mod v0;

use crate::{
	config::{
		api::{AllowedApiCalls, RuntimeRead},
		assets::TrustBackedAssetsInstance,
	},
	fungibles::{self},
	AccountId, Runtime, RuntimeCall, RuntimeOrigin,
};
use codec::{Decode, Encode};
use frame_support::{
	dispatch::{GetDispatchInfo, RawOrigin},
	pallet_prelude::*,
	traits::{Contains, OriginTrait},
};
use pallet_contracts::chain_extension::{
	BufInBufOutState, ChainExtension, Environment, Ext, InitState, RetVal,
};
use pop_primitives::AssetId;
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{traits::Dispatchable, DispatchError};
use sp_std::vec::Vec;

const LOG_TARGET: &str = "pop-api::extension";
const DECODING_FAILED_ERROR: DispatchError = DispatchError::Other("DecodingFailed");
// TODO: issue #93, we can also encode the `pop_primitives::Error::UnknownCall` which means we do use
//  `Error` in the runtime and it should stay in primitives. Perhaps issue #91 will also influence
//  here. Should be looked at together.
const DECODING_FAILED_ERROR_ENCODED: [u8; 4] = [255u8, 0, 0, 0];
const UNKNOWN_CALL_ERROR: DispatchError = DispatchError::Other("UnknownCall");
// TODO: see above.
const UNKNOWN_CALL_ERROR_ENCODED: [u8; 4] = [254u8, 0, 0, 0];

type ContractSchedule<T> = <T as pallet_contracts::Config>::Schedule;

#[derive(Default)]
pub struct PopApiExtension;

impl ChainExtension<Runtime> for PopApiExtension {
	fn call<E: Ext<T = Runtime>>(
		&mut self,
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError> {
		log::debug!(target:LOG_TARGET, " extension called ");
		let mut env = env.buf_in_buf_out();
		// Charge weight for making a call from a contract to the runtime.
		// `debug_message` weight is a good approximation of the additional overhead of going
		// from contract layer to substrate layer.
		// reference: https://github.com/paritytech/ink-examples/blob/b8d2caa52cf4691e0ddd7c919e4462311deb5ad0/psp22-extension/runtime/psp22-extension-example.rs#L236
		let contract_host_weight = ContractSchedule::<Runtime>::get().host_fn_weights;
		env.charge_weight(contract_host_weight.debug_message)?;

		// Extract version and function_id from first two bytes.
		let (version, function_id) = {
			let bytes = env.func_id().to_le_bytes();
			(bytes[0], bytes[1])
		};
		// Extract pallet index and call / key index from last two bytes.
		let (pallet_index, call_index) = {
			let bytes = env.ext_id().to_le_bytes();
			(bytes[0], bytes[1])
		};

		let result = match FuncId::try_from(function_id) {
			Ok(function_id) => {
				// Read encoded parameters from buffer and calculate weight for reading `len` bytes`.
				// reference: https://github.com/paritytech/polkadot-sdk/blob/117a9433dac88d5ac00c058c9b39c511d47749d2/substrate/frame/contracts/src/wasm/runtime.rs#L267
				let len = env.in_len();
				env.charge_weight(contract_host_weight.return_per_byte.saturating_mul(len.into()))?;
				let params = env.read(len)?;
				log::debug!(target: LOG_TARGET, "Read input successfully");
				match function_id {
					FuncId::Dispatch => {
						dispatch::<E>(&mut env, version, pallet_index, call_index, params)
					},
					FuncId::ReadState => {
						read_state::<E>(&mut env, version, pallet_index, call_index, params)
					},
				}
			},
			Err(e) => Err(e),
		};

		match result {
			Ok(_) => Ok(RetVal::Converging(0)),
			Err(e) => Ok(RetVal::Converging(convert_to_status_code(e, version))),
		}
	}
}

fn dispatch<E>(
	env: &mut Environment<E, BufInBufOutState>,
	version: u8,
	pallet_index: u8,
	call_index: u8,
	mut params: Vec<u8>,
) -> Result<(), DispatchError>
where
	E: Ext<T = Runtime>,
	// Runtime: frame_system::Config<RuntimeOrigin = RuntimeOrigin, RuntimeCall = RuntimeCall>,
	// RuntimeOrigin: From<RawOrigin<<Runtime as frame_system::Config>::AccountId>>,
{
	const LOG_PREFIX: &str = " dispatch |";

	// Prefix params with version, pallet, index to simplify decoding.
	params.insert(0, version);
	params.insert(1, pallet_index);
	params.insert(2, call_index);
	let call = <VersionedDispatch>::decode(&mut &params[..]).map_err(|_| DECODING_FAILED_ERROR)?;

	// Contract is the origin by default.
	let origin: RuntimeOrigin = RawOrigin::Signed(env.ext().address().clone()).into();
	match call {
		VersionedDispatch::V0(call) => dispatch_call::<E>(env, call, origin, LOG_PREFIX),
	}
}

fn dispatch_call<E>(
	env: &mut Environment<E, BufInBufOutState>,
	call: RuntimeCall,
	mut origin: RuntimeOrigin,
	log_prefix: &str,
) -> Result<(), DispatchError>
where
	E: Ext<T = Runtime>,
{
	let charged_dispatch_weight = env.charge_weight(call.get_dispatch_info().weight)?;
	log::debug!(target:LOG_TARGET, "{} Inputted RuntimeCall: {:?}", log_prefix, call);
	origin.add_filter(AllowedApiCalls::contains);
	match call.dispatch(origin) {
		Ok(info) => {
			log::debug!(target:LOG_TARGET, "{} success, actual weight: {:?}", log_prefix, info.actual_weight);
			// Refund weight if the actual weight is less than the charged weight.
			if let Some(actual_weight) = info.actual_weight {
				env.adjust_weight(charged_dispatch_weight, actual_weight);
			}
			Ok(())
		},
		Err(err) => {
			log::debug!(target:LOG_TARGET, "{} failed: error: {:?}", log_prefix, err.error);
			Err(err.error)
		},
	}
}

fn read_state<E>(
	env: &mut Environment<E, BufInBufOutState>,
	version: u8,
	pallet_index: u8,
	call_index: u8,
	mut params: Vec<u8>,
) -> Result<(), DispatchError>
where
	E: Ext<T = Runtime>,
	// Runtime: frame_system::Config,
	// Runtime: pallet_api::fungibles::Config,
{
	const LOG_PREFIX: &str = " read_state |";

	// Prefix params with version, pallet, index to simplify decoding, and decode parameters for
	// reading state.
	params.insert(0, version);
	params.insert(1, pallet_index);
	params.insert(2, call_index);
	let read = <VersionedStateRead>::decode(&mut &params[..]).map_err(|_| DECODING_FAILED_ERROR)?;

	// Charge weight for doing one storage read.
	env.charge_weight(<Runtime as frame_system::Config>::DbWeight::get().reads(1_u64))?;
	let result = match read {
		VersionedStateRead::V0(read) => {
			ensure!(AllowedApiCalls::contains(&read), UNKNOWN_CALL_ERROR);
			match read {
				RuntimeRead::Fungibles(key) => fungibles::Pallet::<Runtime>::read_state(key),
			}
		},
	};
	log::trace!(
		target:LOG_TARGET,
		"{} result: {:?}.", LOG_PREFIX, result
	);
	env.write(&result, false, None)
}

/// Wrapper to enable versioning of runtime state reads.
#[derive(Decode, Debug)]
enum VersionedStateRead {
	/// Version zero of state reads.
	#[codec(index = 0)]
	V0(RuntimeRead),
}

/// Wrapper to enable versioning of runtime calls.
#[derive(Decode, Debug)]
enum VersionedDispatch {
	/// Version zero of dispatch calls.
	#[codec(index = 0)]
	V0(RuntimeCall),
}

// Converts a `DispatchError` to a `u32` status code based on the version of the API the contract uses.
// The contract calling the chain extension can convert the status code to the descriptive `Error`.
//
// For `Error` see `pop_primitives::<version>::error::Error`.
//
// The error encoding can vary per version, allowing for flexible and backward-compatible error handling.
// As a result, contracts maintain compatibility across different versions of the runtime.
//
// # Parameters
//
// - `error`: The `DispatchError` encountered during contract execution.
// - `version`: The version of the chain extension, used to determine the known errors.
pub(crate) fn convert_to_status_code(error: DispatchError, version: u8) -> u32 {
	let mut encoded_error: [u8; 4] = match error {
		// "UnknownCall" and "DecodingFailed" are mapped to specific errors in the API and will
		// never change.
		UNKNOWN_CALL_ERROR => UNKNOWN_CALL_ERROR_ENCODED,
		DECODING_FAILED_ERROR => DECODING_FAILED_ERROR_ENCODED,
		_ => {
			let mut encoded_error = error.encode();
			// Resize the encoded value to 4 bytes in order to decode the value in a u32 (4 bytes).
			encoded_error.resize(4, 0);
			encoded_error.try_into().expect("qed, resized to 4 bytes line above")
		},
	};
	match version {
		// If an unknown variant of the `DispatchError` is detected the error needs to be converted
		// into the encoded value of `Error::Other`. This conversion is performed by shifting the bytes one
		// position forward (discarding the last byte as it is not used) and setting the first byte to the
		// encoded value of `Other` (0u8). This ensures the error is correctly categorized as an `Other`
		// variant which provides all the necessary information to debug which error occurred in the runtime.
		//
		// Byte layout explanation:
		// - Byte 0: index of the variant within `Error`
		// - Byte 1:
		//   - Must be zero for `UNIT_ERRORS`.
		//   - Represents the nested error in `SINGLE_NESTED_ERRORS`.
		//   - Represents the first level of nesting in `DOUBLE_NESTED_ERRORS`.
		// - Byte 2:
		//   - Represents the second level of nesting in `DOUBLE_NESTED_ERRORS`.
		// - Byte 3:
		//   - Unused or represents further nested information.
		0 => v0::handle_unknown_error(&mut encoded_error),
		_ => encoded_error = UNKNOWN_CALL_ERROR_ENCODED,
	}
	u32::from_le_bytes(encoded_error)
}

/// Function identifiers used in the Pop API.
///
/// The `FuncId` specifies the available functions that can be called through the Pop API. Each
/// variant corresponds to a specific functionality provided by the API, facilitating the
/// interaction between smart contracts and the runtime.
///
/// - `Dispatch`: Represents a function call to dispatch a runtime call.
/// - `ReadState`: Represents a function call to read the state from the runtime.
/// - `SendXcm`: Represents a function call to send an XCM message.
#[derive(Debug)]
pub enum FuncId {
	Dispatch,
	ReadState,
}

impl TryFrom<u8> for FuncId {
	type Error = DispatchError;

	/// Attempts to convert a `u8` value to its corresponding `FuncId` variant.
	///
	/// If the `u8` value does not match any known function identifier, it returns a
	/// `DispatchError::Other` indicating an unknown function ID.
	fn try_from(func_id: u8) -> Result<Self, Self::Error> {
		let id = match func_id {
			0 => Self::Dispatch,
			1 => Self::ReadState,
			_ => {
				return Err(UNKNOWN_CALL_ERROR);
			},
		};
		Ok(id)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{Assets, Runtime, System};
	use sp_runtime::BuildStorage;

	// Test ensuring `func_id()` and `ext_id()` work as expected, i.e. extracting the first two
	// bytes and the last two bytes, respectively, from a 4 byte array.
	#[test]
	fn test_byte_extraction() {
		use rand::Rng;

		// Helper functions
		fn func_id(id: u32) -> u16 {
			(id & 0x0000FFFF) as u16
		}
		fn ext_id(id: u32) -> u16 {
			(id >> 16) as u16
		}

		// Number of test iterations
		let test_iterations = 1_000_000;

		// Create a random number generator
		let mut rng = rand::thread_rng();

		// Run the test for a large number of random 4-byte arrays
		for _ in 0..test_iterations {
			// Generate a random 4-byte array
			let bytes: [u8; 4] = rng.gen();

			// Convert the 4-byte array to a u32 value
			let value = u32::from_le_bytes(bytes);

			// Extract the first two bytes (least significant 2 bytes)
			let first_two_bytes = func_id(value);

			// Extract the last two bytes (most significant 2 bytes)
			let last_two_bytes = ext_id(value);

			// Check if the first two bytes match the expected value
			assert_eq!([bytes[0], bytes[1]], first_two_bytes.to_le_bytes());

			// Check if the last two bytes match the expected value
			assert_eq!([bytes[2], bytes[3]], last_two_bytes.to_le_bytes());
		}
	}

	// Test showing all the different type of variants and its encoding.
	#[test]
	fn encoding_of_enum() {
		#[derive(Debug, PartialEq, Encode, Decode)]
		enum ComprehensiveEnum {
			SimpleVariant,
			DataVariant(u8),
			NamedFields { w: u8 },
			NestedEnum(InnerEnum),
			OptionVariant(Option<u8>),
			VecVariant(Vec<u8>),
			TupleVariant(u8, u8),
			NestedStructVariant(NestedStruct),
			NestedEnumStructVariant(NestedEnumStruct),
		}

		#[derive(Debug, PartialEq, Encode, Decode)]
		enum InnerEnum {
			A,
			B { inner_data: u8 },
			C(u8),
		}

		#[derive(Debug, PartialEq, Encode, Decode)]
		struct NestedStruct {
			x: u8,
			y: u8,
		}

		#[derive(Debug, PartialEq, Encode, Decode)]
		struct NestedEnumStruct {
			inner_enum: InnerEnum,
		}

		// Creating each possible variant for an enum.
		let enum_simple = ComprehensiveEnum::SimpleVariant;
		let enum_data = ComprehensiveEnum::DataVariant(42);
		let enum_named = ComprehensiveEnum::NamedFields { w: 42 };
		let enum_nested = ComprehensiveEnum::NestedEnum(InnerEnum::B { inner_data: 42 });
		let enum_option = ComprehensiveEnum::OptionVariant(Some(42));
		let enum_vec = ComprehensiveEnum::VecVariant(vec![1, 2, 3, 4, 5]);
		let enum_tuple = ComprehensiveEnum::TupleVariant(42, 42);
		let enum_nested_struct =
			ComprehensiveEnum::NestedStructVariant(NestedStruct { x: 42, y: 42 });
		let enum_nested_enum_struct =
			ComprehensiveEnum::NestedEnumStructVariant(NestedEnumStruct {
				inner_enum: InnerEnum::C(42),
			});

		// Encode and print each variant individually to see their encoded values.
		println!("{:?} -> {:?}", enum_simple, enum_simple.encode());
		println!("{:?} -> {:?}", enum_data, enum_data.encode());
		println!("{:?} -> {:?}", enum_named, enum_named.encode());
		println!("{:?} -> {:?}", enum_nested, enum_nested.encode());
		println!("{:?} -> {:?}", enum_option, enum_option.encode());
		println!("{:?} -> {:?}", enum_vec, enum_vec.encode());
		println!("{:?} -> {:?}", enum_tuple, enum_tuple.encode());
		println!("{:?} -> {:?}", enum_nested_struct, enum_nested_struct.encode());
		println!("{:?} -> {:?}", enum_nested_enum_struct, enum_nested_enum_struct.encode());
	}

	fn new_test_ext() -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	#[test]
	fn encoding_decoding_dispatch_error() {
		use sp_runtime::{ArithmeticError, DispatchError, ModuleError, TokenError};

		new_test_ext().execute_with(|| {
			let error = DispatchError::Module(ModuleError {
				index: 255,
				error: [2, 0, 0, 0],
				message: Some("error message"),
			});
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![3, 255, 2, 0, 0, 0]);
			assert_eq!(
				decoded,
				// `message` is skipped for encoding.
				DispatchError::Module(ModuleError {
					index: 255,
					error: [2, 0, 0, 0],
					message: None
				})
			);

			// Example pallet assets Error into ModuleError.
			let index = <<Runtime as frame_system::Config>::PalletInfo as frame_support::traits::PalletInfo>::index::<
				Assets,
			>()
			.expect("Every active module has an index in the runtime; qed") as u8;
			let mut error =
				pallet_assets::Error::NotFrozen::<Runtime, TrustBackedAssetsInstance>.encode();
			error.resize(MAX_MODULE_ERROR_ENCODED_SIZE, 0);
			let error = DispatchError::Module(ModuleError {
				index,
				error: TryInto::try_into(error).expect("should work"),
				message: None,
			});
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![3, 52, 18, 0, 0, 0]);
			assert_eq!(
				decoded,
				DispatchError::Module(ModuleError {
					index: 52,
					error: [18, 0, 0, 0],
					message: None
				})
			);

			// Example DispatchError::Token
			let error = DispatchError::Token(TokenError::UnknownAsset);
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![7, 4]);
			assert_eq!(decoded, error);

			// Example DispatchError::Arithmetic
			let error = DispatchError::Arithmetic(ArithmeticError::Overflow);
			let encoded = error.encode();
			let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
			assert_eq!(encoded, vec![8, 1]);
			assert_eq!(decoded, error);
		});
	}
}

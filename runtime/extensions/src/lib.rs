pub mod constants;
mod v0;

use codec::Encode;
use constants::*;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::Contains,
};
use pallet_contracts::chain_extension::{
	BufInBufOutState, ChainExtension, Environment, Ext, InitState, RetVal,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{traits::Dispatchable, DispatchError};
use sp_std::vec::Vec;

#[cfg(feature = "pop-devnet")]
pub(crate) use pallet_api::fungibles;
#[cfg(feature = "pop-devnet")]
use pop_primitives::AssetId;

type ContractSchedule<T> = <T as pallet_contracts::Config>::Schedule;

/// Handler to process the parameters from the chain extension environment for read state calls.
pub trait ReadStateParamsHandler {
	fn handle_params<T, E>(
		env: &mut Environment<E, BufInBufOutState>,
		params: Vec<u8>,
	) -> Result<(), DispatchError>
	where
		E: Ext<T = T>,
		T: PopApiExtensionConfig;
}

/// Handler to process the parameters from the chain extension environment for dispatch calls.
pub trait DispatchCallParamsHandler {
	fn handle_params<T, E>(
		env: &mut Environment<E, BufInBufOutState>,
		params: Vec<u8>,
	) -> Result<(), DispatchError>
	where
		E: Ext<T = T>,
		T: PopApiExtensionConfig;
}

#[cfg(feature = "pop-devnet")]
pub trait PopApiExtensionConfig:
	frame_system::Config<RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>>
	+ pallet_assets::Config<Self::AssetInstance, AssetId = AssetId>
	+ fungibles::Config
{
	type AssetInstance;
	type ReadStateParamsHandler: ReadStateParamsHandler;
	type DispatchCallParamsHandler: DispatchCallParamsHandler;
	type AllowedDispatchCalls: Contains<Self::RuntimeCall>;
}

#[cfg(not(feature = "pop-devnet"))]
pub trait PopApiExtensionConfig:
	frame_system::Config<RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>>
{
	type ReadStateParamsHandler: ReadStateParamsHandler;
	type DispatchCallParamsHandler: DispatchCallParamsHandler;
	type AllowedDispatchCalls: Contains<Self::RuntimeCall>;
}

#[derive(Default)]
pub struct PopApiExtension;

/// Extract (version, function_id, pallet_index, call_index) from the payload bytes
fn extract_env<T, E: Ext>(env: &Environment<E, BufInBufOutState>) -> (u8, u8, u8, u8)
where
	E: Ext<T = T>,
{
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

	(version, function_id, pallet_index, call_index)
}

impl<T> ChainExtension<T> for PopApiExtension
where
	T: PopApiExtensionConfig + pallet_contracts::Config,
	T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
	{
		log::debug!(target:LOG_TARGET, " extension called ");
		let mut env = env.buf_in_buf_out();
		// Charge weight for making a call from a contract to the runtime.
		// `debug_message` weight is a good approximation of the additional overhead of going
		// from contract layer to substrate layer.
		// reference: https://github.com/paritytech/ink-examples/blob/b8d2caa52cf4691e0ddd7c919e4462311deb5ad0/psp22-extension/runtime/psp22-extension-example.rs#L236
		let contract_host_weight = ContractSchedule::<T>::get().host_fn_weights;
		env.charge_weight(contract_host_weight.debug_message)?;

		let (version, function_id, pallet_index, call_index) = extract_env(&env);

		let result = match FuncId::try_from(function_id) {
			// Read encoded parameters from buffer and calculate weight for reading `len` bytes`.
			Ok(function_id) => {
				// reference: https://github.com/paritytech/polkadot-sdk/blob/117a9433dac88d5ac00c058c9b39c511d47749d2/substrate/frame/contracts/src/wasm/runtime.rs#L267
				let len = env.in_len();
				env.charge_weight(contract_host_weight.return_per_byte.saturating_mul(len.into()))?;
				let params = env.read(len)?;
				log::debug!(target: LOG_TARGET, "Read input successfully");
				match function_id {
					FuncId::Dispatch => {
						dispatch::<T, E>(&mut env, version, pallet_index, call_index, params)
					},
					FuncId::ReadState => {
						read_state::<T, E>(&mut env, version, pallet_index, call_index, params)
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

fn dispatch<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	version: u8,
	pallet_index: u8,
	call_index: u8,
	mut params: Vec<u8>,
) -> Result<(), DispatchError>
where
	T: PopApiExtensionConfig,
	E: Ext<T = T>,
{
	// Prefix params with version, pallet, index to simplify decoding.
	params.insert(0, version);
	params.insert(1, pallet_index);
	params.insert(2, call_index);
	// Handle the params for the dispatch call.
	T::DispatchCallParamsHandler::handle_params(env, params)
}

pub fn dispatch_call<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	call: T::RuntimeCall,
	origin: T::RuntimeOrigin,
	log_prefix: &str,
) -> Result<(), DispatchError>
where
	T: PopApiExtensionConfig,
	E: Ext<T = T>,
{
	let charged_dispatch_weight = env.charge_weight(call.get_dispatch_info().weight)?;
	log::debug!(target:LOG_TARGET, "{} Inputted RuntimeCall: {:?}", log_prefix, call);
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

fn read_state<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	version: u8,
	pallet_index: u8,
	call_index: u8,
	mut params: Vec<u8>,
) -> Result<(), DispatchError>
where
	T: PopApiExtensionConfig + pallet_contracts::Config,
	E: Ext<T = T>,
{
	// Prefix params with version, pallet, index to simplify decoding, and decode parameters for
	// reading state.
	params.insert(0, version);
	params.insert(1, pallet_index);
	params.insert(2, call_index);
	// Handle the params for the read state call.
	T::ReadStateParamsHandler::handle_params(env, params)
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

#[cfg(all(feature = "pop-devnet", test))]
mod tests {
	use super::*;

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
}

#![cfg_attr(not(feature = "std"), no_std)]

mod constants;
mod mock;
#[cfg(test)]
mod tests;
mod v0;

use codec::Encode;
use constants::*;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::{Contains, OriginTrait},
};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
	BufInBufOutState, ChainExtension, Environment, Ext, InitState, RetVal,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{traits::Dispatchable, DispatchError};
use sp_std::vec::Vec;

type ContractSchedule<T> = <T as pallet_contracts::Config>::Schedule;

pub trait Config:
	frame_system::Config<RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>>
{
	/// A query of runtime state.
	type RuntimeRead: Decode + ReadStateHandler<Self>;
	/// Allowlisted runtime calls and read state calls.
	type AllowedApiCalls: Contains<Self::RuntimeCall> + Contains<Self::RuntimeRead>;
}

/// Trait for handling parameters from the chain extension environment during state read operations.
pub trait ReadStateHandler<T: Config> {
	/// Processes the parameters needed to execute a call within the runtime environment.
	/// Layout of `params`: [version, pallet_index, call_index, ...Vec<u8>].
	fn handle_read(params: T::RuntimeRead) -> Vec<u8>;
}

#[derive(Default)]
pub struct ApiExtension;

/// Extract (version, function_id, pallet_index, call_index) from the payload bytes.
fn extract_env<T, E: Ext<T = T>>(env: &Environment<E, BufInBufOutState>) -> (u8, u8, u8, u8) {
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

impl<T> ChainExtension<T> for ApiExtension
where
	T: Config + pallet_contracts::Config,
	T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	fn call<E: Ext<T = T>>(
		&mut self,
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError> {
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

/// Helper method to decode the byte data to a provided type and throws error if failed.
fn decode_checked<T: Decode>(params: &mut &[u8]) -> Result<T, DispatchError> {
	T::decode(params).map_err(|_| DECODING_FAILED_ERROR)
}

fn read_state<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	version: u8,
	pallet_index: u8,
	call_index: u8,
	mut params: Vec<u8>,
) -> Result<(), DispatchError>
where
	T: Config,
	E: Ext<T = T>,
{
	const LOG_PREFIX: &str = " read_state |";

	// Prefix params with version, pallet, index to simplify decoding.
	params.insert(0, version);
	params.insert(1, pallet_index);
	params.insert(2, call_index);
	let (mut encoded_version, mut encoded_read) = (&params[..1], &params[1..]);
	let version = decode_checked::<VersionedStateRead>(&mut encoded_version)?;

	// Charge weight for doing one storage read.
	env.charge_weight(T::DbWeight::get().reads(1_u64))?;
	let result = match version {
		VersionedStateRead::V0 => {
			let read = decode_checked::<T::RuntimeRead>(&mut encoded_read)?;
			ensure!(T::AllowedApiCalls::contains(&read), UNKNOWN_CALL_ERROR);
			T::RuntimeRead::handle_read(read)
		},
	};
	log::trace!(
		target:LOG_TARGET,
		"{} result: {:?}.", LOG_PREFIX, result
	);
	env.write(&result, false, None)
}

fn dispatch<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	version: u8,
	pallet_index: u8,
	call_index: u8,
	mut params: Vec<u8>,
) -> Result<(), DispatchError>
where
	T: Config,
	E: Ext<T = T>,
{
	const LOG_PREFIX: &str = " dispatch |";

	// Prefix params with version, pallet, index to simplify decoding.
	params.insert(0, version);
	params.insert(1, pallet_index);
	params.insert(2, call_index);
	let call = decode_checked::<VersionedDispatch<T>>(&mut &params[..])?;
	// Contract is the origin by default.
	let origin: T::RuntimeOrigin = RawOrigin::Signed(env.ext().address().clone()).into();
	match call {
		VersionedDispatch::V0(call) => dispatch_call::<T, E>(env, call, origin, LOG_PREFIX),
	}
}

fn dispatch_call<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	call: T::RuntimeCall,
	mut origin: T::RuntimeOrigin,
	log_prefix: &str,
) -> Result<(), DispatchError>
where
	T: Config,
	E: Ext<T = T>,
{
	let charged_dispatch_weight = env.charge_weight(call.get_dispatch_info().weight)?;
	log::debug!(target:LOG_TARGET, "{} Inputted RuntimeCall: {:?}", log_prefix, call);
	origin.add_filter(T::AllowedApiCalls::contains);
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

/// Wrapper to enable versioning of runtime state reads.
#[derive(Decode, Debug)]
enum VersionedStateRead {
	/// Version zero of state reads.
	#[codec(index = 0)]
	V0,
}

/// Wrapper to enable versioning of runtime calls.
#[derive(Decode, Debug)]
enum VersionedDispatch<T: Config> {
	/// Version zero of dispatch calls.
	#[codec(index = 0)]
	V0(T::RuntimeCall),
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

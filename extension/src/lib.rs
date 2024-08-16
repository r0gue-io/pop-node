#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;
mod v0;

use codec::Encode;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo},
	pallet_prelude::*,
	traits::OriginTrait,
};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{
	BufInBufOutState, ChainExtension, Environment, Ext, InitState, RetVal,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{traits::Dispatchable, DispatchError};
use sp_std::vec::Vec;

/// Logging target for categorizing messages from the Pop API extension module.
const LOG_TARGET: &str = "pop-api::extension";

pub const DECODING_FAILED_ERROR: DispatchError = DispatchError::Other("DecodingFailed");
const DECODING_FAILED_ERROR_ENCODED: [u8; 4] = [255u8, 0, 0, 0];
pub const UNKNOWN_CALL_ERROR: DispatchError = DispatchError::Other("UnknownCall");
const UNKNOWN_CALL_ERROR_ENCODED: [u8; 4] = [254u8, 0, 0, 0];

type ContractSchedule<T> = <T as pallet_contracts::Config>::Schedule;

/// Type of the state reader.
pub trait ReadState {
	/// Query of the state read operations.
	type StateQuery: Decode;

	/// Check if a state query is allowed.
	fn contains(c: &Self::StateQuery) -> bool;

	/// Reads state using the provided query, returning the result as a byte vector.
	fn read(read: Self::StateQuery) -> Vec<u8>;

	fn versioned_read_state_handler(
		params: Vec<u8>,
		version: u8,
		pallet_index: u8,
		call_index: u8,
	) -> Result<Self::StateQuery, DispatchError>;
}

/// Type of the dispatch call filter.
pub trait HandleDispatch {
	/// Query of the dispatch calls operations.
	type Call: Decode;

	/// Check if runtime call is allowed.
	fn contains(t: &Self::Call) -> bool;

	fn versioned_dispatch_handler(
		params: Vec<u8>,
		version: u8,
		pallet_index: u8,
		call_index: u8,
	) -> Result<Self::Call, DispatchError>;
}

pub trait ExtractEnv {
	fn extract_env<T, E: Ext<T = T>>(env: &Environment<E, BufInBufOutState>) -> (u8, u8, u8, u8);
}

pub trait ExtensionTrait<T: pallet_contracts::Config + frame_system::Config> {
	type StateReader: ReadState;
	type DispatchHandler: HandleDispatch<Call = <T as frame_system::Config>::RuntimeCall> + 'static;
	type EnvExtractor: ExtractEnv;

	fn call<E: Ext<T = T>>(
		env: &mut Environment<E, BufInBufOutState>,
	) -> Result<RetVal, DispatchError>
	where
		T: frame_system::Config<
			RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
		>,
		E: Ext<T = T>,
	{
		// Charge weight for making a call from a contract to the runtime.
		// `debug_message` weight is a good approximation of the additional overhead of going
		// from contract layer to substrate layer.
		// reference: https://github.com/paritytech/ink-examples/blob/b8d2caa52cf4691e0ddd7c919e4462311deb5ad0/psp22-extension/runtime/psp22-extension-example.rs#L236
		let contract_host_weight = ContractSchedule::<T>::get().host_fn_weights;
		env.charge_weight(contract_host_weight.debug_message)?;

		let (function_id, version, pallet_index, call_index) =
			Self::EnvExtractor::extract_env(&env);

		let result = match FuncId::try_from(function_id) {
			// Read encoded parameters from buffer and calculate weight for reading `len` bytes`.
			Ok(function_id) => {
				// reference: https://github.com/paritytech/polkadot-sdk/blob/117a9433dac88d5ac00c058c9b39c511d47749d2/substrate/frame/contracts/src/wasm/runtime.rs#L267
				let len = env.in_len();
				env.charge_weight(contract_host_weight.return_per_byte.saturating_mul(len.into()))?;
				let params = env.read(len)?;
				match function_id {
					FuncId::Dispatch => {
						Self::dispatch::<E>(env, version, pallet_index, call_index, params)
					},
					FuncId::ReadState => {
						Self::read_state::<E>(env, version, pallet_index, call_index, params)
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

	fn dispatch<E>(
		env: &mut Environment<E, BufInBufOutState>,
		version: u8,
		pallet_index: u8,
		call_index: u8,
		params: Vec<u8>,
	) -> Result<(), DispatchError>
	where
		T: frame_system::Config<
			RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
		>,
		E: Ext<T = T>,
	{
		const LOG_PREFIX: &str = " dispatch |";

		// Decode the parameters in a dispatch call based on the version, pallet index and call index.
		let call = Self::DispatchHandler::versioned_dispatch_handler(
			params,
			version,
			pallet_index,
			call_index,
		)?;
		log::debug!(target:LOG_TARGET, "{} Inputted RuntimeCall: {:?}", LOG_PREFIX, call);
		// Contract is the origin by default.
		let mut origin: T::RuntimeOrigin = RawOrigin::Signed(env.ext().address().clone()).into();
		// Charge pre-dispatch weight.
		let charged_dispatch_weight = env.charge_weight(call.get_dispatch_info().weight)?;
		// Check whether the dispatch call is allowed by the runtime.
		origin.add_filter(Self::DispatchHandler::contains);
		match call.dispatch(origin) {
			Ok(info) => {
				log::debug!(target:LOG_TARGET, "{} success, actual weight: {:?}", LOG_PREFIX, info.actual_weight);
				// Refund weight if the actual weight is less than the charged weight.
				if let Some(actual_weight) = info.actual_weight {
					env.adjust_weight(charged_dispatch_weight, actual_weight);
				}
				Ok(())
			},
			Err(err) => {
				log::debug!(target:LOG_TARGET, "{} failed: error: {:?}", LOG_PREFIX, err.error);
				// Refund weight if the actual weight is less than the charged weight.
				if let Some(actual_weight) = err.post_info.actual_weight {
					env.adjust_weight(charged_dispatch_weight, actual_weight);
				}
				Err(err.error)
			},
		}
	}

	fn read_state<E: Ext<T = T>>(
		env: &mut Environment<E, BufInBufOutState>,
		version: u8,
		pallet_index: u8,
		call_index: u8,
		params: Vec<u8>,
	) -> Result<(), DispatchError> {
		// Charge weight for doing one storage read.
		env.charge_weight(T::DbWeight::get().reads(1_u64))?;
		// Decode the parameters in a state query based on the version, pallet index and call index.
		let read = Self::StateReader::versioned_read_state_handler(
			params,
			version,
			pallet_index,
			call_index,
		)?;
		// Check whether the state query is allowed by the runtime.
		ensure!(Self::StateReader::contains(&read), UNKNOWN_CALL_ERROR);
		// Query state.
		let result = Self::StateReader::read(read);
		log::trace!(
			target:LOG_TARGET,
			"{} result: {:?}.", " read_state |", result
		);
		// Write the result in the buffer for the contract to obtain.
		env.write(&result, false, None)
	}
}

/// Pop API chain extension.
#[derive(Default)]
pub struct ApiExtension<T: pallet_contracts::Config, I: ExtensionTrait<T>>(PhantomData<(T, I)>);

impl<T, I> ChainExtension<T> for ApiExtension<T, I>
where
	T: pallet_contracts::Config
		+ frame_system::Config<
			RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
		>,
	T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
	// Bound the type by the two traits which need to be implemented by the runtime.
	I: ExtensionTrait<T>,
{
	fn call<E: Ext<T = T>>(
		&mut self,
		env: Environment<E, InitState>,
	) -> Result<RetVal, DispatchError> {
		let mut env = env.buf_in_buf_out();
		<I as ExtensionTrait<T>>::call(&mut env)
	}
}

/// Function identifiers used in the Pop API.
///
/// The `FuncId` specifies the available functions that can be called through the Pop API. Each
/// variant corresponds to a specific functionality provided by the API, facilitating the
/// interaction between smart contracts and the runtime.
#[derive(Debug)]
pub enum FuncId {
	/// Represents a function call to dispatch a runtime call.
	Dispatch,
	/// Represents a function call to read the state from the runtime.
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

/// Converts a `DispatchError` to a `u32` status code based on the version of the API the contract uses.
/// The contract calling the chain extension can convert the status code to the descriptive `Error`.
///
/// For `Error` see `pop_primitives::<version>::error::Error`.
///
/// The error encoding can vary per version, allowing for flexible and backward-compatible error handling.
/// As a result, contracts maintain compatibility across different versions of the runtime.
///
/// # Parameters
///
/// - `error`: The `DispatchError` encountered during contract execution.
/// - `version`: The version of the chain extension, used to determine the known errors.
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

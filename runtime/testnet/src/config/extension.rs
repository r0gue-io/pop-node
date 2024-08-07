use crate::{Runtime, RuntimeCall};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, traits::Contains};
use pallet_contracts::chain_extension::{BufInBufOutState, Environment, Ext};
use pop_runtime_extensions::{
	constants::{DECODING_FAILED_ERROR, LOG_TARGET, UNKNOWN_CALL_ERROR},
	PopApiExtensionConfig, StateReadHandler,
};
use sp_core::Get;
use sp_runtime::DispatchError;

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum RuntimeRead {}

/// A type to identify allowed calls to the Runtime from the API.
pub struct AllowedApiCalls;

impl Contains<RuntimeCall> for AllowedApiCalls {
	/// Allowed runtime calls from the API.
	fn contains(_: &RuntimeCall) -> bool {
		false
	}
}

impl Contains<RuntimeRead> for AllowedApiCalls {
	/// Allowed state queries from the API.
	fn contains(_: &RuntimeRead) -> bool {
		false
	}
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
enum VersionedDispatch<T: PopApiExtensionConfig> {
	/// Version zero of dispatch calls.
	#[codec(index = 0)]
	V0(T::RuntimeCall),
}

pub struct ContractExecutionContext;

impl StateReadHandler for ContractExecutionContext {
	fn handle_params<T, E>(
		env: &mut Environment<E, BufInBufOutState>,
		params: Vec<u8>,
	) -> Result<(), DispatchError>
	where
		E: Ext<T = T>,
		T: PopApiExtensionConfig,
	{
		const LOG_PREFIX: &str = " read_state |";

		let read =
			<VersionedStateRead>::decode(&mut &params[..]).map_err(|_| DECODING_FAILED_ERROR)?;

		// Charge weight for doing one storage read.
		env.charge_weight(T::DbWeight::get().reads(1_u64))?;
		let result = match read {
			VersionedStateRead::V0(read) => {
				ensure!(AllowedApiCalls::contains(&read), UNKNOWN_CALL_ERROR);
				vec![0u8]
			},
		};
		log::trace!(
			target:LOG_TARGET,
			"{} result: {:?}.", LOG_PREFIX, result
		);
		env.write(&result, false, None)
	}
}

impl PopApiExtensionConfig for Runtime {
	type StateReadHandler = ContractExecutionContext;
	type AllowedDispatchCalls = AllowedApiCalls;
}

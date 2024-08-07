use crate::{
	config::api::AllowedApiCalls,
	fungibles::{self},
	Runtime,
};
use codec::Decode;
use frame_support::{ensure, traits::Contains};
use pallet_contracts::chain_extension::{BufInBufOutState, Environment, Ext};
use pop_runtime_extension::{
	constants::{DECODING_FAILED_ERROR, LOG_TARGET, UNKNOWN_CALL_ERROR},
	StateReadHandler,
};
use sp_core::Get;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

use super::api::RuntimeRead;

/// Wrapper to enable versioning of runtime state reads.
#[derive(Decode, Debug)]
enum VersionedStateRead {
	/// Version zero of state reads.
	#[codec(index = 0)]
	V0(RuntimeRead),
}

pub struct ContractExecutionContext;

impl StateReadHandler for ContractExecutionContext {
	fn handle_params<T, E>(
		env: &mut Environment<E, BufInBufOutState>,
		params: Vec<u8>,
	) -> Result<(), DispatchError>
	where
		E: Ext<T = T>,
		T: pop_runtime_extension::Config,
	{
		const LOG_PREFIX: &str = " read_state |";

		let read =
			<VersionedStateRead>::decode(&mut &params[..]).map_err(|_| DECODING_FAILED_ERROR)?;

		// Charge weight for doing one storage read.
		env.charge_weight(T::DbWeight::get().reads(1_u64))?;
		let result = match read {
			VersionedStateRead::V0(read) => {
				ensure!(AllowedApiCalls::contains(&read), UNKNOWN_CALL_ERROR);
				match read {
					RuntimeRead::Fungibles(key) => fungibles::Pallet::read_state(key),
				}
			},
		};
		log::trace!(
			target:LOG_TARGET,
			"{} result: {:?}.", LOG_PREFIX, result
		);
		env.write(&result, false, None)
	}
}

impl pop_runtime_extension::Config for Runtime {
	type StateReadHandler = ContractExecutionContext;
	type AllowedDispatchCalls = AllowedApiCalls;
}

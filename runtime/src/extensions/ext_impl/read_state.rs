use cumulus_primitives_core::relay_chain::BlockNumber;
use frame_support::pallet_prelude::*;
use log;
use pallet_contracts::chain_extension::{Environment, Ext, InitState};

const LOG_TARGET: &str = "popapi::extension::read_state";

// SAFE_KEYS should live in pop-api repo, both this runtime and the contract
// can depend on pop-api to be able to interface with each other.
#[derive(codec::Decode)]
enum SafeKeys {
    RelayBlockNumber,
    // 0x45323df7cc47150b3930e2666b0aa313a2bca190d36bd834cc73a38fc213ecbd
}

pub(crate) fn read_state<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
    T: pallet_contracts::Config + frame_system::Config,
    E: Ext<T = T>,
{
    let mut env = env.buf_in_buf_out();
    let len = env.in_len();
    let key: SafeKeys = env.read_as_unbounded(len)?;
    match key {
        SafeKeys::RelayBlockNumber => {
            let block_num: BlockNumber = crate::ParachainSystem::last_relay_block_number();
            log::debug!(
                target:LOG_TARGET,
                "Last Relay Chain Block Number is: {:?}.", block_num
            );
            Ok(())
        }
        _ => Err(DispatchError::Other("Unable to read provided key.")),
    }
}

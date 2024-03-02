use codec::Decode;
use cumulus_primitives_core::relay_chain::BlockNumber;
use frame_support::pallet_prelude::*;
use log;
use pallet_contracts::chain_extension::{Environment, Ext, InitState};
use pop_api_primitives::storage_keys::ParachainSystemKeys;

const LOG_TARGET: &str = "popapi::extension::read_state";

pub(crate) fn read_state<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
    T: pallet_contracts::Config + frame_system::Config,
    E: Ext<T = T>,
{
    let mut env = env.buf_in_buf_out();
    // TODO: Substitue len u32 with pop_api::src::impls::pop_network::StringLimit.
    // Move StringLimit to pop_api_primitives first.
    let len: u32 = env.in_len();
    let key: ParachainSystemKeys = env.read_as_unbounded(len)?;

    match key {
        LastRelayChainBlockNumber => {
            let relay_block_num: BlockNumber =
                cumulus_pallet_parachain_system::LastRelayChainBlockNumber::get();
            log::debug!(
                target:LOG_TARGET,
                "Last Relay Chain Block Number is: {:?}.", relay_block_num
            );
            //Ok(relay_block_num)
            Ok(())
        }
        _ => Err(DispatchError::Other("Unable to read provided key.")),
    }
}

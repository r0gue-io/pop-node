use frame_support::{
    dispatch::{GetDispatchInfo, PostDispatchInfo, RawOrigin},
    pallet_prelude::*,
};
use log;
use pallet_contracts::chain_extension::{Environment, Ext, InitState, SysConfig};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{traits::Dispatchable, DispatchError};

const LOG_TARGET: &str = "popapi::extension::dispatch";

pub(crate) fn dispatch<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
    T: pallet_contracts::Config + frame_system::Config,
    <T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
    <T as SysConfig>::RuntimeCall: Parameter
        + Dispatchable<RuntimeOrigin = <T as SysConfig>::RuntimeOrigin, PostInfo = PostDispatchInfo>
        + GetDispatchInfo
        + From<frame_system::Call<T>>,
    E: Ext<T = T>,
{
    let mut env = env.buf_in_buf_out();

    // charge max weight before reading contract memory
    // TODO: causing "1010: block limits exhausted" error
    // let weight_limit = env.ext().gas_meter().gas_left();
    // let charged_weight = env.charge_weight(weight_limit)?;

    // TODO: debug_message weight is a good approximation of the additional overhead of going
    // from contract layer to substrate layer.

    // input length
    let len = env.in_len();
    let call: <T as SysConfig>::RuntimeCall = env.read_as_unbounded(len)?;

    log::debug!(target:LOG_TARGET, " dispatch inputted RuntimeCall: {:?}", call);

    let sender = env.ext().caller();
    let origin: T::RuntimeOrigin = RawOrigin::Signed(sender.account_id()?.clone()).into();

    // TODO: uncomment once charged_weight is fixed
    // let actual_weight = call.get_dispatch_info().weight;
    // env.adjust_weight(charged_weight, actual_weight);
    let result = call.dispatch(origin);
    match result {
        Ok(info) => {
            log::debug!(target:LOG_TARGET, "dispatch success, actual weight: {:?}", info.actual_weight);
        }
        Err(err) => {
            log::debug!(target:LOG_TARGET, "dispatch failed: error: {:?}", err.error);
            return Err(err.error);
        }
    }
    Ok(())
}

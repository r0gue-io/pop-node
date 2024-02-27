use codec::{
    Decode,
    Encode,
    MaxEncodedLen,
};
use frame_support::{
    dispatch::RawOrigin,
    pallet_prelude::*,
};

use log::{error, log, trace};

use pallet_contracts::chain_extension::{
    ChainExtension,
    Environment,
    Ext,
    InitState,
    RetVal,
    SysConfig,
};
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{
    traits::{
        Dispatchable, Saturating, StaticLookup, Zero
    },
    DispatchError,
};

use crate::{RuntimeCall, RuntimeOrigin};


#[derive(Default)]
pub struct PopApiExtension;

fn convert_err(err_msg: &'static str) -> impl FnOnce(DispatchError) -> DispatchError {
    move |err| {
        trace!(
            target: "runtime",
            "Pop API failed:{:?}",
            err
        );
        DispatchError::Other(err_msg)
    }
}

/// We're using enums for function IDs because contrary to raw u16 it enables
/// exhaustive matching, which results in cleaner code.
#[derive(Debug)]
enum FuncId {
    CallRuntime
}

impl TryFrom<u16> for FuncId {
    type Error = DispatchError;

    fn try_from(func_id: u16) -> Result<Self, Self::Error> {
        let id = match func_id {
            0xfecb => Self::CallRuntime,
            _ => {
                error!("Called an unregistered `func_id`: {:}", func_id);
                return Err(DispatchError::Other("Unimplemented func_id"))
            }
        };

        Ok(id)
    }
}
const LOG_TARGET: &str = "runtime::contracts";
fn call_runtime<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
    T: pallet_contracts::Config,
    <T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
    E: Ext<T = T>,
{
    log::debug!(target:LOG_TARGET, "popapi call_runtime");
    let mut env = env.buf_in_buf_out();

    // TODO: calculate weight and charge fees

    // let base_weight = <T as pallet_assets::Config>::WeightInfo::transfer();
    // debug_message weight is a good approximation of the additional overhead of going
    // from contract layer to substrate layer.
    let len = env.in_len();
    let mut call: RuntimeCall = env.read_as_unbounded(len)?;
    log::debug!(target:LOG_TARGET, "popapi dispatch {:?}", call);

    //TODO: properly set the origin to the sender
    let sender = env.ext().caller();
    let origin: RuntimeOrigin = RawOrigin::Root.into();
    let result = call.dispatch(origin);
    log::debug!(target:LOG_TARGET, "pop api trace {:?}", result);
    Ok(())
}


impl<T> ChainExtension<T> for PopApiExtension
where
    T: pallet_contracts::Config,
    <T as SysConfig>::AccountId: UncheckedFrom<<T as SysConfig>::Hash> + AsRef<[u8]>,
{
    fn call<E: Ext>(
        &mut self,
        env: Environment<E, InitState>,
    ) -> Result<RetVal, DispatchError>
    where
        E: Ext<T = T>,
        <E::T as SysConfig>::AccountId:
            UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
    {
        
        let func_id = FuncId::try_from(env.func_id())?;
        log::debug!(target:LOG_TARGET, "popapi call_hit id: {:?}", func_id);
        match func_id {
            FuncId::CallRuntime => call_runtime::<T, E>(env)?,
        }

        Ok(RetVal::Converging(0))
    }
}

use frame_support::traits::{Contains, OriginTrait};
use frame_support::{
	dispatch::{GetDispatchInfo, RawOrigin},
	pallet_prelude::*,
};
use pallet_contracts::chain_extension::{
	BufInBufOutState, ChainExtension, ChargedAmount, Environment, Ext, InitState, RetVal,
};
use pop_primitives::storage_keys::RuntimeStateKeys;
use sp_core::crypto::UncheckedFrom;
use sp_runtime::{traits::Dispatchable, DispatchError};
use sp_std::vec::Vec;

use crate::{AccountId, AllowedApiCalls, RuntimeCall, RuntimeOrigin};

const LOG_TARGET: &str = "pop-api::extension";

type ContractSchedule<T> = <T as pallet_contracts::Config>::Schedule;

#[derive(Default)]
pub struct PopApiExtension;

impl<T> ChainExtension<T> for PopApiExtension
where
	T: pallet_contracts::Config
		+ frame_system::Config<
			RuntimeOrigin = RuntimeOrigin,
			AccountId = AccountId,
			RuntimeCall = RuntimeCall,
		>,
	T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
	fn call<E: Ext>(&mut self, env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		E: Ext<T = T>,
	{
		log::debug!(target:LOG_TARGET, " extension called ");
		match v0::FuncId::try_from(env.func_id())? {
			v0::FuncId::Dispatch => {
				match dispatch::<T, E>(env) {
					Ok(()) => Ok(RetVal::Converging(0)),
					Err(DispatchError::Module(error)) => {
						// encode status code = pallet index in runtime + error index, allowing for
						// 999 errors
						Ok(RetVal::Converging(
							(error.index as u32 * 1_000) + u32::from_le_bytes(error.error),
						))
					},
					Err(e) => Err(e),
				}
			},
			v0::FuncId::ReadState => {
				read_state::<T, E>(env)?;
				Ok(RetVal::Converging(0))
			},
		}
	}
}

pub mod v0 {
	#[derive(Debug)]
	pub enum FuncId {
		Dispatch,
		ReadState,
	}
}

impl TryFrom<u16> for v0::FuncId {
	type Error = DispatchError;

	fn try_from(func_id: u16) -> Result<Self, Self::Error> {
		let id = match func_id {
			0x0 => Self::Dispatch,
			0x1 => Self::ReadState,
			_ => {
				log::error!("called an unregistered `func_id`: {:}", func_id);
				return Err(DispatchError::Other("unimplemented func_id"));
			},
		};

		Ok(id)
	}
}

fn dispatch_call<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	call: RuntimeCall,
	mut origin: RuntimeOrigin,
	log_prefix: &str,
) -> Result<(), DispatchError>
where
	E: Ext<T = T>,
{
	let charged_dispatch_weight = env.charge_weight(call.get_dispatch_info().weight)?;

	log::debug!(target:LOG_TARGET, "{} inputted RuntimeCall: {:?}", log_prefix, call);

	origin.add_filter(AllowedApiCalls::contains);

	match call.dispatch(origin) {
		Ok(info) => {
			log::debug!(target:LOG_TARGET, "{} success, actual weight: {:?}", log_prefix, info.actual_weight);

			// refund weight if the actual weight is less than the charged weight
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

fn charge_overhead_weight<T, E>(
	env: &mut Environment<E, BufInBufOutState>,
	len: u32,
	log_prefix: &str,
) -> Result<ChargedAmount, DispatchError>
where
	T: pallet_contracts::Config,
	E: Ext<T = T>,
{
	let contract_host_weight = ContractSchedule::<T>::get().host_fn_weights;

	// calculate weight for reading bytes of `len`
	// reference: https://github.com/paritytech/polkadot-sdk/blob/117a9433dac88d5ac00c058c9b39c511d47749d2/substrate/frame/contracts/src/wasm/runtime.rs#L267
	let base_weight: Weight = contract_host_weight.return_per_byte.saturating_mul(len.into());

	// debug_message weight is a good approximation of the additional overhead of going
	// from contract layer to substrate layer.
	// reference: https://github.com/paritytech/ink-examples/blob/b8d2caa52cf4691e0ddd7c919e4462311deb5ad0/psp22-extension/runtime/psp22-extension-example.rs#L236
	let overhead = contract_host_weight.debug_message;

	let charged_weight = env.charge_weight(base_weight.saturating_add(overhead))?;
	log::debug!(target: LOG_TARGET, "{} charged weight: {:?}", log_prefix, charged_weight);

	Ok(charged_weight)
}

fn dispatch<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_contracts::Config,
	RuntimeOrigin: From<RawOrigin<T::AccountId>>,
	E: Ext<T = T>,
{
	const LOG_PREFIX: &str = " dispatch |";

	let mut env = env.buf_in_buf_out();
	let len = env.in_len();

	charge_overhead_weight::<T, E>(&mut env, len, LOG_PREFIX)?;

	// read the input as RuntimeCall
	let call: RuntimeCall = env.read_as_unbounded(len)?;

	// contract is the origin by default
	let origin: RuntimeOrigin = RawOrigin::Signed(env.ext().address().clone()).into();

	dispatch_call::<T, E>(&mut env, call, origin, LOG_PREFIX)
}

fn read_state<T, E>(env: Environment<E, InitState>) -> Result<(), DispatchError>
where
	T: pallet_contracts::Config,
	E: Ext<T = T>,
{
	const LOG_PREFIX: &str = " read_state |";

	let mut env = env.buf_in_buf_out();

	// To be conservative, we charge the weight for reading the input bytes of a fixed-size type.
	let base_weight: Weight = ContractSchedule::<T>::get()
		.host_fn_weights
		.return_per_byte
		.saturating_mul(env.in_len().into());
	let charged_weight = env.charge_weight(base_weight)?;

	log::debug!(target:LOG_TARGET, "{} charged weight: {:?}", LOG_PREFIX, charged_weight);

	let key: RuntimeStateKeys = env.read_as()?;

	let result = match key {
		_ => Vec::<u8>::default(),
	}
	.encode();

	log::trace!(
		target:LOG_TARGET,
		"{} result: {:?}.", LOG_PREFIX, result
	);
	env.write(&result, false, None).map_err(|e| {
		log::trace!(target: LOG_TARGET, "{:?}", e);
		DispatchError::Other("unable to write results to contract memory")
	})
}

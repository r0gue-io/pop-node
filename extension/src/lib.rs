#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode as _;
use core::marker::PhantomData;
pub use decoding::{Decode, Decodes, DecodingFailed, IdentityProcessor, Processor};
pub use environment::{BufIn, BufOut, Environment, Ext};
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo, RawOrigin},
	ensure,
	traits::{Contains, OriginTrait},
	weights::Weight,
};
pub use functions::{
	Converter, DefaultConverter, DispatchCall, ErrorConverter, Function, ReadState, Readable,
};
pub use matching::{Equals, FunctionId, Matches};
use pallet_contracts::chain_extension::{ChainExtension, InitState, RetVal::Converging};
pub use pallet_contracts::chain_extension::{Result, RetVal, State};
use pallet_contracts::WeightInfo;
use sp_core::Get;
use sp_runtime::{traits::Dispatchable, DispatchError};
use sp_std::vec::Vec;

mod decoding;
mod environment;
pub mod functions;
mod matching;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type ContractWeights<T> = <T as pallet_contracts::Config>::WeightInfo;

/// Encoded version of `pallet_contracts::Error::DecodingFailed`, as found within `DispatchError::ModuleError`.
pub const DECODING_FAILED_ERROR: [u8; 4] = [11, 0, 0, 0];

/// A configurable chain extension.
#[derive(Default)]
pub struct Extension<C: Config>(PhantomData<C>);
impl<Runtime, Config> ChainExtension<Runtime> for Extension<Config>
where
	Runtime: pallet_contracts::Config
		+ frame_system::Config<
			RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
		>,
	Config: self::Config<Functions: Function<Config = Runtime>> + 'static,
{
	/// Call the chain extension logic.
	///
	/// # Parameters
	/// - `env`: Access to the remaining arguments and the execution environment.
	fn call<E: pallet_contracts::chain_extension::Ext<T = Runtime>>(
		&mut self,
		env: pallet_contracts::chain_extension::Environment<E, InitState>,
	) -> Result<RetVal> {
		let mut env = environment::Env(env.buf_in_buf_out());
		self.call(&mut env)
	}
}

impl<
		Runtime: pallet_contracts::Config,
		Config: self::Config<Functions: Function<Config = Runtime>>,
	> Extension<Config>
{
	fn call(
		&mut self,
		env: &mut (impl Environment<Config = Runtime> + BufIn + BufOut),
	) -> Result<RetVal> {
		log::trace!(target: Config::LOG_TARGET, "extension called");
		// Charge weight for making a call from a contract to the runtime.
		// `debug_message` weight is a good approximation of the additional overhead of going from contract layer to substrate layer.
		// reference: https://github.com/paritytech/polkadot-sdk/pull/4233/files#:~:text=DebugMessage(len)%20%3D%3E%20T%3A%3AWeightInfo%3A%3Aseal_debug_message(len)%2C
		let len = env.in_len();
		let overhead = ContractWeights::<Runtime>::seal_debug_message(len);
		let charged = env.charge_weight(overhead)?;
		log::debug!(target: Config::LOG_TARGET, "extension call weight charged: len={len}, weight={overhead}, charged={charged:?}");
		// Execute the function
		Config::Functions::execute(env)
	}
}

/// Trait for configuration of the chain extension.
pub trait Config {
	/// The function(s) available with the chain extension.
	type Functions: Function;

	/// The log target.
	const LOG_TARGET: &'static str;
}

/// Trait to enable specification of a log target.
pub trait LogTarget {
	/// The log target.
	const LOG_TARGET: &'static str;
}

impl LogTarget for () {
	const LOG_TARGET: &'static str = "pop-chain-extension";
}

#[test]
fn default_log_target_works() {
	assert!(matches!(<() as LogTarget>::LOG_TARGET, "pop-chain-extension"));
}

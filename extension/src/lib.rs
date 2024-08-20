#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode as _;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo, RawOrigin},
	ensure,
	traits::{Contains, OriginTrait},
	weights::Weight,
};
pub use functions::{
	Decode, Decodes, DispatchCall, Equals, Function, FunctionId, Matches, Processor, ReadState,
};
use pallet_contracts::chain_extension::{
	ChainExtension, InitState, Result, RetVal, RetVal::Converging,
};
pub use pallet_contracts::chain_extension::{Environment, Ext, State};
use sp_core::Get;
use sp_runtime::{traits::Dispatchable, DispatchError};
use std::marker::PhantomData;

mod decoding;
mod functions;
mod matching;
#[cfg(test)]
mod tests;

type Schedule<T> = <T as pallet_contracts::Config>::Schedule;

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
	fn call<E: Ext<T = Runtime>>(&mut self, env: Environment<E, InitState>) -> Result<RetVal> {
		log::trace!(target: Config::LOG_TARGET, "extension called");
		let mut env = env.buf_in_buf_out();
		// Charge weight for making a call from a contract to the runtime.
		// `debug_message` weight is a good approximation of the additional overhead of going from contract layer to substrate layer.
		// reference: https://github.com/paritytech/ink-examples/blob/b8d2caa52cf4691e0ddd7c919e4462311deb5ad0/psp22-extension/runtime/psp22-extension-example.rs#L236
		env.charge_weight(Schedule::<Runtime>::get().host_fn_weights.debug_message)?;
		// Execute the function
		match Config::Functions::execute(&mut env) {
			Ok(r) => Ok(r),
			Err(e) => Config::Error::convert(e, env),
		}
	}
}

/// Trait for configuration of the chain extension.
pub trait Config {
	/// The function(s) available with the chain extension.
	type Functions: Function;
	/// Optional error conversion.
	type Error: ErrorConverter;

	/// The log target.
	const LOG_TARGET: &'static str;
}

/// Trait to be implemented for type handling a read of runtime state.
pub trait Readable {
	/// Determines the weight of the read, used to charge the appropriate weight before the read is performed.
	fn weight(&self) -> Weight;

	/// Performs the read and returns the result.
	fn read(self) -> Vec<u8>;
}

/// Trait to enable specification of a log target.
pub trait LogTarget {
	/// The log target.
	const LOG_TARGET: &'static str;
}

impl LogTarget for () {
	const LOG_TARGET: &'static str = "pop-chain-extension";
}

/// Trait for error conversion.
pub trait ErrorConverter {
	/// Converts the provided error.
	///
	/// # Parameters
	/// - `error` - The error to be converted.
	/// - `env` - The current execution environment.
	fn convert<E: Ext, S: State>(error: DispatchError, env: Environment<E, S>) -> Result<RetVal>;
}

impl ErrorConverter for () {
	fn convert<E: Ext, S: State>(error: DispatchError, _env: Environment<E, S>) -> Result<RetVal> {
		Err(error)
	}
}

/// Trait for providing an error type.
pub trait ErrorProvider {
	/// An error to return.
	const ERROR: DispatchError;
}

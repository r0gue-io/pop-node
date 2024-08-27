#![cfg_attr(not(feature = "std"), no_std)]

use codec::Decode as _;
use core::{fmt::Debug, marker::PhantomData};
pub use decoding::{Decode, Decodes, IdentityProcessor};
pub use environment::{BufIn, BufOut, Environment, Ext};
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo, RawOrigin},
	ensure,
	traits::{Contains, OriginTrait},
	weights::Weight,
};
pub use functions::{DispatchCall, Function, ReadState};
pub use matching::{Equals, FunctionId, Matches};
use pallet_contracts::chain_extension::{ChainExtension, InitState, RetVal::Converging};
pub use pallet_contracts::chain_extension::{Result, RetVal, State};
use pallet_contracts::WeightInfo;
use sp_core::Get;
use sp_runtime::{traits::Dispatchable, DispatchError};
use sp_std::vec::Vec;

mod decoding;
/// Contains traits related to the execution environment of chain extension.
pub mod environment;
mod functions;
/// Contains traits related to matching functions.
pub mod matching;
#[cfg(test)]
mod tests;

/// Describes the weights of the dispatchables of contract module and is also used to construct a default cost schedule.
pub type ContractWeights<T> = <T as pallet_contracts::Config>::WeightInfo;

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
	/// Call the chain extension logic and executed configured functions.
	///
	/// # Parameters
	/// - `env`: Access to the remaining arguments and the execution environment.
	pub fn call(
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

/// Trait to be implemented for a type handling a read of runtime state.
pub trait Readable {
	/// The corresponding type carrying the result of the runtime state read.
	type Result: Debug;

	/// Determines the weight of the read, used to charge the appropriate weight before the read is performed.
	fn weight(&self) -> Weight;

	/// Performs the read and returns the result.
	fn read(self) -> Self::Result;
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
	/// The log target.
	const LOG_TARGET: &'static str;

	/// Converts the provided error.
	///
	/// # Parameters
	/// - `error` - The error to be converted.
	/// - `env` - The current execution environment.
	fn convert(error: DispatchError, env: &impl Environment) -> Result<RetVal>;
}

impl ErrorConverter for () {
	const LOG_TARGET: &'static str = "pop-chain-extension::converters::error";

	fn convert(error: DispatchError, _env: &impl Environment) -> Result<RetVal> {
		Err(error)
	}
}

/// Error to be returned when decoding fails.
pub struct DecodingFailed<C>(PhantomData<C>);
impl<T: pallet_contracts::Config> Get<DispatchError> for DecodingFailed<T> {
	fn get() -> DispatchError {
		pallet_contracts::Error::<T>::DecodingFailed.into()
	}
}

/// Trait for processing a value based on additional information available from the environment.
pub trait Processor {
	/// The type of value to be processed.
	type Value;

	/// The log target.
	const LOG_TARGET: &'static str;

	/// Processes the provided value.
	///
	/// # Parameters
	/// - `value` - The value to be processed.
	/// - `env` - The current execution environment.
	fn process(value: Self::Value, env: &impl Environment) -> Self::Value;
}

impl Processor for () {
	type Value = ();
	const LOG_TARGET: &'static str = "";
	fn process(value: Self::Value, _env: &impl Environment) -> Self::Value {
		value
	}
}

/// Trait for converting a value based on additional information available from the environment.
pub trait Converter {
	/// The type of value to be converted.
	type Source;
	/// The target type.
	type Target;
	/// The log target.
	const LOG_TARGET: &'static str;

	/// Converts the provided value.
	///
	/// # Parameters
	/// - `value` - The value to be converted.
	/// - `env` - The current execution environment.
	fn convert(value: Self::Source, env: &impl Environment) -> Self::Target;
}

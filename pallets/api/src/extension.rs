use core::{fmt::Debug, marker::PhantomData};

use frame_support::traits::Get;
pub use pop_chain_extension::{
	Config, ContractWeightsOf, DecodingFailed, DispatchCall, ReadState, Readable,
};
use pop_chain_extension::{
	Converter, Decodes, Environment, LogTarget, Matches, Processor, Result, RetVal,
};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

/// Encoded version of `pallet_contracts::Error::DecodingFailed`, as found within
/// `DispatchError::ModuleError`.
pub const DECODING_FAILED_ERROR: [u8; 4] = [11, 0, 0, 0];
/// The logging target for the chain extension.
pub const LOG_TARGET: &str = "pop-api::extension";

/// The chain extension used by the API.
pub type Extension<Functions> = pop_chain_extension::Extension<Functions>;
/// Decodes output by prepending bytes from ext_id() + func_id()
pub type DecodesAs<Output, Weight, Error, Logger = ()> =
	Decodes<Output, Weight, Error, Prepender, Logger>;

/// Prepends bytes from ext_id() + func_id() to prefix the encoded input bytes to determine the
/// versioned output
pub struct Prepender;
impl Processor for Prepender {
	/// The type of value to be processed.
	type Value = Vec<u8>;

	/// The log target.
	const LOG_TARGET: &'static str = "pop-api::extension::processor";

	/// Processes the provided value.
	///
	/// # Parameters
	/// - `value` - The value to be processed.
	/// - `env` - The current execution environment.
	fn process(mut value: Self::Value, env: &impl Environment) -> Self::Value {
		// Resolve version, pallet and call index from environment
		let version = version(env);
		let (module, index) = module_and_index(env);
		// Prepend bytes
		value.splice(0..0, [version, module, index]);
		log::debug!(target: Self::LOG_TARGET, "prepender: version={version}, module={module}, index={index}");
		value
	}
}

/// Matches on the first byte of a function identifier only.
pub struct IdentifiedByFirstByteOfFunctionId<T>(PhantomData<T>);
impl<T: Get<u8>> Matches for IdentifiedByFirstByteOfFunctionId<T> {
	fn matches(env: &impl Environment) -> bool {
		func_id(env) == T::get()
	}
}

/// A log target for dispatched calls.
pub struct DispatchCallLogTarget;
impl LogTarget for DispatchCallLogTarget {
	const LOG_TARGET: &'static str = "pop-api::extension::dispatch";
}

/// A log target for state reads.
pub struct ReadStateLogTarget;
impl LogTarget for ReadStateLogTarget {
	const LOG_TARGET: &'static str = "pop-api::extension::read-state";
}

/// Conversion of a `DispatchError` to a versioned error.
pub struct VersionedErrorConverter<E>(PhantomData<E>);
impl<Error: From<(DispatchError, u8)> + Into<u32> + Debug> pop_chain_extension::ErrorConverter
	for VersionedErrorConverter<Error>
{
	/// The log target.
	const LOG_TARGET: &'static str = "pop-api::extension::converters::versioned-error";

	/// Converts the provided error.
	///
	/// # Parameters
	/// - `error` - The error to be converted.
	/// - `env` - The current execution environment.
	fn convert(error: DispatchError, env: &impl Environment) -> Result<RetVal> {
		// Defer to supplied versioned error conversion type
		let version = version(env);
		log::debug!(target: Self::LOG_TARGET, "versioned error converter: error={error:?}, version={version}");
		let error: Error = (error, version).into();
		log::debug!(target: Self::LOG_TARGET, "versioned error converter: converted error={error:?}");
		Ok(RetVal::Converging(error.into()))
	}
}

/// Conversion of a read result to a versioned read result.
pub struct VersionedResultConverter<S, T>(PhantomData<(S, T)>);
impl<Source: Debug, Target: From<(Source, u8)> + Debug> Converter
	for VersionedResultConverter<Source, Target>
{
	/// The type of value to be converted.
	type Source = Source;
	/// The target type.
	type Target = Target;

	/// The log target.
	const LOG_TARGET: &'static str = "pop-api::extension::converters::versioned-result";

	/// Converts the provided value.
	///
	/// # Parameters
	/// - `value` - The value to be converted.
	/// - `env` - The current execution environment.
	fn convert(value: Self::Source, env: &impl Environment) -> Self::Target {
		// Defer to supplied versioned result conversion type
		let version = version(env);
		log::debug!(target: Self::LOG_TARGET, "versioned result converter: result={value:?}, version={version}");
		let converted: Target = (value, version).into();
		log::debug!(target: Self::LOG_TARGET, "versioned result converter: converted result={converted:?}");
		converted
	}
}

fn func_id(env: &impl Environment) -> u8 {
	env.func_id().to_le_bytes()[0]
}

fn module_and_index(env: &impl Environment) -> (u8, u8) {
	let bytes = env.ext_id().to_le_bytes();
	(bytes[0], bytes[1])
}

fn version(env: &impl Environment) -> u8 {
	env.func_id().to_le_bytes()[1]
}

#[cfg(test)]
mod tests {
	use frame_support::pallet_prelude::Weight;
	use pop_chain_extension::Ext;

	use super::*;
	use crate::extension::Prepender;

	#[test]
	fn prepender_works() {
		let func = 0;
		let version = 1;
		let module = 2;
		let index = 3;
		let env = MockEnvironment {
			func_id: u16::from_le_bytes([func, version]),
			ext_id: u16::from_le_bytes([module, index]),
		};
		let data = 42;
		assert_eq!(Prepender::process(vec![data], &env), vec![version, module, index, data]);
	}

	struct MockEnvironment {
		func_id: u16,
		ext_id: u16,
	}
	impl Environment for MockEnvironment {
		type AccountId = ();
		type ChargedAmount = Weight;

		fn func_id(&self) -> u16 {
			self.func_id
		}

		fn ext_id(&self) -> u16 {
			self.ext_id
		}

		fn charge_weight(&mut self, _amount: Weight) -> Result<Self::ChargedAmount> {
			unimplemented!()
		}

		fn adjust_weight(&mut self, _charged: Self::ChargedAmount, _actual_weight: Weight) {
			unimplemented!()
		}

		fn ext(&mut self) -> impl Ext<AccountId = Self::AccountId> {
			unimplemented!()
		}
	}
}

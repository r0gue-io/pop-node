use core::{fmt::Debug, marker::PhantomData};

use frame_support::traits::Get;
pub use pop_chain_extension_revive::{
	Config, ContractWeightsOf, DecodingFailed, DispatchCall, ErrorConverter, ReadState, Readable,
};
use pop_chain_extension_revive::{
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
pub type Extension<Functions> = pop_chain_extension_revive::Extension<Functions>;
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
impl<Error: TryFrom<(DispatchError, u8), Error = DispatchError> + Into<u32> + Debug> ErrorConverter
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
		let error: Error = (error, version).try_into()?;
		log::debug!(target: Self::LOG_TARGET, "versioned error converter: converted error={error:?}");
		Ok(RetVal::Converging(error.into()))
	}
}

/// Conversion of a read result to a versioned read result.
pub struct VersionedResultConverter<S, T>(PhantomData<(S, T)>);
impl<Source: Debug, Target: TryFrom<(Source, u8), Error = DispatchError> + Debug> Converter
	for VersionedResultConverter<Source, Target>
{
	/// The type returned in the event of a conversion error.
	type Error = DispatchError;
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
	fn try_convert(value: Self::Source, env: &impl Environment) -> Result<Self::Target> {
		// Defer to supplied versioned result conversion type.
		let version = version(env);
		log::debug!(target: Self::LOG_TARGET, "versioned result converter: result={value:?}, version={version}");
		let converted: Target = (value, version).try_into()?;
		log::debug!(target: Self::LOG_TARGET, "versioned result converter: converted result={converted:?}");
		Ok(converted)
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
	use pop_chain_extension_revive::Ext;
	use sp_core::ConstU8;

	use super::{DispatchError::*, *};
	use crate::extension::Prepender;

	#[test]
	fn func_id_works() {
		let env = MockEnvironment { func_id: u16::from_le_bytes([1, 2]), ext_id: 0u16 };
		assert_eq!(func_id(&env), 1);
	}

	#[test]
	fn module_and_index_works() {
		let env = MockEnvironment { func_id: 0u16, ext_id: u16::from_le_bytes([2, 3]) };
		assert_eq!(module_and_index(&env), (2, 3));
	}

	#[test]
	fn version_works() {
		let env = MockEnvironment { func_id: u16::from_le_bytes([1, 2]), ext_id: 0u16 };
		assert_eq!(version(&env), 2);
	}

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

	#[test]
	fn identified_by_first_byte_of_function_id_matches() {
		let env = MockEnvironment { func_id: u16::from_le_bytes([1, 2]), ext_id: 0u16 };
		assert!(IdentifiedByFirstByteOfFunctionId::<ConstU8<1>>::matches(&env));
	}

	#[test]
	fn identified_by_first_byte_of_function_id_does_not_match() {
		let env = MockEnvironment { func_id: u16::from_le_bytes([1, 2]), ext_id: 0u16 };
		assert!(!IdentifiedByFirstByteOfFunctionId::<ConstU8<2>>::matches(&env));
	}

	#[test]
	fn dispatch_call_log_target_works() {
		assert!(matches!(
			<DispatchCallLogTarget as LogTarget>::LOG_TARGET,
			"pop-api::extension::dispatch"
		));
	}

	#[test]
	fn read_state_log_target_works() {
		assert!(matches!(
			<ReadStateLogTarget as LogTarget>::LOG_TARGET,
			"pop-api::extension::read-state"
		));
	}

	mod versioned_error {
		use super::{RetVal::Converging, *};

		// Mock versioned error.
		#[derive(Debug)]
		pub enum VersionedError {
			V0(DispatchError),
			V1(DispatchError),
		}

		impl TryFrom<(DispatchError, u8)> for VersionedError {
			type Error = DispatchError;

			fn try_from(value: (DispatchError, u8)) -> Result<Self> {
				let (error, version) = value;
				match version {
					0 => Ok(VersionedError::V0(error)),
					1 => Ok(VersionedError::V1(error)),
					_ => Err(Other("DecodingFailed")),
				}
			}
		}

		impl From<VersionedError> for u32 {
			// Mock conversion based on error and version.
			fn from(value: VersionedError) -> Self {
				match value {
					VersionedError::V0(error) => match error {
						BadOrigin => 1,
						_ => 100,
					},
					VersionedError::V1(error) => match error {
						BadOrigin => 2,
						_ => 200,
					},
				}
			}
		}

		#[test]
		fn versioned_error_converter_works() {
			for (version, error, expected_result) in vec![
				(0, BadOrigin, 1),
				(0, Other("test"), 100),
				(1, BadOrigin, 2),
				(1, Other("test"), 200),
			] {
				let env =
					MockEnvironment { func_id: u16::from_le_bytes([0, version]), ext_id: 0u16 };
				// Because `Retval` does not implement the `Debug` trait the result has to be
				// unwrapped.
				let Ok(Converging(result)) =
					VersionedErrorConverter::<VersionedError>::convert(error, &env)
				else {
					panic!("should not happen")
				};
				assert_eq!(result, expected_result);
			}
		}

		#[test]
		fn versioned_error_converter_fails_when_invalid_version() {
			let version = 2;
			let env = MockEnvironment { func_id: u16::from_le_bytes([0, version]), ext_id: 0u16 };
			let result = VersionedErrorConverter::<VersionedError>::convert(BadOrigin, &env).err();
			assert_eq!(result, Some(Other("DecodingFailed")));
		}
	}

	mod versioned_result {
		use VersionedRuntimeResult::{V0, V1};

		use super::*;

		// Mock versioned runtime result.
		#[derive(Debug, PartialEq)]
		pub enum VersionedRuntimeResult {
			V0(u8),
			V1(u8),
		}

		impl TryFrom<(u8, u8)> for VersionedRuntimeResult {
			type Error = DispatchError;

			// Mock conversion based on result and version.
			fn try_from(value: (u8, u8)) -> Result<Self> {
				let (result, version) = value;
				// Per version there is a specific upper bound allowed.
				match version {
					0 if result <= 50 => Ok(V0(result)),
					0 if result > 50 => Ok(V0(50)),
					1 if result <= 100 => Ok(V1(result)),
					1 if result > 100 => Ok(V1(100)),
					_ => Err(Other("DecodingFailed")),
				}
			}
		}

		#[test]
		fn versioned_result_converter_works() {
			for (version, value, expected_result) in vec![
				(0, 10, Ok(V0(10))),
				// `V0` has 50 as upper bound and therefore caps the value.
				(0, 100, Ok(V0(50))),
				(1, 10, Ok(V1(10))),
				// Different upper bound for `V1`.
				(1, 100, Ok(V1(100))),
			] {
				let env =
					MockEnvironment { func_id: u16::from_le_bytes([0, version]), ext_id: 0u16 };
				let result = VersionedResultConverter::<u8, VersionedRuntimeResult>::try_convert(
					value, &env,
				);
				assert_eq!(result, expected_result);
			}
		}

		#[test]
		fn versioned_result_converter_fails_when_invalid_version() {
			let env = MockEnvironment { func_id: u16::from_le_bytes([0, 2]), ext_id: 0u16 };
			let result =
				VersionedResultConverter::<u8, VersionedRuntimeResult>::try_convert(10, &env).err();
			assert_eq!(result, Some(Other("DecodingFailed")));
		}
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

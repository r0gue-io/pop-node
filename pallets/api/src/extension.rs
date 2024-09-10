use core::{fmt::Debug, marker::PhantomData};

use frame_support::traits::Get;
pub use pop_chain_extension::{
	Config, ContractWeightsOf, DecodingFailed, DispatchCall, ErrorConverter, ReadState, Readable,
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
impl<Error: From<(DispatchError, u8)> + Into<u32> + Debug> ErrorConverter
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
	use sp_core::ConstU8;

	use super::{DispatchError::*, *};
	use crate::extension::Prepender;

	#[test]
	fn func_id_works() {
		let env = MockEnvironment { func_id: u16::from_le_bytes([1, 2]), ext_id: 0u16 };
		assert_eq!(func_id(&env), 1);
		let env = MockEnvironment { func_id: u16::from_le_bytes([2, 1]), ext_id: 0u16 };
		assert_eq!(func_id(&env), 2);
	}

	#[test]
	fn module_and_index_works() {
		let env = MockEnvironment { func_id: 0u16, ext_id: u16::from_le_bytes([2, 3]) };
		assert_eq!(module_and_index(&env), (2, 3));
		let env = MockEnvironment { func_id: 0u16, ext_id: u16::from_le_bytes([3, 2]) };
		assert_eq!(module_and_index(&env), (3, 2));
	}

	#[test]
	fn version_works() {
		let env = MockEnvironment { func_id: u16::from_le_bytes([1, 2]), ext_id: 0u16 };
		assert_eq!(version(&env), 2);
		let env = MockEnvironment { func_id: u16::from_le_bytes([2, 1]), ext_id: 0u16 };
		assert_eq!(version(&env), 1);
	}

	#[test]
	fn prepender_works() {
		let env = MockEnvironment {
			func_id: u16::from_le_bytes([1, 2]),
			ext_id: u16::from_le_bytes([3, 4]),
		};
		assert_eq!(Prepender::process(vec![0u8], &env), vec![2, 3, 4, 0]);
		assert_eq!(Prepender::process(vec![0u8, 5, 10], &env), vec![2, 3, 4, 0, 5, 10]);

		let env = MockEnvironment {
			func_id: u16::from_le_bytes([2, 1]),
			ext_id: u16::from_le_bytes([4, 3]),
		};
		assert_eq!(Prepender::process(vec![0u8], &env), vec![1, 4, 3, 0]);
		assert_eq!(Prepender::process(vec![0u8, 5, 10], &env), vec![1, 4, 3, 0, 5, 10]);
	}

	#[test]
	fn identified_by_first_byte_of_function_id_matches() {
		let env = MockEnvironment { func_id: u16::from_le_bytes([1, 2]), ext_id: 0u16 };
		assert!(IdentifiedByFirstByteOfFunctionId::<ConstU8<1>>::matches(&env));
		let env = MockEnvironment { func_id: u16::from_le_bytes([2, 1]), ext_id: 0u16 };
		assert!(IdentifiedByFirstByteOfFunctionId::<ConstU8<2>>::matches(&env));
	}

	#[test]
	fn identified_by_first_byte_of_function_id_does_not_match() {
		let env = MockEnvironment { func_id: u16::from_le_bytes([1, 2]), ext_id: 0u16 };
		assert!(!IdentifiedByFirstByteOfFunctionId::<ConstU8<2>>::matches(&env));
		let env = MockEnvironment { func_id: u16::from_le_bytes([2, 1]), ext_id: 0u16 };
		assert!(!IdentifiedByFirstByteOfFunctionId::<ConstU8<1>>::matches(&env));
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

	#[test]
	fn versioned_error_converter_works() {
		// Mock versioned error.
		#[derive(Debug)]
		pub enum VersionedError {
			V0(DispatchError),
			V1(DispatchError),
		}

		impl From<(DispatchError, u8)> for VersionedError {
			fn from(value: (DispatchError, u8)) -> Self {
				let (error, version) = value;
				match version {
					0 => VersionedError::V0(error),
					1 => VersionedError::V1(error),
					_ => unimplemented!(),
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

		for (version, error, expected_result) in vec![
			(0, BadOrigin, 1),
			(0, CannotLookup, 100),
			(1, BadOrigin, 2),
			(1, CannotLookup, 200),
		] {
			let env = MockEnvironment { func_id: u16::from_le_bytes([0, version]), ext_id: 0u16 };
			let RetVal::Converging(result) =
				VersionedErrorConverter::<VersionedError>::convert(error, &env)
					.expect("should always result `Ok`")
			else {
				unimplemented!();
			};
			assert_eq!(result, expected_result);
		}
	}

	#[test]
	fn versioned_result_converter_works() {
		// Mock versioned runtime result.
		#[derive(Debug, PartialEq)]
		pub enum VersionedRuntimeResult {
			V0(u8),
			V1(u8),
		}

		impl From<(u8, u8)> for VersionedRuntimeResult {
			// Mock conversion based on result and version.
			fn from(value: (u8, u8)) -> Self {
				let (result, version) = value;
				match version {
					0 if result <= 50 => VersionedRuntimeResult::V0(result),
					0 if result > 50 => VersionedRuntimeResult::V0(50),
					1 if result <= 100 => VersionedRuntimeResult::V1(result),
					1 if result > 100 => VersionedRuntimeResult::V1(100),
					_ => unimplemented!(),
				}
			}
		}

		for (version, value, expected_result) in vec![
			(0, 10, VersionedRuntimeResult::V0(10)),
			(0, 100, VersionedRuntimeResult::V0(50)),
			(1, 10, VersionedRuntimeResult::V1(10)),
			(1, 100, VersionedRuntimeResult::V1(100)),
		] {
			let env = MockEnvironment { func_id: u16::from_le_bytes([0, version]), ext_id: 0u16 };
			let result =
				VersionedResultConverter::<u8, VersionedRuntimeResult>::convert(value, &env);
			assert_eq!(result, expected_result);
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

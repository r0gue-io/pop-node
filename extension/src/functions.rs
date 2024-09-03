use super::*;
use core::fmt::Debug;

/// A chain extension function.
pub trait Function {
	/// The configuration of the contracts module.
	type Config: pallet_contracts::Config;
	/// Optional error conversion.
	type Error: ErrorConverter;

	/// Executes the function.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn execute(
		env: &mut (impl Environment<Config = Self::Config> + BufIn + BufOut),
	) -> Result<RetVal>;
}

/// A function for dispatching a runtime call.
pub struct DispatchCall<M, C, D, F, E = (), L = ()>(PhantomData<(M, C, D, F, E, L)>);
impl<
		Matcher: Matches,
		Config: pallet_contracts::Config
			+ frame_system::Config<
				RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
			>,
		Decoder: Decode<Output: codec::Decode + Into<<Config as frame_system::Config>::RuntimeCall>>,
		Filter: Contains<<Config as frame_system::Config>::RuntimeCall> + 'static,
		Error: ErrorConverter,
		Logger: LogTarget,
	> Function for DispatchCall<Matcher, Config, Decoder, Filter, Error, Logger>
{
	/// The configuration of the contracts module.
	type Config = Config;
	/// Optional error conversion.
	type Error = Error;

	/// Executes the function.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn execute(env: &mut (impl Environment<Config = Config> + BufIn)) -> Result<RetVal> {
		// Decode runtime call.
		// TODO: should the error returned from decoding failure be converted into a versioned error, or always be pallet_contracts::Error::DecodingFailed?
		let call = Decoder::decode(env)?.into();
		log::debug!(target: Logger::LOG_TARGET, "decoded: call={call:?}");
		// Charge weight before dispatch.
		let dispatch_info = call.get_dispatch_info();
		log::debug!(target: Logger::LOG_TARGET, "pre-dispatch info: dispatch_info={dispatch_info:?}");
		let charged = env.charge_weight(dispatch_info.weight)?;
		log::debug!(target: Logger::LOG_TARGET, "pre-dispatch weight charged: charged={charged:?}");
		// Contract is the origin by default.
		let origin = RawOrigin::Signed(env.ext().address().clone());
		log::debug!(target: Logger::LOG_TARGET, "contract origin: origin={origin:?}");
		let mut origin: Config::RuntimeOrigin = origin.into();
		// Ensure call allowed.
		origin.add_filter(Filter::contains);
		// Dispatch call.
		let result = call.dispatch(origin);
		log::debug!(target: Logger::LOG_TARGET, "dispatched: result={result:?}");
		// Adjust weight.
		let weight = frame_support::dispatch::extract_actual_weight(&result, &dispatch_info);
		env.adjust_weight(charged, weight);
		log::debug!(target: Logger::LOG_TARGET, "weight adjusted: weight={weight:?}");
		match result {
			Ok(_) => Ok(Converging(0)),
			Err(e) => Error::convert(e.error, env),
		}
	}
}

impl<M: Matches, C, D, F, E, L> Matches for DispatchCall<M, C, D, F, E, L> {
	fn matches(env: &impl Environment) -> bool {
		M::matches(env)
	}
}

/// A function for reading runtime state.
pub struct ReadState<M, C, R, D, F, RC = DefaultConverter<<R as Readable>::Result>, E = (), L = ()>(
	PhantomData<(M, C, R, D, F, RC, E, L)>,
);
impl<
		Matcher: Matches,
		Config: pallet_contracts::Config,
		Read: Readable + Debug,
		Decoder: Decode<Output: codec::Decode + Into<Read>>,
		Filter: Contains<Read>,
		ResultConverter: Converter<Source = Read::Result, Target: Into<Vec<u8>>>,
		Error: ErrorConverter,
		Logger: LogTarget,
	> Function for ReadState<Matcher, Config, Read, Decoder, Filter, ResultConverter, Error, Logger>
{
	/// The configuration of the contracts module.
	type Config = Config;
	/// Optional error conversion.
	type Error = Error;

	/// Executes the function.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn execute(env: &mut (impl Environment + BufIn + BufOut)) -> Result<RetVal> {
		// Decode runtime state read
		// TODO: should the error returned from decoding failure be converted into a versioned error, or always be pallet_contracts::Error::DecodingFailed?
		let read = Decoder::decode(env)?.into();
		log::debug!(target: Logger::LOG_TARGET, "decoded: read={read:?}");
		// Charge weight before read
		let weight = read.weight();
		let charged = env.charge_weight(weight)?;
		log::trace!(target: Logger::LOG_TARGET, "pre-read weight charged: weight={weight}, charged={charged:?}");
		// Ensure read allowed
		ensure!(Filter::contains(&read), frame_system::Error::<Config>::CallFiltered);
		let result = read.read();
		log::debug!(target: Logger::LOG_TARGET, "read: result={result:?}");
		// Perform any final conversion. Any implementation is expected to charge weight as appropriate.
		let result = ResultConverter::convert(result, env).into();
		log::debug!(target: Logger::LOG_TARGET, "converted: result={result:?}");
		// Charge weight before read
		let weight = ContractWeights::<Config>::seal_input_per_byte(1); // use unit weight as write function handles multiplication
		log::trace!(target: Logger::LOG_TARGET, "return result to contract: weight_per_byte={weight}");
		// Charge appropriate weight for writing to contract, based on input length, prior to decoding.
		// TODO: check parameters (allow_skip, weight_per_byte)
		// TODO: confirm whether potential error from writing to the buffer needs to be converted to a versioned error (suspect not)
		env.write(&result, false, Some(weight))?;
		Ok(Converging(0))
	}
}

impl<M: Matches, C, R, D, F, RC, E, L> Matches for ReadState<M, C, R, D, F, RC, E, L> {
	fn matches(env: &impl Environment) -> bool {
		M::matches(env)
	}
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

/// A default converter, for converting (encoding) from some type into a byte array.
pub struct DefaultConverter<T>(PhantomData<T>);
impl<T: Into<Vec<u8>>> Converter for DefaultConverter<T> {
	type Source = T;
	type Target = Vec<u8>;
	const LOG_TARGET: &'static str = "";

	fn convert(value: Self::Source, _env: &impl Environment) -> Self::Target {
		value.into()
	}
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

// Support tuples of at least one function (required for type resolution) and a maximum of ten.
#[impl_trait_for_tuples::impl_for_tuples(1, 10)]
#[tuple_types_custom_trait_bound(Function + Matches)]
impl<Runtime: pallet_contracts::Config> Function for Tuple {
	for_tuples!( where #( Tuple: Function<Config=Runtime> )* );
	type Config = Runtime;
	type Error = ();

	fn execute(
		env: &mut (impl Environment<Config = Self::Config> + BufIn + BufOut),
	) -> Result<RetVal> {
		// Attempts to match a specified extension/function identifier to its corresponding function, as configured by the runtime.
		for_tuples!( #(
            if Tuple::matches(env) {
                return Tuple::execute(env)
            }
        )* );

		// Otherwise returns error indicating an unmatched request.
		log::error!("no function could be matched");
		Err(pallet_contracts::Error::<Self::Config>::DecodingFailed.into())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{charge_weight_filtering_read_state, read_from_buffer_weight};
	use crate::{
		mock::INVALID_FUNC_ID,
		tests::{
			charge_weight_filtering_dispatch_call, function_dispatch_call_weight,
			function_read_state_weight,
		},
	};
	use codec::Encode;
	use frame_support::traits::{Everything, Nothing};
	use frame_system::Call;
	use mock::{
		new_test_ext, DispatchExtFuncId, DispatchExtNoopFuncId, Functions, MockEnvironment,
		MockExt, ReadExtFuncId, ReadExtNoopFuncId, RuntimeCall, RuntimeRead, RuntimeResult, Test,
	};

	fn environment(id: u32, buffer: Vec<u8>) -> MockEnvironment<MockExt> {
		MockEnvironment::new(id, buffer, MockExt::default())
	}

	fn test_environments(
		id: u32,
		buffer: Vec<u8>,
	) -> (MockEnvironment<MockExt>, MockEnvironment<MockExt>) {
		// Initialize environment with no function execution.
		let noop_env = MockEnvironment::default();
		// Initialize environment with function execution.
		let env = environment(id, buffer);
		assert_eq!(env.charged(), noop_env.charged());
		(env, noop_env)
	}

	enum AtLeastOneByte {}
	impl Contains<Vec<u8>> for AtLeastOneByte {
		fn contains(input: &Vec<u8>) -> bool {
			input.len() > 0
		}
	}

	enum LargerThan100 {}
	impl Contains<u8> for LargerThan100 {
		fn contains(input: &u8) -> bool {
			*input > 100
		}
	}

	enum MustBeEven {}
	impl Contains<u8> for MustBeEven {
		fn contains(input: &u8) -> bool {
			*input % 2 == 0
		}
	}

	#[test]
	fn filtering_works() {
		fn contains<C: Contains<T>, T>(input: T, expected: bool) {
			assert_eq!(C::contains(&input), expected);
		}
		contains::<Everything, u32>(42, true);
		contains::<Everything, Vec<u8>>(vec![1, 2, 3, 4], true);
		contains::<Nothing, u32>(42, false);
		contains::<Nothing, Vec<u8>>(vec![1, 2, 3, 4], false);
		contains::<AtLeastOneByte, Vec<u8>>(vec![], false);
		contains::<AtLeastOneByte, Vec<u8>>(vec![1], true);
		contains::<AtLeastOneByte, Vec<u8>>(vec![1, 2, 3, 4], true);
		contains::<LargerThan100, u8>(100, false);
		contains::<LargerThan100, u8>(101, true);
		contains::<MustBeEven, u8>(100, true);
		contains::<MustBeEven, u8>(101, false);
	}

	mod dispatch_call_tests {
		use super::*;

		#[test]
		fn filtering_dispatch_call_noop_function_fails() {
			new_test_ext().execute_with(|| {
				let call = RuntimeCall::System(Call::remark_with_event {
					remark: "pop".as_bytes().to_vec(),
				});
				let mut env = environment(DispatchExtNoopFuncId::get(), call.encode());
				let error = frame_system::Error::<Test>::CallFiltered.into();
				let expected = <() as ErrorConverter>::convert(error, &mut env).err();
				assert_eq!(Functions::execute(&mut env).err(), expected);
			})
		}

		#[test]
		fn filtering_dispatch_call_noop_function_charge_weights() {
			new_test_ext().execute_with(|| {
				let call = RuntimeCall::System(Call::remark_with_event {
					remark: "pop".as_bytes().to_vec(),
				});
				let (mut env, mut noop_env) =
					test_environments(DispatchExtNoopFuncId::get(), call.encode());
				assert!(Functions::execute(&mut env).is_err());
				// Check that the two environments charged the same weights.
				charge_weight_filtering_dispatch_call(
					&mut noop_env,
					call.encode().len() as u32,
					call,
					env.ext().address().clone(),
				);
				assert_eq!(env.charged(), noop_env.charged());
			})
		}

		#[test]
		fn execute_dispatch_call_function_works() {
			new_test_ext().execute_with(|| {
				let call = RuntimeCall::System(Call::remark_with_event {
					remark: "pop".as_bytes().to_vec(),
				});
				let mut env = environment(DispatchExtFuncId::get(), call.encode());
				assert!(matches!(Functions::execute(&mut env), Ok(Converging(0))));
			})
		}

		#[test]
		fn execute_dispatch_call_function_charge_weights() {
			new_test_ext().execute_with(|| {
				let call = RuntimeCall::System(Call::remark_with_event {
					remark: "pop".as_bytes().to_vec(),
				});
				let encoded_call = call.encode();
				let (mut env, mut noop_env) =
					test_environments(DispatchExtFuncId::get(), encoded_call.clone());
				assert!(Functions::execute(&mut env).is_ok());
				// Check that the two environments charged the same weights.
				assert_eq!(
					env.charged(),
					function_dispatch_call_weight(
						&mut noop_env,
						encoded_call.len() as u32,
						call,
						env.ext().address().clone()
					)
				);
			})
		}

		#[test]
		fn execute_dispatch_call_function_invalid_input_fails() {
			new_test_ext().execute_with(|| {
				// Invalid encoded runtime call.
				let input = vec![0, 99];
				let mut env = environment(DispatchExtFuncId::get(), input.clone());
				let error = pallet_contracts::Error::<Test>::DecodingFailed.into();
				let expected = <() as ErrorConverter>::convert(error, &mut env).err();
				assert_eq!(Functions::execute(&mut env).err(), expected);
			})
		}

		#[test]
		fn execute_dispatch_call_function_invalid_input_charge_weights() {
			new_test_ext().execute_with(|| {
				// Invalid encoded runtime call.
				let input = vec![0, 99];
				let mut env = environment(DispatchExtFuncId::get(), input.clone());
				assert!(Functions::execute(&mut env).is_err());
				assert_eq!(env.charged(), read_from_buffer_weight(input.len() as u32,));
			})
		}
	}

	mod read_state_tests {
		use super::*;

		#[test]
		fn filtering_read_state_noop_function_fails() {
			new_test_ext().execute_with(|| {
				let read = RuntimeRead::Ping;
				let mut env = environment(ReadExtNoopFuncId::get(), read.encode());
				let error = frame_system::Error::<Test>::CallFiltered.into();
				let expected = <() as ErrorConverter>::convert(error, &mut env).err();
				assert_eq!(Functions::execute(&mut env).err(), expected);
			})
		}

		#[test]
		fn filtering_read_state_noop_function_charge_weights() {
			new_test_ext().execute_with(|| {
				let read = RuntimeRead::Ping;
				let (mut env, mut noop_env) =
					test_environments(ReadExtNoopFuncId::get(), read.encode());
				assert!(Functions::execute(&mut env).is_err());
				// Check that the two environments charged the same weights.
				charge_weight_filtering_read_state(&mut noop_env, read.encode().len() as u32, read);
				assert_eq!(env.charged(), noop_env.charged());
			})
		}

		#[test]
		fn execute_read_state_function_works() {
			let read = RuntimeRead::Ping;
			let expected = "pop".as_bytes().encode();
			let mut env = environment(ReadExtFuncId::get(), read.encode());
			assert!(matches!(Functions::execute(&mut env), Ok(Converging(0))));
			// Check if the contract environment buffer is written correctly.
			assert_eq!(env.buffer, expected);
		}

		#[test]
		fn execute_read_state_function_charge_weights() {
			let read = RuntimeRead::Ping;
			let encoded_read = read.encode();
			let read_result = RuntimeResult::Pong("pop".to_string());
			let (mut env, mut noop_env) =
				test_environments(ReadExtFuncId::get(), encoded_read.clone());
			assert!(Functions::execute(&mut env).is_ok());
			// Check that the two environments charged the same weights.
			assert_eq!(
				env.charged(),
				function_read_state_weight(
					&mut noop_env,
					encoded_read.len() as u32,
					read,
					read_result,
				)
			);
		}

		#[test]
		fn execute_read_state_function_invalid_input_fails() {
			// Invalid encoded runtime state read.
			let input = vec![0];
			let mut env = environment(ReadExtFuncId::get(), input.clone());
			let error = pallet_contracts::Error::<Test>::DecodingFailed.into();
			let expected = <() as ErrorConverter>::convert(error, &mut env).err();
			assert_eq!(Functions::execute(&mut env).err(), expected);
		}

		#[test]
		fn execute_read_state_function_invalid_input_charge_weights() {
			// Invalid encoded runtime state read.
			let input = vec![0];
			let mut env = environment(ReadExtFuncId::get(), input.clone());
			assert!(Functions::execute(&mut env).is_err());
			assert_eq!(env.charged(), read_from_buffer_weight(input.len() as u32));
		}
	}

	#[test]
	fn execute_invalid_function_fails() {
		let input = vec![];
		let mut env = environment(INVALID_FUNC_ID, input.clone());
		let error = pallet_contracts::Error::<Test>::DecodingFailed.into();
		let expected = <() as ErrorConverter>::convert(error, &mut env).err();
		assert_eq!(Functions::execute(&mut env).err(), expected);
	}

	#[test]
	fn execute_invalid_function_no_charge_weights() {
		let input = vec![];
		let mut env = environment(INVALID_FUNC_ID, input.clone());
		assert!(Functions::execute(&mut env).is_err());
		// No weight charged as execution begins after the overhead weight charge and immediately errors.
		assert_eq!(env.charged(), Weight::default());
	}

	#[test]
	fn default_error_conversion_works() {
		let env = MockEnvironment::new(0, vec![42], MockExt::default());
		assert!(matches!(
			<() as ErrorConverter>::convert(DispatchError::BadOrigin, &env),
			Err(DispatchError::BadOrigin)
		));
	}

	#[test]
	fn default_conversion_works() {
		let env = MockEnvironment::default();
		let source = "pop".to_string();
		assert_eq!(DefaultConverter::<String>::convert(source.clone(), &env), source.as_bytes());
	}
}

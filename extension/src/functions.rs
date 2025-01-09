use core::fmt::Debug;

use super::*;

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
		env: &mut (impl Environment<AccountId = AccountIdOf<Self::Config>> + BufIn + BufOut),
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
		Decoder: Decode<Output: codec::Decode + Into<RuntimeCallOf<Config>>>,
		Filter: Contains<RuntimeCallOf<Config>> + 'static,
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
	fn execute(
		env: &mut (impl Environment<AccountId = Config::AccountId> + BufIn),
	) -> Result<RetVal> {
		// Decode runtime call.
		let call = Decoder::decode(env)?.into();
		log::debug!(target: Logger::LOG_TARGET, "decoded: call={call:?}");
		// Charge weight before dispatch.
		let dispatch_info = call.get_dispatch_info();
		log::debug!(target: Logger::LOG_TARGET, "pre-dispatch info: dispatch_info={dispatch_info:?}");
		let charged = env.charge_weight(dispatch_info.call_weight)?;
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
		ResultConverter: Converter<Source = Read::Result, Target: Into<Vec<u8>>, Error = DispatchError>,
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
		// Perform any final conversion. Any implementation is expected to charge weight as
		// appropriate.
		let result = ResultConverter::try_convert(result, env)?.into();
		log::debug!(target: Logger::LOG_TARGET, "converted: result={result:?}");
		// Charge appropriate weight for writing to contract, based on result length.
		let weight = ContractWeightsOf::<Config>::seal_input(result.len() as u32);
		let charged = env.charge_weight(weight)?;
		log::trace!(target: Logger::LOG_TARGET, "return result to contract: weight={weight}, charged={charged:?}");
		env.write(&result, false, None)?; // weight charged above
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

	/// Determines the weight of the read, used to charge the appropriate weight before the read is
	/// performed.
	fn weight(&self) -> Weight;

	/// Performs the read and returns the result.
	fn read(self) -> Self::Result;
}

/// Trait for fallible conversion of a value based on additional information available from the
/// environment.
pub trait Converter {
	/// The error type returned when conversion fails.
	type Error;
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
	fn try_convert(
		value: Self::Source,
		env: &impl Environment,
	) -> core::result::Result<Self::Target, Self::Error>;
}

/// A default converter, for converting (encoding) from some type into a byte array.
pub struct DefaultConverter<T>(PhantomData<T>);
impl<T: Into<Vec<u8>>> Converter for DefaultConverter<T> {
	/// The error type returned when conversion fails.
	type Error = DispatchError;
	/// The type of value to be converted.
	type Source = T;
	/// The target type.
	type Target = Vec<u8>;

	const LOG_TARGET: &'static str = "";

	fn try_convert(
		value: Self::Source,
		_env: &impl Environment,
	) -> core::result::Result<Self::Target, Self::Error> {
		Ok(value.into())
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
	type Config = Runtime;
	type Error = ();

	for_tuples!( where #( Tuple: Function<Config=Runtime> )* );

	fn execute(
		env: &mut (impl Environment<AccountId = AccountIdOf<Self::Config>> + BufIn + BufOut),
	) -> Result<RetVal> {
		// Attempts to match a specified extension/function identifier to its corresponding
		// function, as configured by the runtime.
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
	use codec::Encode;
	use frame_support::traits::{Everything, Nothing};
	use frame_system::Call;
	use mock::{new_test_ext, Functions, MockEnvironment, RuntimeCall, RuntimeRead, Test};
	use sp_core::ConstU32;

	use super::*;
	use crate::{
		extension::{read_from_buffer_weight, write_to_contract_weight},
		matching::WithFuncId,
		mock::{Noop, NoopFuncId, INVALID_FUNC_ID},
	};

	type FuncId = ConstU32<42>;

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
	fn contains_works() {
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

	mod dispatch_call {
		use super::*;

		type DispatchCall = DispatchCallWithFilter<Everything>;
		type DispatchCallWithFilter<Filter> = super::DispatchCall<
			WithFuncId<FuncId>,
			Test,
			Decodes<RuntimeCall, ContractWeightsOf<Test>, DecodingFailed<Test>>,
			Filter,
		>;

		#[test]
		fn dispatch_call_filtering_works() {
			let call =
				RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() });
			let mut env = MockEnvironment::new(FuncId::get(), call.encode());
			let error = frame_system::Error::<Test>::CallFiltered.into();
			let expected = <() as ErrorConverter>::convert(error, &mut env).err();
			assert_eq!(DispatchCallWithFilter::<Nothing>::execute(&mut env).err(), expected);
		}

		#[test]
		fn dispatch_call_filtered_charges_weight() {
			let call =
				RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() });
			let encoded_call = call.encode();
			let mut env = MockEnvironment::new(FuncId::get(), encoded_call.clone());
			assert!(DispatchCallWithFilter::<Nothing>::execute(&mut env).is_err());
			assert_eq!(
				env.charged(),
				read_from_buffer_weight(encoded_call.len() as u32)
					+ call.get_dispatch_info().call_weight
			);
		}

		#[test]
		fn dispatch_call_works() {
			new_test_ext().execute_with(|| {
				let call = RuntimeCall::System(Call::remark_with_event {
					remark: "pop".as_bytes().to_vec(),
				});
				let mut env = MockEnvironment::new(FuncId::get(), call.encode());
				assert!(matches!(DispatchCall::execute(&mut env), Ok(Converging(0))));
			})
		}

		#[test]
		fn dispatch_call_returns_error() {
			new_test_ext().execute_with(|| {
				let call = RuntimeCall::System(Call::set_code { code: "pop".as_bytes().to_vec() });
				let mut env = MockEnvironment::new(FuncId::get(), call.encode());
				let error = DispatchCall::execute(&mut env).err();
				let expected =
					<() as ErrorConverter>::convert(DispatchError::BadOrigin, &env).err();
				assert_eq!(error, expected);
			})
		}

		#[test]
		fn dispatch_call_charges_weight() {
			new_test_ext().execute_with(|| {
				let call = RuntimeCall::System(Call::remark_with_event {
					remark: "pop".as_bytes().to_vec(),
				});
				let encoded_call = call.encode();
				let mut env = MockEnvironment::new(FuncId::get(), encoded_call.clone());
				assert!(DispatchCall::execute(&mut env).is_ok());
				assert_eq!(
					env.charged(),
					read_from_buffer_weight(encoded_call.len() as u32)
						+ call.get_dispatch_info().call_weight
				);
			})
		}

		#[test]
		fn dispatch_call_adjusts_weight() {
			let migrate_weight = <Test as pallet_contracts::Config>::WeightInfo::migrate();
			let migration_noop_weight =
				<Test as pallet_contracts::Config>::WeightInfo::migration_noop();
			new_test_ext().execute_with(|| {
				// Attempt to perform non-existent migration with additional weight limit specified.
				let extra_weight = Weight::from_parts(123456789, 12345);
				let weight_limit = migration_noop_weight.saturating_add(extra_weight);
				let call = RuntimeCall::Contracts(pallet_contracts::Call::migrate { weight_limit });
				let encoded_call = call.encode();
				let mut env = MockEnvironment::new(FuncId::get(), encoded_call.clone());
				let expected: DispatchError =
					pallet_contracts::Error::<Test>::NoMigrationPerformed.into();
				assert_eq!(DispatchCall::execute(&mut env).err().unwrap(), expected);
				// Ensure pre-dispatch weight is weight function + weight limit
				assert_eq!(call.get_dispatch_info().call_weight, migrate_weight + weight_limit);
				assert_eq!(
					env.charged(),
					read_from_buffer_weight(encoded_call.len() as u32)
						+ call.get_dispatch_info().call_weight
						- extra_weight
				);
			})
		}

		#[test]
		fn dispatch_call_with_invalid_input_returns_error() {
			// Invalid encoded runtime call.
			let input = vec![0, 99];
			let mut env = MockEnvironment::new(FuncId::get(), input.clone());
			let error = pallet_contracts::Error::<Test>::DecodingFailed.into();
			let expected = <() as ErrorConverter>::convert(error, &mut env).err();
			assert_eq!(DispatchCall::execute(&mut env).err(), expected);
		}

		#[test]
		fn dispatch_call_with_invalid_input_charges_weight() {
			// Invalid encoded runtime call.
			let input = vec![0, 99];
			let mut env = MockEnvironment::new(FuncId::get(), input.clone());
			assert!(DispatchCall::execute(&mut env).is_err());
			assert_eq!(env.charged(), read_from_buffer_weight(input.len() as u32,));
		}
	}

	mod read_state {
		use super::*;
		use crate::mock::{RuntimeResult, UppercaseConverter};

		type ReadState = ReadStateWithFilter<Everything>;
		type ReadStateWithFilter<Filter> = super::ReadState<
			WithFuncId<FuncId>,
			Test,
			RuntimeRead,
			Decodes<RuntimeRead, ContractWeightsOf<Test>, DecodingFailed<Test>>,
			Filter,
		>;
		type ReadStateWithResultConverter<ResultConverter> = super::ReadState<
			WithFuncId<FuncId>,
			Test,
			RuntimeRead,
			Decodes<RuntimeRead, ContractWeightsOf<Test>, DecodingFailed<Test>>,
			Everything,
			ResultConverter,
		>;

		#[test]
		fn read_state_filtering_works() {
			let read = RuntimeRead::Ping;
			let mut env = MockEnvironment::new(FuncId::get(), read.encode());
			let error = frame_system::Error::<Test>::CallFiltered.into();
			let expected = <() as ErrorConverter>::convert(error, &mut env).err();
			assert_eq!(ReadStateWithFilter::<Nothing>::execute(&mut env).err(), expected);
		}

		#[test]
		fn read_state_filtered_charges_weight() {
			let read = RuntimeRead::Ping;
			let encoded_read = read.encode();
			let mut env = MockEnvironment::new(FuncId::get(), encoded_read.clone());
			assert!(ReadStateWithFilter::<Nothing>::execute(&mut env).is_err());
			assert_eq!(
				env.charged(),
				read_from_buffer_weight(encoded_read.len() as u32) + read.weight()
			);
		}

		#[test]
		fn read_state_works() {
			let read = RuntimeRead::Ping;
			let expected = "pop".as_bytes().encode();
			let mut env = MockEnvironment::new(FuncId::get(), read.encode());
			assert!(matches!(ReadState::execute(&mut env), Ok(Converging(0))));
			// Check if the contract environment buffer is written correctly.
			assert_eq!(env.buffer, expected);
		}

		#[test]
		fn read_state_result_conversion_works() {
			let read = RuntimeRead::Ping;
			let expected = RuntimeResult::Pong("pop".to_string());
			let mut env = MockEnvironment::new(FuncId::get(), read.encode());
			assert!(matches!(
				ReadStateWithResultConverter::<UppercaseConverter>::execute(&mut env),
				Ok(Converging(0))
			));
			// Check if the contract environment buffer is written correctly.
			assert_eq!(Ok(&env.buffer), UppercaseConverter::try_convert(expected, &env).as_ref());
		}

		#[test]
		fn read_state_charges_weight() {
			let read = RuntimeRead::Ping;
			let encoded_read = read.encode();
			let mut env = MockEnvironment::new(FuncId::get(), encoded_read.clone());
			assert!(ReadState::execute(&mut env).is_ok());
			let expected = "pop".as_bytes().encode();
			assert_eq!(
				env.charged(),
				read_from_buffer_weight(encoded_read.len() as u32)
					+ read.weight() + write_to_contract_weight(expected.len() as u32)
			);
		}

		#[test]
		fn read_state_with_invalid_input_returns_error() {
			// Invalid encoded runtime state read.
			let input = vec![0];
			let mut env = MockEnvironment::new(FuncId::get(), input.clone());
			let error = pallet_contracts::Error::<Test>::DecodingFailed.into();
			let expected = <() as ErrorConverter>::convert(error, &mut env).err();
			assert_eq!(ReadState::execute(&mut env).err(), expected);
		}

		#[test]
		fn read_state_with_invalid_input_charges_weight() {
			// Invalid encoded runtime state read.
			let input = vec![0];
			let mut env = MockEnvironment::new(FuncId::get(), input.clone());
			assert!(ReadState::execute(&mut env).is_err());
			assert_eq!(env.charged(), read_from_buffer_weight(input.len() as u32));
		}
	}

	#[test]
	fn execute_tuple_matches_and_executes_function() {
		type Functions = (Noop<WithFuncId<NoopFuncId>, Test>,);
		let mut env = MockEnvironment::new(NoopFuncId::get(), vec![]);
		assert!(matches!(Functions::execute(&mut env), Ok(Converging(0))));
	}

	#[test]
	fn execute_tuple_with_invalid_function_fails() {
		let input = vec![];
		let mut env = MockEnvironment::new(INVALID_FUNC_ID, input.clone());
		let error = pallet_contracts::Error::<Test>::DecodingFailed.into();
		let expected = <() as ErrorConverter>::convert(error, &mut env).err();
		assert_eq!(Functions::execute(&mut env).err(), expected);
	}

	#[test]
	fn execute_tuple_with_invalid_function_does_not_charge_weight() {
		let input = vec![];
		let mut env = MockEnvironment::new(INVALID_FUNC_ID, input.clone());
		assert!(Functions::execute(&mut env).is_err());
		// No weight charged as no function in the `Functions` tuple is matched to charge weight.
		// See extension tests for extension call weight charges.
		assert_eq!(env.charged(), Weight::default());
	}

	#[test]
	fn default_error_conversion_works() {
		let env = MockEnvironment::default();
		assert!(matches!(
			<() as ErrorConverter>::convert(DispatchError::BadOrigin, &env),
			Err(DispatchError::BadOrigin)
		));
	}

	#[test]
	fn default_conversion_works() {
		let env = MockEnvironment::default();
		let source = "pop".to_string();
		assert_eq!(
			DefaultConverter::try_convert(source.clone(), &env),
			Ok(source.as_bytes().into())
		);
	}
}

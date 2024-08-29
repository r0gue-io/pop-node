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
		// Decode runtime read
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
	use crate::mock::{DispatchCallFuncId, Environment, Ext, ReadStateFuncId};
	use codec::Encode;
	use mock::{RuntimeRead, RuntimeResult};

	#[test]
	fn execute_dispatch_call_function_works() {
		let mut env =
			mock::Environment::new(DispatchCallFuncId::get(), vec![], mock::Ext::default());
		assert!(matches!(mock::Functions::execute(&mut env), Ok(Converging(0))));
	}

	#[test]
	fn execute_read_state_function_works() {
		let mut env = mock::Environment::new(ReadStateFuncId::get(), vec![], mock::Ext::default());
		assert!(matches!(mock::Functions::execute(&mut env), Ok(Converging(0))));
	}

	#[test]
	fn execute_read_state_write_to_memory_works() {
		let mut env = mock::Environment::new(
			ReadStateFuncId::get(),
			[0u8.encode(), RuntimeRead::Ping.encode()].concat(),
			mock::Ext::default(),
		);
		assert!(matches!(mock::Functions::execute(&mut env), Ok(Converging(0))));
		assert_eq!(env.buffer, RuntimeResult::Pong("pop".to_string()).encode());
	}

	#[test]
	fn execute_read_state_invalid() {
		let mut env =
			mock::Environment::new(ReadStateFuncId::get(), vec![0, 0], mock::Ext::default());
		let error = pallet_contracts::Error::<mock::Test>::DecodingFailed.into();
		let expected = <() as ErrorConverter>::convert(error, &mut env).err();
		assert_eq!(mock::Functions::execute(&mut env).err(), expected);
	}

	#[test]
	fn execute_invalid_function() {
		let mut env = mock::Environment::new(0, vec![], mock::Ext::default());
		let error = pallet_contracts::Error::<mock::Test>::DecodingFailed.into();
		let expected = <() as ErrorConverter>::convert(error, &mut env).err();
		assert_eq!(mock::Functions::execute(&mut env).err(), expected);
	}

	#[test]
	fn default_error_conversion_works() {
		let env = Environment::new(0, [0u8; 42].to_vec(), Ext::default());
		assert!(matches!(
			<() as ErrorConverter>::convert(DispatchError::BadOrigin, &env),
			Err(DispatchError::BadOrigin)
		));
	}

	#[test]
	fn default_conversion_works() {
		let env = Environment::default();
		let source = "pop".to_string();
		assert_eq!(DefaultConverter::<String>::convert(source.clone(), &env), source.as_bytes());
	}
}

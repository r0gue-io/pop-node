use super::*;
use core::fmt::Debug;
pub use decoding::{Decode, Decodes};
pub use matching::{Equals, FunctionId, Matches};
use pallet_contracts::chain_extension::{BufIn, BufOut};

/// A chain extension function.
pub trait Function {
	/// The configuration of the contracts module.
	type Config: pallet_contracts::Config;

	/// Executes the function.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn execute<E: Ext<T = Self::Config>, S: BufIn + BufOut>(
		env: &mut Environment<E, S>,
	) -> Result<RetVal>;
}

// Support tuples of at least one function (required for type resolution) and a maximum of ten.
#[impl_trait_for_tuples::impl_for_tuples(1, 10)]
#[tuple_types_custom_trait_bound(Function + Matches)]
impl<Runtime: pallet_contracts::Config> Function for Tuple {
	for_tuples!( where #( Tuple: Function<Config=Runtime> )* );
	type Config = Runtime;
	fn execute<E: Ext<T = Self::Config>, S: BufIn + BufOut>(
		env: &mut Environment<E, S>,
	) -> Result<RetVal> {
		// Attempts to match a specified extension/function identifier to its corresponding function, as configured by the runtime.
		for_tuples!( #(
            if Tuple::matches(&env) {
                return Tuple::execute(env)
            }
        )* );

		// Otherwise returns error indicating an unmatched request.
		Err(pallet_contracts::Error::<Self::Config>::DecodingFailed.into())
	}
}

/// A function for dispatching a runtime call.
pub struct DispatchCall<C, D, M, F, L = ()>(PhantomData<(C, D, M, F, L)>);
impl<
		Config: pallet_contracts::Config
			+ frame_system::Config<
				RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
			>,
		Decoder: Decode<Output: codec::Decode + Into<<Config as frame_system::Config>::RuntimeCall>>,
		Matcher: Matches,
		Filter: Contains<<Config as frame_system::Config>::RuntimeCall> + 'static,
		Logger: LogTarget,
	> Function for DispatchCall<Config, Decoder, Matcher, Filter, Logger>
{
	/// The configuration of the contracts module.
	type Config = Config;

	/// Executes the function.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn execute<E: Ext<T = Config>, S: BufIn + BufOut>(
		env: &mut Environment<E, S>,
	) -> Result<RetVal> {
		// Decode runtime call.
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
		result.map(|_| Converging(0)).map_err(|e| e.error)
	}
}

impl<C, D, M: Matches, F, L> Matches for DispatchCall<C, D, M, F, L> {
	fn matches<E: Ext, S: State>(env: &Environment<E, S>) -> bool {
		M::matches(env)
	}
}

/// A function for reading runtime state.
pub struct ReadState<C, R, D, M, F, RC = DefaultConverter<<R as Readable>::Result>, L = ()>(
	PhantomData<(C, R, D, M, F, RC, L)>,
);
impl<
		Config: pallet_contracts::Config,
		Read: Readable + Debug,
		Decoder: Decode<Output: codec::Decode + Into<Read>>,
		Matcher: Matches,
		Filter: Contains<Read>,
		ResultConverter: Converter<Source = Read::Result, Target: Into<Vec<u8>>>,
		Logger: LogTarget,
	> Function for ReadState<Config, Read, Decoder, Matcher, Filter, ResultConverter, Logger>
{
	/// The configuration of the contracts module.
	type Config = Config;

	/// Executes the function.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn execute<E: Ext<T = Config>, S: BufIn + BufOut>(
		env: &mut Environment<E, S>,
	) -> Result<RetVal> {
		// Decode runtime read
		let read = Decoder::decode(env)?.into();
		log::debug!(target: Logger::LOG_TARGET, "decoded: read={read:?}");
		// Charge weight before read
		let charged = env.charge_weight(read.weight())?;
		log::trace!(target: Logger::LOG_TARGET, "pre-read weight charged: charged={charged:?}");
		// Ensure read allowed
		ensure!(Filter::contains(&read), frame_system::Error::<Config>::CallFiltered);
		let result = read.read();
		log::debug!(target: Logger::LOG_TARGET, "read: result={result:?}");
		// Perform any final conversion. Any implementation is expected to charge weight as appropriate.
		let result = ResultConverter::convert(result, env).into();
		// TODO: check parameters (allow_skip, weight_per_byte)
		env.write(&result, false, Some(Schedule::<Config>::get().host_fn_weights.input_per_byte))?;
		Ok(Converging(0))
	}
}

impl<C, R, D, M: Matches, F, RC, L> Matches for ReadState<C, R, D, M, F, RC, L> {
	fn matches<E: Ext, S: State>(env: &Environment<E, S>) -> bool {
		M::matches(env)
	}
}

/// A default converter, for converting (encoding) from some type into a byte array.
pub struct DefaultConverter<T>(PhantomData<T>);
impl<T: Into<Vec<u8>>> Converter for DefaultConverter<T> {
	type Source = T;
	type Target = Vec<u8>;
	const LOG_TARGET: &'static str = "";

	fn convert<E: Ext, S: State>(value: Self::Source, _env: &Environment<E, S>) -> Self::Target {
		value.into()
	}
}
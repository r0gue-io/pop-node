use super::*;
pub use decoding::{Decode, Decodes, Processor};
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
		env: Environment<E, S>,
	) -> Result<RetVal>;
}

#[impl_trait_for_tuples::impl_for_tuples(1, 3)]
#[tuple_types_custom_trait_bound(Function + Matches)]
impl<Runtime: pallet_contracts::Config> Function for Tuple {
	for_tuples!( where #( Tuple: Function<Config=Runtime> )* );
	type Config = Runtime;
	fn execute<E: Ext<T = Self::Config>, S: BufIn + BufOut>(
		env: Environment<E, S>,
	) -> Result<RetVal> {
		// Attempts to match a specified extension/function identifier to its corresponding function, as configured by the runtime.
		for_tuples!( #(
            if Tuple::matches(env.ext_id(), env.func_id()) {
                return Tuple::execute(env)
            }
        )* );

		// Otherwise returns `DispatchError::Other` indicating an unmatched request.
		// TODO: improve error so we can determine if its an invalid function vs unknown runtime call/read
		Err(UNKNOWN_CALL_ERROR)
	}
}

/// A function for dispatching a runtime call.
pub struct DispatchCall<C, D, M, F>(PhantomData<(C, D, M, F)>);
impl<
		Config: pallet_contracts::Config
			+ frame_system::Config<
				RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
			>,
		Decoder: Decode<Output: codec::Decode + Into<<Config as frame_system::Config>::RuntimeCall>>,
		Matcher: Matches,
		Filter: Contains<<Config as frame_system::Config>::RuntimeCall> + 'static,
	> Function for DispatchCall<Config, Decoder, Matcher, Filter>
{
	/// The configuration of the contracts module.
	type Config = Config;

	/// Executes the function.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn execute<E: Ext<T = Config>, S: BufIn + BufOut>(
		mut env: Environment<E, S>,
	) -> Result<RetVal> {
		const LOG_PREFIX: &str = " dispatch |";

		// Decode runtime call.
		let call = Decoder::decode(&mut env)?.into();
		// TODO: log::debug!(target:LOG_TARGET, "{} Inputted RuntimeCall: {:?}", LOG_PREFIX, call);
		// Charge weight before dispatch.
		let dispatch_info = call.get_dispatch_info();
		let charged = env.charge_weight(dispatch_info.weight)?;
		// TODO: log::trace! dispatch_info and charged
		// Contract is the origin by default.
		let mut origin: Config::RuntimeOrigin =
			RawOrigin::Signed(env.ext().address().clone()).into();
		// Ensure call allowed.
		origin.add_filter(Filter::contains);
		// Dispatch call.
		let result = call.dispatch(origin);
		// TODO: log::debug!(target:LOG_TARGET, "{} result, actual weight: {:?}", LOG_PREFIX, info.actual_weight);
		// Adjust weight.
		let weight = frame_support::dispatch::extract_actual_weight(&result, &dispatch_info);
		env.adjust_weight(charged, weight);
		// TODO: conversion of error to 'versioned' status code
		result.map(|_| Converging(0)).map_err(|e| e.error)
	}
}

impl<C, D, M: Matches, F> Matches for DispatchCall<C, D, M, F> {
	fn matches(ext_id: u16, func_id: u16) -> bool {
		M::matches(ext_id, func_id)
	}
}

/// A function for reading runtime state.
pub struct ReadState<C, R, D, M, F>(PhantomData<(C, R, D, M, F)>);
impl<
		Config: pallet_contracts::Config,
		Read: Readable,
		Decoder: Decode<Output: codec::Decode + Into<Read>>,
		Matcher: Matches,
		Filter: Contains<Read>,
	> Function for ReadState<Config, Read, Decoder, Matcher, Filter>
{
	/// The configuration of the contracts module.
	type Config = Config;

	/// Executes the function.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn execute<E: Ext<T = Config>, S: BufIn + BufOut>(
		mut env: Environment<E, S>,
	) -> Result<RetVal> {
		const LOG_PREFIX: &str = " dispatch |";

		// Decode runtime read
		let read = Decoder::decode(&mut env)?.into();
		// TODO: log::debug!(target:LOG_TARGET, "{} Inputted RuntimeRead: {:?}", LOG_PREFIX, read);
		// Charge weight before read
		let _charged = env.charge_weight(read.weight())?;
		// TODO: log::trace! charged
		// Ensure read allowed
		ensure!(Filter::contains(&read), frame_system::Error::<Config>::CallFiltered);
		let result = read.read();
		// TODO: log::debug!(target:LOG_TARGET, "{} result: {:?}", LOG_PREFIX, result);
		// TODO: check parameters (allow_skip, weight_per_byte)
		env.write(&result, false, Some(Schedule::<Config>::get().host_fn_weights.input_per_byte))?;
		// TODO: conversion of error to 'versioned' status code
		Ok(Converging(0))
	}
}

impl<C, R, D, M: Matches, F> Matches for ReadState<C, R, D, M, F> {
	fn matches(ext_id: u16, func_id: u16) -> bool {
		M::matches(ext_id, func_id)
	}
}

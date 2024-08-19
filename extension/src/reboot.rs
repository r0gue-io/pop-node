use crate::{DECODING_FAILED_ERROR, UNKNOWN_CALL_ERROR};
use codec::Decode;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo, RawOrigin},
	ensure,
	traits::{Contains, OriginTrait},
	weights::Weight,
};
pub use functions::{
	builders::{Builder, DefaultBuilder, PrefixBuilder},
	matching::{FirstByte, FunctionIdMatcher, Matches},
	DispatchCall, Function, ReadState,
};
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, Result, RetVal, RetVal::Converging,
};
use sp_core::Get;
use sp_runtime::traits::Dispatchable;
use std::marker::PhantomData;

type Schedule<T> = <T as pallet_contracts::Config>::Schedule;

/// A configurable chain extension.
#[derive(Default)]
pub struct Extension<C: Config>(PhantomData<C>);
impl<Config, F> ChainExtension<Config> for Extension<F>
where
	Config: pallet_contracts::Config
		+ frame_system::Config<
			RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
		>,
	F: Functions<Function: Function<C = C>> + 'static,
{
	/// Call the chain extension logic.
	///
	/// # Parameters
	/// - `env`: Access to the remaining arguments and the execution environment.
	fn call<E: Ext<T = Config>>(&mut self, env: Environment<E, InitState>) -> Result<RetVal> {
		let mut env = env.buf_in_buf_out();
		env.charge_weight(Schedule::<Config>::get().host_fn_weights.debug_message)?;
		F::Functions::execute(env)
	}
}

/// Trait for configuration of the chain extension.
pub trait Config {
	/// The function(s) available with the chain extension.
	type Functions: Function;
}

// Trait to be implemented for type handling a read of runtime state
pub trait RuntimeRead {
	fn weight(&self) -> Weight;
	fn read(self) -> Vec<u8>;
}

mod functions {
	use super::*;
	use builders::Builder;
	use matching::Matches;
	use pallet_contracts::chain_extension::{BufIn, BufOut};

	// A chain extension function.
	pub trait Function {
		type C: Config;
		fn execute<E: Ext<T = Self::C>>(env: Environment<E, BufInBufOutState>) -> Result<RetVal>;
	}

	#[impl_trait_for_tuples::impl_for_tuples(1, 3)]
	#[tuple_types_custom_trait_bound(Function + Matches)]
	impl<C: Config> Function for Tuple {
		for_tuples!( where #( Tuple: Function<C=C> )* );
		type C = C;
		fn execute<E: Ext<T = Self::C>>(env: Environment<E, BufInBufOutState>) -> Result<RetVal> {
			for_tuples!( #(
			if Tuple::matches(env.func_id()) {
				return Tuple::execute(env)
			}
		)* );

			Err(UNKNOWN_CALL_ERROR)
		}
	}

	// Function implementation for dispatching a call.
	pub struct DispatchCall<C, B, D, Matcher, Filter>(PhantomData<(C, B, D, Matcher, Filter)>);
	impl<
			C: Config
				+ frame_system::Config<
					RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
				>,
			B: Builder<C = C, Output = Dispatch>,
			Dispatch: Decode + Into<<C as frame_system::Config>::RuntimeCall>,
			Matcher: Matches,
			Filter: Contains<<C as frame_system::Config>::RuntimeCall> + 'static,
		> Function for DispatchCall<C, B, Dispatch, Matcher, Filter>
	{
		type C = C;

		fn execute<E: Ext<T = C>>(mut env: Environment<E, BufInBufOutState>) -> Result<RetVal> {
			// Build call
			let call = B::build(&mut env)?.into();
			// Charge weight before dispatch
			let dispatch_info = call.get_dispatch_info();
			let charged = env.charge_weight(dispatch_info.weight)?;
			// Ensure call allowed
			let mut origin: C::RuntimeOrigin =
				RawOrigin::Signed(env.ext().address().clone()).into();
			origin.add_filter(Filter::contains);
			// Dispatch call
			let result = call.dispatch(origin);
			// Adjust weight
			let weight = frame_support::dispatch::extract_actual_weight(&result, &dispatch_info);
			env.adjust_weight(charged, weight);
			result.map(|_| Converging(0)).map_err(|e| e.error)
		}
	}

	impl<C, B, D, M: Matches, F> Matches for DispatchCall<C, B, D, M, F> {
		fn matches(func_id: u16) -> bool {
			M::matches(func_id)
		}
	}

	// Function implementation for reading state.
	pub struct ReadState<C, B, R, RR, M, F>(PhantomData<(C, B, R, RR, M, F)>);
	impl<
			C: Config,
			B: Builder<C = C, Output = R>,
			R: Decode + Into<RR>,
			RR: RuntimeRead,
			Matcher: Matches,
			Filter: Contains<RR>,
		> Function for ReadState<C, B, R, RR, Matcher, Filter>
	{
		type C = C;

		fn execute<E: Ext<T = C>>(mut env: Environment<E, BufInBufOutState>) -> Result<RetVal> {
			// Build call
			let read = B::build(&mut env)?.into();

			// Charge weight before read
			env.charge_weight(read.weight())?;
			// Ensure read allowed
			ensure!(Filter::contains(&read), frame_system::Error::<Config>::CallFiltered);
			// TODO: check parameters (allow_skip, weight_per_byte)
			env.write(
				&read.read(),
				false,
				Some(Schedule::<Config>::get().host_fn_weights.input_per_byte),
			)?;
			Ok(Converging(0))
		}
	}

	impl<C, B, R, RR, M: Matches, F> Matches for ReadState<C, B, R, RR, M, F> {
		fn matches(func_id: u16) -> bool {
			M::matches(func_id)
		}
	}

	pub mod builders {
		use super::*;

		// Trait for building some output (call/read) from the environment.
		pub trait Builder {
			type C: Config;
			type Output: Decode;
			fn build<E: Ext<T = Self::C>>(
				env: &mut Environment<E, BufInBufOutState>,
			) -> Result<Self::Output>;
		}

		// Default implementation which charges appropriate weight before building the output.
		pub struct DefaultBuilder<C, O>(PhantomData<(C, O)>);
		impl<C: Config, Output: Decode> Builder for DefaultBuilder<C, Output> {
			type C = C;
			type Output = Output;

			fn build<E: Ext<T = Self::C>>(
				env: &mut Environment<E, BufInBufOutState>,
			) -> Result<Self::Output> {
				let contract_host_weight = <C as Config>::Schedule::get().host_fn_weights;
				let len = env.in_len();
				env.charge_weight(contract_host_weight.return_per_byte.saturating_mul(len.into()))?;
				let params = env.read(len)?;
				Output::decode(&mut &params[..]).map_err(|_| DECODING_FAILED_ERROR)
			}
		}

		// Implementation which charges appropriate weight before building the output, prefixed with additional data.
		pub struct PrefixBuilder<C, O>(PhantomData<(C, O)>);
		impl<C: Config, Output: Decode> Builder for PrefixBuilder<C, Output> {
			type C = C;
			type Output = Output;

			fn build<E: Ext<T = Self::C>>(
				env: &mut Environment<E, BufInBufOutState>,
			) -> Result<Self::Output> {
				let contract_host_weight = <C as Config>::Schedule::get().host_fn_weights;
				let len = env.in_len();
				env.charge_weight(contract_host_weight.return_per_byte.saturating_mul(len.into()))?;
				let mut params = env.read(len)?;

				// TODO: refactor out into own type, merging default and prefix into a single implementation with optional prefix handling
				let version = env.func_id().to_le_bytes()[1];
				let (pallet_index, call_index) = {
					let bytes = env.ext_id().to_le_bytes();
					(bytes[0], bytes[1])
				};
				params.insert(0, version);
				params.insert(1, pallet_index);
				params.insert(2, call_index);

				Output::decode(&mut &params[..]).map_err(|_| DECODING_FAILED_ERROR)
			}
		}
	}

	pub mod matching {
		use super::*;

		// Simple trait for determining of a function matches some identifier
		pub trait Matches {
			fn matches(func_id: u16) -> bool;
		}

		// Default implementation
		pub struct FunctionIdMatcher<T>(PhantomData<T>);
		impl<T: Get<u16>> Matches for FunctionIdMatcher<T> {
			fn matches(function_id: u16) -> bool {
				function_id == T::get()
			}
		}

		// Implementation which matches on the first byte only
		pub struct FirstByte<T>(PhantomData<T>);
		impl<T: Get<u8>> Matches for FirstByte<T> {
			fn matches(function_id: u16) -> bool {
				let bytes = function_id.to_le_bytes();
				bytes[0] == T::get()
			}
		}
	}
}

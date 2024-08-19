use crate::{DECODING_FAILED_ERROR, UNKNOWN_CALL_ERROR};
use codec::Decode as _;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo, RawOrigin},
	ensure,
	traits::{Contains, OriginTrait},
	weights::Weight,
};
pub use functions::{
	decoding::{Decode, Decodes, Processor},
	matching::{FirstByteOfFunctionId, FunctionIdMatcher, Matches},
	DispatchCall, Function, ReadState,
};
use pallet_contracts::{
	chain_extension::{
		BufInBufOutState, ChainExtension, Environment, Ext, InitState, Result, RetVal,
		RetVal::Converging,
	},
	Config,
};
use sp_core::Get;
use sp_runtime::traits::Dispatchable;
use std::marker::PhantomData;

#[derive(Default)]
pub struct ApiExtension<F>(PhantomData<F>);
impl<C, F> ChainExtension<C> for ApiExtension<F>
where
	C: Config
		+ frame_system::Config<
			RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
		>,
	F: Functions<Function: Function<Runtime = C>> + 'static,
{
	fn call<E: Ext<T = C>>(&mut self, env: Environment<E, InitState>) -> Result<RetVal> {
		let mut env = env.buf_in_buf_out();

		let contract_host_weight = <C as Config>::Schedule::get().host_fn_weights;
		env.charge_weight(contract_host_weight.debug_message)?;

		F::Function::execute::<E>(env)
	}
}

// Simple trait to allow configuration of chain extension functions (workaround for Default requirement)
pub trait Functions {
	type Function: Function;
}

// Trait to be implemented for type handling a read of runtime state
pub trait RuntimeRead {
	fn weight(&self) -> Weight;
	fn read(self) -> Vec<u8>;
}

mod functions {
	use super::*;
	use decoding::Decode;
	use matching::Matches;

	/// A chain extension function.
	pub trait Function {
		type Runtime: Config;
		fn execute<E: Ext<T = Self::Runtime>>(
			env: Environment<E, BufInBufOutState>,
		) -> Result<RetVal>;
	}

	#[impl_trait_for_tuples::impl_for_tuples(1, 3)]
	#[tuple_types_custom_trait_bound(Function + Matches)]
	impl<Runtime: Config> Function for Tuple {
		for_tuples!( where #( Tuple: Function<Runtime=Runtime> )* );
		type Runtime = Runtime;
		fn execute<E: Ext<T = Self::Runtime>>(
			env: Environment<E, BufInBufOutState>,
		) -> Result<RetVal> {
			for_tuples!( #(
			if Tuple::matches(env.func_id()) {
				return Tuple::execute(env)
			}
		)* );

			Err(UNKNOWN_CALL_ERROR)
		}
	}

	// Function implementation for dispatching a call.
	pub struct DispatchCall<R, D, M, F>(PhantomData<(R, D, M, F)>);
	impl<
			Runtime: Config
				+ frame_system::Config<
					RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
				>,
			Decoder: Decode<Output: codec::Decode + Into<<Runtime as frame_system::Config>::RuntimeCall>>,
			Matcher: Matches,
			Filter: Contains<<Runtime as frame_system::Config>::RuntimeCall> + 'static,
		> Function for DispatchCall<Runtime, Decoder, Matcher, Filter>
	{
		type Runtime = Runtime;

		fn execute<E: Ext<T = Runtime>>(
			mut env: Environment<E, BufInBufOutState>,
		) -> Result<RetVal> {
			// Build call
			let call = Decoder::decode(&mut env)?.into();
			// Charge weight before dispatch
			let charged_weight = env.charge_weight(call.get_dispatch_info().weight)?;
			// Ensure call allowed
			let mut origin: Runtime::RuntimeOrigin =
				RawOrigin::Signed(env.ext().address().clone()).into();
			origin.add_filter(Filter::contains);

			let (result, weight) = match call.dispatch(origin) {
				Ok(info) => (Ok(()), info.actual_weight),
				Err(err) => (Err(err.error), err.post_info.actual_weight),
			};
			// Adjust post-dispatch weight
			if let Some(actual_weight) = weight {
				env.adjust_weight(charged_weight, actual_weight);
			}
			result.map(|_| Converging(0))
		}
	}

	impl<R, D, M: Matches, F> Matches for DispatchCall<R, D, M, F> {
		fn matches(func_id: u16) -> bool {
			M::matches(func_id)
		}
	}

	// Function implementation for reading state.
	pub struct ReadState<R, D, RR, M, F>(PhantomData<(R, D, RR, M, F)>);
	impl<
			Runtime: Config,
			Decoder: Decode<Output: codec::Decode + Into<Read>>,
			Read: RuntimeRead,
			Matcher: Matches,
			Filter: Contains<Read>,
		> Function for ReadState<Runtime, Decoder, Read, Matcher, Filter>
	{
		type Runtime = Runtime;

		fn execute<E: Ext<T = Runtime>>(
			mut env: Environment<E, BufInBufOutState>,
		) -> Result<RetVal> {
			// Build runtime read
			let read = Decoder::decode(&mut env)?.into();
			// Charge weight before read
			env.charge_weight(read.weight())?;
			// Ensure call allowed
			// TODO: use filtered error so easier to determine if it is decoding error or a filter blocking the call
			ensure!(Filter::contains(&read), UNKNOWN_CALL_ERROR);
			// TODO: check remaining parameters (allow_skip, weight per byte)
			env.write(&read.read(), false, None)?;
			Ok(Converging(0))
		}
	}

	impl<R, D, RR, M: Matches, F> Matches for ReadState<R, D, RR, M, F> {
		fn matches(func_id: u16) -> bool {
			M::matches(func_id)
		}
	}

	/// Provides functionality for processing/decoding data read from contract memory.
	pub mod decoding {
		use super::*;
		use pallet_contracts::chain_extension::{BufIn, State};
		use sp_runtime::DispatchError;

		/// Trait for decoding data read from contract memory.
		pub trait Decode {
			/// The output type to be decoded.
			type Output: codec::Decode;
			/// An optional processor, for performing any additional processing before decoding.
			type Processor: Processor;

			/// The error to return if decoding fails.
			const ERROR: DispatchError = DECODING_FAILED_ERROR;

			/// Decodes data read from contract memory.
			///
			/// # Parameters
			/// - `env` - The current execution environment.
			fn decode<E: Ext, S: BufIn>(env: &mut Environment<E, S>) -> Result<Self::Output> {
				// Charge appropriate weight prior to decoding.
				let contract_host_weight = <E::T as Config>::Schedule::get().host_fn_weights;
				let len = env.in_len();
				env.charge_weight(contract_host_weight.return_per_byte.saturating_mul(len.into()))?;
				// Read input supplied by contract.
				let mut input = env.read(len)?;
				// Perform any additional processing required. Any implementation is expected to charge weight as appropriate.
				Self::Processor::process(&mut input, env);
				// Finally decode and return.
				Self::Output::decode(&mut &input[..]).map_err(|_| Self::ERROR)
			}
		}

		/// Default implementation for decoding data read from contract memory.
		pub struct Decodes<O, P = ()>(PhantomData<(O, P)>);
		impl<Output: codec::Decode, Processor_: Processor> Decode for Decodes<Output, Processor_> {
			type Output = Output;
			type Processor = Processor_;
		}

		/// Trait for processing a value based on additional information available from the environment.
		pub trait Processor {
			/// Processes the provided value.
			///
			/// # Parameters
			/// - `value` - The value to be processed.
			/// - `env` - The current execution environment.
			fn process<E: Ext, S: State>(value: &mut Vec<u8>, env: &mut Environment<E, S>);
		}

		impl Processor for () {
			fn process<E: Ext, S: State>(_value: &mut Vec<u8>, _env: &mut Environment<E, S>) {}
		}
	}

	pub mod matching {
		use super::*;

		// Simple trait for determining if a `Function` matches a function identifier
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

		// Implementation which matches on the first byte of a function identifier only
		pub struct FirstByteOfFunctionId<T>(PhantomData<T>);
		impl<T: Get<u8>> Matches for FirstByteOfFunctionId<T> {
			fn matches(function_id: u16) -> bool {
				let bytes = function_id.to_le_bytes();
				bytes[0] == T::get()
			}
		}
	}
}

pub mod pop_api {
	use super::functions::decoding::Processor;
	use pallet_contracts::chain_extension::{Environment, Ext, State};

	// Use bytes from func_id() + ext_id() to prefix the encoded input bytes to determine the versioned output
	// TODO: implementation is technically specific to pop-api so should be moved elsewhere - e.g. pallet-api
	pub struct Prepender;
	impl Processor for Prepender {
		fn process<E: Ext, S: State>(value: &mut Vec<u8>, env: &mut Environment<E, S>) {
			// TODO: revisit the ordering based on specced standard
			// Resolve version, pallet and call index from environment
			let version = env.func_id().to_le_bytes()[1];
			let (pallet_index, call_index) = {
				let bytes = env.ext_id().to_le_bytes();
				(bytes[0], bytes[1])
			};
			// Prepend bytes
			value.insert(0, version);
			value.insert(1, pallet_index);
			value.insert(2, call_index);
		}
	}
}

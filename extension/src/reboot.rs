use crate::{DECODING_FAILED_ERROR, UNKNOWN_CALL_ERROR};
use codec::Decode as _;
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo, RawOrigin},
	ensure,
	traits::{Contains, OriginTrait},
	weights::Weight,
};
pub use functions::{
	Decode, Decodes, DispatchCall, Equals, Function, FunctionId, Matches, Processor, ReadState,
	Readable,
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
	F: self::Config<Functions: Function<Config = Config>> + 'static,
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

mod functions {
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
			for_tuples!( #(
			if Tuple::matches(env.ext_id(), env.func_id()) {
				return Tuple::execute(env)
			}
		)* );

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
			// Decode runtime call
			let call = Decoder::decode(&mut env)?.into();
			// Charge weight before dispatch
			let dispatch_info = call.get_dispatch_info();
			let charged = env.charge_weight(dispatch_info.weight)?;
			// Ensure call allowed
			let mut origin: Config::RuntimeOrigin =
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
			// Decode runtime read
			let read = Decoder::decode(&mut env)?.into();
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

	impl<C, R, D, M: Matches, F> Matches for ReadState<C, R, D, M, F> {
		fn matches(ext_id: u16, func_id: u16) -> bool {
			M::matches(ext_id, func_id)
		}
	}

	/// Trait to be implemented for type handling a read of runtime state
	pub trait Readable {
		/// Determines the weight of the read, used to charge the appropriate weight before the read is performed.
		fn weight(&self) -> Weight;

		/// Performs the read and returns the result.
		fn read(self) -> Vec<u8>;
	}

	mod decoding {
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
				let len = env.in_len();
				env.charge_weight(
					Schedule::<E::T>::get()
						.host_fn_weights
						.return_per_byte
						.saturating_mul(len.into()),
				)?;
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

	mod matching {
		use super::*;

		/// Trait for matching a function.
		pub trait Matches {
			/// Determines whether a function is a match.
			///
			/// # Parameters
			/// - `ext_id` - The specified chain extension identifier.
			/// - `func_id` - The specified function identifier.
			fn matches(ext_id: u16, func_id: u16) -> bool;
		}

		/// Matches on an extension and function identifier.
		pub struct Equals<E, F>(PhantomData<(E, F)>);
		impl<E: Get<u16>, F: Get<u16>> Matches for Equals<E, F> {
			fn matches(ext_id: u16, func_id: u16) -> bool {
				ext_id == E::get() && func_id == F::get()
			}
		}

		/// Matches on a function identifier only.
		pub struct FunctionId<T>(PhantomData<T>);
		impl<T: Get<u16>> Matches for FunctionId<T> {
			fn matches(_ext_id: u16, func_id: u16) -> bool {
				func_id == T::get()
			}
		}
	}
}

// TODO: below implementations are technically specific to pop-api so should be moved elsewhere - e.g. pallet-api
pub mod pop_api {
	use super::{Decodes, Matches, Processor};
	use core::marker::PhantomData;
	use pallet_contracts::chain_extension::{Environment, Ext, State};
	use sp_core::Get;

	pub type PopApi<Functions> = super::Extension<Functions>;

	// Use bytes from func_id() + ext_id() to prefix the encoded input bytes to determine the versioned output
	pub type DecodesAs<Output> = Decodes<Output, Prepender>;

	// Use bytes from func_id() + ext_id() to prefix the encoded input bytes to determine the versioned output
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

	/// Matches on the first byte of a function identifier only.
	pub struct FirstByteOfFunctionId<T>(PhantomData<T>);
	impl<T: Get<u8>> Matches for FirstByteOfFunctionId<T> {
		fn matches(_ext_id: u16, func_id: u16) -> bool {
			let bytes = func_id.to_le_bytes();
			bytes[0] == T::get()
		}
	}
}

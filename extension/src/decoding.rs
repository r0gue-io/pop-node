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
	/// The log target.
	const LOG_TARGET: &'static str;

	/// Decodes data read from contract memory.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn decode<E: Ext, S: BufIn>(env: &mut Environment<E, S>) -> Result<Self::Output> {
		// Charge appropriate weight, based on input length, prior to decoding.
		// reference: https://github.com/paritytech/polkadot-sdk/blob/117a9433dac88d5ac00c058c9b39c511d47749d2/substrate/frame/contracts/src/wasm/runtime.rs#L267
		let len = env.in_len();
		let weight = Schedule::<E::T>::get()
			.host_fn_weights
			.return_per_byte
			.saturating_mul(len.into());
		env.charge_weight(weight)?;
		log::debug!(target: Self::LOG_TARGET, "pre-decode weight charged: len={len}, weight={weight}");
		// Read encoded input supplied by contract for buffer.
		let mut input = env.read(len)?;
		log::debug!(target: Self::LOG_TARGET, "input read: input={input:?}");
		// Perform any additional processing required. Any implementation is expected to charge weight as appropriate.
		Self::Processor::process(&mut input, env);
		// Finally decode and return.
		Self::Output::decode(&mut &input[..]).map_err(|_| Self::ERROR)
	}
}

/// Default implementation for decoding data read from contract memory.
pub struct Decodes<O, P = (), L = ()>(PhantomData<(O, P, L)>);
impl<Output: codec::Decode, Processor_: Processor, Logger: LogTarget> Decode
	for Decodes<Output, Processor_, Logger>
{
	type Output = Output;
	type Processor = Processor_;
	const LOG_TARGET: &'static str = Logger::LOG_TARGET;
}

/// Trait for processing a value based on additional information available from the environment.
pub trait Processor {
	/// The log target.
	const LOG_TARGET: &'static str;

	/// Processes the provided value.
	///
	/// # Parameters
	/// - `value` - The value to be processed.
	/// - `env` - The current execution environment.
	fn process<E: Ext, S: State>(value: &mut Vec<u8>, env: &mut Environment<E, S>);
}

impl Processor for () {
	const LOG_TARGET: &'static str = "";

	fn process<E: Ext, S: State>(_value: &mut Vec<u8>, _env: &mut Environment<E, S>) {}
}

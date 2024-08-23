use super::*;
use pallet_contracts::chain_extension::BufIn;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

/// Trait for decoding data read from contract memory.
pub trait Decode {
	/// The output type to be decoded.
	type Output: codec::Decode;
	/// An optional processor, for performing any additional processing on data read from the contract before decoding.
	type Processor: Processor<Value = Vec<u8>>;
	/// The error to return if decoding fails.
	type Error: Get<DispatchError>;

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
		input = Self::Processor::process(input, env);
		// Finally decode and return.
		Self::Output::decode(&mut &input[..]).map_err(|_| Self::Error::get())
	}
}

/// Default implementation for decoding data read from contract memory.
pub struct Decodes<O, E, P = (), L = ()>(PhantomData<(O, E, P, L)>);
impl<
		Output: codec::Decode,
		Error: Get<DispatchError>,
		ValueProcessor: Processor<Value = Vec<u8>>,
		Logger: LogTarget,
	> Decode for Decodes<Output, Error, ValueProcessor, Logger>
{
	type Output = Output;
	type Processor = ValueProcessor;
	type Error = Error;
	const LOG_TARGET: &'static str = Logger::LOG_TARGET;
}

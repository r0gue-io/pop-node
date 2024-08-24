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
		// Charge appropriate weight for copying from contract, based on input length, prior to decoding.
		// reference: https://github.com/paritytech/polkadot-sdk/pull/4233/files#:~:text=CopyToContract(len)%20%3D%3E%20T%3A%3AWeightInfo%3A%3Aseal_return(len)%2C
		let len = env.in_len();
		let weight = ContractWeights::<E::T>::seal_return(len);
		let charged = env.charge_weight(weight)?;
		log::debug!(target: Self::LOG_TARGET, "pre-decode weight charged: len={len}, weight={weight}, charged={charged:?}");
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
pub struct Decodes<O, E, P = IdentityProcessor, L = ()>(PhantomData<(O, E, P, L)>);
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

/// Default processor implementation which just passes through the value unchanged.
pub struct IdentityProcessor;
impl Processor for IdentityProcessor {
	type Value = Vec<u8>;
	const LOG_TARGET: &'static str = "";

	fn process<E: Ext, S: State>(value: Self::Value, _env: &Environment<E, S>) -> Self::Value {
		value
	}
}

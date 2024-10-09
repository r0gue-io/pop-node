#![cfg_attr(not(feature = "std"), no_std)]

use core::marker::PhantomData;

pub use decoding::{Decode, Decodes, DecodingFailed, Identity, Processor};
pub use environment::{BufIn, BufOut, Environment, Ext};
use frame_support::{
	dispatch::{GetDispatchInfo, PostDispatchInfo, RawOrigin},
	ensure,
	traits::{Contains, OriginTrait},
	weights::Weight,
};
pub use functions::{
	Converter, DefaultConverter, DispatchCall, ErrorConverter, Function, ReadState, Readable,
};
pub use matching::{Equals, FunctionId, Matches};
use pallet_revive::{
	chain_extension::{ChainExtension, RetVal::Converging},
	WeightInfo,
};
pub use pallet_revive::{
	chain_extension::{Result, RetVal},
	wasm::Memory,
};
use sp_core::Get;
use sp_runtime::{traits::Dispatchable, DispatchError};
use sp_std::vec::Vec;

mod decoding;
mod environment;
mod functions;
mod matching;
// Mock runtime/environment for unit/integration testing.
#[cfg(test)]
mod mock;
// Integration tests using proxy contract and mock runtime.
#[cfg(test)]
mod tests;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type ContractWeightsOf<T> = <T as pallet_revive::Config>::WeightInfo;
type RuntimeCallOf<T> = <T as frame_system::Config>::RuntimeCall;

/// A configurable chain extension.
#[derive(Default)]
pub struct Extension<C: Config>(PhantomData<C>);
impl<Runtime, Config> ChainExtension<Runtime> for Extension<Config>
where
	Runtime: pallet_revive::Config
		+ frame_system::Config<
			RuntimeCall: GetDispatchInfo + Dispatchable<PostInfo = PostDispatchInfo>,
		>,
	Config: self::Config<Functions: Function<Config = Runtime>> + 'static,
{
	/// Call the chain extension logic.
	///
	/// # Parameters
	/// - `env`: Access to the remaining arguments and the execution environment.
	fn call<E: pallet_revive::chain_extension::Ext<T = Runtime>, M: ?Sized + Memory<E::T>>(
		&mut self,
		env: pallet_revive::chain_extension::Environment<E, M>,
	) -> Result<RetVal> {
		let mut env = environment::Env(env);
		self.call(&mut env)
	}
}

impl<
		Runtime: pallet_revive::Config,
		Config: self::Config<Functions: Function<Config = Runtime>>,
	> Extension<Config>
{
	fn call(
		&mut self,
		env: &mut (impl Environment<AccountId = Runtime::AccountId> + BufIn + BufOut),
	) -> Result<RetVal> {
		log::trace!(target: Config::LOG_TARGET, "extension called");
		// Charge weight for making a call from a contract to the runtime.
		// `debug_message` weight is a good approximation of the additional overhead of going from
		// contract layer to substrate layer. reference: https://github.com/paritytech/polkadot-sdk/pull/4233/files#:~:text=DebugMessage(len)%20%3D%3E%20T%3A%3AWeightInfo%3A%3Aseal_debug_message(len)%2C
		let len = env.in_len();
		let overhead = ContractWeightsOf::<Runtime>::seal_debug_message(len);
		let charged = env.charge_weight(overhead)?;
		log::debug!(target: Config::LOG_TARGET, "extension call weight charged: len={len}, weight={overhead}, charged={charged:?}");
		// Execute the function
		Config::Functions::execute(env)
	}
}

/// Trait for configuration of the chain extension.
pub trait Config {
	/// The function(s) available with the chain extension.
	type Functions: Function;

	/// The log target.
	const LOG_TARGET: &'static str;
}

/// Trait to enable specification of a log target.
pub trait LogTarget {
	/// The log target.
	const LOG_TARGET: &'static str;
}

impl LogTarget for () {
	const LOG_TARGET: &'static str = "pop-chain-extension";
}

#[test]
fn default_log_target_works() {
	assert!(matches!(<() as LogTarget>::LOG_TARGET, "pop-chain-extension"));
}

#[cfg(test)]
mod extension {
	use codec::Encode;
	use frame_system::Call;

	use super::*;
	use crate::mock::{
		new_test_ext, DispatchExtFuncId, MockEnvironment, NoopFuncId, ReadExtFuncId, RuntimeCall,
		RuntimeRead, Test, INVALID_FUNC_ID,
	};

	#[test]
	fn call_works() {
		let input = vec![2, 2];
		let mut env = MockEnvironment::new(NoopFuncId::get(), input.clone());
		let mut extension = Extension::<mock::Config>::default();
		assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
		// Charges weight.
		assert_eq!(env.charged(), overhead_weight(input.len() as u32))
	}

	#[test]
	fn calling_unknown_function_fails() {
		let input = vec![2, 2];
		// No function registered for id 0.
		let mut env = MockEnvironment::new(INVALID_FUNC_ID, input.clone());
		let mut extension = Extension::<mock::Config>::default();
		assert!(matches!(
			extension.call(&mut env),
			Err(error) if error == pallet_revive::Error::<Test>::DecodingFailed.into()
		));
		// Charges weight.
		assert_eq!(env.charged(), overhead_weight(input.len() as u32))
	}

	#[test]
	fn dispatch_call_works() {
		new_test_ext().execute_with(|| {
			let call =
				RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() });
			let encoded_call = call.encode();
			let mut env = MockEnvironment::new(DispatchExtFuncId::get(), encoded_call.clone());
			let mut extension = Extension::<mock::Config>::default();
			assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
			// Charges weight.
			assert_eq!(
				env.charged(),
				overhead_weight(encoded_call.len() as u32) +
					read_from_buffer_weight(encoded_call.len() as u32) +
					call.get_dispatch_info().weight
			);
		});
	}

	#[test]
	fn dispatch_call_with_invalid_input_returns_error() {
		// Invalid encoded runtime call.
		let input = vec![0u8, 99];
		let mut env = MockEnvironment::new(DispatchExtFuncId::get(), input.clone());
		let mut extension = Extension::<mock::Config>::default();
		assert!(extension.call(&mut env).is_err());
		// Charges weight.
		assert_eq!(
			env.charged(),
			overhead_weight(input.len() as u32) + read_from_buffer_weight(input.len() as u32)
		);
	}

	#[test]
	fn read_state_works() {
		let read = RuntimeRead::Ping;
		let encoded_read = read.encode();
		let expected = "pop".as_bytes().encode();
		let mut env = MockEnvironment::new(ReadExtFuncId::get(), encoded_read.clone());
		let mut extension = Extension::<mock::Config>::default();
		assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
		// Charges weight.
		assert_eq!(
			env.charged(),
			overhead_weight(encoded_read.len() as u32) +
				read_from_buffer_weight(encoded_read.len() as u32) +
				read.weight() +
				write_to_contract_weight(expected.len() as u32)
		);
		// Check if the contract environment buffer is written correctly.
		assert_eq!(env.buffer, expected);
	}

	#[test]
	fn read_state_with_invalid_input_returns_error() {
		let input = vec![0u8, 99];
		let mut env = MockEnvironment::new(
			ReadExtFuncId::get(),
			// Invalid runtime state read.
			input.clone(),
		);
		let mut extension = Extension::<mock::Config>::default();
		assert!(extension.call(&mut env).is_err());
		// Charges weight.
		assert_eq!(
			env.charged(),
			overhead_weight(input.len() as u32) + read_from_buffer_weight(input.len() as u32)
		);
	}

	// Weight charged for calling into the runtime from a contract.
	fn overhead_weight(input_len: u32) -> Weight {
		ContractWeightsOf::<Test>::seal_debug_message(input_len)
	}

	// Weight charged for reading function call input from buffer.
	pub(crate) fn read_from_buffer_weight(input_len: u32) -> Weight {
		ContractWeightsOf::<Test>::seal_return(input_len)
	}

	// Weight charged for writing to contract memory.
	pub(crate) fn write_to_contract_weight(len: u32) -> Weight {
		ContractWeightsOf::<Test>::seal_input(len)
	}
}

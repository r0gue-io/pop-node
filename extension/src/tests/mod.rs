use crate::mock::RuntimeResult;
use crate::tests::utils::decoding_failed_weight;
use crate::{
	environment::Ext,
	mock::{
		new_test_ext, Config, DispatchCallEverthingFuncId, MockEnvironment, MockExt, NoopFuncId,
		ReadStateEverthingFuncId, RuntimeCall, RuntimeRead, Test,
	},
	Environment, Extension,
};
use codec::Encode;
use frame_system::Call;
use pallet_contracts::chain_extension::RetVal::Converging;
pub(crate) use utils::{
	extension_call_dispatch_call_weight, extension_call_read_state_weight,
	function_dispatch_call_weight, function_read_state_weight, overhead_weight,
	read_from_buffer_weight,
};

mod contract;
mod utils;

#[test]
fn extension_call_works() {
	let input = vec![2, 2];
	let mut env = MockEnvironment::new(NoopFuncId::get(), input.clone(), MockExt::default());
	let mut extension = Extension::<Config>::default();
	assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
	assert_eq!(env.charged(), overhead_weight(input.len() as u32))
}

#[test]
fn extension_for_unknown_function_works() {
	let input = vec![2, 2];
	// No function registered for id 0.
	let mut env = MockEnvironment::new(0, input.clone(), MockExt::default());
	let mut extension = Extension::<Config>::default();
	assert!(matches!(
		extension.call(&mut env),
		Err(error) if error == pallet_contracts::Error::<Test>::DecodingFailed.into()
	));
	// Charges weight.
	assert_eq!(env.charged(), overhead_weight(input.len() as u32))
}

#[test]
fn extension_call_dispatch_call_works() {
	new_test_ext().execute_with(|| {
		let call =
			RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() });
		// Insert one byte due to the `RemoveFirstByte` conversion configuration of `DispatchCallEverythingFuncId`.
		let encoded_call = [0u8.encode(), call.encode()].concat();
		let mut default_env = MockEnvironment::default();
		let mut env = MockEnvironment::new(
			DispatchCallEverthingFuncId::get(),
			encoded_call.clone(),
			MockExt::default(),
		);
		let mut extension = Extension::<Config>::default();
		assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
		assert_eq!(
			env.charged(),
			extension_call_dispatch_call_weight(
				&mut default_env,
				encoded_call.len() as u32,
				call,
				env.ext().address().clone()
			)
		);
	});
}

#[test]
fn extension_call_dispatch_call_invalid() {
	// Invalid encoded runtime call.
	let input = vec![0u8, 99];
	let mut env =
		MockEnvironment::new(DispatchCallEverthingFuncId::get(), input.clone(), MockExt::default());
	let mut extension = Extension::<Config>::default();
	assert!(extension.call(&mut env).is_err());
	assert_eq!(env.charged(), decoding_failed_weight(input.len() as u32));
}

#[test]
fn extension_call_read_state_works() {
	let read = RuntimeRead::Ping;
	// Insert one byte due to the `RemoveFirstByte` conversion configuration of `ReadStateEverythingFuncId`.
	let encoded_read = [0u8.encode(), read.encode()].concat();
	let read_result = RuntimeResult::Pong("pop".to_string());
	let expected = "pop".as_bytes().encode();
	let mut default_env = MockEnvironment::default();
	let mut env = MockEnvironment::new(
		ReadStateEverthingFuncId::get(),
		encoded_read.clone(),
		MockExt::default(),
	);
	let mut extension = Extension::<Config>::default();
	assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
	// Check that the two environments charged the same weights.
	assert_eq!(
		env.charged(),
		extension_call_read_state_weight(
			&mut default_env,
			encoded_read.len() as u32,
			read,
			read_result,
		)
	);
	// Check if the contract environment buffer is written correctly.
	assert_eq!(env.buffer, expected);
}

#[test]
fn extension_call_read_state_invalid() {
	let input = vec![0u8, 99];
	let mut env = MockEnvironment::new(
		ReadStateEverthingFuncId::get(),
		// Invalid runtime state read.
		input.clone(),
		MockExt::default(),
	);
	let mut extension = Extension::<Config>::default();
	assert!(extension.call(&mut env).is_err());
	assert_eq!(env.charged(), decoding_failed_weight(input.len() as u32));
}

use crate::{
	mock::{
		new_test_ext, Config, DispatchExtFuncId, MockEnvironment, NoopFuncId, ReadExtFuncId,
		RuntimeCall, RuntimeRead, RuntimeResult, Test,
	},
	tests::utils::decoding_failed_weight,
	Environment as _, Ext as _, Extension,
};
use codec::Encode;
use frame_system::Call;
use pallet_contracts::chain_extension::RetVal::Converging;
pub(crate) use utils::{
	charge_weight_filtering_read_state, extension_call_dispatch_call_weight,
	extension_call_read_state_weight, function_read_state_weight, overhead_weight,
	read_from_buffer_weight,
};

mod contract;
mod utils;

#[test]
fn extension_call_works() {
	let input = vec![2, 2];
	let mut env = MockEnvironment::new(NoopFuncId::get(), input.clone());
	let mut extension = Extension::<Config>::default();
	assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
	assert_eq!(env.charged(), overhead_weight(input.len() as u32))
}

#[test]
fn extension_for_unknown_function_fails() {
	let input = vec![2, 2];
	// No function registered for id 0.
	let mut env = MockEnvironment::new(0, input.clone());
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
		let encoded_call = call.encode();
		let mut default_env = MockEnvironment::default();
		let mut env = MockEnvironment::new(DispatchExtFuncId::get(), encoded_call.clone());
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
	let mut env = MockEnvironment::new(DispatchExtFuncId::get(), input.clone());
	let mut extension = Extension::<Config>::default();
	assert!(extension.call(&mut env).is_err());
	assert_eq!(env.charged(), decoding_failed_weight(input.len() as u32));
}

#[test]
fn extension_call_read_state_works() {
	let read = RuntimeRead::Ping;
	let encoded_read = read.encode();
	let read_result = RuntimeResult::Pong("pop".to_string());
	let expected = "pop".as_bytes().encode();
	let mut default_env = MockEnvironment::default();
	let mut env = MockEnvironment::new(ReadExtFuncId::get(), encoded_read.clone());
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
		ReadExtFuncId::get(),
		// Invalid runtime state read.
		input.clone(),
	);
	let mut extension = Extension::<Config>::default();
	assert!(extension.call(&mut env).is_err());
	assert_eq!(env.charged(), decoding_failed_weight(input.len() as u32));
}
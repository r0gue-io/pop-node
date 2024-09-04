use crate::{
	mock::{
		new_test_ext, Config, DispatchExtFuncId, MockEnvironment, NoopFuncId, ReadExtFuncId,
		RuntimeCall, RuntimeRead, Test, *,
	},
	ContractWeights, Extension, Readable, Weight,
};
use codec::Encode;
use frame_support::dispatch::GetDispatchInfo;
use frame_system::Call;
use pallet_contracts::{chain_extension::RetVal::Converging, WeightInfo};

mod contract;

#[test]
fn extension_call_works() {
	let input = vec![2, 2];
	let mut env = MockEnvironment::new(NoopFuncId::get(), input.clone());
	let mut extension = Extension::<Config>::default();
	assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
	// Charges weight.
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
		let mut env = MockEnvironment::new(DispatchExtFuncId::get(), encoded_call.clone());
		let mut extension = Extension::<Config>::default();
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
fn extension_call_dispatch_call_invalid() {
	// Invalid encoded runtime call.
	let input = vec![0u8, 99];
	let mut env = MockEnvironment::new(DispatchExtFuncId::get(), input.clone());
	let mut extension = Extension::<Config>::default();
	assert!(extension.call(&mut env).is_err());
	// Charges weight.
	assert_eq!(
		env.charged(),
		overhead_weight(input.len() as u32) + read_from_buffer_weight(input.len() as u32)
	);
}

#[test]
fn extension_call_read_state_works() {
	let read = RuntimeRead::Ping;
	let encoded_read = read.encode();
	let expected = "pop".as_bytes().encode();
	let mut env = MockEnvironment::new(ReadExtFuncId::get(), encoded_read.clone());
	let mut extension = Extension::<Config>::default();
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
fn extension_call_read_state_invalid() {
	let input = vec![0u8, 99];
	let mut env = MockEnvironment::new(
		ReadExtFuncId::get(),
		// Invalid runtime state read.
		input.clone(),
	);
	let mut extension = Extension::<Config>::default();
	assert!(extension.call(&mut env).is_err());
	// Charges weight.
	assert_eq!(
		env.charged(),
		overhead_weight(input.len() as u32) + read_from_buffer_weight(input.len() as u32)
	);
}

// Weight charged for calling into the runtime from a contract.
fn overhead_weight(input_len: u32) -> Weight {
	ContractWeights::<Test>::seal_debug_message(input_len)
}

// Weight charged for reading function call input from buffer.
pub(crate) fn read_from_buffer_weight(input_len: u32) -> Weight {
	ContractWeights::<Test>::seal_return(input_len)
}

// Weight charged for writing to contract memory.
pub(crate) fn write_to_contract_weight(len: u32) -> Weight {
	ContractWeights::<Test>::seal_input(len)
}

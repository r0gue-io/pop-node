use crate::{
	environment::{BufOut, Ext},
	functions::{Converter, Readable},
	mock::*,
	ContractWeights, DefaultConverter, Dispatchable, Environment as _, GetDispatchInfo, RawOrigin,
	Weight,
};
use frame_support::assert_ok;
use pallet_contracts::WeightInfo;

// Weight charged for calling into the runtime from a contract.
pub(crate) fn overhead_weight(input_len: u32) -> Weight {
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

// Weight charged after decoding failed.
pub(crate) fn decoding_failed_weight(input_len: u32) -> Weight {
	overhead_weight(input_len) + read_from_buffer_weight(input_len)
}

// Weight charged for executing a dispatch call with the chain extension.
pub(crate) fn extension_call_dispatch_call_weight<E: Ext<Config = Test> + Clone + Default>(
	default_env: &mut Environment<E>,
	input_len: u32,
	call: RuntimeCall,
	contract: <Test as frame_system::Config>::AccountId,
) -> Weight {
	assert_ok!(default_env.charge_weight(ContractWeights::<Test>::seal_debug_message(input_len)));
	function_dispatch_call_weight(default_env, input_len, call, contract)
}

// Weight charged for executing the function dispatch call.
pub(crate) fn function_dispatch_call_weight<E: Ext<Config = Test> + Clone + Default>(
	default_env: &mut Environment<E>,
	input_len: u32,
	call: RuntimeCall,
	contract: <Test as frame_system::Config>::AccountId,
) -> Weight {
	assert_ok!(default_env.charge_weight(read_from_buffer_weight(input_len)));
	// Charge pre-dispatch weight.
	let dispatch_info = call.get_dispatch_info();
	let charged = default_env.charge_weight(dispatch_info.weight).expect("should work");
	// Dispatch call.
	let origin: <Test as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(contract).into();
	let result = call.dispatch(origin);
	// Adjust weight.
	let weight = frame_support::dispatch::extract_actual_weight(&result, &dispatch_info);
	default_env.adjust_weight(charged, weight);
	default_env.charged()
}

// Weight charged for executing a runtime state read with the chain extension.
pub(crate) fn extension_call_read_state_weight<E: Ext<Config = Test> + Clone + Default>(
	default_env: &mut Environment<E>,
	input_len: u32,
	read: RuntimeRead,
	read_result: RuntimeResult,
) -> Weight {
	assert_ok!(default_env.charge_weight(ContractWeights::<Test>::seal_debug_message(input_len)));
	function_read_state_weight(default_env, input_len, read, read_result)
}

// Weight charged for executing the function runtime read state.
pub(crate) fn function_read_state_weight<E: Ext<Config = Test> + Clone + Default>(
	default_env: &mut Environment<E>,
	input_len: u32,
	read: RuntimeRead,
	read_result: RuntimeResult,
) -> Weight {
	assert_ok!(default_env.charge_weight(read_from_buffer_weight(input_len)));
	// Charge weight for reading state.
	assert_ok!(default_env.charge_weight(read.weight()));
	// Charge weight for writing to contract buffer, based on input length, after conversion.
	let result = DefaultConverter::<RuntimeResult>::convert(read_result, default_env);
	let weight_per_byte = ContractWeights::<Test>::seal_input_per_byte(1);
	assert_ok!(default_env.write(&result, false, Some(weight_per_byte)));
	default_env.charged()
}

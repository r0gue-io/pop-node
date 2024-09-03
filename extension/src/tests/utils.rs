use crate::mock::*;
use crate::{
	environment::{BufOut, Ext},
	functions::{Converter, Readable},
	ContractWeights, DefaultConverter, Dispatchable, Environment, GetDispatchInfo, RawOrigin,
	Weight,
};
use codec::Encode;
use core::fmt::Debug;
use frame_support::assert_ok;
use pallet_contracts::{Code, CollectEvents, ContractExecResult, Determinism, WeightInfo};
use std::path::Path;

/// Initializing a new contract file if it does not exist.
pub(crate) fn initialize_contract(contract_path: &str) -> Vec<u8> {
	if !Path::new(contract_path).exists() {
		use contract_build::*;
		let manifest_path = ManifestPath::new("contract/Cargo.toml").unwrap();
		let args = ExecuteArgs {
			build_artifact: BuildArtifacts::CodeOnly,
			build_mode: BuildMode::Debug,
			manifest_path,
			output_type: OutputType::Json,
			verbosity: Verbosity::Quiet,
			skip_wasm_validation: true,
			..Default::default()
		};
		execute(args).unwrap();
	}
	std::fs::read(contract_path).unwrap()
}

/// Instantiating the contract.
pub(crate) fn instantiate(contract: Vec<u8>) -> AccountId {
	let result = Contracts::bare_instantiate(
		ALICE,
		0,
		GAS_LIMIT,
		None,
		Code::Upload(contract),
		function_selector("new"),
		Default::default(),
		DEBUG_OUTPUT,
		CollectEvents::UnsafeCollect,
	);
	log::debug!("instantiate result: {result:?}");
	let result = result.result.unwrap();
	assert!(!result.result.did_revert());
	result.account_id
}

/// Perform a call to a specified contract.
pub(crate) fn call(
	contract: AccountId,
	func_id: u32,
	input: impl Encode + Debug,
	gas_limit: Weight,
) -> ContractExecResult<Balance, EventRecord> {
	log::debug!("call: func_id={func_id}, input={input:?}");
	let result = Contracts::bare_call(
		ALICE,
		contract,
		0,
		gas_limit,
		None,
		[function_selector("call"), (func_id, input.encode()).encode()].concat(),
		DEBUG_OUTPUT,
		CollectEvents::UnsafeCollect,
		Determinism::Enforced,
	);
	log::debug!("gas consumed: {:?}", result.gas_consumed);
	log::debug!("call result: {result:?}");
	result
}

/// Construct the hashed bytes as a selector of function.
pub(crate) fn function_selector(name: &str) -> Vec<u8> {
	sp_io::hashing::blake2_256(name.as_bytes())[0..4].to_vec()
}

// Weight charged for calling into the runtime from a contract.
pub(crate) fn overhead_weight(input_len: u32) -> Weight {
	ContractWeights::<Test>::seal_debug_message(input_len)
}

// Weight charged for reading function call input from buffer.
pub(crate) fn read_from_buffer_weight(input_len: u32) -> Weight {
	ContractWeights::<Test>::seal_return(input_len)
}

// Weight charged after decoding failed.
pub(crate) fn decoding_failed_weight(input_len: u32) -> Weight {
	overhead_weight(input_len) + read_from_buffer_weight(input_len)
}

// Weight charged for executing a dispatch call with the chain extension.
pub(crate) fn extension_call_dispatch_call_weight<E: Ext<Config = Test> + Clone>(
	default_env: &mut MockEnvironment<E>,
	input_len: u32,
	call: RuntimeCall,
	contract: <Test as frame_system::Config>::AccountId,
) -> Weight {
	assert_ok!(default_env.charge_weight(ContractWeights::<Test>::seal_debug_message(input_len)));
	function_dispatch_call_weight(default_env, input_len, call, contract)
}

// Environment charges weight for before the dispatch call.
pub(crate) fn charge_weight_filtering_dispatch_call<E: Ext<Config = Test> + Clone>(
	default_env: &mut MockEnvironment<E>,
	input_len: u32,
	call: RuntimeCall,
	contract: <Test as frame_system::Config>::AccountId,
) {
	let dispatch_info = call.get_dispatch_info();
	assert_ok!(default_env.charge_weight(read_from_buffer_weight(input_len)));
	// Charge pre-dispatch weight.
	default_env.charge_weight(dispatch_info.weight).expect("should work");
	// Dispatch call.
	let origin: <Test as frame_system::Config>::RuntimeOrigin = RawOrigin::Signed(contract).into();
	assert_ok!(call.dispatch(origin));
}

// Environment charges weight for before the dispatch call.
pub(crate) fn charge_weight_filtering_read_state<E: Ext<Config = Test> + Clone>(
	default_env: &mut MockEnvironment<E>,
	input_len: u32,
	read: RuntimeRead,
) {
	assert_ok!(default_env.charge_weight(read_from_buffer_weight(input_len)));
	// Charge weight for reading state.
	assert_ok!(default_env.charge_weight(read.weight()));
}

// Weight charged for executing the function dispatch call.
pub(crate) fn function_dispatch_call_weight<E: Ext<Config = Test> + Clone>(
	default_env: &mut MockEnvironment<E>,
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
pub(crate) fn extension_call_read_state_weight<E: Ext<Config = Test> + Clone>(
	default_env: &mut MockEnvironment<E>,
	input_len: u32,
	read: RuntimeRead,
	read_result: RuntimeResult,
) -> Weight {
	assert_ok!(default_env.charge_weight(ContractWeights::<Test>::seal_debug_message(input_len)));
	function_read_state_weight(default_env, input_len, read, read_result)
}

// Weight charged for executing the function runtime read state.
pub(crate) fn function_read_state_weight<E: Ext<Config = Test> + Clone>(
	default_env: &mut MockEnvironment<E>,
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

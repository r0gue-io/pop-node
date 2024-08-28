use crate::mock::{Test as Runtime, *};
use codec::Encode;
use core::fmt::Debug;
use frame_support::{pallet_prelude::Weight, traits::fungible::Inspect};
use pallet_contracts::{Code, CollectEvents, ContractExecResult, Determinism};
use std::path::Path;

pub(crate) type AccountId = <Test as frame_system::Config>::AccountId;
pub(crate) type Balance = <<Test as pallet_contracts::Config>::Currency as Inspect<
	<Test as frame_system::Config>::AccountId,
>>::Balance;
pub(crate) type EventRecord = frame_system::EventRecord<
	<Test as frame_system::Config>::RuntimeEvent,
	<Test as frame_system::Config>::Hash,
>;

pub(crate) const ALICE: u64 = 1;
pub(crate) const DEBUG_OUTPUT: pallet_contracts::DebugInfo =
	pallet_contracts::DebugInfo::UnsafeDebug;
pub(crate) const GAS_LIMIT: Weight = Weight::from_parts(100_000_000_000, 3 * 1024 * 1024);
pub(crate) const INIT_AMOUNT: <Runtime as pallet_balances::Config>::Balance = 100_000_000;
pub(crate) const INVALID_FUNC_ID: u32 = 0;

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
/// TODO: Parameters
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
/// TODO: Parameters
pub(crate) fn function_selector(name: &str) -> Vec<u8> {
	sp_io::hashing::blake2_256(name.as_bytes())[0..4].to_vec()
}

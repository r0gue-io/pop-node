#![cfg(test)]
use super::*;
use crate::{Assets, Balances, Contracts, Runtime, System};
use pallet_contracts::{Code, CollectEvents, Determinism, ExecReturnValue};
use sp_runtime::{traits::Hash, AccountId32, BuildStorage};

mod local_fungibles;

const DEBUG_OUTPUT: pallet_contracts::DebugInfo = pallet_contracts::DebugInfo::UnsafeDebug;

const ALICE: AccountId32 = AccountId32::new([1_u8; 32]);
const BOB: AccountId32 = AccountId32::new([2_u8; 32]);
const INIT_VALUE: u128 = 100_000_000 * UNIT;
const GAS_LIMIT: Weight = Weight::from_parts(100_000_000_000, 3 * 1024 * 1024);

fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.expect("Frame system builds valid default genesis config");

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INIT_VALUE), (BOB, INIT_VALUE)],
	}
	.assimilate_storage(&mut t)
	.expect("Pallet balances storage can be assimilated");

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn load_wasm_module<T>(path: &str) -> std::io::Result<(Vec<u8>, <T::Hashing as Hash>::Output)>
where
	T: frame_system::Config,
{
	let wasm_binary = std::fs::read(path)?;
	let code_hash = T::Hashing::hash(&wasm_binary);
	Ok((wasm_binary, code_hash))
}

fn function_selector(name: &str) -> Vec<u8> {
	let hash = sp_io::hashing::blake2_256(name.as_bytes());
	[hash[0..4].to_vec()].concat()
}

fn do_bare_call(
	addr: AccountId32,
	input: Vec<u8>,
	value: u128,
) -> Result<ExecReturnValue, DispatchError> {
	let result = Contracts::bare_call(
		ALICE,
		addr.into(),
		value.into(),
		GAS_LIMIT,
		None,
		input,
		DEBUG_OUTPUT,
		CollectEvents::Skip,
		Determinism::Enforced,
	);
	log::debug!("Contract debug buffer - {:?}", String::from_utf8(result.debug_message.clone()));
	log::debug!("result: {:?}", result);
	result.result
}

// Deploy, instantiate and return contract address.
fn instantiate(contract: &str, init_value: u128) -> AccountId32 {
	let (wasm_binary, _) =
		load_wasm_module::<Runtime>(contract).expect("could not read .wasm file");
	let result = Contracts::bare_instantiate(
		ALICE,
		init_value,
		GAS_LIMIT,
		None,
		Code::Upload(wasm_binary),
		function_selector("new"),
		vec![],
		DEBUG_OUTPUT,
		CollectEvents::Skip,
	)
	.result
	.unwrap();
	assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
	result.account_id
}

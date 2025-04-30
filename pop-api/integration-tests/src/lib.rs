#![cfg(test)]

use frame_support::{
	assert_ok,
	pallet_prelude::{Decode, Encode},
	traits::fungibles::{
		approvals::Inspect as _, metadata::Inspect as _, roles::Inspect as _, Inspect,
	},
	weights::Weight,
};
use pallet_contracts::{Code, CollectEvents, Determinism, ExecReturnValue};
#[cfg(feature = "devnet")]
use pop_runtime_devnet::{Assets, Contracts, Nfts, Runtime, RuntimeOrigin, System, UNIT};
#[cfg(feature = "testnet")]
use pop_runtime_testnet::{Assets, Contracts, Nfts, Runtime, RuntimeOrigin, System, UNIT};
use sp_runtime::{AccountId32, BuildStorage, DispatchError};
use utils::*;

mod fungibles;
#[cfg(feature = "testnet")]
mod messaging;
#[cfg(feature = "devnet")]
mod nonfungibles;
mod utils;

type Balance = u128;

const ALICE: AccountId32 = AccountId32::new([1_u8; 32]);
const BOB: AccountId32 = AccountId32::new([2_u8; 32]);
const DEBUG_OUTPUT: pallet_contracts::DebugInfo = pallet_contracts::DebugInfo::UnsafeDebug;
const FERDIE: AccountId32 = AccountId32::new([3_u8; 32]);
const GAS_LIMIT: Weight = Weight::from_parts(500_000_000_000, 100 * 1024 * 1024);
const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
const INIT_VALUE: Balance = 100 * UNIT;

fn new_test_ext() -> sp_io::TestExternalities {
	let _ = env_logger::try_init();

	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.expect("Frame system builds valid default genesis config");

	pallet_balances::GenesisConfig::<Runtime> {
		// FERDIE has no initial balance.
		balances: vec![(ALICE, INIT_AMOUNT), (BOB, INIT_AMOUNT)],
	}
	.assimilate_storage(&mut t)
	.expect("Pallet balances storage can be assimilated");

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		#[cfg(feature = "testnet")]
		pallet_timestamp::Pallet::<Runtime>::set_timestamp(1u64);
		System::set_block_number(1)
		}
	);
	ext
}

fn function_selector(name: &str) -> Vec<u8> {
	let hash = sp_io::hashing::blake2_256(name.as_bytes());
	[hash[0..4].to_vec()].concat()
}

fn bare_call(
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
	log::info!("contract exec result={result:?}");
	result.result
}

// Deploy, instantiate and return contract address.
fn instantiate(contract: &str, init_value: u128, salt: Vec<u8>) -> AccountId32 {
	let wasm_binary = std::fs::read(contract).expect("could not read .wasm file");

	let result = Contracts::bare_instantiate(
		ALICE,
		init_value,
		GAS_LIMIT,
		None,
		Code::Upload(wasm_binary),
		function_selector("new"),
		salt,
		DEBUG_OUTPUT,
		CollectEvents::Skip,
	)
	.result
	.unwrap();
	assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
	result.account_id
}

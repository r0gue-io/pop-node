#![cfg(test)]

use frame_support::{
	assert_ok,
	traits::fungibles::{
		approvals::Inspect as _, metadata::Inspect as _, roles::Inspect as _, Inspect,
	},
	weights::Weight,
};
use pallet_revive::{AddressMapper, Code, CollectEvents, ExecReturnValue};
use pop_runtime_devnet::{
	config::ismp::Router, Assets, Messaging, Nfts, Revive, Runtime, RuntimeOrigin, System, UNIT,
};
use scale::{Decode, Encode};
use sp_runtime::{
	app_crypto::sp_core,
	offchain::{testing::TestOffchainExt, OffchainDbExt},
	AccountId32, BuildStorage, DispatchError,
};

mod environment;
mod fungibles;
mod incentives;
mod messaging;
mod nonfungibles;

type Balance = u128;

const ALICE: AccountId32 = AccountId32::new([1_u8; 32]);
const BOB: AccountId32 = AccountId32::new([2_u8; 32]);
const DEBUG_OUTPUT: pallet_revive::DebugInfo = pallet_revive::DebugInfo::UnsafeDebug;
const FERDIE: AccountId32 = AccountId32::new([3_u8; 32]);
const GAS_LIMIT: Weight = Weight::from_parts(500_000_000_000, 3 * 1024 * 1024);
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
	let (offchain, _state) = TestOffchainExt::new();
	ext.register_extension(OffchainDbExt::new(offchain));
	ext.execute_with(|| System::set_block_number(1));
	// register account mappings
	ext.execute_with(|| {
		Revive::map_account(RuntimeOrigin::signed(ALICE)).unwrap();
		Revive::map_account(RuntimeOrigin::signed(BOB)).unwrap();
	});
	ext
}

fn function_selector(name: &str) -> Vec<u8> {
	let hash = sp_io::hashing::blake2_256(name.as_bytes());
	[hash[0..4].to_vec()].concat()
}

fn bare_call(
	addr: sp_core::H160,
	input: Vec<u8>,
	value: u128,
) -> Result<ExecReturnValue, DispatchError> {
	let result = Revive::bare_call(
		RuntimeOrigin::signed(ALICE),
		addr.into(),
		value.into(),
		GAS_LIMIT,
		1 * 1_000_000_000_000,
		input,
		DEBUG_OUTPUT,
		CollectEvents::Skip,
	);
	log::info!("contract exec result={result:?}");
	result.result
}

// Deploy, instantiate and return contract address.
fn instantiate(
	contract: &str,
	init_value: u128,
	data: Vec<u8>,
	_salt: Vec<u8>,
) -> (sp_core::H160, AccountId32) {
	let wasm_binary = std::fs::read(contract).expect("could not read .wasm file");

	let result = Revive::bare_instantiate(
		RuntimeOrigin::signed(ALICE),
		init_value,
		GAS_LIMIT,
		1 * 1_000_000_000_000,
		Code::Upload(wasm_binary),
		data,
		None,
		DEBUG_OUTPUT,
		CollectEvents::Skip,
	)
	.result
	.unwrap();
	assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
	(result.addr, pallet_revive::AccountId32Mapper::<Runtime>::to_account_id(&result.addr))
}

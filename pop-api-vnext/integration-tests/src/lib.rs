#![cfg(test)]

use std::path::Path;

use frame_support::{assert_ok, weights::Weight};
use pallet_assets::{Instance1, NextAssetId};
use pallet_revive::{
	precompiles::alloy::{primitives as alloy, sol_types::SolCall},
	test_utils::{ALICE, BOB, CHARLIE},
	AccountId32Mapper, AddressMapper, Code, DepositLimit, ExecReturnValue, H160, U256,
};
#[cfg(feature = "devnet")]
use pop_runtime_devnet::{
	AccountId, Assets, Balance, Messaging, Revive, Runtime, RuntimeEvent, RuntimeOrigin, System,
	UNIT,
};
use sp_runtime::{BuildStorage, DispatchError};

mod fungibles;
mod messaging;

const INIT_AMOUNT: Balance = 100_000_000 * UNIT;

type TokenId = u32;

// Get the last event from `pallet-revive`.
fn last_contract_event(address: &H160) -> Vec<u8> {
	let events = System::read_events_for_pallet::<pallet_revive::Event<Runtime>>();
	let contract_events = events
		.iter()
		.filter_map(|event| match event {
			pallet_revive::Event::<Runtime>::ContractEmitted { contract, data, .. }
				if contract == address =>
				Some(data.as_slice()),
			_ => None,
		})
		.collect::<Vec<&[u8]>>();
	contract_events
		.last()
		.expect("expected an event for the specified contract")
		.to_vec()
}

fn bare_call(
	origin: RuntimeOrigin,
	dest: H160,
	value: Balance,
	gas_limit: Weight,
	storage_deposit_limit: DepositLimit<Balance>,
	data: Vec<u8>,
) -> Result<ExecReturnValue, DispatchError> {
	let result = Revive::bare_call(origin, dest, value, gas_limit, storage_deposit_limit, data);
	log::info!("contract exec result={result:?}");
	result.result
}

fn blake_selector(input: &str) -> [u8; 4] {
	sp_io::hashing::blake2_256(input.as_bytes())[0..4]
		.try_into()
		.expect("hash length > 4")
}

// Deploy, instantiate and return contract address.
fn instantiate(
	origin: RuntimeOrigin,
	contract: impl AsRef<Path>,
	value: Balance,
	gas_limit: Weight,
	storage_deposit_limit: DepositLimit<Balance>,
	data: Vec<u8>,
	salt: Option<[u8; 32]>,
) -> H160 {
	let binary = std::fs::read(contract).expect("could not read .polkavm file");

	let result = Revive::bare_instantiate(
		origin,
		value,
		gas_limit,
		storage_deposit_limit,
		Code::Upload(binary),
		data,
		salt,
	)
	.result
	.unwrap();
	assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
	result.addr
}

fn keccak_selector(input: &str) -> [u8; 4] {
	sp_io::hashing::keccak_256(input.as_bytes())[0..4]
		.try_into()
		.expect("hash length > 4")
}

fn to_account_id(address: &H160) -> AccountId {
	AccountId32Mapper::<Runtime>::to_account_id(address)
}

fn to_address(account: &AccountId) -> H160 {
	AccountId32Mapper::<Runtime>::to_address(account)
}

pub(crate) struct ExtBuilder {
	assets: Option<Vec<(TokenId, AccountId, bool, Balance)>>,
	asset_accounts: Option<Vec<(TokenId, AccountId, Balance)>>,
	asset_metadata: Option<Vec<(TokenId, Vec<u8>, Vec<u8>, u8)>>,
}

impl ExtBuilder {
	pub(crate) fn new() -> Self {
		Self { assets: None, asset_accounts: None, asset_metadata: None }
	}

	pub(crate) fn with_assets(mut self, assets: Vec<(TokenId, AccountId, bool, Balance)>) -> Self {
		self.assets = Some(assets);
		self
	}

	pub(crate) fn with_asset_balances(
		mut self,
		accounts: Vec<(TokenId, AccountId, Balance)>,
	) -> Self {
		self.asset_accounts = Some(accounts);
		self
	}

	pub(crate) fn with_asset_metadata(
		mut self,
		metadata: Vec<(TokenId, Vec<u8>, Vec<u8>, u8)>,
	) -> Self {
		self.asset_metadata = Some(metadata);
		self
	}

	pub(crate) fn build(mut self) -> sp_io::TestExternalities {
		let _ = env_logger::try_init();

		let mut t = frame_system::GenesisConfig::<Runtime>::default()
			.build_storage()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Runtime> {
			// DJANGO has no initial balance.
			balances: vec![(ALICE, INIT_AMOUNT), (BOB, INIT_AMOUNT), (CHARLIE, INIT_AMOUNT)],
			..Default::default()
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		pallet_assets::GenesisConfig::<Runtime, pallet_assets::Instance1> {
			assets: self.assets.take().unwrap_or_default(),
			metadata: self.asset_metadata.take().unwrap_or_default(),
			accounts: self.asset_accounts.take().unwrap_or_default(),
			next_asset_id: Some(0),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);
			NextAssetId::<Runtime, Instance1>::put(1);
		});

		ext
	}
}

#[test]
fn selectors_work() {
	// Constructors currently still use blake encoding
	assert_eq!(hex::encode(blake_selector("new")), "9bae9d5e");

	// Erc20 selectors
	assert_eq!(hex::encode(keccak_selector("exists()")), "267c4ae4");
	assert_eq!(hex::encode(keccak_selector("totalSupply()")), "18160ddd");
	assert_eq!(hex::encode(keccak_selector("balanceOf(address)")), "70a08231");
}

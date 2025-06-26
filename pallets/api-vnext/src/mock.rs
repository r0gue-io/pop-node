use codec::Compact;
use frame_support::{
	derive_impl,
	sp_runtime::{traits::AccountIdLookup, AccountId32, BuildStorage},
	traits::{AsEnsureOriginWithArg, Get},
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_assets::AutoIncAssetId;
pub(crate) use pallet_revive::test_utils::{ALICE, BOB, CHARLIE};

use super::fungibles;

pub(crate) type AccountId = AccountId32;
pub(crate) type Balance = u128;
type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type ExistentialDeposit = <Test as pallet_balances::Config>::ExistentialDeposit;
// For terminology in tests.
pub(crate) type TokenId = u32;

pub(crate) const ERC20: u16 = 2;
pub(crate) const FUNGIBLES: u16 = 1;
pub(crate) const UNIT: Balance = 10_000_000_000;

#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask,
		RuntimeViewFunction
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type Assets = pallet_assets::Pallet<Runtime>;
	#[runtime::pallet_index(1)]
	pub type Balances = pallet_balances::Pallet<Runtime>;
	#[runtime::pallet_index(2)]
	pub type Contracts = pallet_revive::Pallet<Runtime>;
	#[runtime::pallet_index(3)]
	pub type System = frame_system::Pallet<Runtime>;
	#[runtime::pallet_index(4)]
	pub type Timestamp = pallet_timestamp::Pallet<Runtime>;
	#[runtime::pallet_index(5)]
	pub type Fungibles = fungibles::Pallet<Runtime>;
}

#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig)]
impl pallet_assets::Config for Test {
	type AssetIdParameter = Compact<TokenId>;
	type Balance = Balance;
	type CallbackHandle = AutoIncAssetId<Test>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = Balance;
}

#[derive_impl(pallet_revive::config_preludes::TestDefaultConfig)]
impl pallet_revive::Config for Test {
	type AddressMapper = pallet_revive::AccountId32Mapper<Self>;
	type Currency = Balances;
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	type Precompiles = (
		fungibles::precompiles::v0::Fungibles<FUNGIBLES, Test>,
		fungibles::precompiles::erc20::v0::Erc20<ERC20, Test>,
	);
	type Time = Timestamp;
	type UploadOrigin = EnsureSigned<Self::AccountId>;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type Block = Block;
	type Lookup = AccountIdLookup<Self::AccountId, ()>;
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig)]
impl pallet_timestamp::Config for Test {}

impl fungibles::Config for Test {
	type WeightInfo = ();
}

pub(crate) struct ExtBuilder {
	assets: Option<Vec<(TokenId, AccountId, bool, Balance)>>,
	asset_accounts: Option<Vec<(TokenId, AccountId, Balance)>>,
	asset_metadata: Option<Vec<(TokenId, Vec<u8>, Vec<u8>, u8)>>,
	balances: Vec<(AccountId, Balance)>,
}
impl ExtBuilder {
	pub(crate) fn new() -> Self {
		Self {
			assets: None,
			asset_accounts: None,
			asset_metadata: None,
			balances: vec![(ALICE, ExistentialDeposit::get())],
		}
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

	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub(crate) fn build(mut self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: self.balances, ..Default::default() }
			.assimilate_storage(&mut t)
			.unwrap();

		pallet_assets::GenesisConfig::<Test> {
			assets: self.assets.take().unwrap_or_default(),
			metadata: self.asset_metadata.take().unwrap_or_default(),
			accounts: self.asset_accounts.take().unwrap_or_default(),
			next_asset_id: Some(0),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub(crate) fn build_with_env<R>(
		self,
		execute: impl FnOnce(pallet_revive::precompiles::run::CallSetup<Test>) -> R,
	) -> R {
		self.build().execute_with(|| {
			let call_setup = pallet_revive::precompiles::run::CallSetup::<Test>::default();
			// let (ext, _) = call_setup.ext();
			execute(call_setup)
		})
	}
}

pub(crate) fn signed(account: AccountId) -> RuntimeOrigin {
	RuntimeOrigin::signed(account)
}

pub(crate) fn root() -> RuntimeOrigin {
	RuntimeOrigin::root()
}

pub(crate) fn none() -> RuntimeOrigin {
	RuntimeOrigin::none()
}

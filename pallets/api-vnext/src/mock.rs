use codec::Compact;
use frame_support::{
	derive_impl,
	sp_runtime::{traits::AccountIdLookup, AccountId32, BuildStorage},
	traits::AsEnsureOriginWithArg,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_assets::AutoIncAssetId;
use pallet_revive::precompiles::run::CallSetup;
pub(crate) use pallet_revive::test_utils::ALICE;

pub(crate) type Balance = u128;
type Block = frame_system::mocking::MockBlock<Test>;
type Erc20<const PREFIX: u16, I> = crate::Erc20<PREFIX, Test, I>;
type Fungibles<const FIXED: u16, I> = crate::Fungibles<FIXED, Test, I>;

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
	pub type Assets = pallet_assets::Pallet<Runtime, Instance1>;
	#[runtime::pallet_index(1)]
	pub type Balances = pallet_balances::Pallet<Runtime>;
	#[runtime::pallet_index(2)]
	pub type Contracts = pallet_revive::Pallet<Runtime>;
	#[runtime::pallet_index(3)]
	pub type System = frame_system::Pallet<Runtime>;
	#[runtime::pallet_index(4)]
	pub type Timestamp = pallet_timestamp::Pallet<Runtime>;
}

#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig)]
impl pallet_assets::Config<pallet_assets::Instance1> for Test {
	type AssetIdParameter = Compact<u32>;
	type Balance = Balance;
	type CallbackHandle = AutoIncAssetId<Test, pallet_assets::Instance1>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId32>;
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
	type Precompiles =
		(Fungibles<100, pallet_assets::Instance1>, Erc20<101, pallet_assets::Instance1>);
	type Time = Timestamp;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId32;
	type Block = Block;
	type Lookup = AccountIdLookup<Self::AccountId, ()>;
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig)]
impl pallet_timestamp::Config for Test {}

pub(crate) struct ExtBuilder {
	assets: Option<Vec<(u32, AccountId32, bool, u128)>>,
	asset_accounts: Option<Vec<(u32, AccountId32, u128)>>,
	asset_metadata: Option<Vec<(u32, Vec<u8>, Vec<u8>, u8)>>,
}
impl ExtBuilder {
	pub(crate) fn new() -> Self {
		Self { assets: None, asset_accounts: None, asset_metadata: None }
	}

	pub(crate) fn with_assets(mut self, assets: Vec<(u32, AccountId32, bool, u128)>) -> Self {
		self.assets = Some(assets);
		self
	}

	pub(crate) fn with_asset_balances(mut self, accounts: Vec<(u32, AccountId32, u128)>) -> Self {
		self.asset_accounts = Some(accounts);
		self
	}

	pub(crate) fn with_asset_metadata(
		mut self,
		metadata: Vec<(u32, Vec<u8>, Vec<u8>, u8)>,
	) -> Self {
		self.asset_metadata = Some(metadata);
		self
	}

	pub(crate) fn build(mut self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Test> {
			balances: vec![(ALICE, 10_000_000_000)],
			..Default::default()
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_assets::GenesisConfig::<Test, pallet_assets::Instance1> {
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

	pub(crate) fn build_with_env<R>(self, execute: impl FnOnce(CallSetup<Test>) -> R) -> R {
		self.build().execute_with(|| {
			let call_setup = CallSetup::<Test>::default();
			// let (ext, _) = call_setup.ext();
			execute(call_setup)
		})
	}
}

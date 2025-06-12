use codec::{Compact, Decode, DecodeWithMemTracking, Encode};
use frame_support::{
	derive_impl, parameter_types,
	sp_runtime::{
		traits::{AccountIdLookup, IdentifyAccount, Lazy, Verify},
		AccountId32, BuildStorage,
	},
	traits::{AsEnsureOriginWithArg, ConstU128, ConstU32, ConstU64},
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_assets::AutoIncAssetId;
use pallet_nfts::PalletFeatures;
use pallet_revive::precompiles::run::CallSetup;
pub(crate) use pallet_revive::test_utils::ALICE;
use scale_info::TypeInfo;

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
	pub type Nfts = pallet_nfts::Pallet<Runtime, Instance1>;
	#[runtime::pallet_index(4)]
	pub type System = frame_system::Pallet<Runtime>;
	#[runtime::pallet_index(5)]
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

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, DecodeWithMemTracking, TypeInfo)]
pub struct Noop;

impl IdentifyAccount for Noop {
	type AccountId = AccountId32;

	fn into_account(self) -> Self::AccountId {
		AccountId32::new([0; 32])
	}
}

impl Verify for Noop {
	type Signer = Noop;

	fn verify<L: Lazy<[u8]>>(
		&self,
		_msg: L,
		_signer: &<Self::Signer as IdentifyAccount>::AccountId,
	) -> bool {
		false
	}
}

parameter_types! {
	pub NftsPalletFeatures: PalletFeatures = PalletFeatures::all_enabled();
}

impl pallet_nfts::Config<pallet_nfts::Instance1> for Test {
	type ApprovalsLimit = ConstU32<10>;
	type AttributeDepositBase = ConstU128<1>;
	type BlockNumberProvider = frame_system::Pallet<Test>;
	type CollectionApprovalDeposit = ConstU128<1>;
	type CollectionBalanceDeposit = ConstU128<1>;
	type CollectionDeposit = ConstU128<2>;
	type CollectionId = u32;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId32>>;
	type Currency = Balances;
	type DepositPerByte = ConstU128<1>;
	type Features = NftsPalletFeatures;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type ItemAttributesApprovalsLimit = ConstU32<2>;
	type ItemDeposit = ConstU128<1>;
	type ItemId = u32;
	type KeyLimit = ConstU32<50>;
	type Locker = ();
	type MaxAttributesPerCall = ConstU32<2>;
	type MaxDeadlineDuration = ConstU64<10000>;
	type MaxTips = ConstU32<10>;
	type MetadataDepositBase = ConstU128<1>;
	type OffchainPublic = Noop;
	type OffchainSignature = Noop;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ConstU32<50>;
	type ValueLimit = ConstU32<50>;
	type WeightInfo = ();
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

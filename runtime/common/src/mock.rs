use frame_support::{
	derive_impl, parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU32},
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot, EnsureSigned};
use pallet_contracts::{
	config_preludes::{
		CodeHashLockupDepositPercent, DefaultDepositLimit, DepositPerByte, DepositPerItem,
		MaxDelegateDependencies,
	},
	DefaultAddressGenerator, Frame, Schedule,
};

type HashOf<T> = <T as frame_system::Config>::Hash;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		Assets: pallet_assets::<Instance1>,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		Contracts: pallet_contracts,
  Fungibles: pallet_api::fungibles,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = u64;
	type Block = frame_system::mocking::MockBlock<Test>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type ReserveIdentifier = [u8; 8];
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig as pallet_timestamp::DefaultConfig)]
impl pallet_timestamp::Config for Test {}

impl pallet_contracts::Config for Test {
	type AddressGenerator = DefaultAddressGenerator;
	type ApiVersion = ();
	type CallFilter = ();
	// TestFilter;
	type CallStack = [Frame<Self>; 5];
	type ChainExtension = ();
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Currency = Balances;
	type Debug = ();
	// TestDebug;
	type DefaultDepositLimit = DefaultDepositLimit;
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type Environment = ();
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	type MaxCodeLen = ConstU32<{ 100 * 1024 }>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type MaxDelegateDependencies = MaxDelegateDependencies;
	type MaxStorageKeyLen = ConstU32<128>;
	type Migrations = ();
	// crate::migration::codegen::BenchMigrations;
	type Randomness = Test;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Schedule = MySchedule;
	type Time = Timestamp;
	type UnsafeUnstableInterface = ();
	// UnstableInterface;
	type UploadOrigin = EnsureSigned<Self::AccountId>;
	type WeightInfo = ();
	type WeightPrice = ();
	// Self;
	type Xcm = ();
}

type AssetsInstance = pallet_assets::Instance1;
#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig as pallet_assets::DefaultConfig)]
impl pallet_assets::Config<AssetsInstance> for Test {
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<u64>>;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<u64>;
	type Freezer = ();
	type RuntimeEvent = RuntimeEvent;
}

impl pallet_api::fungibles::Config for Test {
	type AssetsInstance = AssetsInstance;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

parameter_types! {
	pub MySchedule: Schedule<Test> = {
		let schedule = <Schedule<Test>>::default();
		schedule
	};
}

impl frame_support::traits::Randomness<HashOf<Test>, BlockNumberFor<Test>> for Test {
	fn random(_subject: &[u8]) -> (HashOf<Test>, BlockNumberFor<Test>) {
		(Default::default(), Default::default())
	}
}

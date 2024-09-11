use crate::api::Config;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	sp_runtime::{testing::H256, Perbill},
	traits::{AsEnsureOriginWithArg, ConstBool, ConstU128, ConstU32, Randomness},
	weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_api::fungibles;
use pallet_contracts::{DefaultAddressGenerator, Frame, Schedule};
use sp_runtime::traits::{AccountIdLookup, Convert};
use sp_std::vec::Vec;

mod api;

pub(crate) type AccountId = AccountId32;
pub(crate) type AssetId = u32;
pub(crate) type Balance = u128;

// Define the runtime type as a collection of pallets
construct_runtime!(
	pub enum Runtime {
		System: frame_system,
		Assets: pallet_assets,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		Contracts: pallet_contracts,
		Fungibles: fungibles = 150,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<<Runtime as pallet_balances::Config>::Balance>;
	type AccountId = AccountId;
	type Block = frame_system::mocking::MockBlock<Runtime>;
	type Lookup = AccountIdLookup<AccountId, ()>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type ExistentialDeposit = ConstU128<1>;
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const AssetDeposit: u128 = 1;
	pub const AssetAccountDeposit: u128 = 10;
	pub const ApprovalDeposit: u128 = 1;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: u128 = 1;
	pub const MetadataDepositPerByte: u128 = 1;
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type RemoveItemsLimit = ConstU32<5>;
	type AssetId = AssetId;
	type AssetIdParameter = u32;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type AssetDeposit = ConstU128<1>;
	type AssetAccountDeposit = ConstU128<10>;
	type MetadataDepositBase = ConstU128<1>;
	type MetadataDepositPerByte = ConstU128<1>;
	type ApprovalDeposit = ConstU128<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig as pallet_timestamp::DefaultConfig)]
impl pallet_timestamp::Config for Runtime {}

// Configure pallet contracts
impl Randomness<H256, u64> for Runtime {
	fn random(_subject: &[u8]) -> (H256, u64) {
		(Default::default(), Default::default())
	}
}

// Configure pallet contracts
pub enum SandboxRandomness {}
impl Randomness<H256, u64> for SandboxRandomness {
	fn random(_subject: &[u8]) -> (H256, u64) {
		unreachable!("No randomness")
	}
}

type BalanceOf = <Runtime as pallet_balances::Config>::Balance;
impl Convert<Weight, BalanceOf> for Runtime {
	fn convert(w: Weight) -> BalanceOf {
		w.ref_time().into()
	}
}

parameter_types! {
	pub SandboxSchedule: Schedule<Runtime> = {
		let schedule = <Schedule<Runtime>>::default();
		schedule
	};
	pub DeletionWeightLimit: Weight = Weight::zero();
	pub DefaultDepositLimit: BalanceOf = 10_000_000;
	pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
	pub MaxDelegateDependencies: u32 = 32;
}

impl pallet_contracts::Config for Runtime {
	type AddressGenerator = DefaultAddressGenerator;
	type ApiVersion = ();
	type CallFilter = ();
	// TestFilter;
	type CallStack = [Frame<Self>; 5];
	type ChainExtension = api::Extension<Config>;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Currency = Balances;
	type Debug = drink::pallet_contracts_debugging::DrinkDebug;
	// TestDebug;
	type DefaultDepositLimit = DefaultDepositLimit;
	type DepositPerByte = ConstU128<1>;
	type DepositPerItem = ConstU128<1>;
	type Environment = ();
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	type MaxCodeLen = ConstU32<{ 123 * 1024 }>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type MaxDelegateDependencies = MaxDelegateDependencies;
	type MaxStorageKeyLen = ConstU32<128>;
	type Migrations = ();
	// crate::migration::codegen::BenchMigrations;
	type Randomness = SandboxRandomness;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Schedule = SandboxSchedule;
	type Time = Timestamp;
	type UnsafeUnstableInterface = ConstBool<false>;
	// UnstableInterface;
	type UploadOrigin = EnsureSigned<Self::AccountId>;
	type WeightInfo = ();
	type WeightPrice = Self;
	// Self;
	type Xcm = ();
}

drink::create_sandbox!(PopSandbox, Runtime);

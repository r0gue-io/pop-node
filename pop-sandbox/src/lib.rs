use crate::api::Config;
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	sp_runtime::{testing::H256, Perbill},
	traits::{AsEnsureOriginWithArg, ConstBool, ConstU128, ConstU32, Randomness},
	weights::Weight,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_api::fungibles;
use pallet_contracts::{DefaultAddressGenerator, Frame};
use sp_runtime::traits::{AccountIdLookup, Convert};
use sp_std::vec::Vec;

mod api;
// Provides utlity methods to interact with the sandbox.
pub mod utils;

pub(crate) type AccountId = AccountId32;
pub(crate) type AssetId = u32;
pub(crate) type Balance = u128;

pub const MILLIUNIT: Balance = UNIT / 1_000; // 10_000_000

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	(items as Balance * UNIT + (bytes as Balance) * (5 * MILLIUNIT / 100)) / 10
}

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

fn schedule<T: pallet_contracts::Config>() -> pallet_contracts::Schedule<T> {
	pallet_contracts::Schedule {
		limits: pallet_contracts::Limits {
			runtime_memory: 1024 * 1024 * 1024,
			..Default::default()
		},
		..Default::default()
	}
}

parameter_types! {
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub Schedule: pallet_contracts::Schedule<Runtime> = schedule::<Runtime>();
	pub const DefaultDepositLimit: Balance = deposit(1024, 1024 * 1024);
	pub const CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
}

impl pallet_contracts::Config for Runtime {
	type AddressGenerator = DefaultAddressGenerator;
	type ApiVersion = ();
	// C
	type CallFilter = ();
	type CallStack = [Frame<Self>; 23];
	type ChainExtension = api::Extension<Config>;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Currency = Balances;
	// D
	type Debug = drink::pallet_contracts_debugging::DrinkDebug;
	type DefaultDepositLimit = DefaultDepositLimit;
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	// E
	type Environment = ();
	// I
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	// M
	type MaxCodeLen = ConstU32<{ 256 * 1024 }>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type MaxDelegateDependencies = ConstU32<32>;
	type MaxStorageKeyLen = ConstU32<128>;
	type Migrations = ();
	// R
	type Randomness = SandboxRandomness;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	// S
	type Schedule = Schedule;
	// T
	type Time = Timestamp;
	// U
	type UnsafeUnstableInterface = ConstBool<false>;
	type UploadOrigin = EnsureSigned<Self::AccountId>;
	// W
	type WeightInfo = ();
	type WeightPrice = Self;
	// X
	type Xcm = ();
}

drink::create_sandbox!(PopSandbox, Runtime);

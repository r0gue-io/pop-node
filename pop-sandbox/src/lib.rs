use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	construct_runtime, derive_impl, parameter_types,
	sp_runtime::{testing::H256, Perbill},
	traits::{AsEnsureOriginWithArg, ConstU32, ConstU64, Randomness},
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_api::fungibles;
use pallet_contracts::{DefaultAddressGenerator, Frame, Schedule};
use pop_chain_extension::{CallFilter, ReadState};
use sp_std::vec::Vec;

pub(crate) type AccountId = u64;
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
		Fungibles: fungibles
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = AccountId;
	type Block = frame_system::mocking::MockBlock<Runtime>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Runtime {
	type AccountStore = System;
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
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<u64>>;
	type ForceOrigin = EnsureRoot<u64>;
	type AssetDeposit = ConstU64<1>;
	type AssetAccountDeposit = ConstU64<10>;
	type MetadataDepositBase = ConstU64<1>;
	type MetadataDepositPerByte = ConstU64<1>;
	type ApprovalDeposit = ConstU64<1>;
	type StringLimit = ConstU32<50>;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

impl fungibles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetsInstance = ();
	type WeightInfo = ();
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig as pallet_timestamp::DefaultConfig)]
impl pallet_timestamp::Config for Runtime {}

// Configure pallet contracts
impl Randomness<H256, u64> for Runtime {
	fn random(_subject: &[u8]) -> (H256, u64) {
		(Default::default(), Default::default())
	}
}

// A query of runtime state.
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
#[repr(u8)]
pub enum RuntimeRead {
	/// Fungible token queries.
	#[codec(index = 150)]
	Fungibles(fungibles::Read<Runtime>),
}

/// A struct that implement requirements for the Pop API chain extension.
#[derive(Default)]
pub struct Extension;
impl ReadState for Extension {
	type StateQuery = RuntimeRead;

	fn contains(c: &Self::StateQuery) -> bool {
		use fungibles::Read::*;
		matches!(
			c,
			RuntimeRead::Fungibles(
				TotalSupply(..) |
					BalanceOf { .. } |
					Allowance { .. } |
					TokenName(..) | TokenSymbol(..) |
					TokenDecimals(..) |
					AssetExists(..)
			)
		)
	}

	fn read(read: RuntimeRead) -> Vec<u8> {
		match read {
			RuntimeRead::Fungibles(key) => fungibles::Pallet::read_state(key),
		}
	}
}

impl CallFilter for Extension {
	type Call = RuntimeCall;

	fn contains(c: &Self::Call) -> bool {
		use fungibles::Call::*;
		matches!(
			c,
			RuntimeCall::Fungibles(
				transfer { .. } |
					transfer_from { .. } |
					approve { .. } | increase_allowance { .. } |
					decrease_allowance { .. } |
					create { .. } | set_metadata { .. } |
					start_destroy { .. } |
					clear_metadata { .. } |
					mint { .. } | burn { .. }
			)
		)
	}
}

parameter_types! {
	pub MySchedule: Schedule<Runtime> = {
		let schedule = <Schedule<Runtime>>::default();
		schedule
	};
	pub static DepositPerByte: <Runtime as pallet_balances::Config>::Balance = 1;
	pub const DepositPerItem: <Runtime as pallet_balances::Config>::Balance = 2;
	pub static MaxDelegateDependencies: u32 = 32;
	pub static MaxTransientStorageSize: u32 = 4 * 1024;
	pub static CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
	pub static DefaultDepositLimit: <Runtime as pallet_balances::Config>::Balance = 10_000_000;
}

impl pallet_contracts::Config for Runtime {
	type AddressGenerator = DefaultAddressGenerator;
	type CallFilter = ();
	// TestFilter;
	type CallStack = [Frame<Self>; 5];
	// type ChainExtension = pop_chain_extension::ApiExtension<Extension>;
	type ChainExtension = ();
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type Currency = Balances;
	type Debug = ();
	// TestDebug;
	type DefaultDepositLimit = DefaultDepositLimit;
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type Environment = ();
	type MaxCodeLen = ConstU32<{ 100 * 1024 }>;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type MaxDelegateDependencies = MaxDelegateDependencies;
	type MaxStorageKeyLen = ConstU32<128>;
	type Migrations = ();
	// crate::migration::codegen::BenchMigrations;
	type Randomness = Runtime;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Schedule = MySchedule;
	type Time = Timestamp;
	type UnsafeUnstableInterface = ();
	// UnstableInterface;
	type WeightInfo = ();
	type WeightPrice = ();
	// Self;
	type Xcm = ();
}

// Implement `crate::Sandbox` trait

/// Default initial balance for the default account.
pub const INITIAL_BALANCE: u64 = 1_000_000_000_000_000;
pub const DEFAULT_ACCOUNT: AccountId = 1;

pub struct PopSandbox {
	ext: drink::ink_sandbox::TestExternalities,
}

impl ::std::default::Default for PopSandbox {
	fn default() -> Self {
		let ext = drink::ink_sandbox::macros::BlockBuilder::<Runtime>::new_ext(vec![(
			DEFAULT_ACCOUNT,
			INITIAL_BALANCE,
		)]);
		Self { ext }
	}
}

impl drink::ink_sandbox::Sandbox for PopSandbox {
	type Runtime = Runtime;

	fn execute_with<T>(&mut self, execute: impl FnOnce() -> T) -> T {
		self.ext.execute_with(execute)
	}

	fn dry_run<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T {
		// Make a backup of the backend.
		let backend_backup = self.ext.as_backend();
		// Run the action, potentially modifying storage. Ensure, that there are no pending changes
		// that would affect the reverted backend.
		let result = action(self);
		self.ext.commit_all().expect("Failed to commit changes");

		// Restore the backend.
		self.ext.backend = backend_backup;
		result
	}

	fn register_extension<E: ::core::any::Any + drink::ink_sandbox::Extension>(&mut self, ext: E) {
		self.ext.register_extension(ext);
	}

	fn initialize_block(
		height: drink::ink_sandbox::frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
		parent_hash: <Self::Runtime as drink::ink_sandbox::frame_system::Config>::Hash,
	) {
		drink::ink_sandbox::macros::BlockBuilder::<Self::Runtime>::initialize_block(
			height,
			parent_hash,
		)
	}

	fn finalize_block(
		height: drink::ink_sandbox::frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
	) -> <Self::Runtime as drink::ink_sandbox::frame_system::Config>::Hash {
		drink::ink_sandbox::macros::BlockBuilder::<Self::Runtime>::finalize_block(height)
	}

	fn default_actor() -> drink::ink_sandbox::AccountIdFor<Self::Runtime> {
		DEFAULT_ACCOUNT
	}

	fn get_metadata() -> drink::ink_sandbox::RuntimeMetadataPrefixed {
		Self::Runtime::metadata()
	}

	fn convert_account_to_origin(
		account: drink::ink_sandbox::AccountIdFor<Self::Runtime>,
	) -> <<Self::Runtime as drink::ink_sandbox::frame_system::Config>::RuntimeCall as drink::ink_sandbox::frame_support::sp_runtime::traits::Dispatchable>::RuntimeOrigin{
		Some(account).into()
	}
}

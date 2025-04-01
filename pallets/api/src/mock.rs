use ::xcm::latest::{Junction, Junctions, Location};
use codec::{Decode, Encode};
use frame_support::{
	derive_impl,
	dispatch::PostDispatchInfo,
	pallet_prelude::{DispatchResultWithPostInfo, EnsureOrigin, Pays},
	parameter_types,
	traits::{
		AsEnsureOriginWithArg, ConstU128, ConstU32, ConstU64, Everything, Hooks, OriginTrait,
	},
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot, EnsureSigned};
use pallet_nfts::PalletFeatures;
use pallet_xcm::{Origin, TestWeightInfo};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Lazy, TryConvert, Verify},
	BuildStorage,
};

use crate::messaging::{Call, CallbackExecutor, Messages, NotifyQueryHandler};

pub(crate) const ALICE: AccountId = 1;
pub(crate) const BOB: AccountId = 2;
pub(crate) const CHARLIE: AccountId = 3;
pub(crate) const RESPONSE: AccountId = 4;
pub(crate) const RESPONSE_LOCATION: Location = Location { parents: 1, interior: Junctions::Here };
pub(crate) const FEE_ACCOUNT: AccountId = 5;
pub(crate) const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
pub(crate) const UNIT: Balance = 10_000_000_000;

type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type AccountId = u64;
pub(crate) type Balance = u128;
// For terminology in tests.
pub(crate) type TokenId = u32;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Assets: pallet_assets::<Instance1>,
		Balances: pallet_balances,
		Fungibles: crate::fungibles,
		Nfts: pallet_nfts::<Instance1>,
		NonFungibles: crate::nonfungibles,
		Messaging: crate::messaging,
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<u128>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type Block = Block;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type Nonce = u64;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = SS58Prefix;
	type SystemWeightInfo = ();
	type Version = ();
}

impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = Balance;
	type DoneSlashHandler = ();
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<1>;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = ();
}

type AssetsInstance = pallet_assets::Instance1;
impl pallet_assets::Config<AssetsInstance> for Test {
	type ApprovalDeposit = ConstU128<1>;
	type AssetAccountDeposit = ConstU128<10>;
	type AssetDeposit = ConstU128<1>;
	type AssetId = TokenId;
	type AssetIdParameter = TokenId;
	type Balance = Balance;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
	type CallbackHandle = ();
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type Currency = Balances;
	type Extra = ();
	type ForceOrigin = EnsureRoot<u64>;
	type Freezer = ();
	type MetadataDepositBase = ConstU128<1>;
	type MetadataDepositPerByte = ConstU128<1>;
	type RemoveItemsLimit = ConstU32<5>;
	type RuntimeEvent = RuntimeEvent;
	type StringLimit = ConstU32<50>;
	type WeightInfo = ();
}

impl crate::fungibles::Config for Test {
	type AssetsInstance = AssetsInstance;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

parameter_types! {
	pub storage Features: PalletFeatures = PalletFeatures::all_enabled();
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub struct Noop;

impl IdentifyAccount for Noop {
	type AccountId = AccountId;

	fn into_account(self) -> Self::AccountId {
		0
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

#[cfg(feature = "runtime-benchmarks")]
impl pallet_nfts::pallet::BenchmarkHelper<u32, u32, Noop, u64, Noop> for () {
	fn collection(i: u16) -> u32 {
		i.into()
	}

	fn item(i: u16) -> u32 {
		i.into()
	}

	fn signer() -> (Noop, u64) {
		unimplemented!()
	}

	fn sign(_signer: &Noop, _message: &[u8]) -> Noop {
		unimplemented!()
	}
}

type NftsInstance = pallet_nfts::Instance1;
impl pallet_nfts::Config<NftsInstance> for Test {
	type ApprovalsLimit = ConstU32<10>;
	type AttributeDepositBase = ConstU128<1>;
	type CollectionApprovalDeposit = ConstU128<1>;
	type CollectionBalanceDeposit = ConstU128<1>;
	type CollectionDeposit = ConstU128<2>;
	type CollectionId = u32;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<u64>>;
	type Currency = Balances;
	type DepositPerByte = ConstU128<1>;
	type Features = Features;
	type ForceOrigin = frame_system::EnsureRoot<Self::AccountId>;
	#[cfg(feature = "runtime-benchmarks")]
	type Helper = ();
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

impl crate::nonfungibles::Config for Test {
	type NftsInstance = NftsInstance;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

/// Will return half of the weight in the post info.
/// Mocking a successfull execution, with refund.
pub struct AlwaysSuccessfullCallbackExecutor<T>(T);
impl<T: crate::messaging::Config> CallbackExecutor<T> for AlwaysSuccessfullCallbackExecutor<T> {
	fn execute(
		account: <T as frame_system::Config>::AccountId,
		data: Vec<u8>,
		weight: sp_runtime::Weight,
	) -> frame_support::dispatch::DispatchResultWithPostInfo {
		DispatchResultWithPostInfo::Ok(PostDispatchInfo {
			actual_weight: Some(weight),
			pays_fee: Pays::Yes,
		})
	}

	fn execution_weight() -> sp_runtime::Weight {
		Default::default()
	}
}

parameter_types! {
	pub const OnChainByteFee: Balance = 10;
	pub const OffChainByteFee: Balance = 5;
	pub const MaxXcmQueryTimeoutsPerBlock: u32 = 10;
	pub const FeeAccount: AccountId = FEE_ACCOUNT;
}

pub struct MockNotifyQuery<T>(T);
impl<T: crate::messaging::Config> NotifyQueryHandler<T> for MockNotifyQuery<T> {
	type WeightInfo = ();

	fn new_notify_query(
		responder: impl Into<Location>,
		notify: Call<T>,
		timeout: BlockNumberFor<T>,
		match_querier: impl Into<Location>,
	) -> u64 {
		get_next_query_id()
	}
}

// Just done based on the count of the queries created
// Problematic if one is removed.
pub fn get_next_query_id() -> u64 {
	Messages::<Test>::iter().count() as u64
}

impl crate::messaging::Config for Test {
	type CallbackExecutor = AlwaysSuccessfullCallbackExecutor<Test>;
	type Deposit = Balances;
	type FeeAccount = FeeAccount;
	type IsmpDispatcher = MockIsmpDispatcher;
	type MaxContextLen = ConstU32<64>;
	type MaxDataLen = ConstU32<1024>;
	type MaxKeyLen = ConstU32<1000>;
	type MaxKeys = ConstU32<10>;
	type MaxRemovals = ConstU32<1024>;
	type MaxResponseLen = ConstU32<1024>;
	type MaxXcmQueryTimeoutsPerBlock = MaxXcmQueryTimeoutsPerBlock;
	type OffChainByteFee = OffChainByteFee;
	type OnChainByteFee = OnChainByteFee;
	type OriginConverter = AccountToLocation;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightToFee = RefTimePlusProofTime;
	type Xcm = MockNotifyQuery<Test>;
	type XcmResponseOrigin = EnsureRootWithResponseSuccess;
}

pub struct RefTimePlusProofTime;
impl sp_weights::WeightToFee for RefTimePlusProofTime {
	type Balance = Balance;

	fn weight_to_fee(weight: &sp_weights::Weight) -> Self::Balance {
		(weight.ref_time() + weight.proof_size()) as u128
	}
}

#[derive(Default)]
pub struct MockIsmpDispatcher;
impl ismp::dispatcher::IsmpDispatcher for MockIsmpDispatcher {
	type Account = AccountId;
	type Balance = Balance;

	fn dispatch_request(
		&self,
		_request: ismp::dispatcher::DispatchRequest,
		_fee: ismp::dispatcher::FeeMetadata<Self::Account, Self::Balance>,
	) -> Result<H256, anyhow::Error> {
		Ok(Default::default())
	}

	fn dispatch_response(
		&self,
		_response: ismp::router::PostResponse,
		_fee: ismp::dispatcher::FeeMetadata<Self::Account, Self::Balance>,
	) -> Result<H256, anyhow::Error> {
		Ok(Default::default())
	}
}

pub struct AccountToLocation;

impl TryConvert<RuntimeOrigin, Location> for AccountToLocation {
	fn try_convert(origin: RuntimeOrigin) -> Result<Location, RuntimeOrigin> {
		let signer = origin.into_signer();
		let l = Junctions::from(Junction::AccountIndex64 {
			network: None,
			index: signer.expect("No account id, required."),
		})
		.into_location();
		Ok(l)
	}
}

pub struct EnsureRootWithResponseSuccess;
impl EnsureOrigin<RuntimeOrigin> for EnsureRootWithResponseSuccess {
	type Success = Location;

	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		if let Ok(_) = EnsureRoot::ensure_origin(o.clone()) {
			Ok(RESPONSE_LOCATION)
		} else {
			Err(o)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<O, ()> {
		todo!()
	}
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.expect("Frame system builds valid default genesis config");

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(ALICE, INIT_AMOUNT),
			(BOB, INIT_AMOUNT),
			(CHARLIE, INIT_AMOUNT),
			(FEE_ACCOUNT, INIT_AMOUNT),
		],
	}
	.assimilate_storage(&mut t)
	.expect("Pallet balances storage can be assimilated");

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
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

pub fn next_block() {
	let next_block: u64 = System::block_number() + 1;
	System::set_block_number(next_block);
	Messaging::on_initialize(next_block);
}

pub fn run_to(block_number: u64) {
	while System::block_number() < block_number {
		next_block();
	}
}

use cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
use frame_support::{
	derive_impl,
	pallet_prelude::ConstU32,
	parameter_types,
	traits::{ConstU64, Contains, EverythingBut},
};
use parachains_common::{Balance, Hash};
use polkadot_runtime_common::BlockHashCount;
use sp_runtime::traits::{AccountIdLookup, BlakeTwo256};

use crate::{
	weights::RocksDbWeight, AccountId, AggregateMessageOrigin, Aura, BalancesCall, Block,
	BlockExecutionWeight, BlockLength, BlockWeights, DispatchClass, ExtrinsicBaseWeight,
	MessageQueue, Nonce, PalletInfo, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
	RuntimeTask, RuntimeVersion, Weight, XcmpQueue, AVERAGE_ON_INITIALIZE_RATIO,
	BLOCK_PROCESSING_VELOCITY, MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO,
	RELAY_CHAIN_SLOT_DURATION_MILLIS, UNINCLUDED_SEGMENT_CAPACITY, VERSION,
};

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u16 = 0;
}

/// A type to identify filtered calls.
pub struct FilteredCalls;
impl Contains<RuntimeCall> for FilteredCalls {
	fn contains(c: &RuntimeCall) -> bool {
		use BalancesCall::*;
		matches!(
			c,
			RuntimeCall::Balances(
				force_adjust_total_issuance { .. } |
					force_set_balance { .. } |
					force_transfer { .. } |
					force_unreserve { .. }
			)
		)
	}
}

#[derive_impl(frame_system::config_preludes::ParaChainDefaultConfig)]
impl frame_system::Config for Runtime {
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The basic call filter to use in dispatchable. Supports everything as the default.
	type BaseCallFilter = EverythingBut<FilteredCalls>;
	/// The block type.
	type Block = Block;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Weight information for the extensions of this pallet.
	type ExtensionsWeightInfo = ();
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The default hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<Self::AccountId, ()>;
	type MaxConsumers = ConstU32<16>;
	type MultiBlockMigrator = ();
	/// The index type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	/// Converts a module to the index of the module, injected by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	type PostInherents = ();
	type PostTransactions = ();
	type PreInherents = ();
	/// The aggregated dispatch type available for extrinsics, injected by
	/// `construct_runtime!`.
	type RuntimeCall = RuntimeCall;
	/// The ubiquitous event type injected by `construct_runtime!`.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type injected by `construct_runtime!`.
	type RuntimeOrigin = RuntimeOrigin;
	/// The aggregated Task type, injected by `construct_runtime!`.
	type RuntimeTask = RuntimeTask;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type SingleBlockMigrations = ();
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = frame_system::weights::SubstrateWeight<Self>;
	/// Runtime version.
	type Version = Version;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

#[docify::export]
pub type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	RELAY_CHAIN_SLOT_DURATION_MILLIS,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

impl cumulus_pallet_parachain_system::Config for Runtime {
	type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
	type ConsensusHook = ConsensusHook;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = XcmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type RuntimeEvent = RuntimeEvent;
	type SelectCore = cumulus_pallet_parachain_system::DefaultCoreSelector<Runtime>;
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type WeightInfo = ();
	type XcmpMessageHandler = XcmpQueue;
}

impl pallet_timestamp::Config for Runtime {
	type MinimumPeriod = ConstU64<0>;
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type WeightInfo = ();
}

impl parachain_info::Config for Runtime {}

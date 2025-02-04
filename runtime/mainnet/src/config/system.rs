use cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
use frame_support::traits::{ConstU32, ConstU64, Contains, EnqueueWithOrigin, EverythingBut};
use pallet_balances::Call as BalancesCall;
use polkadot_runtime_common::BlockHashCount;
use pop_runtime_common::{
	Nonce, AVERAGE_ON_INITIALIZE_RATIO, MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO,
};
use sp_runtime::traits::AccountIdLookup;

// Allowing `unsued_imports` here because `Executive` is being used in the
// `register_validate_block` macro but yet the compiler warns about it not being used.
// Duplicating the crate import to reduce the amount of imports that could pass unnoticed
// because of this.
#[allow(unused_imports)]
use crate::Executive;
use crate::{
	parameter_types,
	weights::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
	AccountId, AggregateMessageOrigin, Aura, Balance, BlakeTwo256, Block, BlockLength,
	BlockWeights, DispatchClass, Hash, MessageQueue, PalletInfo, Runtime, RuntimeCall,
	RuntimeEvent, RuntimeOrigin, RuntimeTask, RuntimeVersion, Weight, XcmpQueue,
	BLOCK_PROCESSING_VELOCITY, RELAY_CHAIN_SLOT_DURATION_MILLIS, UNINCLUDED_SEGMENT_CAPACITY,
	VERSION,
};

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u16 = 0;
	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	// The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
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
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

#[docify::export]
/// Provides means to manage the backlog of unincluded blocks.
pub type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	RELAY_CHAIN_SLOT_DURATION_MILLIS,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

#[docify::export(register_validate_block)]
cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
	type ConsensusHook = ConsensusHook;
	type DmpQueue = EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = XcmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type RuntimeEvent = RuntimeEvent;
	type SelectCore = cumulus_pallet_parachain_system::LookaheadCoreSelector<Runtime>;
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type WeightInfo = cumulus_pallet_parachain_system::weights::SubstrateWeight<Runtime>;
	type XcmpMessageHandler = XcmpQueue;
}

impl pallet_timestamp::Config for Runtime {
	type MinimumPeriod = ConstU64<0>;
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

impl parachain_info::Config for Runtime {}

#[cfg(test)]
mod tests {
	use alloc::borrow::Cow;
	use std::any::TypeId;

	use cumulus_primitives_core::relay_chain::MAX_POV_SIZE;
	use frame_support::{
		dispatch::PerDispatchClass, traits::Get, weights::constants::WEIGHT_REF_TIME_PER_SECOND,
	};
	use frame_system::limits::WeightsPerClass;
	use sp_core::crypto::AccountId32;
	use sp_runtime::generic;

	use super::*;
	use crate::{Header, Perbill, UncheckedExtrinsic, Weight};

	mod system {
		use pallet_balances::AdjustmentDirection;
		use sp_runtime::MultiAddress;
		use BalancesCall::*;

		use super::*;

		#[test]
		fn account_id_is_32_bytes() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::AccountId>(),
				TypeId::of::<AccountId32>(),
			);
		}

		#[test]
		fn base_call_filter_allows_everything_but_filtered_calls() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::BaseCallFilter>(),
				TypeId::of::<EverythingBut<FilteredCalls>>(),
			);
			let dummy_acc = MultiAddress::from(AccountId32::from([0; 32]));
			let filtered_calls = [
				RuntimeCall::Balances(force_adjust_total_issuance {
					direction: AdjustmentDirection::Increase,
					delta: 0,
				}),
				RuntimeCall::Balances(force_set_balance { who: dummy_acc.clone(), new_free: 0 }),
				RuntimeCall::Balances(force_transfer {
					source: dummy_acc.clone(),
					dest: dummy_acc.clone(),
					value: 1,
				}),
				RuntimeCall::Balances(force_unreserve { who: dummy_acc, amount: 0 }),
			];

			for call in filtered_calls {
				assert!(FilteredCalls::contains(&call));
				assert!(!<<Runtime as frame_system::Config>::BaseCallFilter>::contains(&call));
			}
		}

		#[test]
		fn block_configured() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::Block>(),
				TypeId::of::<generic::Block<Header, UncheckedExtrinsic>>(),
			);
		}

		#[test]
		fn block_hash_count() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::BlockHashCount>(),
				TypeId::of::<BlockHashCount>(),
			);
			assert_eq!(BlockHashCount::get(), 4096);
		}

		#[test]
		fn block_length_restricted_by_pov() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::BlockLength>(),
				TypeId::of::<RuntimeBlockLength>(),
			);
			// Normal extrinsics limited to 75% of max PoV size
			assert_eq!(
				*RuntimeBlockLength::get().max.get(DispatchClass::Normal),
				Perbill::from_percent(75) * MAX_POV_SIZE
			);
			// Operational / mandatory (inherents) limited to max PoV size
			assert_eq!(
				*RuntimeBlockLength::get().max.get(DispatchClass::Operational),
				MAX_POV_SIZE
			);
			assert_eq!(*RuntimeBlockLength::get().max.get(DispatchClass::Mandatory), MAX_POV_SIZE);
		}

		#[test]
		fn block_weights_restricted_by_dispatch_class() {
			// Two seconds compute per 6s block, max PoV size
			let max_block_weight =
				Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), MAX_POV_SIZE as u64);
			let base_extrinsic = ExtrinsicBaseWeight::get();
			let expected_per_class = PerDispatchClass::new(|dc| match dc {
				DispatchClass::Normal => {
					let max_total = Perbill::from_percent(75) * max_block_weight; // 75% of block
					WeightsPerClass {
						base_extrinsic,
						max_extrinsic: Some(
							max_total -
								(Perbill::from_percent(5) * max_block_weight) -
								base_extrinsic,
						), // max - average initialise ratio - base weight
						max_total: Some(max_total),
						reserved: Some(Weight::zero()),
					}
				},
				DispatchClass::Operational => WeightsPerClass {
					base_extrinsic,
					max_extrinsic: Some(
						max_block_weight -
							(Perbill::from_percent(5) * max_block_weight) -
							base_extrinsic,
					),
					max_total: Some(max_block_weight),
					reserved: Some(Perbill::from_percent(25) * max_block_weight),
				},
				DispatchClass::Mandatory => WeightsPerClass {
					base_extrinsic,
					max_extrinsic: None,
					max_total: None,
					reserved: None,
				},
			});

			assert_eq!(MAXIMUM_BLOCK_WEIGHT, max_block_weight);
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::BlockWeights>(),
				TypeId::of::<RuntimeBlockWeights>(),
			);
			assert_eq!(RuntimeBlockWeights::get().base_block, BlockExecutionWeight::get());
			assert_eq!(RuntimeBlockWeights::get().max_block, max_block_weight);

			let actual = RuntimeBlockWeights::get();
			for class in
				[DispatchClass::Normal, DispatchClass::Operational, DispatchClass::Mandatory]
			{
				let actual = actual.per_class.get(class);
				let expected = expected_per_class.get(class);
				println!("{class:?}\nactual   {actual:?}\nexpected {expected:?}\n");
				assert_eq!(actual.base_extrinsic, expected.base_extrinsic);
				assert_eq!(actual.max_extrinsic, expected.max_extrinsic);
				assert_eq!(actual.max_total, expected.max_total);
				assert_eq!(actual.reserved, expected.reserved);
			}
		}

		#[test]
		fn db_weight_uses_rocks_db() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::DbWeight>(),
				TypeId::of::<RocksDbWeight>(),
			);
		}

		#[test]
		fn uses_blake2_hashing() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::Hashing>(),
				TypeId::of::<BlakeTwo256>(),
			);
		}

		#[test]
		fn uses_multi_address_lookup() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::Lookup>(),
				TypeId::of::<AccountIdLookup<AccountId, ()>>(),
			);
		}

		#[test]
		fn max_consumers_limited_to_16() {
			assert_eq!(<<Runtime as frame_system::Config>::MaxConsumers as Get<u32>>::get(), 16);
		}

		#[test]
		fn multi_block_migrator_disabled() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::MultiBlockMigrator>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn nonce_uses_u32() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::Nonce>(),
				TypeId::of::<u32>(),
			);
		}

		#[test]
		fn account_killed_handler_disabled() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::OnKilledAccount>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn new_account_handler_disabled() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::OnNewAccount>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn set_code_handler_managed_by_parachain_system() {
			// Runtime upgrades orchestrated by parachain system pallet
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::OnSetCode>(),
				TypeId::of::<cumulus_pallet_parachain_system::ParachainSetCode<Runtime>>(),
			);
		}

		#[test]
		fn post_inherent_handler_disabled() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::PostInherents>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn post_transactions_handler_disabled() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::PostTransactions>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn pre_inherent_handler_disabled() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::PreInherents>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn single_block_migrations_is_empty() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::SingleBlockMigrations>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn ss58_prefix_matches_relay() {
			assert_eq!(<<Runtime as frame_system::Config>::SS58Prefix>::get(), 0);
		}

		#[test]
		fn does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as frame_system::Config>::SystemWeightInfo>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		#[ignore]
		fn extensions_do_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as frame_system::Config>::ExtensionsWeightInfo>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn versions_runtime() {
			assert_eq!(
				TypeId::of::<<Runtime as frame_system::Config>::Version>(),
				TypeId::of::<Version>(),
			);

			assert_eq!(Version::get().spec_name, Cow::Borrowed("pop"));
			assert_eq!(Version::get().impl_name, Cow::Borrowed("pop"));
			assert_eq!(Version::get().spec_version, 100);
		}
	}

	mod parachain_system {
		use super::*;

		#[test]
		fn ensures_relay_block_increases_monotonically() {
			assert_eq!(
			TypeId::of::<
				<Runtime as cumulus_pallet_parachain_system::Config>::CheckAssociatedRelayNumber,
			>(),
			TypeId::of::<RelayNumberMonotonicallyIncreases>(),
		);
		}

		#[test]
		fn manages_unincluded_block_authoring() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_parachain_system::Config>::ConsensusHook>(),
				TypeId::of::<
					cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
						Runtime,
						6000, // Relay chain slot duration of 6s.
						1,    // Blocks per slot.
						3,    // Max unincluded blocks accepted by runtime simultaneously
					>,
				>(),
			);
		}

		#[test]
		fn enqueues_dmp_messages() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_parachain_system::Config>::DmpQueue>(),
				TypeId::of::<EnqueueWithOrigin<MessageQueue, RelayOrigin>>(),
			);
		}

		#[test]
		fn event_handler_disabled() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_parachain_system::Config>::OnSystemEvent>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn outbound_messages_sourced_from_xcmp_queue() {
			assert_eq!(
				TypeId::of::<
					<Runtime as cumulus_pallet_parachain_system::Config>::OutboundXcmpMessageSource,
				>(),
				TypeId::of::<XcmpQueue>(),
			);
		}

		#[test]
		fn reserves_weight_for_dmp_messages() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_parachain_system::Config>::ReservedDmpWeight>(
				),
				TypeId::of::<ReservedDmpWeight>(),
			);
			assert_eq!(ReservedDmpWeight::get(), MAXIMUM_BLOCK_WEIGHT / 4);
		}

		#[test]
		fn reserves_weight_for_xcmp_messages() {
			assert_eq!(
				TypeId::of::<
					<Runtime as cumulus_pallet_parachain_system::Config>::ReservedXcmpWeight,
				>(),
				TypeId::of::<ReservedXcmpWeight>(),
			);
			assert_eq!(ReservedXcmpWeight::get(), MAXIMUM_BLOCK_WEIGHT / 4);
		}

		#[test]
		fn uses_lookahead_core_selector() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_parachain_system::Config>::SelectCore>(),
				TypeId::of::<cumulus_pallet_parachain_system::LookaheadCoreSelector::<Runtime>>(),
			);
		}

		#[test]
		fn looks_up_para_id_from_parachain_info() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_parachain_system::Config>::SelfParaId>(),
				TypeId::of::<parachain_info::Pallet<Runtime>>(),
			);
		}

		#[test]
		fn does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as cumulus_pallet_parachain_system::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn uses_xcmp_queue_as_xcmp_message_handler() {
			assert_eq!(
				TypeId::of::<
					<Runtime as cumulus_pallet_parachain_system::Config>::XcmpMessageHandler,
				>(),
				TypeId::of::<XcmpQueue>(),
			);
		}
	}

	mod timestamp {
		use super::*;

		#[test]
		fn min_period_is_zero() {
			assert_eq!(
				<<Runtime as pallet_timestamp::Config>::MinimumPeriod as Get<u64>>::get(),
				0
			);
		}

		#[test]
		fn uses_u64_moment() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_timestamp::Config>::Moment>(),
				TypeId::of::<u64>(),
			);
		}

		#[test]
		fn handler_is_aura() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_timestamp::Config>::OnTimestampSet>(),
				TypeId::of::<Aura>(),
			);
		}

		#[test]
		fn does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_timestamp::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}
}

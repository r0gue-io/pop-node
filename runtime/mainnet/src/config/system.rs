use polkadot_runtime_common::BlockHashCount;
use pop_runtime_common::{
	Nonce, AVERAGE_ON_INITIALIZE_RATIO, MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO,
};
use sp_runtime::traits::AccountIdLookup;

use crate::{
	parameter_types,
	weights::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
	AccountId, Balance, BlakeTwo256, Block, BlockLength, BlockWeights, DispatchClass, Everything,
	Hash, PalletInfo, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, RuntimeTask,
	RuntimeVersion, VERSION,
};

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u16 = 0;
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
}

impl frame_system::Config for Runtime {
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The basic call filter to use in dispatchable. Supports everything as the default.
	type BaseCallFilter = Everything;
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
	type MaxConsumers = frame_support::traits::ConstU32<16>;
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
	use crate::{Header, UncheckedExtrinsic, Weight};

	#[test]
	fn base_call_filter_allows_everything() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::BaseCallFilter>(),
			TypeId::of::<Everything>(),
		);
	}

	#[test]
	fn system_account_id_is_32_bytes() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::AccountId>(),
			TypeId::of::<AccountId32>(),
		);
	}

	#[test]
	fn system_block_configured() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::Block>(),
			TypeId::of::<generic::Block<Header, UncheckedExtrinsic>>(),
		);
	}

	#[test]
	fn system_block_hash_count() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::BlockHashCount>(),
			TypeId::of::<BlockHashCount>(),
		);
		assert_eq!(BlockHashCount::get(), 4096);
	}

	#[test]
	fn system_block_length_restricted_by_pov() {
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
		assert_eq!(*RuntimeBlockLength::get().max.get(DispatchClass::Operational), MAX_POV_SIZE);
		assert_eq!(*RuntimeBlockLength::get().max.get(DispatchClass::Mandatory), MAX_POV_SIZE);
	}

	#[test]
	fn system_block_weights_restricted_by_dispatch_class() {
		let max_block_weight =
			// Two seconds compute per 6s block, max PoV size
			Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), MAX_POV_SIZE as u64);
		let base_extrinsic = ExtrinsicBaseWeight::get();
		let expected_per_class = PerDispatchClass::new(|dc| match dc {
			DispatchClass::Normal => {
				let max_total = Perbill::from_percent(75) * max_block_weight; // 75% of block
				WeightsPerClass {
					base_extrinsic,
					max_extrinsic: Some(
						max_total - (Perbill::from_percent(5) * max_block_weight) - base_extrinsic,
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
		for class in [DispatchClass::Normal, DispatchClass::Operational, DispatchClass::Mandatory] {
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
	fn system_db_weight_uses_rocks_db() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::DbWeight>(),
			TypeId::of::<RocksDbWeight>(),
		);
	}

	#[test]
	fn system_uses_blake2_hashing() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::Hashing>(),
			TypeId::of::<BlakeTwo256>(),
		);
	}

	#[test]
	fn system_uses_multi_address_lookup() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::Lookup>(),
			TypeId::of::<AccountIdLookup<AccountId, ()>>(),
		);
	}

	#[test]
	fn system_max_consumers_limited_to_16() {
		assert_eq!(<<Runtime as frame_system::Config>::MaxConsumers as Get<u32>>::get(), 16);
	}

	#[test]
	fn system_multi_block_migrator_disabled() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::MultiBlockMigrator>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn system_nonce_uses_u32() {
		assert_eq!(TypeId::of::<<Runtime as frame_system::Config>::Nonce>(), TypeId::of::<u32>(),);
	}

	#[test]
	fn system_account_killed_handler_disabled() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::OnKilledAccount>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn system_new_account_handler_disabled() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::OnNewAccount>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn system_set_code_handler_managed_by_parachain_system() {
		// Runtime upgrades orchestrated by parachain system pallet
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::OnSetCode>(),
			TypeId::of::<cumulus_pallet_parachain_system::ParachainSetCode<Runtime>>(),
		);
	}

	#[test]
	fn system_post_inherent_handler_disabled() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::PostInherents>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn system_post_transactions_handler_disabled() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::PostTransactions>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn system_pre_inherent_handler_disabled() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::PreInherents>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn system_single_block_migrations_is_empty() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::SingleBlockMigrations>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn system_ss58_prefix_matches_relay() {
		assert_eq!(<<Runtime as frame_system::Config>::SS58Prefix>::get(), 0);
	}

	#[test]
	fn system_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as frame_system::Config>::SystemWeightInfo>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	#[ignore]
	fn system_extensions_do_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as frame_system::Config>::ExtensionsWeightInfo>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn system_versions_runtime() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::Version>(),
			TypeId::of::<Version>(),
		);

		assert_eq!(Version::get().spec_name, Cow::Borrowed("pop"));
		assert_eq!(Version::get().impl_name, Cow::Borrowed("pop"));
		assert_eq!(Version::get().spec_version, 100);
	}
}

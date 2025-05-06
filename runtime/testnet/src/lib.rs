#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod apis;
// Public due to integration tests crate.
pub mod config;

/// The genesis state presets available.
pub mod genesis;
mod weights;

extern crate alloc;

use alloc::{borrow::Cow, vec::Vec};

// ISMP imports
use ::ismp::{
	consensus::{ConsensusClientId, StateMachineHeight, StateMachineId},
	host::StateMachine,
	router::{Request, Response},
};
use sp_runtime::{traits::ValidateUnsigned};
pub use apis::{RuntimeApi, RUNTIME_API_VERSIONS};
use config::system::ConsensusHook;
use cumulus_primitives_core::AggregateMessageOrigin;
use cumulus_primitives_storage_weight_reclaim::StorageWeightReclaim;
use frame_metadata_hash_extension::CheckMetadataHash;
use frame_support::{
	dispatch::DispatchClass,
	parameter_types,
	traits::{ConstBool, ConstU32, ConstU64, ConstU8, EitherOfDiverse, VariantCountOf},
	weights::{
		ConstantMultiplier, Weight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
	PalletId,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	CheckGenesis, CheckMortality, CheckNonZeroSender, CheckNonce, CheckSpecVersion, CheckTxVersion,
	CheckWeight, EnsureRoot,
};
use pallet_api::{fungibles, messaging};
use pallet_balances::Call as BalancesCall;
use pallet_nfts_sdk as pallet_nfts;
use pallet_transaction_payment::ChargeTransactionPayment;
// Polkadot imports
use polkadot_runtime_common::SlowAdjustingFeeUpdate;
pub use pop_runtime_common::{
	deposit, AuraId, Balance, BlockNumber, Hash, Nonce, Signature, AVERAGE_ON_INITIALIZE_RATIO,
	BLOCK_PROCESSING_VELOCITY, DAYS, EXISTENTIAL_DEPOSIT, HOURS, MAXIMUM_BLOCK_WEIGHT, MICRO_UNIT,
	MILLI_UNIT, MINUTES, NORMAL_DISPATCH_RATIO, RELAY_CHAIN_SLOT_DURATION_MILLIS, SLOT_DURATION,
	UNINCLUDED_SEGMENT_CAPACITY, UNIT,
};
use smallvec::smallvec;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, IdentifyAccount, Verify},
};
pub use sp_runtime::{ExtrinsicInclusionMode, MultiAddress, Perbill, Permill};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use weights::{BlockExecutionWeight, ExtrinsicBaseWeight};
// XCM Imports
use xcm::latest::prelude::BodyId;

use crate::config::assets::TrustBackedAssetsInstance;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Record of an event happening.
pub type EventRecord = frame_system::EventRecord<
	<Runtime as frame_system::Config>::RuntimeEvent,
	<Runtime as frame_system::Config>::Hash,
>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type TxExtension = (
	CheckNonZeroSender<Runtime>,
	CheckSpecVersion<Runtime>,
	CheckTxVersion<Runtime>,
	CheckGenesis<Runtime>,
	CheckMortality<Runtime>,
	CheckNonce<Runtime>,
	CheckWeight<Runtime>,
	ChargeTransactionPayment<Runtime>,
	StorageWeightReclaim<Runtime>,
	CheckMetadataHash<Runtime>,
);

/// EthExtra converts an unsigned Call::eth_transact into a CheckedExtrinsic.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct EthExtraImpl;

impl pallet_revive::evm::runtime::EthExtra for EthExtraImpl {
	type Config = Runtime;
	type Extension = TxExtension;

	fn get_eth_extension(nonce: u32, tip: Balance) -> Self::Extension {
		(
			CheckNonZeroSender::<Runtime>::new(),
			CheckSpecVersion::<Runtime>::new(),
			CheckTxVersion::<Runtime>::new(),
			CheckGenesis::<Runtime>::new(),
			CheckMortality::from(generic::Era::Immortal),
			CheckNonce::<Runtime>::from(nonce),
			CheckWeight::<Runtime>::new(),
			ChargeTransactionPayment::<Runtime>::from(tip),
			StorageWeightReclaim::<Runtime>::new(),
			CheckMetadataHash::<Runtime>::new(false),
		)
	}
}

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	pallet_revive::evm::runtime::UncheckedExtrinsic<Address, Signature, EthExtraImpl>;

/// Migrations to apply on runtime upgrade.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
pub type Migrations = (
	cumulus_pallet_xcmp_queue::migration::v5::MigrateV4ToV5<Runtime>,
	// Unreleased.
	pallet_assets::migration::next_asset_id::SetNextAssetId<
		// Higher AssetId on testnet live is `7_045`,
		// rounded up to 10_000.
		ConstU32<10_000>,
		Runtime,
		TrustBackedAssetsInstance,
	>,
	// Permanent.
	pallet_contracts::Migration<Runtime>,
	pallet_xcm::migration::MigrateToLatestXcmVersion<Runtime>,
);

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
		// we map to 1/10 of that, or 1/10 MILLIUNIT
		let p = MILLI_UNIT / 10;
		let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	use sp_runtime::{
		generic,
		traits::{BlakeTwo256, Hash as HashT},
	};

	use super::*;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque block hash type.
	pub type Hash = <BlakeTwo256 as HashT>::Output;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: Cow::Borrowed("pop"),
	impl_name: Cow::Borrowed("pop"),
	authoring_version: 1,
	#[allow(clippy::zero_prefixed_literal)]
	spec_version: 00_05_02,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 2,
	system_version: 1,
};

// Prints debug output of the `contracts` pallet to stdout if the node is
// started with `-lruntime::contracts=debug`.
const CONTRACTS_DEBUG_OUTPUT: pallet_contracts::DebugInfo =
	pallet_contracts::DebugInfo::UnsafeDebug;
const CONTRACTS_EVENTS: pallet_contracts::CollectEvents =
	pallet_contracts::CollectEvents::UnsafeCollect;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

impl cumulus_pallet_xcmp_queue::migration::v5::V5Config for Runtime {
	// This must be the same as the `ChannelInfo` from the `Config`:
	type ChannelList = ParachainSystem;
}

#[frame_support::runtime]
mod runtime {
	// Create the runtime by composing the FRAME pallets that were previously configured.
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
		RuntimeTask
	)]
	pub struct Runtime;

	// System support stuff.
	#[runtime::pallet_index(0)]
	pub type System = frame_system::Pallet<Runtime>;
	#[runtime::pallet_index(1)]
	pub type ParachainSystem = cumulus_pallet_parachain_system::Pallet<Runtime>;
	#[runtime::pallet_index(2)]
	pub type Timestamp = pallet_timestamp::Pallet<Runtime>;
	#[runtime::pallet_index(3)]
	pub type ParachainInfo = parachain_info::Pallet<Runtime>;

	// Monetary stuff.
	#[runtime::pallet_index(10)]
	pub type Balances = pallet_balances::Pallet<Runtime>;
	#[runtime::pallet_index(11)]
	pub type TransactionPayment = pallet_transaction_payment::Pallet<Runtime>;
	#[runtime::pallet_index(12)]
	pub type Treasury = pallet_treasury::Pallet<Runtime>;

	// Governance
	#[runtime::pallet_index(15)]
	pub type Sudo = pallet_sudo;
	#[runtime::pallet_index(16)]
	pub type Council = pallet_collective::Pallet<Runtime, Instance1>;
	#[runtime::pallet_index(18)]
	pub type Motion = pallet_motion;

	// Collator support. The order of these 4 are important and shall not change.
	#[runtime::pallet_index(20)]
	pub type Authorship = pallet_authorship::Pallet<Runtime>;
	#[runtime::pallet_index(21)]
	pub type CollatorSelection = pallet_collator_selection::Pallet<Runtime>;
	#[runtime::pallet_index(22)]
	pub type Session = pallet_session::Pallet<Runtime>;
	#[runtime::pallet_index(23)]
	pub type Aura = pallet_aura::Pallet<Runtime>;
	#[runtime::pallet_index(24)]
	pub type AuraExt = cumulus_pallet_aura_ext;

	// Scheduler
	#[runtime::pallet_index(28)]
	pub type Scheduler = pallet_scheduler;

	// Preimage
	#[runtime::pallet_index(29)]
	pub type Preimage = pallet_preimage;

	// XCM helpers.
	#[runtime::pallet_index(30)]
	pub type XcmpQueue = cumulus_pallet_xcmp_queue::Pallet<Runtime>;
	#[runtime::pallet_index(31)]
	pub type PolkadotXcm = pallet_xcm::Pallet<Runtime>;
	#[runtime::pallet_index(32)]
	pub type CumulusXcm = cumulus_pallet_xcm::Pallet<Runtime>;
	#[runtime::pallet_index(33)]
	pub type MessageQueue = pallet_message_queue::Pallet<Runtime>;

	// ISMP
	#[runtime::pallet_index(38)]
	#[runtime::disable_unsigned]
	pub type Ismp = pallet_ismp::Pallet<Runtime>;
	#[runtime::pallet_index(39)]
	pub type IsmpParachain = ismp_parachain::Pallet<Runtime>;

	// Contracts
	#[runtime::pallet_index(40)]
	pub type Contracts = pallet_contracts::Pallet<Runtime>;
	#[runtime::pallet_index(60)]
	pub type Revive = pallet_revive::Pallet<Runtime>;

	// Proxy
	#[runtime::pallet_index(41)]
	pub type Proxy = pallet_proxy::Pallet<Runtime>;
	// Multisig
	#[runtime::pallet_index(42)]
	pub type Multisig = pallet_multisig::Pallet<Runtime>;
	// Utility
	#[runtime::pallet_index(43)]
	pub type Utility = pallet_utility::Pallet<Runtime>;

	// Assets
	#[runtime::pallet_index(50)]
	pub type Nfts = pallet_nfts::Pallet<Runtime>;
	#[runtime::pallet_index(51)]
	pub type NftFractionalization = pallet_nft_fractionalization::Pallet<Runtime>;
	#[runtime::pallet_index(52)]
	pub type Assets = pallet_assets::Pallet<Runtime, Instance1>;

	// Pop API
	#[runtime::pallet_index(150)]
	pub type Fungibles = fungibles::Pallet<Runtime>;
	#[runtime::pallet_index(152)]
	pub type Messaging = messaging::Pallet<Runtime>;
}

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[fungibles, Fungibles]
		[pallet_balances, Balances]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_message_queue, MessageQueue]
		[pallet_sudo, Sudo]
		[pallet_collator_selection, CollatorSelection]
		[cumulus_pallet_parachain_system, ParachainSystem]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
	);
}

// We move some impls outside so we can easily use them with `docify`.
impl Runtime {
	#[docify::export]
	fn impl_slot_duration() -> sp_consensus_aura::SlotDuration {
		sp_consensus_aura::SlotDuration::from_millis(SLOT_DURATION)
	}

	#[docify::export]
	fn impl_can_build_upon(
		included_hash: <Block as BlockT>::Hash,
		slot: cumulus_primitives_aura::Slot,
	) -> bool {
		ConsensusHook::can_build_upon(included_hash, slot)
	}
}

#[docify::export(register_validate_block)]
cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}

// Ensures that the account id lookup does not perform any state reads. When this changes,
// `pallet_api::fungibles` dispatchables need to be re-evaluated.
#[test]
fn test_lookup_config() {
	use std::any::TypeId;
	assert_eq!(
		TypeId::of::<<Runtime as frame_system::Config>::Lookup>(),
		TypeId::of::<sp_runtime::traits::AccountIdLookup<sp_runtime::AccountId32, ()>>()
	);
}

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod apis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
#[cfg(feature = "runtime-benchmarks")]
use benchmarks::*;

pub mod config;

/// The genesis state presets available.
pub mod genesis;

extern crate alloc;

use alloc::{borrow::Cow, vec::Vec};

pub use apis::{RuntimeApi, RUNTIME_API_VERSIONS};
use config::{monetary::deposit, system::ConsensusHook};
use cumulus_primitives_core::AggregateMessageOrigin;
use frame_metadata_hash_extension::CheckMetadataHash;
use frame_support::{
	dispatch::DispatchClass,
	parameter_types,
	traits::{
		fungible::HoldConsideration, tokens::imbalance::ResolveTo, ConstBool, ConstU32, ConstU64,
		ConstU8, EqualPrivilegeOnly, LinearStoragePrice, VariantCountOf,
	},
	weights::{ConstantMultiplier, Weight},
	PalletId,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	CheckGenesis, CheckMortality, CheckNonZeroSender, CheckNonce, CheckSpecVersion, CheckTxVersion,
	CheckWeight, EnsureRoot,
};
use pallet_nfts_sdk as pallet_nfts;
use pallet_transaction_payment::ChargeTransactionPayment;
// Polkadot imports
use polkadot_runtime_common::SlowAdjustingFeeUpdate;
pub use pop_runtime_common::{
	weights, AuraId, Balance, BlockNumber, Hash, Nonce, Signature, AVERAGE_ON_INITIALIZE_RATIO,
	BLOCK_PROCESSING_VELOCITY, DAYS, EXISTENTIAL_DEPOSIT, HOURS, MAXIMUM_BLOCK_WEIGHT, MICRO_UNIT,
	MILLI_UNIT, MINUTES, NORMAL_DISPATCH_RATIO, RELAY_CHAIN_SLOT_DURATION_MILLIS, SLOT_DURATION,
	UNINCLUDED_SEGMENT_CAPACITY, UNIT,
};
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

use crate::config::assets::TrustBackedAssetsInstance;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// The SignedExtension to the basic transaction logic.
pub type TxExtension = cumulus_pallet_weight_reclaim::StorageWeightReclaim<
	Runtime,
	(
		CheckNonZeroSender<Runtime>,
		CheckSpecVersion<Runtime>,
		CheckTxVersion<Runtime>,
		CheckGenesis<Runtime>,
		CheckMortality<Runtime>,
		CheckNonce<Runtime>,
		CheckWeight<Runtime>,
		ChargeTransactionPayment<Runtime>,
		CheckMetadataHash<Runtime>,
	),
>;

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
			CheckMetadataHash::<Runtime>::new(false),
		)
			.into()
	}
}

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	pallet_revive::evm::runtime::UncheckedExtrinsic<Address, Signature, EthExtraImpl>;

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
type Migrations = (
	// Unreleased.
	config::governance::initiate_council_migration::SetCouncilors,
	pallet_assets::migration::next_asset_id::SetNextAssetId<
		ConstU32<1>,
		Runtime,
		TrustBackedAssetsInstance,
	>,
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
	spec_version: 00_01_00,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	system_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
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
		RuntimeTask,
		RuntimeViewFunction
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
	#[runtime::pallet_index(4)]
	pub type WeightReclaim = cumulus_pallet_weight_reclaim::Pallet<Runtime>;

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

	// Contracts (using pallet-revive)
	#[runtime::pallet_index(40)]
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
	#[runtime::pallet_index(52)]
	pub type Assets = pallet_assets::Pallet<Runtime, Instance1>;
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

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use sp_runtime::MultiSignature;

	use super::*;

	#[test]
	fn block_header_configured() {
		assert_eq!(TypeId::of::<Header>(), TypeId::of::<generic::Header<u32, BlakeTwo256>>());
	}

	#[test]
	fn unchecked_extrinsic_configured() {
		assert_eq!(
			TypeId::of::<UncheckedExtrinsic>(),
			TypeId::of::<
				pallet_revive::evm::runtime::UncheckedExtrinsic<
					// Multiple address formats supported.
					MultiAddress<AccountId, ()>,
					// The signature scheme(s) supported.
					MultiSignature,
					// The transaction extensions that has an additional extension to convert
					// an eth transaction into a checked extrinsic.
					EthExtraImpl,
				>,
			>(),
		);
	}

	#[test]
	fn transaction_extension_checks() {
		assert_eq!(
			TypeId::of::<TxExtension>(),
			TypeId::of::<
				cumulus_pallet_weight_reclaim::StorageWeightReclaim<
					Runtime,
					(
						// Ensures sender is not the zero address.
						CheckNonZeroSender<Runtime>,
						// Ensures the runtime version included within in the transaction is the
						// same as at present.
						CheckSpecVersion<Runtime>,
						// Ensures the transaction version included in the transaction is the same
						// as at present.
						CheckTxVersion<Runtime>,
						// Genesis hash check to provide replay protection between different
						// networks.
						CheckGenesis<Runtime>,
						// Checks transaction mortality.
						CheckMortality<Runtime>,
						// Nonce check and increment to give replay protection for transactions.
						CheckNonce<Runtime>,
						// Block resource (weight) limit check.
						CheckWeight<Runtime>,
						// Require the transactor pay for the transaction, optionally including a
						// tip to gain additional priority in the queue.
						ChargeTransactionPayment<Runtime>,
						// Extension for optionally verifying the metadata hash.
						CheckMetadataHash<Runtime>
					),
				>,
			>(),
		);
	}
}

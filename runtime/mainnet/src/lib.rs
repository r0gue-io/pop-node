#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod apis;
pub mod config;
mod weights;

extern crate alloc;

use alloc::{borrow::Cow, vec::Vec};

pub use apis::{RuntimeApi, RUNTIME_API_VERSIONS};
use config::{
	governance::SudoAddress,
	system::{ConsensusHook, RuntimeBlockWeights},
	xcm::{RelayLocation, XcmOriginToTransactDispatchOrigin},
};
use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use cumulus_primitives_storage_weight_reclaim::StorageWeightReclaim;
use frame_metadata_hash_extension::CheckMetadataHash;
use frame_support::{
	dispatch::DispatchClass,
	parameter_types,
	traits::{
		fungible,
		fungible::HoldConsideration,
		tokens::{imbalance::ResolveTo, PayFromAccount, UnityAssetBalanceConversion},
		ConstBool, ConstU32, ConstU64, ConstU8, Contains, EitherOfDiverse, EqualPrivilegeOnly,
		EverythingBut, Imbalance, LinearStoragePrice, NeverEnsureOrigin, OnUnbalanced,
		TransformOrigin, VariantCountOf,
	},
	weights::{ConstantMultiplier, Weight},
	PalletId,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	CheckGenesis, CheckMortality, CheckNonZeroSender, CheckNonce, CheckSpecVersion, CheckTxVersion,
	CheckWeight, EnsureRoot,
};
use pallet_transaction_payment::ChargeTransactionPayment;
use pallet_xcm::{EnsureXcm, IsVoiceOfBody};
use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
// Polkadot imports
use polkadot_runtime_common::SlowAdjustingFeeUpdate;
pub use pop_runtime_common::{
	AuraId, Balance, BlockNumber, Hash, Nonce, Signature, AVERAGE_ON_INITIALIZE_RATIO,
	BLOCK_PROCESSING_VELOCITY, DAYS, EXISTENTIAL_DEPOSIT, HOURS, MAXIMUM_BLOCK_WEIGHT, MICRO_UNIT,
	MILLI_UNIT, MINUTES, NORMAL_DISPATCH_RATIO, RELAY_CHAIN_SLOT_DURATION_MILLIS, SLOT_DURATION,
	UNINCLUDED_SEGMENT_CAPACITY, UNIT,
};
use sp_core::crypto::Ss58Codec;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{
		AccountIdConversion, BlakeTwo256, Block as BlockT, IdentifyAccount, IdentityLookup, Verify,
	},
};
pub use sp_runtime::{ExtrinsicInclusionMode, MultiAddress, Perbill, Permill};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
// XCM Imports
use xcm::latest::prelude::BodyId;

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

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
type Migrations = ();

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	// src: https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/constants/src/polkadot.rs#L70
	(items as Balance * 20 * UNIT + (bytes as Balance) * 100 * fee::MILLICENTS) / 100
}

/// Constants related to Polkadot fee payment.
/// Source: https://github.com/polkadot-fellows/runtimes/blob/main/system-parachains/constants/src/polkadot.rs#L65C47-L65C58
pub mod fee {
	use frame_support::{
		pallet_prelude::Weight,
		weights::{
			constants::ExtrinsicBaseWeight, FeePolynomial, WeightToFeeCoefficient,
			WeightToFeeCoefficients, WeightToFeePolynomial,
		},
	};
	use pop_runtime_common::{Balance, MILLI_UNIT};
	use smallvec::smallvec;
	pub use sp_runtime::Perbill;

	pub const CENTS: Balance = MILLI_UNIT * 10; // 100_000_000
	pub const MILLICENTS: Balance = CENTS / 1_000; // 100_000

	/// Cost of every transaction byte at Polkadot system parachains.
	///
	/// It is the Relay Chain (Polkadot) `TransactionByteFee` / 20.
	pub const TRANSACTION_BYTE_FEE: Balance = MILLICENTS / 2;

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl frame_support::weights::WeightToFee for WeightToFee {
		type Balance = Balance;

		fn weight_to_fee(weight: &Weight) -> Self::Balance {
			let time_poly: FeePolynomial<Balance> = RefTimeToFee::polynomial().into();
			let proof_poly: FeePolynomial<Balance> = ProofSizeToFee::polynomial().into();

			// Take the maximum instead of the sum to charge by the more scarce resource.
			time_poly.eval(weight.ref_time()).max(proof_poly.eval(weight.proof_size()))
		}
	}

	/// Maps the reference time component of `Weight` to a fee.
	pub struct RefTimeToFee;
	impl WeightToFeePolynomial for RefTimeToFee {
		type Balance = Balance;

		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// In Polkadot, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
			// The standard system parachain configuration is 1/20 of that, as in 1/200 CENT.
			let p = CENTS;
			let q = 200 * Balance::from(ExtrinsicBaseWeight::get().ref_time());

			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}

	/// Maps the proof size component of `Weight` to a fee.
	pub struct ProofSizeToFee;
	impl WeightToFeePolynomial for ProofSizeToFee {
		type Balance = Balance;

		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// Map 20kb proof to 1 CENT.
			let p = CENTS;
			let q = 20_000;

			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
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

impl pallet_authorship::Config for Runtime {
	type EventHandler = (CollatorSelection,);
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
}

parameter_types! {
	// increase ED 100 times to match system chains: 1_000_000_000
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT * 100;
}

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DoneSlashHandler = ();
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = fee::TRANSACTION_BYTE_FEE;
	pub SudoAddress: AccountId = AccountId::from_ss58check("15NMV2JX1NeMwarQiiZvuJ8ixUcvayFDcu1F9Wz1HNpSc8gP").expect("sudo address is valid SS58");
	pub MaintenanceAccount: AccountId = AccountId::from_ss58check("1Y3M8pnn3rJcxQn46SbocHcUHYfs4j8W2zHX7XNK99LGSVe").expect("maintenance address is valid SS58");
}

/// DealWithFees is used to handle fees and tips in the OnChargeTransaction trait,
/// by implementing OnUnbalanced.
pub struct DealWithFees;
impl OnUnbalanced<fungible::Credit<AccountId, Balances>> for DealWithFees {
	fn on_unbalanceds(
		mut fees_then_tips: impl Iterator<Item = fungible::Credit<AccountId, Balances>>,
	) {
		if let Some(mut fees) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
			}

			let split = fees.ration(50, 50);

			ResolveTo::<TreasuryAccount, Balances>::on_unbalanced(split.0);
			ResolveTo::<MaintenanceAccount, Balances>::on_unbalanced(split.1);
		}
	}
}

/// The type responsible for payment in pallet_transaction_payment.
pub type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, DealWithFees>;

impl pallet_transaction_payment::Config for Runtime {
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type OnChargeTransaction = OnChargeTransaction;
	type OperationalFeeMultiplier = ConstU8<5>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type WeightToFee = fee::WeightToFee;
}

parameter_types! {
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
	pub MessageQueueIdleServiceWeight: Weight = Perbill::from_percent(20) * RuntimeBlockWeights::get().max_block;
}

impl pallet_message_queue::Config for Runtime {
	type HeapSize = ConstU32<{ 64 * 1024 }>;
	type IdleMaxServiceWeight = MessageQueueIdleServiceWeight;
	type MaxStale = ConstU32<8>;
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor =
		pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor = xcm_builder::ProcessXcmMessage<
		AggregateMessageOrigin,
		xcm_executor::XcmExecutor<config::xcm::XcmConfig>,
		RuntimeCall,
	>;
	// The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)` origin:
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type RuntimeEvent = RuntimeEvent;
	type ServiceWeight = MessageQueueServiceWeight;
	type Size = u32;
	type WeightInfo = ();
}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type ChannelInfo = ParachainSystem;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	// Limit the number of messages and signals a HRMP channel can have at most
	type MaxActiveOutboundChannels = ConstU32<128>;
	type MaxInboundSuspended = ConstU32<1_000>;
	// Limit the number of HRMP channels
	type MaxPageSize = ConstU32<{ 103 * 1024 }>;
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
	type RuntimeEvent = RuntimeEvent;
	type VersionWrapper = PolkadotXcm;
	type WeightInfo = ();
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
}

parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl pallet_session::Config for Runtime {
	type Keys = SessionKeys;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type RuntimeEvent = RuntimeEvent;
	// Essentially just Aura, but let's be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type SessionManager = CollatorSelection;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type WeightInfo = ();
}

#[docify::export(aura_config)]
impl pallet_aura::Config for Runtime {
	type AllowMultipleBlocksPerSlot = ConstBool<true>;
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<100_000>;
	type SlotDuration = ConstU64<SLOT_DURATION>;
}

parameter_types! {
	pub const PotId: PalletId = PalletId(*b"PotStake");
	// StakingAdmin pluralistic body.
	pub const StakingAdminBodyId: BodyId = BodyId::Defense;
}

/// We allow root and the StakingAdmin to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	EnsureXcm<IsVoiceOfBody<RelayLocation, StakingAdminBodyId>>,
>;

impl pallet_collator_selection::Config for Runtime {
	type Currency = Balances;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type MaxCandidates = ConstU32<0>;
	type MaxInvulnerables = ConstU32<20>;
	type MinEligibleCollators = ConstU32<3>;
	type PotId = PotId;
	type RuntimeEvent = RuntimeEvent;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
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
}

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
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

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use sp_runtime::MultiSignature;
	use frame_support::{dispatch::GetDispatchInfo, pallet_prelude::Encode};
	use pallet_balances::AdjustmentDirection;
	use pallet_transaction_payment::OnChargeTransaction as OnChargeTransactionT;
	use sp_keyring::AccountKeyring as Keyring;
	use sp_runtime::{traits::Dispatchable, MultiSignature};
	use BalancesCall::*;
	use RuntimeCall::Balances as BalancesRuntimeCall;

	use super::*;
	use crate::Balances;
	#[test]
	fn filtering_force_adjust_total_issuance_works() {
		assert!(FilteredCalls::contains(&BalancesRuntimeCall(force_adjust_total_issuance {
			direction: AdjustmentDirection::Increase,
			delta: 0
		})));
	}

	#[test]
	fn filtering_force_set_balance_works() {
		assert!(FilteredCalls::contains(&BalancesRuntimeCall(force_set_balance {
			who: MultiAddress::Address32([0u8; 32]),
			new_free: 0,
		})));
	}

	#[test]
	fn filtering_force_transfer_works() {
		assert!(FilteredCalls::contains(&BalancesRuntimeCall(force_transfer {
			source: MultiAddress::Address32([0u8; 32]),
			dest: MultiAddress::Address32([0u8; 32]),
			value: 0,
		})));
	}

	#[test]
	fn filtering_force_unreserve_works() {
		assert!(FilteredCalls::contains(&BalancesRuntimeCall(force_unreserve {
			who: MultiAddress::Address32([0u8; 32]),
			amount: 0
		})));
	}

	#[test]
	fn filtering_configured() {
		assert_eq!(
			TypeId::of::<<Runtime as frame_system::Config>::BaseCallFilter>(),
			TypeId::of::<EverythingBut<FilteredCalls>>(),
		);
	}

	#[test]
	fn ed_is_correct() {
		assert_eq!(ExistentialDeposit::get(), EXISTENTIAL_DEPOSIT * 100);
		assert_eq!(ExistentialDeposit::get(), 1_000_000_000);
	}

	#[test]
	fn units_are_correct() {
		// UNIT should have 10 decimals
		assert_eq!(UNIT, 10_000_000_000);
		assert_eq!(MILLI_UNIT, 10_000_000);
		assert_eq!(MICRO_UNIT, 10_000);

		// fee specific units
		assert_eq!(fee::CENTS, 100_000_000);
		assert_eq!(fee::MILLICENTS, 100_000);
	}

	#[test]
	fn transaction_byte_fee_is_correct() {
		assert_eq!(fee::TRANSACTION_BYTE_FEE, 50_000);
	}

	#[test]
	fn deposit_works() {
		const UNITS: Balance = 10_000_000_000;
		const DOLLARS: Balance = UNITS; // 10_000_000_000
		const CENTS: Balance = DOLLARS / 100; // 100_000_000
		const MILLICENTS: Balance = CENTS / 1_000; // 100_000

		// https://github.com/polkadot-fellows/runtimes/blob/e220854a081f30183999848ce6c11ca62647bcfa/relay/polkadot/constants/src/lib.rs#L36
		fn relay_deposit(items: u32, bytes: u32) -> Balance {
			items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS
		}

		// https://github.com/polkadot-fellows/runtimes/blob/e220854a081f30183999848ce6c11ca62647bcfa/system-parachains/constants/src/polkadot.rs#L70
		fn system_para_deposit(items: u32, bytes: u32) -> Balance {
			relay_deposit(items, bytes) / 100
		}

		assert_eq!(deposit(2, 64), system_para_deposit(2, 64))
	}

	#[test]
	fn treasury_account_is_pallet_id_truncated() {
		assert_eq!(TreasuryAccount::get(), TREASURY_PALLET_ID.into_account_truncating());
	}

	pub fn new_test_ext() -> sp_io::TestExternalities {
		let initial_balance = 100_000_000 * UNIT;
		let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![
				(TreasuryAccount::get(), initial_balance),
				(MaintenanceAccount::get(), initial_balance),
				(Keyring::Alice.to_account_id(), initial_balance),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();
		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}

	#[test]
	fn transaction_payment_charges_fees_via_balances_and_funds_treasury_and_maintenance_equally() {
		new_test_ext().execute_with(|| {
			let who: AccountId = Keyring::Alice.to_account_id();
			let call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
			let fee = UNIT / 10;
			let tip = UNIT / 2;
			let fee_plus_tip = fee + tip;
			let treasury_balance = Balances::free_balance(&TreasuryAccount::get());
			let maintenance_balance = Balances::free_balance(&MaintenanceAccount::get());
			let who_balance = Balances::free_balance(&who);
			let dispatch_info = call.get_dispatch_info();

			// NOTE: OnChargeTransaction functions expect tip to be included within fee
			let liquidity_info =
				<OnChargeTransaction as OnChargeTransactionT<Runtime>>::withdraw_fee(
					&who,
					&call,
					&dispatch_info,
					fee_plus_tip,
					0,
				)
				.unwrap();
			<OnChargeTransaction as OnChargeTransactionT<Runtime>>::correct_and_deposit_fee(
				&who,
				&dispatch_info,
				&call.dispatch(RuntimeOrigin::signed(who.clone())).unwrap(),
				fee_plus_tip,
				0,
				liquidity_info,
			)
			.unwrap();

			let treasury_expected_balance = treasury_balance + (fee_plus_tip / 2);
			let maintenance_expected_balance = maintenance_balance + (fee_plus_tip / 2);
			let who_expected_balance = who_balance - fee_plus_tip;

			assert!(treasury_balance != 0);
			assert!(maintenance_expected_balance != 0);

			assert_eq!(Balances::free_balance(&TreasuryAccount::get()), treasury_expected_balance);
			assert_eq!(
				Balances::free_balance(&MaintenanceAccount::get()),
				maintenance_expected_balance
			);
			assert_eq!(Balances::free_balance(&who), who_expected_balance);
		})
	}

	#[test]
	fn test_fees_and_tip_split() {
		new_test_ext().execute_with(|| {
			let fee_amount = 10;
			let fee = <Balances as fungible::Balanced<AccountId>>::issue(fee_amount);
			let tip_amount = 20;
			let tip = <Balances as fungible::Balanced<AccountId>>::issue(tip_amount);
			let treasury_balance = Balances::free_balance(&TreasuryAccount::get());
			let maintenance_balance = Balances::free_balance(&MaintenanceAccount::get());
			DealWithFees::on_unbalanceds(vec![fee, tip].into_iter());

			// Each to get 50%, total is 30 so 15 each.
			assert_eq!(
				Balances::free_balance(&TreasuryAccount::get()),
				treasury_balance + ((fee_amount + tip_amount) / 2)
			);
			assert_eq!(
				Balances::free_balance(&MaintenanceAccount::get()),
				maintenance_balance + ((fee_amount + tip_amount) / 2)
			);
		});
	}

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
			TypeId::of::<(
				// Ensures sender is not the zero address.
				CheckNonZeroSender<Runtime>,
				// Ensures the runtime version included within in the transaction is the same as at
				// present.
				CheckSpecVersion<Runtime>,
				// Ensures the transaction version included in the transaction is the same as at
				// present.
				CheckTxVersion<Runtime>,
				// Genesis hash check to provide replay protection between different networks.
				CheckGenesis<Runtime>,
				// Checks transaction mortality.
				CheckMortality<Runtime>,
				// Nonce check and increment to give replay protection for transactions.
				CheckNonce<Runtime>,
				// Block resource (weight) limit check.
				CheckWeight<Runtime>,
				// Require the transactor pay for the transaction, optionally including a tip to
				// gain additional priority in the queue.
				ChargeTransactionPayment<Runtime>,
				// Checks the size of the node-side storage proof before and after executing a
				// given extrinsic.
				StorageWeightReclaim<Runtime>,
				// Extension for optionally verifying the metadata hash.
				CheckMetadataHash<Runtime>
			)>(),
		);
	}

	mod treasury {
		use super::*;

		#[test]
		fn asset_kind_is_nothing() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::AssetKind>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn balance_converter_is_set() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::BalanceConverter>(),
				TypeId::of::<UnityAssetBalanceConversion>(),
			);
		}

		#[cfg(feature = "runtime-benchmarks")]
		#[test]
		fn benchmark_helper_is_correct_type() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::BenchmarkHelper>(),
				TypeId::of::<BenchmarkHelper>(),
			);
		}

		#[test]
		fn beneficiary_is_account_id() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::Beneficiary>(),
				TypeId::of::<AccountId>(),
			);
		}

		#[test]
		fn beneficiary_lookup_is_identity_lookup() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::BeneficiaryLookup>(),
				TypeId::of::<IdentityLookup<AccountId>>(),
			);
		}

		#[test]
		fn block_number_provider_is_set() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::BlockNumberProvider>(),
				TypeId::of::<System>(),
			);
		}

		#[test]
		fn burn_is_nothing() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::Burn>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn max_approvals_is_set() {
			assert_eq!(<Runtime as pallet_treasury::Config>::MaxApprovals::get(), 100);
		}

		#[test]
		fn pallet_id_is_set() {
			assert_eq!(
				<Runtime as pallet_treasury::Config>::PalletId::get().encode(),
				PalletId(*b"treasury").encode()
			);
		}

		#[test]
		fn paymaster_is_correct_type() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::Paymaster>(),
				TypeId::of::<TreasuryPaymaster<<Runtime as pallet_treasury::Config>::Currency>>(),
			);
		}

		#[test]
		fn payout_period_is_set() {
			assert_eq!(<Runtime as pallet_treasury::Config>::PayoutPeriod::get(), 30 * DAYS);
		}

		#[test]
		fn reject_origin_is_correct() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::RejectOrigin>(),
				TypeId::of::<EnsureRoot<AccountId>>(),
			);
		}
		#[test]
		fn spend_funds_is_correct() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::SpendFunds>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn spend_origin_is_correct() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::SpendOrigin>(),
				TypeId::of::<NeverEnsureOrigin<Balance>>(),
			);
		}

		#[test]
		fn spend_period_is_six_days() {
			assert_eq!(<Runtime as pallet_treasury::Config>::SpendPeriod::get(), 6 * DAYS);
		}

		#[test]
		fn type_of_on_charge_transaction_is_correct() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_transaction_payment::Config>::OnChargeTransaction>(
				),
				TypeId::of::<OnChargeTransaction>(),
			);
		}

		#[test]
		fn weight_info_is_not_default() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_treasury::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}
}

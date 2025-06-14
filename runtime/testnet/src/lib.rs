#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// Public due to integration tests crate.
pub mod config;

/// The genesis state presets available.
pub mod genesis;
mod weights;

extern crate alloc;

use alloc::{borrow::Cow, vec, vec::Vec};

// ISMP imports
use ::ismp::{
	consensus::{ConsensusClientId, StateMachineHeight, StateMachineId},
	host::StateMachine,
	router::{Request, Response},
};
use codec::Encode;
use config::system::ConsensusHook;
use cumulus_pallet_parachain_system::RelayChainState;
use cumulus_primitives_core::AggregateMessageOrigin;
use frame_metadata_hash_extension::CheckMetadataHash;
use frame_support::{
	dispatch::{DispatchClass, DispatchInfo},
	genesis_builder_helper::{build_state, get_preset},
	parameter_types,
	traits::{
		tokens::nonfungibles_v2::Inspect, ConstBool, ConstU32, ConstU64, ConstU8, EitherOfDiverse,
		VariantCountOf,
	},
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
use pallet_ismp::offchain::{Leaf, Proof, ProofKeys};
use pallet_nfts_sdk as pallet_nfts;
use pallet_revive::{
	evm::{runtime::EthExtra, H160},
	AddressMapper,
};
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
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H256, U256};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, Get, IdentifyAccount, TransactionExtension, Verify},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};
pub use sp_runtime::{ExtrinsicInclusionMode, MultiAddress, Perbill, Permill};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use weights::{BlockExecutionWeight, ExtrinsicBaseWeight};
// XCM Imports
use xcm::{latest::prelude::BodyId, VersionedAsset, VersionedLocation};

use crate::config::{assets::TrustBackedAssetsInstance, system::RuntimeBlockWeights};

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

/// Migrations to apply on runtime upgrade.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
pub type Migrations = (
	// Note the multi-block migrations configured in pallet_migrations are not present here.
	// Permanent.
	cumulus_pallet_aura_ext::migration::MigrateV0ToV1<Runtime>,
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
	spec_version: 00_05_04,
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
	#[runtime::pallet_index(5)]
	pub type MultiBlockMigrations = pallet_migrations::Pallet<Runtime>;

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
		[pallet_migrations, MultiBlockMigrations]
		[pallet_sudo, Sudo]
		[pallet_collator_selection, CollatorSelection]
		[cumulus_pallet_parachain_system, ParachainSystem]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
		[cumulus_pallet_weight_reclaim, WeightReclaim]
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

impl_runtime_apis! {

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			Runtime::impl_slot_duration()
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> ExtrinsicInclusionMode{
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl frame_support::view_functions::runtime_api::RuntimeViewFunction<Block> for Runtime {
		fn execute_view_function(
			id: frame_support::view_functions::ViewFunctionId,
			input: Vec<u8>
		) -> Result<Vec<u8>, frame_support::view_functions::ViewFunctionDispatchError> {
			Runtime::execute_view_function(id, input)
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_contracts::ContractsApi<Block, AccountId, Balance, BlockNumber, Hash, EventRecord>
		for Runtime
	{
		fn call(
			origin: AccountId,
			dest: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			input_data: Vec<u8>,
		) -> pallet_contracts::ContractExecResult<Balance, EventRecord> {
			let gas_limit = gas_limit.unwrap_or(RuntimeBlockWeights::get().max_block);
			Contracts::bare_call(
				origin,
				dest,
				value,
				gas_limit,
				storage_deposit_limit,
				input_data,
				CONTRACTS_DEBUG_OUTPUT,
				CONTRACTS_EVENTS,
				pallet_contracts::Determinism::Enforced,
			)
		}

		fn instantiate(
			origin: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			code: pallet_contracts::Code<Hash>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> pallet_contracts::ContractInstantiateResult<AccountId, Balance, EventRecord>
		{
			let gas_limit = gas_limit.unwrap_or(RuntimeBlockWeights::get().max_block);
			Contracts::bare_instantiate(
				origin,
				value,
				gas_limit,
				storage_deposit_limit,
				code,
				data,
				salt,
				CONTRACTS_DEBUG_OUTPUT,
				CONTRACTS_EVENTS,
			)
		}

		fn upload_code(
			origin: AccountId,
			code: Vec<u8>,
			storage_deposit_limit: Option<Balance>,
			determinism: pallet_contracts::Determinism,
		) -> pallet_contracts::CodeUploadResult<Hash, Balance>
		{
			Contracts::bare_upload_code(origin, code, storage_deposit_limit, determinism)
		}

		fn get_storage(
			address: AccountId,
			key: Vec<u8>,
		) -> pallet_contracts::GetStorageResult {
			Contracts::get_storage(address, key)
		}
	}

	impl cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			included_hash: <Block as BlockT>::Hash,
			slot: cumulus_primitives_aura::Slot,
		) -> bool {
			Runtime::impl_can_build_upon(included_hash, slot)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	impl pallet_nfts_runtime_api::NftsApi<Block, AccountId, u32, u32> for Runtime {
		fn owner(collection: u32, item: u32) -> Option<AccountId> {
			<Nfts as Inspect<AccountId>>::owner(&collection, &item)
		}

		fn collection_owner(collection: u32) -> Option<AccountId> {
			<Nfts as Inspect<AccountId>>::collection_owner(&collection)
		}

		fn attribute(
			collection: u32,
			item: u32,
			key: Vec<u8>,
		) -> Option<Vec<u8>> {
			<Nfts as Inspect<AccountId>>::attribute(&collection, &item, &key)
		}

		fn custom_attribute(
			account: AccountId,
			collection: u32,
			item: u32,
			key: Vec<u8>,
		) -> Option<Vec<u8>> {
			<Nfts as Inspect<AccountId>>::custom_attribute(
				&account,
				&collection,
				&item,
				&key,
			)
		}

		fn system_attribute(
			collection: u32,
			item: Option<u32>,
			key: Vec<u8>,
		) -> Option<Vec<u8>> {
			<Nfts as Inspect<AccountId>>::system_attribute(&collection, item.as_ref(), &key)
		}

		fn collection_attribute(collection: u32, key: Vec<u8>) -> Option<Vec<u8>> {
			<Nfts as Inspect<AccountId>>::collection_attribute(&collection, &key)
		}
	}

	impl pallet_revive::ReviveApi<Block, AccountId, Balance, Nonce, BlockNumber> for Runtime
	{
		fn balance(address: H160) -> U256 {
			Revive::evm_balance(&address)
		}

		fn block_gas_limit() -> U256 {
			Revive::evm_block_gas_limit()
		}

		fn gas_price() -> U256 {
			Revive::evm_gas_price()
		}

		fn nonce(address: H160) -> Nonce {
			let account = <Runtime as pallet_revive::Config>::AddressMapper::to_account_id(&address);
			System::account_nonce(account)
		}

		fn eth_transact(tx: pallet_revive::evm::GenericTransaction) -> Result<pallet_revive::EthTransactInfo<Balance>, pallet_revive::EthTransactError>
		{
			let blockweights: BlockWeights = <Runtime as frame_system::Config>::BlockWeights::get();

			let tx_fee = |pallet_call, mut dispatch_info: DispatchInfo| {
				let call = RuntimeCall::Revive(pallet_call);
				dispatch_info.extension_weight = EthExtraImpl::get_eth_extension(0, 0u32.into()).weight(&call);
				let uxt: UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic::new_bare(call).into();

				pallet_transaction_payment::Pallet::<Runtime>::compute_fee(
					uxt.encoded_size() as u32,
					&dispatch_info,
					0u32.into(),
				)
			};

			Revive::bare_eth_transact(
				tx,
				blockweights.max_block,
				tx_fee,
			)
		}

		fn call(
			origin: AccountId,
			dest: H160,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			input_data: Vec<u8>,
		) -> pallet_revive::ContractResult<pallet_revive::ExecReturnValue, Balance> {
			Revive::bare_call(
				RuntimeOrigin::signed(origin),
				dest,
				value,
				gas_limit.unwrap_or(RuntimeBlockWeights::get().max_block),
				pallet_revive::DepositLimit::Balance(storage_deposit_limit.unwrap_or(u128::MAX)),
				input_data,
			)
		}

		fn instantiate(
			origin: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			code: pallet_revive::Code,
			data: Vec<u8>,
			salt: Option<[u8; 32]>,
		) -> pallet_revive::ContractResult<pallet_revive::InstantiateReturnValue, Balance>
		{
			Revive::bare_instantiate(
				RuntimeOrigin::signed(origin),
				value,
				gas_limit.unwrap_or(RuntimeBlockWeights::get().max_block),
				pallet_revive::DepositLimit::Balance(storage_deposit_limit.unwrap_or(u128::MAX)),
				code,
				data,
				salt,
			)
		}

		fn upload_code(
			origin: AccountId,
			code: Vec<u8>,
			storage_deposit_limit: Option<Balance>,
		) -> pallet_revive::CodeUploadResult<Balance>
		{
			Revive::bare_upload_code(
				RuntimeOrigin::signed(origin),
				code,
				storage_deposit_limit.unwrap_or(u128::MAX),
			)
		}

		fn get_storage(
			address: H160,
			key: [u8; 32],
		) -> pallet_revive::GetStorageResult {
			Revive::get_storage(
				address,
				key
			)
		}

		fn trace_block(
			block: Block,
			config: pallet_revive::evm::TracerConfig
		) -> Vec<(u32, pallet_revive::evm::CallTrace)> {
			use pallet_revive::tracing::trace;
			let mut tracer = config.build(Revive::evm_gas_from_weight);
			let mut traces = vec![];
			let (header, extrinsics) = block.deconstruct();

			Executive::initialize_block(&header);
			for (index, ext) in extrinsics.into_iter().enumerate() {
				trace(&mut tracer, || {
					let _ = Executive::apply_extrinsic(ext);
				});

				if let Some(tx_trace) = tracer.collect_traces().pop() {
					traces.push((index as u32, tx_trace));
				}
			}

			traces
		}

		fn trace_tx(
			block: Block,
			tx_index: u32,
			config: pallet_revive::evm::TracerConfig
		) -> Option<pallet_revive::evm::CallTrace> {
			use pallet_revive::tracing::trace;
			let mut tracer = config.build(Revive::evm_gas_from_weight);
			let (header, extrinsics) = block.deconstruct();

			Executive::initialize_block(&header);
			for (index, ext) in extrinsics.into_iter().enumerate() {
				if index as u32 == tx_index {
					trace(&mut tracer, || {
						let _ = Executive::apply_extrinsic(ext);
					});
					break;
				} else {
					let _ = Executive::apply_extrinsic(ext);
				}
			}

			tracer.collect_traces().pop()
		}

		fn trace_call(
			tx: pallet_revive::evm::GenericTransaction,
			config: pallet_revive::evm::TracerConfig)
			-> Result<pallet_revive::evm::CallTrace, pallet_revive::EthTransactError>
		{
			use pallet_revive::tracing::trace;
			let mut tracer = config.build(Revive::evm_gas_from_weight);
			let result = trace(&mut tracer, || Self::eth_transact(tx));

			if let Some(trace) = tracer.collect_traces().pop() {
				Ok(trace)
			} else if let Err(err) = result {
				Err(err)
			} else {
				Ok(Default::default())
			}
		}
	}

	impl pallet_ismp_runtime_api::IsmpRuntimeApi<Block, <Block as BlockT>::Hash> for Runtime {
		fn host_state_machine() -> StateMachine {
			<Runtime as pallet_ismp::Config>::HostStateMachine::get()
		}

		fn challenge_period(id: StateMachineId) -> Option<u64> {
			Ismp::challenge_period(id)
		}

		/// Generate a proof for the provided leaf indices
		fn generate_proof(
			keys: ProofKeys
		) -> Result<(Vec<Leaf>, Proof<<Block as BlockT>::Hash>), sp_mmr_primitives::Error> {
			Ismp::generate_proof(keys)
		}

		/// Fetch all ISMP events
		fn block_events() -> Vec<::ismp::events::Event> {
			Ismp::block_events()
		}

		/// Fetch all ISMP events and their extrinsic metadata
		fn block_events_with_metadata() -> Vec<(::ismp::events::Event, Option<u32>)> {
			Ismp::block_events_with_metadata()
		}

		/// Return the scale encoded consensus state
		fn consensus_state(id: ConsensusClientId) -> Option<Vec<u8>> {
			Ismp::consensus_states(id)
		}

		/// Return the timestamp this client was last updated in seconds
		fn state_machine_update_time(height: StateMachineHeight) -> Option<u64> {
			Ismp::state_machine_update_time(height)
		}

		/// Return the latest height of the state machine
		fn latest_state_machine_height(id: StateMachineId) -> Option<u64> {
			Ismp::latest_state_machine_height(id)
		}

		/// Get actual requests
		fn requests(commitments: Vec<H256>) -> Vec<Request> {
			Ismp::requests(commitments)
		}

		/// Get actual requests
		fn responses(commitments: Vec<H256>) -> Vec<Response> {
			Ismp::responses(commitments)
		}
	}

	impl ismp_parachain_runtime_api::IsmpParachainApi<Block> for Runtime {
		fn para_ids() -> Vec<u32> {
			IsmpParachain::para_ids()
		}

		fn current_relay_chain_state() -> RelayChainState {
			IsmpParachain::current_relay_chain_state()
		}
	}

	impl xcm_runtime_apis::trusted_query::TrustedQueryApi<Block> for Runtime {
		fn is_trusted_reserve(asset: VersionedAsset, location: VersionedLocation) -> xcm_runtime_apis::trusted_query::XcmTrustedQueryResult {
			PolkadotXcm::is_trusted_reserve(asset, location)
		}
		fn is_trusted_teleporter(asset: VersionedAsset, location: VersionedLocation) -> xcm_runtime_apis::trusted_query::XcmTrustedQueryResult {
			PolkadotXcm::is_trusted_teleporter(asset, location)
		}
	}

	impl xcm_runtime_apis::authorized_aliases::AuthorizedAliasersApi<Block> for Runtime {
		fn authorized_aliasers(target: VersionedLocation) -> Result<
			Vec<xcm_runtime_apis::authorized_aliases::OriginAliaser>,
			xcm_runtime_apis::authorized_aliases::Error
		> {
			PolkadotXcm::authorized_aliasers(target)
		}
		fn is_authorized_alias(origin: VersionedLocation, target: VersionedLocation) -> Result<
			bool,
			xcm_runtime_apis::authorized_aliases::Error
		> {
			PolkadotXcm::is_authorized_alias(origin, target)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect,
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::BenchmarkList;
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		#[allow(non_local_definitions)]
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, alloc::string::String> {
			use frame_benchmarking::{BenchmarkError, BenchmarkBatch};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {
				fn setup_set_code_requirements(code: &Vec<u8>) -> Result<(), BenchmarkError> {
					ParachainSystem::initialize_for_set_code_benchmark(code.len() as u32);
					Ok(())
				}

				fn verify_set_code() {
					System::assert_last_event(cumulus_pallet_parachain_system::Event::<Runtime>::ValidationFunctionStored.into());
				}
			}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, genesis::get_preset)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			genesis::presets()
		}
	}
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

#[test]
fn metadata_api_implemented() {
	use core::ops::Deref;

	use codec::Decode;
	use frame_metadata::{RuntimeMetadata, RuntimeMetadataPrefixed};
	const V16_UNSTABLE: u32 = u32::MAX;

	fn assert<T: sp_api::runtime_decl_for_metadata::Metadata<Block>>() {
		let opaque_meta: OpaqueMetadata = T::metadata();
		let prefixed_meta = RuntimeMetadataPrefixed::decode(&mut &opaque_meta.deref()[..]).unwrap();
		assert_eq!(prefixed_meta, Runtime::metadata());
		// Always returns metadata v14:
		// https://github.com/paritytech/polkadot-sdk/blob/c36d3066c082b769f20c31dfdbae77d8fd027a0d/substrate/frame/support/procedural/src/construct_runtime/expand/metadata.rs#L151
		let RuntimeMetadata::V14(_) = prefixed_meta.1 else {
			panic!("Expected metadata V14");
		};

		assert_eq!(T::metadata_versions(), vec![14, 15, V16_UNSTABLE]);

		let version = 15;
		let opaque_meta = T::metadata_at_version(version).expect("V15 should exist");
		let prefixed_meta_bytes = opaque_meta.deref();
		assert_eq!(prefixed_meta_bytes, Runtime::metadata_at_version(version).unwrap().deref());
		let prefixed_meta = RuntimeMetadataPrefixed::decode(&mut &prefixed_meta_bytes[..]).unwrap();
		// Ensure that we have the V15 variant.
		let RuntimeMetadata::V15(metadata) = prefixed_meta.1 else {
			panic!("Expected metadata V15");
		};
		assert!(!metadata.apis.is_empty());
		assert!(!metadata.pallets.is_empty());

		// Ensure metadata v16 is not provided.
		assert!(T::metadata_at_version(16).is_none());
	}
	sp_io::TestExternalities::new_empty().execute_with(|| assert::<Runtime>());
}

use core::marker::PhantomData;

use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use frame_support::{
	parameter_types,
	traits::{
		tokens::imbalance::ResolveTo, ConstU32, ContainsPair, Everything, Get, Nothing,
		TransformOrigin,
	},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use parachains_common::{
	message_queue::{NarrowOriginToSibling, ParaIdToSibling},
	xcm_config::{
		AllSiblingSystemParachains, ParentRelayOrSiblingParachains, RelayOrOtherSystemParachains,
	},
};
use polkadot_parachain_primitives::primitives::Sibling;
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use sp_runtime::traits::AccountIdConversion;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds,
	FrameTransactionalProcessor, FungibleAdapter, IsConcrete, ParentIsPreset, RelayChainAsNative,
	SendXcmFeeToAccount, SiblingParachainAsNative, SiblingParachainConvertsVia,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit,
	TrailingSetTopicAsId, UsingComponents, WithComputedOrigin, WithUniqueTopic,
	XcmFeeManagerFromComponents,
};
use xcm_executor::XcmExecutor;

use crate::{
	config::{governance::SudoAddress, monetary::fee::WeightToFee, system::RuntimeBlockWeights},
	AccountId, AllPalletsWithSystem, Balances, ParachainInfo, ParachainSystem,
	PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
	XcmpQueue, Perbill,
};

parameter_types! {
	pub const RelayLocation: Location = Location::parent();
	pub AssetHub: Location = Location::new(1, [Parachain(1000)]);
	pub const RelayNetwork: Option<NetworkId> = Some(Polkadot);
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorLocation = [GlobalConsensus(RelayNetwork::get().unwrap()), Parachain(ParachainInfo::parachain_id().into())].into();
	pub MessageQueueIdleServiceWeight: Weight = Perbill::from_percent(20) * RuntimeBlockWeights::get().max_block;
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
	pub TreasuryAccount: AccountId = PalletId(*b"treasury").into_account_truncating();
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
		xcm_executor::XcmExecutor<XcmConfig>,
		RuntimeCall,
	>;
	// The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)` origin:
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type RuntimeEvent = RuntimeEvent;
	type ServiceWeight = MessageQueueServiceWeight;
	type Size = u32;
	type WeightInfo = pallet_message_queue::weights::SubstrateWeight<Self>;
}

/// Type for specifying how a `Location` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = FungibleAdapter<
	// Use this currency:
	Balances,
	// Use this currency when it is a fungible asset matching the given location or name:
	IsConcrete<RelayLocation>,
	// Do a simple punn to convert an AccountId32 Location into a native chain account ID:
	LocationToAccountId,
	// Our chain's account ID type (we can't get away without mentioning it explicitly):
	AccountId,
	// We don't track any teleports.
	(),
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

pub type Barrier = TrailingSetTopicAsId<(
	TakeWeightCredit,
	AllowKnownQueryResponses<PolkadotXcm>,
	WithComputedOrigin<
		(
			AllowTopLevelPaidExecutionFrom<Everything>,
			AllowSubscriptionsFrom<ParentRelayOrSiblingParachains>,
		),
		UniversalLocation,
		ConstU32<8>,
	>,
)>;

/// Asset filter that allows native/relay asset if coming from a certain location.
// Borrowed from https://github.com/paritytech/polkadot-sdk/blob/ea458d0b95d819d31683a8a09ca7973ae10b49be/cumulus/parachains/runtimes/testing/penpal/src/xcm_config.rs#L239 for now
pub struct NativeAssetFrom<T>(PhantomData<T>);
impl<T: Get<Location>> ContainsPair<Asset, Location> for NativeAssetFrom<T> {
	fn contains(asset: &Asset, origin: &Location) -> bool {
		let loc = T::get();
		&loc == origin &&
			matches!(asset, Asset { id: AssetId(asset_loc), fun: Fungible(_a) }
			if *asset_loc == Location::from(Parent))
	}
}

/// Combinations of (Asset, Location) pairs which we trust as reserves.
pub type TrustedReserves = NativeAssetFrom<AssetHub>;

/// Locations that will not be charged fees in the executor,
/// either execution or delivery.
/// We only waive fees for system functions, which these locations represent.
pub type WaivedLocations = (RelayOrOtherSystemParachains<AllSiblingSystemParachains, Runtime>,);

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type Aliasers = Nothing;
	type AssetClaims = PolkadotXcm;
	type AssetExchanger = ();
	type AssetLocker = ();
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type AssetTrap = PolkadotXcm;
	type Barrier = Barrier;
	type CallDispatcher = RuntimeCall;
	type FeeManager = XcmFeeManagerFromComponents<
		WaivedLocations,
		SendXcmFeeToAccount<Self::AssetTransactor, TreasuryAccount>,
	>;
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type HrmpNewChannelOpenRequestHandler = ();
	type IsReserve = TrustedReserves;
	type IsTeleporter = ();
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type MessageExporter = ();
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type ResponseHandler = PolkadotXcm;
	type RuntimeCall = RuntimeCall;
	type SafeCallFilter = Everything;
	type SubscriptionService = PolkadotXcm;
	type Trader = UsingComponents<
		WeightToFee,
		RelayLocation,
		AccountId,
		Balances,
		ResolveTo<TreasuryAccount, Balances>,
	>;
	type TransactionalProcessor = FrameTransactionalProcessor;
	type UniversalAliases = Nothing;
	type UniversalLocation = UniversalLocation;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type XcmRecorder = PolkadotXcm;
	type XcmSender = XcmRouter;
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = WithUniqueTopic<(
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
)>;

impl pallet_xcm::Config for Runtime {
	type AdminOrigin = EnsureRoot<AccountId>;
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type MaxLockers = ConstU32<8>;
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type SovereignAccountOf = LocationToAccountId;
	type TrustedLockers = ();
	type UniversalLocation = UniversalLocation;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type WeightInfo = pallet_xcm::TestWeightInfo;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmReserveTransferFilter = Everything;
	type XcmRouter = XcmRouter;
	type XcmTeleportFilter = Nothing;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type ChannelInfo = ParachainSystem;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type MaxActiveOutboundChannels = ConstU32<128>;
	type MaxInboundSuspended = ConstU32<128>;
	type MaxPageSize = ConstU32<{ 103 * 1024 }>;
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
	type RuntimeEvent = RuntimeEvent;
	type VersionWrapper = PolkadotXcm;
	type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Runtime>;
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use super::*;

	// Only reserve accepted is the relay asset from Asset Hub.
	#[test]
	fn reserves() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::IsReserve>(),
			TypeId::of::<NativeAssetFrom<AssetHub>>(),
		);
	}

	#[test]
	fn asset_hub_as_relay_asset_reserve() {
		assert!(TrustedReserves::contains(
			&Asset::from((AssetId::from(Parent), Fungibility::from(100u128))),
			&AssetHub::get(),
		));
	}

	#[test]
	fn relay_as_relay_asset_reserve_fails() {
		let relay_asset = Asset::from((AssetId::from(Parent), Fungibility::from(100u128)));
		assert!(!TrustedReserves::contains(&relay_asset, &Parent.into()));
	}

	// Decline native asset from another parachain.
	#[test]
	fn decline_sibling_native_assets() {
		let chain_x = Location::new(1, [Parachain(4242)]);
		let chain_x_asset =
			Asset::from((AssetId::from(chain_x.clone()), Fungibility::from(100u128)));
		assert!(!TrustedReserves::contains(&chain_x_asset, &chain_x));
	}

	// Decline non native asset from another parachain. Either a native asset as foreign asset on
	// another parachain or a local asset from e.g. `pallet-assets`.
	#[test]
	fn decline_sibling_non_native_assets() {
		// Native asset X of chain Y example.
		let chain_x = Location::new(1, [Parachain(4242)]);
		let chain_y = Location::new(1, [Parachain(6969)]);
		let chain_x_asset = Asset::from((AssetId::from(chain_x), Fungibility::from(100u128)));
		assert!(!TrustedReserves::contains(&chain_x_asset, &chain_y));
		// `pallet-assets` example.
		let usd = Location::new(1, [Parachain(1000), PalletInstance(50), GeneralIndex(1337)]);
		let usd_asset = Asset::from((AssetId::from(usd), Fungibility::from(100u128)));
		assert!(!TrustedReserves::contains(&usd_asset, &chain_y));
	}

	#[test]
	fn message_queue_heap_size() {
		assert_eq!(
			<<Runtime as pallet_message_queue::Config>::HeapSize as Get<u32>>::get(),
			64 * 1024
		);
	}

	#[test]
	fn message_queue_limits_idle_max_service_weight() {
		assert_eq!(
			<<Runtime as pallet_message_queue::Config>::IdleMaxServiceWeight as Get<Weight>>::get(),
			Perbill::from_percent(20) * RuntimeBlockWeights::get().max_block
		);
	}

	#[test]
	fn message_queue_limits_max_stale_pages() {
		assert_eq!(<<Runtime as pallet_message_queue::Config>::MaxStale as Get<u32>>::get(), 8);
	}

	#[test]
	fn message_queue_processing_delegated_to_executor() {
		#[cfg(feature = "runtime-benchmarks")]
		type MessageProcessor =
			pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;
		#[cfg(not(feature = "runtime-benchmarks"))]
		type MessageProcessor = xcm_builder::ProcessXcmMessage<
			AggregateMessageOrigin,
			XcmExecutor<XcmConfig>,
			RuntimeCall,
		>;
		assert_eq!(
			TypeId::of::<<Runtime as pallet_message_queue::Config>::MessageProcessor>(),
			TypeId::of::<MessageProcessor>()
		);
	}

	#[test]
	fn message_queue_change_handler_uses_xcmp_queue() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_message_queue::Config>::QueueChangeHandler>(),
			TypeId::of::<NarrowOriginToSibling<XcmpQueue>>()
		);
	}

	#[test]
	fn message_queue_paused_query_handler_uses_xcmp_queue() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_message_queue::Config>::QueuePausedQuery>(),
			TypeId::of::<NarrowOriginToSibling<XcmpQueue>>()
		);
	}

	#[test]
	fn message_queue_limits_service_weight() {
		assert_eq!(
			<<Runtime as pallet_message_queue::Config>::ServiceWeight as Get<Weight>>::get(),
			Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block
		);
	}

	#[test]
	fn message_queue_uses_u32_page_size() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_message_queue::Config>::Size>(),
			TypeId::of::<u32>()
		);
	}

	#[test]
	fn message_queue_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_message_queue::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn location_to_account_id_matches_configuration() {
		assert_eq!(
			TypeId::of::<LocationToAccountId>(),
			TypeId::of::<(
				ParentIsPreset<AccountId>,
				SiblingParachainConvertsVia<Sibling, AccountId>,
				AccountId32Aliases<RelayNetwork, AccountId>,
			)>()
		);
	}

	#[test]
	fn local_asset_transactor_matches_configuration() {
		assert_eq!(
			TypeId::of::<LocalAssetTransactor>(),
			TypeId::of::<
				FungibleAdapter<
					Balances,
					IsConcrete<RelayLocation>,
					LocationToAccountId,
					AccountId,
					(),
				>,
			>()
		);
	}

	#[test]
	fn xcm_origin_to_transact_dispatch_origin_matches_configuration() {
		assert_eq!(
			TypeId::of::<XcmOriginToTransactDispatchOrigin>(),
			TypeId::of::<(
				SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
				RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
				SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
				SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
				XcmPassthrough<RuntimeOrigin>,
			)>()
		);
	}

	#[test]
	fn barrier_configuration() {
		assert_eq!(
			TypeId::of::<Barrier>(),
			TypeId::of::<
				TrailingSetTopicAsId<(
					TakeWeightCredit,
					AllowKnownQueryResponses<PolkadotXcm>,
					WithComputedOrigin<
						(
							AllowTopLevelPaidExecutionFrom<Everything>,
							AllowSubscriptionsFrom<ParentRelayOrSiblingParachains>,
						),
						UniversalLocation,
						ConstU32<8>,
					>,
				)>,
			>()
		);
	}

	#[test]
	fn xcm_executor_does_not_have_aliasers() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::Aliasers>(),
			TypeId::of::<Nothing>(),
		);
	}

	#[test]
	fn xcm_executor_asset_claims_via_xcm() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::AssetClaims>(),
			TypeId::of::<PolkadotXcm>(),
		);
	}

	#[test]
	fn xcm_executor_asset_exchanger_is_disabled() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::AssetExchanger>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn xcm_executor_asset_locker_is_disabled() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::AssetLocker>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn xcm_executor_uses_local_asset_transactor() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::AssetTransactor>(),
			TypeId::of::<LocalAssetTransactor>(),
		);
	}

	#[test]
	fn xcm_executor_traps_assets_via_xcm() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::AssetTrap>(),
			TypeId::of::<PolkadotXcm>(),
		);
	}

	#[test]
	fn xcm_executor_configures_barrier() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::Barrier>(),
			TypeId::of::<Barrier>(),
		);
	}

	#[test]
	fn xcm_executor_call_dispatcher_is_runtime_call() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::CallDispatcher>(),
			TypeId::of::<RuntimeCall>(),
		);
	}

	#[test]
	fn xcm_executor_fee_manager_resolves_to_treasury() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::FeeManager>(),
			TypeId::of::<
				XcmFeeManagerFromComponents<
					WaivedLocations,
					SendXcmFeeToAccount<
						<XcmConfig as xcm_executor::Config>::AssetTransactor,
						TreasuryAccount,
					>,
				>,
			>(),
		);
	}

	#[test]
	fn xcm_executor_hrmp_accepted_handler_is_disabled() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::HrmpChannelAcceptedHandler>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn xcm_executor_hrmp_closed_handler_is_disabled() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::HrmpChannelClosingHandler>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn xcm_executor_hrmp_new_request_handler_is_disabled() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::HrmpNewChannelOpenRequestHandler>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn xcm_executor_is_reser_is_trusted_reserves() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::IsReserve>(),
			TypeId::of::<TrustedReserves>(),
		);
	}

	#[test]
	fn xcm_executor_does_not_configure_teleporters() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::IsTeleporter>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn xcm_executor_limits_assets_in_holdings() {
		assert_eq!(
			<<XcmConfig as xcm_executor::Config>::MaxAssetsIntoHolding as Get<u32>>::get(),
			64,
		);
	}

	#[test]
	fn xcm_executor_message_exporter_is_dissabled() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::MessageExporter>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn xcm_executor_converts_origin() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::OriginConverter>(),
			TypeId::of::<XcmOriginToTransactDispatchOrigin>(),
		);
	}

	#[test]
	fn xcm_executor_uses_all_pallets_with_system() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::PalletInstancesInfo>(),
			TypeId::of::<AllPalletsWithSystem>(),
		);
	}

	#[test]
	fn xcm_executor_transact_filter_allows_everything() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::SafeCallFilter>(),
			TypeId::of::<Everything>(),
		);
	}

	#[test]
	fn xcm_executor_handles_version_subscriptions_via_xcm() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::SubscriptionService>(),
			TypeId::of::<PolkadotXcm>(),
		);
	}

	#[test]
	fn xcm_executor_trader_is_configured() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::Trader>(),
			TypeId::of::<
				UsingComponents<
					WeightToFee,
					RelayLocation,
					AccountId,
					Balances,
					ResolveTo<TreasuryAccount, Balances>,
				>,
			>(),
		);
	}

	#[test]
	fn xcm_executor_transactional_processor_uses_frame() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::TransactionalProcessor>(),
			TypeId::of::<FrameTransactionalProcessor>(),
		);
	}

	#[test]
	fn xcm_executor_universal_aliases_disabled() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::UniversalAliases>(),
			TypeId::of::<Nothing>(),
		);
	}

	#[test]
	fn xcm_executor_weigher_uses_fixed_wieght_bounds() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::Weigher>(),
			TypeId::of::<FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>>(),
		);
	}

	#[test]
	fn xcm_executor_uses_xcm_as_recorder_for_dry_runs() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::XcmRecorder>(),
			TypeId::of::<PolkadotXcm>(),
		);
	}

	#[test]
	fn xcm_executor_uses_ump_for_relay_and_xcmp_for_paras() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::XcmSender>(),
			TypeId::of::<
				WithUniqueTopic<(
					cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
					XcmpQueue,
				)>,
			>(),
		);
	}

	#[test]
	fn xcm_executor_routes_query_responses() {
		assert_eq!(
			TypeId::of::<<XcmConfig as xcm_executor::Config>::ResponseHandler>(),
			TypeId::of::<PolkadotXcm>(),
		);
	}

	#[test]
	fn pallet_xcm_admin_origin_ensures_root() {
		assert_eq!(
			TypeId::of::<<XcmConfig as pallet_xcm::Config>::AdminOrigin>(),
			TypeId::of::<EnsureRoot<AccountId>>(),
		);
	}

	#[test]
	fn pallet_xcm_advertises_current_xcm_version() {
		assert_eq!(
			TypeId::of::<<XcmConfig as pallet_xcm::Config>::AdvertisedXcmVersion>(),
			TypeId::of::<pallet_xcm::CurrentXcmVersion>(),
		);
	}

	#[test]
	fn pallet_xcm_uses_balances() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::Currency>(),
			TypeId::of::<Balances>(),
		);
	}

	#[test]
	fn pallet_xcm_asset_matching_is_disabled() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::CurrencyMatcher>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn pallet_xcm_message_execute_xcm_origin_uses_signed_to_accountid32() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::ExecuteXcmOrigin>(),
			TypeId::of::<
				EnsureXcmOrigin<
					RuntimeOrigin,
					SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>,
				>,
			>(),
		);
	}

	#[test]
	fn pallet_xcm_limits_max_lockers() {
		assert_eq!(<<Runtime as pallet_xcm::Config>::MaxLockers as Get<u32>>::get(), 8);
	}

	#[test]
	fn pallet_xcm_max_remote_lock_consumers_is_0() {
		assert_eq!(<<Runtime as pallet_xcm::Config>::MaxRemoteLockConsumers as Get<u32>>::get(), 0);
	}

	#[test]
	fn pallet_xcm_remote_consider_identifier_is_disabled() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::RemoteLockConsumerIdentifier>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn pallet_xcm_send_xcm_origin_is_configured() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::SendXcmOrigin>(),
			TypeId::of::<
				EnsureXcmOrigin<
					RuntimeOrigin,
					SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>,
				>,
			>(),
		);
	}

	#[test]
	fn pallet_xcm_sovereign_account_uses_location_converter() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::SovereignAccountOf>(),
			TypeId::of::<LocationToAccountId>(),
		);
	}

	#[test]
	fn pallet_xcm_does_not_have_trusted_lockers() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::TrustedLockers>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn pallet_xcm_universal_location_is_configured() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::UniversalLocation>(),
			TypeId::of::<UniversalLocation>(),
		);
	}

	#[test]
	fn pallet_xcm_weigher_uses_fixed_weight_bounds() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::Weigher>(),
			TypeId::of::<FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>>(),
		);
	}

	#[test]
	fn pallet_xcm_does_not_use_default_weights() {
		assert_ne!(TypeId::of::<<Runtime as pallet_xcm::Config>::WeightInfo>(), TypeId::of::<()>(),);
	}

	#[test]
	fn pallet_xcmp_queue_channel_info_via_parachain_system() {
		assert_eq!(
			TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::ChannelInfo>(),
			TypeId::of::<ParachainSystem>()
		);
	}

	#[test]
	fn pallet_xcm_execute_filters_nothing() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::XcmExecuteFilter>(),
			TypeId::of::<Everything>(),
		);
	}

	#[test]
	fn pallet_xcm_executor_is_configured() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::XcmExecutor>(),
			TypeId::of::<XcmExecutor<XcmConfig>>(),
		);
	}

	#[test]
	fn pallet_xcm_reserve_transfer_filter_only_allows_dot_from_ah() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::XcmReserveTransferFilter>(),
			TypeId::of::<NativeAssetFrom<AssetHub>>(),
		);
	}

	#[test]
	fn pallet_xcm_router_uses_ump_for_relay_and_xcmp_for_para() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::XcmRouter>(),
			TypeId::of::<
				WithUniqueTopic<(
					cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
					XcmpQueue,
				)>,
			>(),
		);
	}

	#[test]
	fn pallet_xcm_teleport_filter_is_nothing() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_xcm::Config>::XcmTeleportFilter>(),
			TypeId::of::<Nothing>(),
		);
	}

	#[test]
	fn pallet_xcm_declares_version_discovery_queue_size() {
		assert_eq!(<Runtime as pallet_xcm::Config>::VERSION_DISCOVERY_QUEUE_SIZE, 100);
	}

	#[test]
	fn pallet_xcmp_queue_controller_origin_ensures_root() {
		assert_eq!(
			TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::ControllerOrigin>(),
			TypeId::of::<EnsureRoot<AccountId>>()
		);
	}

	#[test]
	fn pallet_xcmp_queue_controller_origin_converter_configuration() {
		assert_eq!(
			TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::ControllerOriginConverter>(
			),
			TypeId::of::<XcmOriginToTransactDispatchOrigin>()
		);
	}

	#[test]
	fn pallet_xcmp_queue_limits_outbound_channels() {
		assert_eq!(
			<<Runtime as cumulus_pallet_xcmp_queue::Config>::MaxActiveOutboundChannels as Get<
				u32,
			>>::get(),
			128
		);
	}

	#[test]
	fn pallet_xcmp_queue_limits_inbound_suspended_channels() {
		assert_eq!(
			<<Runtime as cumulus_pallet_xcmp_queue::Config>::MaxInboundSuspended as Get<u32>>::get(
			),
			128
		);
	}

	#[test]
	fn pallet_xcmp_queue_limits_hrmp_message_page_size() {
		assert_eq!(
			<<Runtime as cumulus_pallet_xcmp_queue::Config>::MaxPageSize as Get<u32>>::get(),
			103 * 1024
		);
	}

	#[test]
	#[ignore]
	fn pallet_xcmp_queue_price_for_sibling_delivery() {
		assert_eq!(
			TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::PriceForSiblingDelivery>(),
			TypeId::of::<NoPriceForMessageDelivery<ParaId>>()
		);
	}

	#[test]
	fn pallet_xcmp_queue_versions_xcm() {
		assert_eq!(
			TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::VersionWrapper>(),
			TypeId::of::<PolkadotXcm>(),
		);
	}

	#[test]
	fn pallet_xcmp_queue_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn pallet_xcmp_queue_uses_message_queue() {
		assert_eq!(
			TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::XcmpQueue>(),
			TypeId::of::<
				TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>,
			>(),
		);
	}
}

use core::marker::PhantomData;

use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use frame_support::{
	parameter_types,
	traits::{
		tokens::imbalance::ResolveTo, ConstU32, Contains, ContainsPair, Equals, Everything, Get,
		Nothing, TransformOrigin,
	},
	weights::Weight,
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use parachains_common::{
	message_queue::{NarrowOriginToSibling, ParaIdToSibling},
	xcm_config::ParentRelayOrSiblingParachains,
};
use polkadot_parachain_primitives::primitives::Sibling;
use polkadot_runtime_common::xcm_sender::ExponentialPrice;
use sp_runtime::Vec;
use xcm::latest::prelude::*;
use xcm_builder::{
	AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
	AllowTopLevelPaidExecutionFrom, EnsureXcmOrigin, FrameTransactionalProcessor, FungibleAdapter,
	IsConcrete, ParentIsPreset, RelayChainAsNative, SendXcmFeeToAccount, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeWeightCredit, TrailingSetTopicAsId, UsingComponents,
	WeightInfoBounds, WithComputedOrigin, WithUniqueTopic, XcmFeeManagerFromComponents,
};
use xcm_executor::XcmExecutor;

use crate::{
	config::{
		monetary::{
			fee::{WeightToFee, CENTS},
			TransactionByteFee, TreasuryAccount,
		},
		system::RuntimeBlockWeights,
	},
	weights, AccountId, AllPalletsWithSystem, Balances, MessageQueue, ParachainInfo,
	ParachainSystem, Perbill, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin,
	XcmpQueue,
};

parameter_types! {
	pub const RelayLocation: Location = Location::parent();
	pub AssetHub: Location = Location::new(1, [Parachain(1000)]);
	pub const RelayNetwork: Option<NetworkId> = Some(Polkadot);
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorLocation = [GlobalConsensus(RelayNetwork::get().unwrap()), Parachain(ParachainInfo::parachain_id().into())].into();
	pub MessageQueueIdleServiceWeight: Weight = Perbill::from_percent(20) * RuntimeBlockWeights::get().max_block;
	pub MessageQueueServiceWeight: Weight = Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block;
	pub const BaseDeliveryFee: u128 = CENTS.saturating_mul(3);
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

/// Rules defining whether we should execute a given XCM.
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

/// Filter to determine if all specified assets are supported, used with
/// reserve-transfers.
pub struct FilterByAssets<Assets>(PhantomData<Assets>);
impl<Assets: Contains<Location>> Contains<(Location, Vec<Asset>)> for FilterByAssets<Assets> {
	fn contains(t: &(Location, Vec<Asset>)) -> bool {
		t.1.iter().all(|a| Assets::contains(&a.id.0))
	}
}

/// Combinations of (Asset, Location) pairs which we trust as reserves.
pub type TrustedReserves = NativeAssetFrom<AssetHub>;

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
}

/// XCM executor configuration.
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
		// No locations have waived fees.
		(),
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
	type Weigher =
		WeightInfoBounds<weights::xcm::PopXcmWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;
	type XcmRecorder = PolkadotXcm;
	type XcmSender = XcmRouter;
}

/// Convert an `Origin` into an `AccountId32`.
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
	type MaxRemoteLockConsumers = ();
	type RemoteLockConsumerIdentifier = ();
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type SovereignAccountOf = LocationToAccountId;
	type TrustedLockers = ();
	type UniversalLocation = UniversalLocation;
	type Weigher =
		WeightInfoBounds<weights::xcm::PopXcmWeight<RuntimeCall>, RuntimeCall, MaxInstructions>;
	type WeightInfo = weights::pallet_xcm::WeightInfo<Runtime>;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmReserveTransferFilter = FilterByAssets<Equals<RelayLocation>>;
	type XcmRouter = XcmRouter;
	type XcmTeleportFilter = Nothing;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

/// Means to price the delivery of an XCM to a sibling chain.
pub type PriceForSiblingDelivery =
	ExponentialPrice<RelayLocation, BaseDeliveryFee, TransactionByteFee, XcmpQueue>;

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type ChannelInfo = ParachainSystem;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	// Limit the number of messages and signals a HRMP channel can have at most.
	type MaxActiveOutboundChannels = ConstU32<128>;
	// Limits the number of inbound channels that we can suspend at the same time.
	// A value close to the number of possible channels seems to be a sensible configuration.
	type MaxInboundSuspended = ConstU32<128>;
	// Limit the number of HRMP channels.
	// note: https://github.com/polkadot-fellows/runtimes/blob/76d1fa680d00c3e447e40199e7b2250862ad4bfa/system-parachains/asset-hubs/asset-hub-polkadot/src/lib.rs#L692C2-L693C90
	type MaxPageSize = ConstU32<{ 103 * 1024 }>;
	type PriceForSiblingDelivery = PriceForSiblingDelivery;
	type RuntimeEvent = RuntimeEvent;
	type VersionWrapper = PolkadotXcm;
	type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Runtime>;
	// Enqueue XCMP messages from siblings for later processing.
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use polkadot_runtime_common::xcm_sender::*;
	use polkadot_runtime_parachains::FeeTracker;
	use sp_runtime::FixedPointNumber;
	use xcm_executor::traits::{FeeManager, FeeReason};

	use super::*;
	use crate::System;

	fn new_test_ext() -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::new_empty();
		ext.execute_with(|| System::set_block_number(1));
		ext
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

	mod message_queue {
		use super::*;

		#[test]
		fn heap_size() {
			assert_eq!(
				<<Runtime as pallet_message_queue::Config>::HeapSize as Get<u32>>::get(),
				64 * 1024
			);
		}

		#[test]
		fn limits_idle_max_service_weight() {
			assert_eq!(
				<<Runtime as pallet_message_queue::Config>::IdleMaxServiceWeight as Get<Weight>>::get(),
				Perbill::from_percent(20) * RuntimeBlockWeights::get().max_block
			);
		}

		#[test]
		fn limits_max_stale_pages() {
			assert_eq!(<<Runtime as pallet_message_queue::Config>::MaxStale as Get<u32>>::get(), 8);
		}

		#[test]
		fn processing_delegated_to_executor() {
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
		fn change_handler_uses_xcmp_queue() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_message_queue::Config>::QueueChangeHandler>(),
				TypeId::of::<NarrowOriginToSibling<XcmpQueue>>()
			);
		}

		#[test]
		fn paused_query_handler_uses_xcmp_queue() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_message_queue::Config>::QueuePausedQuery>(),
				TypeId::of::<NarrowOriginToSibling<XcmpQueue>>()
			);
		}

		#[test]
		fn limits_service_weight() {
			assert_eq!(
				<<Runtime as pallet_message_queue::Config>::ServiceWeight as Get<Weight>>::get(),
				Perbill::from_percent(35) * RuntimeBlockWeights::get().max_block
			);
		}

		#[test]
		fn uses_u32_page_size() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_message_queue::Config>::Size>(),
				TypeId::of::<u32>()
			);
		}

		#[test]
		fn does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_message_queue::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}
	}

	mod xcm_executor_config {
		use super::*;

		mod reserves {
			use super::*;

			// There's only one reserve and it is AssetHub for DOT token.
			#[test]
			fn only_reserve_is_ah_for_dot() {
				assert_eq!(
					TypeId::of::<<XcmConfig as xcm_executor::Config>::IsReserve>(),
					TypeId::of::<NativeAssetFrom<AssetHub>>(),
				);
			}

			#[test]
			fn asset_hub_as_relay_asset_reserve() {
				assert!(<TrustedReserves as ContainsPair<Asset, Location>>::contains(
					&Asset::from((AssetId::from(Parent), Fungibility::from(100u128))),
					&AssetHub::get(),
				));
			}

			#[test]
			fn relay_as_relay_asset_reserve_fails() {
				let relay_asset = Asset::from((AssetId::from(Parent), Fungibility::from(100u128)));
				assert!(!<TrustedReserves as ContainsPair<Asset, Location>>::contains(
					&relay_asset,
					&Parent.into()
				));
			}

			// Decline native asset from another parachain.
			#[test]
			fn decline_sibling_native_assets() {
				let chain_x = Location::new(1, [Parachain(4242)]);
				let chain_x_asset =
					Asset::from((AssetId::from(chain_x.clone()), Fungibility::from(100u128)));
				assert!(!<TrustedReserves as ContainsPair<Asset, Location>>::contains(
					&chain_x_asset,
					&chain_x
				));
			}

			// Decline non native asset from another parachain. Either a native asset as foreign
			// asset on another parachain or a local asset from e.g. `pallet-assets`.
			#[test]
			fn decline_sibling_non_native_assets() {
				// Native asset X of chain Y example.
				let chain_x = Location::new(1, [Parachain(4242)]);
				let chain_y = Location::new(1, [Parachain(6969)]);
				let chain_x_asset =
					Asset::from((AssetId::from(chain_x), Fungibility::from(100u128)));
				assert!(!<TrustedReserves as ContainsPair<Asset, Location>>::contains(
					&chain_x_asset,
					&chain_y
				));
				// `pallet-assets` example.
				let usd =
					Location::new(1, [Parachain(1000), PalletInstance(50), GeneralIndex(1337)]);
				let usd_asset = Asset::from((AssetId::from(usd), Fungibility::from(100u128)));
				assert!(!<TrustedReserves as ContainsPair<Asset, Location>>::contains(
					&usd_asset, &chain_y
				));
			}
		}

		#[test]
		fn does_not_have_aliasers() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::Aliasers>(),
				TypeId::of::<Nothing>(),
			);
		}

		#[test]
		fn asset_claims_via_xcm() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::AssetClaims>(),
				TypeId::of::<PolkadotXcm>(),
			);
		}

		#[test]
		fn asset_exchanger_is_disabled() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::AssetExchanger>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn asset_locker_is_disabled() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::AssetLocker>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn uses_local_asset_transactor() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::AssetTransactor>(),
				TypeId::of::<LocalAssetTransactor>(),
			);
		}

		#[test]
		fn traps_assets_via_xcm() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::AssetTrap>(),
				TypeId::of::<PolkadotXcm>(),
			);
		}

		#[test]
		fn configures_barrier() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::Barrier>(),
				TypeId::of::<Barrier>(),
			);
		}

		#[test]
		fn call_dispatcher_is_runtime_call() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::CallDispatcher>(),
				TypeId::of::<RuntimeCall>(),
			);
		}

		#[test]
		fn fee_manager_resolves_to_treasury() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::FeeManager>(),
				TypeId::of::<
					XcmFeeManagerFromComponents<
						(),
						SendXcmFeeToAccount<
							<XcmConfig as xcm_executor::Config>::AssetTransactor,
							TreasuryAccount,
						>,
					>,
				>(),
			);
		}

		#[test]
		fn no_locations_are_waived() {
			let locations = [
				Location::here(),
				Location::parent(),
				Location::new(1, [Parachain(1000)]),
				Location::new(1, [Parachain(1000), PalletInstance(50), GeneralIndex(1984)]),
				Location::new(
					1,
					[Parachain(1000), AccountId32 { network: None, id: Default::default() }],
				),
			];
			for location in locations {
				assert!(!<<XcmConfig as xcm_executor::Config>::FeeManager>::is_waived(
					Some(&location),
					FeeReason::TransferReserveAsset
				));
			}
		}

		#[test]
		fn hrmp_accepted_handler_is_disabled() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::HrmpChannelAcceptedHandler>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn hrmp_closed_handler_is_disabled() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::HrmpChannelClosingHandler>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn hrmp_new_request_handler_is_disabled() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::HrmpNewChannelOpenRequestHandler>(
				),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn trusted_reserves_are_provided() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::IsReserve>(),
				TypeId::of::<TrustedReserves>(),
			);
		}

		#[test]
		fn does_not_configure_teleporters() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::IsTeleporter>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn limits_assets_in_holdings() {
			assert_eq!(
				<<XcmConfig as xcm_executor::Config>::MaxAssetsIntoHolding as Get<u32>>::get(),
				64,
			);
		}

		#[test]
		fn message_exporter_is_disabled() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::MessageExporter>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn converts_origin() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::OriginConverter>(),
				TypeId::of::<XcmOriginToTransactDispatchOrigin>(),
			);
		}

		#[test]
		fn uses_all_pallets_with_system() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::PalletInstancesInfo>(),
				TypeId::of::<AllPalletsWithSystem>(),
			);
		}

		#[test]
		fn routes_query_responses() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::ResponseHandler>(),
				TypeId::of::<PolkadotXcm>(),
			);
		}

		#[test]
		fn transact_filter_allows_everything() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::SafeCallFilter>(),
				TypeId::of::<Everything>(),
			);
		}

		#[test]
		fn handles_version_subscriptions_via_xcm() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::SubscriptionService>(),
				TypeId::of::<PolkadotXcm>(),
			);
		}

		#[test]
		fn trader_is_configured() {
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
		fn transactional_processor_uses_frame() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::TransactionalProcessor>(),
				TypeId::of::<FrameTransactionalProcessor>(),
			);
		}

		#[test]
		fn universal_aliases_disabled() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::UniversalAliases>(),
				TypeId::of::<Nothing>(),
			);
		}

		#[test]
		fn universal_location_is_set() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::UniversalLocation>(),
				TypeId::of::<UniversalLocation>(),
			);
		}

		#[test]
		fn weigher_uses_fixed_wieght_bounds() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::Weigher>(),
				TypeId::of::<
					WeightInfoBounds<
						weights::xcm::PopXcmWeight<RuntimeCall>,
						RuntimeCall,
						MaxInstructions,
					>,
				>(),
			);
		}

		#[test]
		fn uses_xcm_as_recorder_for_dry_runs() {
			assert_eq!(
				TypeId::of::<<XcmConfig as xcm_executor::Config>::XcmRecorder>(),
				TypeId::of::<PolkadotXcm>(),
			);
		}

		#[test]
		fn uses_ump_for_relay_and_xcmp_for_paras() {
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
	}

	mod pallet_xcm_configuration {
		use super::*;

		#[test]
		fn admin_origin_ensures_root() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::AdminOrigin>(),
				TypeId::of::<EnsureRoot<AccountId>>(),
			);
		}

		#[test]
		fn advertises_current_xcm_version() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::AdvertisedXcmVersion>(),
				TypeId::of::<pallet_xcm::CurrentXcmVersion>(),
			);
		}

		#[test]
		fn uses_balances() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::Currency>(),
				TypeId::of::<Balances>(),
			);
		}

		#[test]
		fn asset_matching_is_disabled() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::CurrencyMatcher>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn message_execute_xcm_origin_uses_signed_to_accountid32() {
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
		fn limits_max_lockers() {
			assert_eq!(<<Runtime as pallet_xcm::Config>::MaxLockers as Get<u32>>::get(), 8);
		}

		#[test]
		fn max_remote_lock_consumers_is_0() {
			assert_eq!(
				<<Runtime as pallet_xcm::Config>::MaxRemoteLockConsumers as Get<u32>>::get(),
				0
			);
		}

		#[test]
		fn remote_consider_identifier_is_disabled() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::RemoteLockConsumerIdentifier>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn send_xcm_origin_is_configured() {
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
		fn sovereign_account_uses_location_converter() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::SovereignAccountOf>(),
				TypeId::of::<LocationToAccountId>(),
			);
		}

		#[test]
		fn does_not_have_trusted_lockers() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::TrustedLockers>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn universal_location_is_configured() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::UniversalLocation>(),
				TypeId::of::<UniversalLocation>(),
			);
		}

		#[test]
		fn weigher_uses_fixed_weight_bounds() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::Weigher>(),
				TypeId::of::<
					WeightInfoBounds<
						weights::xcm::PopXcmWeight<RuntimeCall>,
						RuntimeCall,
						MaxInstructions,
					>,
				>(),
			);
		}

		#[test]
		fn does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn execute_filters_nothing() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::XcmExecuteFilter>(),
				TypeId::of::<Everything>(),
			);
		}

		#[test]
		fn executor_is_configured() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::XcmExecutor>(),
				TypeId::of::<XcmExecutor<XcmConfig>>(),
			);
		}

		#[test]
		fn reserve_transfer_only_allows_relay_asset() {
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::XcmReserveTransferFilter>(),
				TypeId::of::<FilterByAssets<Equals<RelayLocation>>>(),
			);

			// Filter assets that are not the relay asset.
			let assets = [
				vec![Asset { id: AssetId(Location::new(1, Parachain(1000))), fun: Fungible(0) }],
				vec![
					Asset { id: AssetId(Location::parent()), fun: Fungible(0) },
					// Assets other than the relay asset:
					Asset { id: AssetId(AssetHub::get()), fun: Fungible(0) },
					Asset {
						id: AssetId(Location::new(
							1,
							[Parachain(1000), PalletInstance(50), GeneralIndex(1984)],
						)),
						fun: Fungible(0),
					},
				],
			];
			for asset in assets {
				// AssetHub is used for the location as it should not be possible to send the
				// derivative of the relay asset to another chain because it functions as the
				// reserve.
				assert!(!<<Runtime as pallet_xcm::Config>::XcmReserveTransferFilter as Contains<
					(Location, Vec<Asset>),
				>>::contains(&(AssetHub::get(), asset)));
			}

			// Allow only relay asset.
			assert!(<<Runtime as pallet_xcm::Config>::XcmReserveTransferFilter as Contains<(
				Location,
				Vec<Asset>
			)>>::contains(&(
				AssetHub::get(),
				vec![Asset { id: AssetId(Location::parent()), fun: Fungible(0) }]
			)));
		}

		#[test]
		fn router_uses_ump_for_relay_and_xcmp_for_para() {
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
		fn teleport_filters_everything() {
			// Nothing can be teleported.
			assert_eq!(
				TypeId::of::<<Runtime as pallet_xcm::Config>::XcmTeleportFilter>(),
				TypeId::of::<Nothing>(),
			);
		}

		#[test]
		fn declares_version_discovery_queue_size() {
			assert_eq!(<Runtime as pallet_xcm::Config>::VERSION_DISCOVERY_QUEUE_SIZE, 100);
		}
	}

	mod pallet_xcmp_queue {
		use super::*;

		#[test]
		fn channel_info_via_parachain_system() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::ChannelInfo>(),
				TypeId::of::<ParachainSystem>()
			);
		}

		#[test]
		fn controller_origin_ensures_root() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::ControllerOrigin>(),
				TypeId::of::<EnsureRoot<AccountId>>()
			);
		}

		#[test]
		fn controller_origin_converter_configuration() {
			assert_eq!(
				TypeId::of::<
					<Runtime as cumulus_pallet_xcmp_queue::Config>::ControllerOriginConverter,
				>(),
				TypeId::of::<XcmOriginToTransactDispatchOrigin>()
			);
		}

		#[test]
		fn limits_outbound_channels() {
			assert_eq!(
				<<Runtime as cumulus_pallet_xcmp_queue::Config>::MaxActiveOutboundChannels as Get<
					u32,
				>>::get(),
				128
			);
		}

		#[test]
		fn limits_inbound_suspended_channels() {
			assert_eq!(
				<<Runtime as cumulus_pallet_xcmp_queue::Config>::MaxInboundSuspended as Get<u32>>::get(),
				128
			);
		}

		#[test]
		fn limits_hrmp_message_page_size() {
			assert_eq!(
				<<Runtime as cumulus_pallet_xcmp_queue::Config>::MaxPageSize as Get<u32>>::get(),
				103 * 1024
			);
		}

		#[test]
		fn price_for_sibling_delivery() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::PriceForSiblingDelivery>(
				),
				TypeId::of::<
					ExponentialPrice<RelayLocation, BaseDeliveryFee, TransactionByteFee, XcmpQueue>,
				>()
			);

			new_test_ext().execute_with(|| {
				type ExponentialDeliveryPrice =
					ExponentialPrice<RelayLocation, BaseDeliveryFee, TransactionByteFee, XcmpQueue>;
				let id: ParaId = 420.into();
				let b: u128 = BaseDeliveryFee::get();
				let m: u128 = TransactionByteFee::get();

				// F * (B + msg_length * M)
				// A: RelayLocation
				// B: BaseDeliveryFee
				// M: TransactionByteFee
				// F: XcmpQueue
				//
				// message_length = 1
				let result: u128 = XcmpQueue::get_fee_factor(id).saturating_mul_int(b + m);
				assert_eq!(
					ExponentialDeliveryPrice::price_for_delivery(id, &Xcm(vec![])),
					(RelayLocation::get(), result).into()
				);
			})
		}

		#[test]
		fn versions_xcm() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::VersionWrapper>(),
				TypeId::of::<PolkadotXcm>(),
			);
		}

		#[test]
		fn does_not_use_default_weights() {
			assert_ne!(
				TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::WeightInfo>(),
				TypeId::of::<()>(),
			);
		}

		#[test]
		fn uses_message_queue() {
			assert_eq!(
				TypeId::of::<<Runtime as cumulus_pallet_xcmp_queue::Config>::XcmpQueue>(),
				TypeId::of::<
					TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>,
				>(),
			);
		}
	}
}

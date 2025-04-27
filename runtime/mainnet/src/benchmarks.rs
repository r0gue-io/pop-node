use alloc::{boxed::Box, vec};

use cumulus_primitives_core::ParaId;
use frame_benchmarking::BenchmarkError;
use frame_support::parameter_types;
pub use pallet_xcm::benchmarking::Pallet as PalletXcmBenchmark;
use xcm::prelude::{
	Asset, AssetId, Fungible, Here, InteriorLocation, Junction, Location, NetworkId, Response,
};
use xcm_executor::traits::ConvertLocation;

use crate::{
	config::{
		monetary::ExistentialDeposit,
		xcm::{
			AssetHub, LocationToAccountId, PriceForParentDelivery, PriceForSiblingDelivery,
			RelayLocation, XcmConfig,
		},
	},
	Runtime, *,
};

/// Pallet that benchmarks XCM's `AssetTransactor` trait via `Fungible`.
pub type XcmFungible = pallet_xcm_benchmarks::fungible::Pallet<Runtime>;
/// Pallet that serves no other purpose than benchmarking raw XCMs.
pub type XcmGeneric = pallet_xcm_benchmarks::generic::Pallet<Runtime>;

frame_benchmarking::define_benchmarks!(
	// Ordered as per runtime
	// System
	[frame_system, SystemBench::<Runtime>]
	[frame_system_extensions, SystemExtensionsBench::<Runtime>]
	[cumulus_pallet_parachain_system, ParachainSystem]
	[pallet_timestamp, Timestamp]
	[cumulus_pallet_weight_reclaim, WeightReclaim]
	// Monetary
	[pallet_balances, Balances]
	[pallet_transaction_payment, TransactionPayment]
	[pallet_treasury, Treasury]
	// Governance
	[pallet_sudo, Sudo]
	[pallet_collective, Council]
	[pallet_motion, Motion]
	// Collation support
	[pallet_collator_selection, CollatorSelection]
	[pallet_session, SessionBench::<Runtime>]
	// Scheduler
	[pallet_scheduler, Scheduler]
	// Preimage
	[pallet_preimage, Preimage]
	// XCM
	[cumulus_pallet_xcmp_queue, XcmpQueue]
	[pallet_xcm, PalletXcmBenchmark::<Runtime>]
	[pallet_xcm_benchmarks::fungible, XcmFungible]
	[pallet_xcm_benchmarks::generic, XcmGeneric]
	[pallet_message_queue, MessageQueue]
	// Contracts
	[pallet_revive, Revive]
	// Proxy
	[pallet_proxy, Proxy]
	// Multisig
	[pallet_multisig, Multisig]
	// Utility
	[pallet_utility, Utility]
	// Assets
	[pallet_nfts, Nfts]
	[pallet_assets, Assets]
);

parameter_types! {
	/// Delivery helpers will deposit this amount to the local origin used in the benchmarks.
	pub ExistentialDepositAsset: Option<Asset> = Some((
		RelayLocation::get(),
		ExistentialDeposit::get()
	).into());
	pub const AssetHubParaId: ParaId = ParaId::new(1000);
}

type DeliveryHelper = (
	cumulus_primitives_utility::ToParentDeliveryHelper<
		XcmConfig,
		ExistentialDepositAsset,
		PriceForParentDelivery,
	>,
	polkadot_runtime_common::xcm_sender::ToParachainDeliveryHelper<
		XcmConfig,
		ExistentialDepositAsset,
		PriceForSiblingDelivery,
		AssetHubParaId,
		ParachainSystem,
	>,
);

impl pallet_xcm::benchmarking::Config for Runtime {
	type DeliveryHelper = DeliveryHelper;

	fn reachable_dest() -> Option<Location> {
		Some(RelayLocation::get())
	}

	fn teleportable_asset_and_dest() -> Option<(Asset, Location)> {
		// No assets can be teleported.
		None
	}

	fn reserve_transferable_asset_and_dest() -> Option<(Asset, Location)> {
		ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(AssetHubParaId::get());

		let who = frame_benchmarking::whitelisted_caller();
		// Give some multiple of the existential deposit.
		let balance = ExistentialDeposit::get() * 10_000;
		let _ =
			<Balances as frame_support::traits::Currency<_>>::make_free_balance_be(&who, balance);
		let ah_on_pop: AccountId = LocationToAccountId::convert_location(&AssetHub::get())?;
		let _ = <Balances as frame_support::traits::Currency<_>>::make_free_balance_be(
			&ah_on_pop, balance,
		);

		// We can do reserve transfers of relay native asset to AH.
		Some((
			Asset { fun: Fungible(ExistentialDeposit::get()), id: AssetId(RelayLocation::get()) },
			AssetHub::get(),
		))
	}

	fn set_up_complex_asset_transfer(
	) -> Option<(xcm::prelude::Assets, u32, Location, Box<dyn FnOnce()>)> {
		ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(AssetHubParaId::get());
		// Pop can only reserve transfer DOT.
		// This test needs to be adapted as the features grow.
		let dest = AssetHub::get();

		let fee_amount = ExistentialDeposit::get();
		let fee_asset: Asset = (RelayLocation::get(), fee_amount).into();

		let who = frame_benchmarking::whitelisted_caller();
		// Give some multiple of the existential deposit.
		let balance = fee_amount + ExistentialDeposit::get() * 10_000;
		let _ =
			<Balances as frame_support::traits::Currency<_>>::make_free_balance_be(&who, balance);
		// Verify initial balance.
		assert_eq!(Balances::free_balance(&who), balance);

		let assets: xcm::prelude::Assets = vec![fee_asset.clone()].into();

		// Verify transferred successfully.
		let verify = Box::new(move || {
			// Verify native balance after transfer, decreased by transferred fee amount
			// (plus transport fees).
			assert!(Balances::free_balance(&who) <= balance - fee_amount);
		});
		Some((assets, 0, dest, verify))
	}

	fn get_asset() -> Asset {
		Asset { id: AssetId(RelayLocation::get()), fun: Fungible(ExistentialDeposit::get()) }
	}
}

impl pallet_xcm_benchmarks::Config for Runtime {
	type AccountIdConverter = LocationToAccountId;
	type DeliveryHelper = DeliveryHelper;
	type XcmConfig = XcmConfig;

	fn valid_destination() -> Result<Location, BenchmarkError> {
		Ok(RelayLocation::get())
	}

	fn worst_case_holding(_depositable_count: u32) -> xcm::prelude::Assets {
		// Pop only allows relay's native asset to be used cross chain for now.
		vec![Asset { id: AssetId(RelayLocation::get()), fun: Fungible(u128::MAX) }].into()
	}
}

impl pallet_xcm_benchmarks::generic::Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type TransactAsset = Balances;

	fn worst_case_response() -> (u64, Response) {
		let notify = frame_system::Call::remark { remark: vec![] };
		PolkadotXcm::new_notify_query(Location::here(), notify, 10, Location::here());
		(0u64, Response::ExecutionResult(None))
	}

	fn worst_case_asset_exchange(
	) -> Result<(xcm::prelude::Assets, xcm::prelude::Assets), BenchmarkError> {
		// Pop doesn't support asset exchange for now.
		Err(BenchmarkError::Skip)
	}

	fn universal_alias() -> Result<(Location, Junction), BenchmarkError> {
		// Pop's `UniversalAliases` is configured to `Nothing`.
		Err(BenchmarkError::Skip)
	}

	fn transact_origin_and_runtime_call() -> Result<(Location, RuntimeCall), BenchmarkError> {
		Ok((RelayLocation::get(), frame_system::Call::remark_with_event { remark: vec![] }.into()))
	}

	fn subscribe_origin() -> Result<Location, BenchmarkError> {
		Ok(RelayLocation::get())
	}

	fn claimable_asset() -> Result<(Location, Location, xcm::prelude::Assets), BenchmarkError> {
		let origin = AssetHub::get();
		let assets: xcm::prelude::Assets = (AssetId(RelayLocation::get()), 1_000 * UNIT).into();
		let ticket = Location { parents: 0, interior: Here };
		Ok((origin, ticket, assets))
	}

	fn fee_asset() -> Result<Asset, BenchmarkError> {
		Ok(Asset { id: AssetId(RelayLocation::get()), fun: Fungible(1_000_000 * UNIT) })
	}

	fn unlockable_asset() -> Result<(Location, Location, Asset), BenchmarkError> {
		// Pop doesn't configure `AssetLocker` yet.
		Err(BenchmarkError::Skip)
	}

	fn export_message_origin_and_destination(
	) -> Result<(Location, NetworkId, InteriorLocation), BenchmarkError> {
		// Pop doesn't configure `MessageExporter` yet.
		Err(BenchmarkError::Skip)
	}

	fn alias_origin() -> Result<(Location, Location), BenchmarkError> {
		// Pop's `Aliasers` is configured to `Nothing`.
		Err(BenchmarkError::Skip)
	}
}

parameter_types! {
	pub TrustedReserve: Option<(Location, Asset)> = Some((AssetHub::get(), Asset::from((RelayLocation::get(), UNIT))));
	// We don't set any trusted teleporters in our XCM config, but we need this for the benchmarks.
	pub TrustedTeleporter: Option<(Location, Asset)> = Some((
		AssetHub::get(),
		Asset::from((RelayLocation::get(), UNIT)),
	));
}
impl pallet_xcm_benchmarks::fungible::Config for Runtime {
	type CheckedAccount = ();
	type TransactAsset = Balances;
	type TrustedReserve = TrustedReserve;
	type TrustedTeleporter = TrustedTeleporter;

	fn get_asset() -> Asset {
		Asset { id: AssetId(RelayLocation::get()), fun: Fungible(10 * UNIT) }
	}
}

use alloc::{boxed::Box, vec};

use cumulus_primitives_core::ParaId;
use frame_support::parameter_types;
pub use pallet_xcm::benchmarking::Pallet as PalletXcmBenchmark;
use xcm::prelude::{
	Asset, AssetId, Fungible, GeneralIndex, Location, PalletInstance, Parachain, Parent, ParentThen,
};

use super::*;
use crate::{
	config::{
		assets::TrustBackedAssetsInstance,
		monetary::ExistentialDeposit,
		xcm::{AssetHub, PriceForSiblingDelivery, RelayLocation, XcmConfig},
	},
	Runtime,
};

frame_benchmarking::define_benchmarks!(
	// Ordered as per runtime
	// System
	[frame_system, SystemBench::<Runtime>]
	[frame_system_extensions, SystemExtensionsBench::<Runtime>]
	[cumulus_pallet_parachain_system, ParachainSystem]
	[pallet_timestamp, Timestamp]
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
	pub ExistentialDepositAsset: Option<Asset> = Some((
		RelayLocation::get(),
		ExistentialDeposit::get()
	).into());
	pub const RandomParaId: ParaId = ParaId::new(43211234);
}

impl pallet_xcm::benchmarking::Config for Runtime {
	type DeliveryHelper = (
		polkadot_runtime_common::xcm_sender::ToParachainDeliveryHelper<
			XcmConfig,
			ExistentialDepositAsset,
			PriceForSiblingDelivery,
			RandomParaId,
			ParachainSystem,
		>,
	);

	fn reachable_dest() -> Option<Location> {
		Some(Location::parent())
	}

	fn teleportable_asset_and_dest() -> Option<(Asset, Location)> {
		// No assets can be teleported.
		None
	}

	fn reserve_transferable_asset_and_dest() -> Option<(Asset, Location)> {
		ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(RandomParaId::get());

		// We can do reserve transfers of relay native asset to RandomPara.
		Some((
			Asset { fun: Fungible(ExistentialDeposit::get()), id: AssetId(Location::from(Parent)) },
			ParentThen(Parachain(RandomParaId::get().into()).into()).into(),
		))
	}

	fn set_up_complex_asset_transfer(
	) -> Option<(xcm::prelude::Assets, u32, Location, Box<dyn FnOnce()>)> {
		ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(RandomParaId::get());
		// Transfer to RandomPara some local asset (local-reserve-transfer) while paying
		// fees using teleported native token.
		// (We don't care that dest doesn't accept incoming unknown local asset)
		let dest = ParentThen(Parachain(RandomParaId::get().into()).into()).into();

		let fee_amount = ExistentialDeposit::get();
		let fee_asset: Asset = (Location::parent(), fee_amount).into();

		let who = frame_benchmarking::whitelisted_caller();
		// Give some multiple of the existential deposit
		let balance = fee_amount + ExistentialDeposit::get() * 1000;
		let _ =
			<Balances as frame_support::traits::Currency<_>>::make_free_balance_be(&who, balance);
		// verify initial balance
		assert_eq!(Balances::free_balance(&who), balance);

		// set up local asset
		let asset_amount = 10u128;
		let initial_asset_amount = asset_amount * 10;
		let (asset_id, ..) = pallet_assets::benchmarking::create_default_minted_asset::<
			Runtime,
			pallet_assets::Instance1,
		>(true, initial_asset_amount);
		let asset_location =
			Location::new(0, [PalletInstance(52), GeneralIndex(u32::from(asset_id).into())]);
		let transfer_asset: Asset = (asset_location, asset_amount).into();

		let assets: xcm::prelude::Assets = vec![fee_asset.clone(), transfer_asset].into();
		let fee_index = if assets.get(0).unwrap().eq(&fee_asset) { 0 } else { 1 };

		// verify transferred successfully
		let verify = Box::new(move || {
			// verify native balance after transfer, decreased by transferred fee amount
			// (plus transport fees)
			assert!(Balances::free_balance(&who) <= balance - fee_amount);
			// verify asset balance decreased by exactly transferred amount
			assert_eq!(Assets::balance(asset_id.into(), &who), initial_asset_amount - asset_amount,);
		});
		Some((assets, fee_index as u32, dest, verify))
	}

	fn get_asset() -> Asset {
		Asset { id: AssetId(Location::parent()), fun: Fungible(ExistentialDeposit::get()) }
	}
}

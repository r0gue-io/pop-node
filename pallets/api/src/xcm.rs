pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::{ensure_signed, pallet_prelude::*, WeightInfo};
	use sp_runtime::AccountId32 as AccountId;
	use sp_std::{boxed::Box, vec};
	use xcm::prelude::*;

	type XcmOf<T> = pallet_xcm::Pallet<T>;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_xcm::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Weight information for dispatchables in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn ah_transfer(
			origin: OriginFor<T>,
			to: AccountId,
			value: u128,
			fee: u128,
		) -> DispatchResult {
			let _ = ensure_signed(origin.clone())?;

			let asset_hub_para_id: u32 = 1000;
			let destination = Location::new(1, Parachain(asset_hub_para_id));
			let beneficiary = Location::new(0, AccountId32 { network: None, id: to.into() });

			// Define the assets
			let asset: Asset = (Location::parent(), value).into();
			let fee_asset: Asset = (Location::parent(), fee).into();

			// XCM instructions to be executed on AssetHub
			let xcm_on_destination = Xcm(vec![
				BuyExecution { fees: fee_asset.clone(), weight_limit: WeightLimit::Unlimited },
				DepositAsset { assets: Wild(All.into()), beneficiary: beneficiary.clone() },
			]);

			// Construct the full XCM message
			let message: Xcm<<T as pallet_xcm::Config>::RuntimeCall> = Xcm(vec![
				// Withdraw the total amount (value + fee) from the contract's account
				WithdrawAsset((vec![asset.clone(), fee_asset.clone()]).into()),
				// Initiate the reserve-based transfer
				InitiateReserveWithdraw {
					assets: vec![asset.clone()].into(),
					reserve: destination.clone(),
					xcm: xcm_on_destination,
				},
			]);

			let _hash =
				XcmOf::<T>::execute(origin, Box::new(VersionedXcm::V4(message)), Weight::MAX);
			Ok(())
		}
	}
}

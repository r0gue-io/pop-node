#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::WithPostDispatchInfo, pallet_prelude::*, traits::fungibles::Inspect,
	};
	use frame_system::pallet_prelude::*;
	use pallet_assets::WeightInfo;
	use sp_runtime::traits::StaticLookup;

	type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
	type AssetIdOf<T> = <pallet_assets::Pallet<T, TrustBackedAssetsInstance> as Inspect<
		<T as frame_system::Config>::AccountId,
	>>::AssetId;
	type AssetIdParameterOf<T> =
		<T as pallet_assets::Config<TrustBackedAssetsInstance>>::AssetIdParameter;
	type Assets<T> = pallet_assets::Pallet<T, TrustBackedAssetsInstance>;
	type AssetsWeightInfo<T> = <T as pallet_assets::Config<TrustBackedAssetsInstance>>::WeightInfo;
	type BalanceOf<T> = <pallet_assets::Pallet<T, TrustBackedAssetsInstance> as Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;
	// Should be defined in primitives.
	type TrustBackedAssetsInstance = pallet_assets::Instance1;

	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_assets::Config<TrustBackedAssetsInstance>
	{
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(AssetsWeightInfo::<T>::transfer_keep_alive())]
		pub fn transfer(
			origin: OriginFor<T>,
			id: AssetIdParameterOf<T>,
			target: AccountIdLookupOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			Assets::<T>::transfer_keep_alive(origin, id, target, amount)
		}

		#[pallet::call_index(1)]
		#[pallet::weight(AssetsWeightInfo::<T>::cancel_approval())]
		pub fn approve(
			origin: OriginFor<T>,
			id: AssetIdParameterOf<T>,
			spender: AccountIdLookupOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			Assets::<T>::cancel_approval(origin.clone(), id.clone(), spender.clone())
				.map_err(|e| e.with_weight(AssetsWeightInfo::<T>::cancel_approval()))?;
			Assets::<T>::approve_transfer(origin, id, spender, value).map_err(|e| {
				e.with_weight(
					AssetsWeightInfo::<T>::cancel_approval()
						+ AssetsWeightInfo::<T>::approve_transfer(),
				)
			})?;
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn total_supply(id: AssetIdOf<T>) -> BalanceOf<T> {
			Assets::<T>::total_supply(id)
		}
	}
}

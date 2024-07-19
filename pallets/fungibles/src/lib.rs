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
	use frame_support::{pallet_prelude::*, traits::fungibles::Inspect};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::StaticLookup;

	type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
	type AssetIdOf<T> = <pallet_assets::Pallet<T, TrustBackedAssetsInstance> as Inspect<
		<T as frame_system::Config>::AccountId,
	>>::AssetId;
	type AssetIdParameterOf<T> =
		<T as pallet_assets::Config<TrustBackedAssetsInstance>>::AssetIdParameter;
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

	use pallet_assets::WeightInfo;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet_assets::Config<TrustBackedAssetsInstance>>::WeightInfo::transfer_keep_alive())]
		pub fn transfer(
			origin: OriginFor<T>,
			id: AssetIdParameterOf<T>,
			target: AccountIdLookupOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			pallet_assets::Pallet::<T, TrustBackedAssetsInstance>::transfer_keep_alive(
				origin, id, target, amount,
			)
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn total_supply(id: AssetIdOf<T>) -> BalanceOf<T> {
			pallet_assets::Pallet::<T, TrustBackedAssetsInstance>::total_supply(id)
		}
	}
}

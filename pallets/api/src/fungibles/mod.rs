/// The fungibles pallet serves as a wrapper around the pallet_assets, offering a streamlined
/// interface for interacting with fungible assets. The goal is to provide a simplified, consistent
/// API that adheres to standards in the smart contract space.
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod tests;

use frame_support::traits::fungibles::{metadata::Inspect as MetadataInspect, Inspect};
pub use pallet::*;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AssetIdOf<T> = <pallet_assets::Pallet<T, AssetsInstanceOf<T>> as Inspect<
	<T as frame_system::Config>::AccountId,
>>::AssetId;
type AssetIdParameterOf<T> = <T as pallet_assets::Config<AssetsInstanceOf<T>>>::AssetIdParameter;
type AssetsOf<T> = pallet_assets::Pallet<T, AssetsInstanceOf<T>>;
type AssetsInstanceOf<T> = <T as Config>::AssetsInstance;
type AssetsWeightInfoOf<T> = <T as pallet_assets::Config<AssetsInstanceOf<T>>>::WeightInfo;
type BalanceOf<T> = <pallet_assets::Pallet<T, AssetsInstanceOf<T>> as Inspect<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo, WithPostDispatchInfo},
		pallet_prelude::*,
		traits::fungibles::approvals::Inspect as ApprovalInspect,
	};
	use frame_system::pallet_prelude::*;
	use pallet_assets::WeightInfo;
	use sp_runtime::{traits::StaticLookup, Saturating};
	use sp_std::vec::Vec;

	/// State reads for the fungibles api with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config> {
		/// Total token supply for a given asset ID.
		#[codec(index = 0)]
		TotalSupply(AssetIdOf<T>),
		/// Account balance for a given asset ID.
		#[codec(index = 1)]
		BalanceOf(AssetIdOf<T>, AccountIdOf<T>),
		/// Allowance for a spender approved by an owner, for a given asset ID.
		#[codec(index = 2)]
		Allowance(AssetIdOf<T>, AccountIdOf<T>, AccountIdOf<T>),
		/// Token name for a given asset ID.
		#[codec(index = 8)]
		TokenName(AssetIdOf<T>),
		/// Token symbol for a given asset ID.
		#[codec(index = 9)]
		TokenSymbol(AssetIdOf<T>),
		/// Token decimals for a given asset ID.
		#[codec(index = 10)]
		TokenDecimals(AssetIdOf<T>),
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config<Self::AssetsInstance> {
		/// The instance of pallet assets it is tightly coupled to.
		type AssetsInstance;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfers `value` amount of tokens from the caller's account to account `to`, with additional
		/// `data` in unspecified format.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		/// * `to` - The recipient account.
		/// * `value` - The number of tokens to transfer.
		#[pallet::call_index(3)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::transfer_keep_alive())]
		pub fn transfer(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			target: AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let target = T::Lookup::unlookup(target);
			AssetsOf::<T>::transfer_keep_alive(origin, id.into(), target, amount)
		}

		/// Approves an account to spend a specified number of tokens on behalf of the caller.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		/// * `spender` - The account that is allowed to spend the tokens.
		/// * `value` - The number of tokens to approve.
		#[pallet::call_index(5)]
		#[pallet::weight(T::DbWeight::get().reads(1) + AssetsWeightInfoOf::<T>::cancel_approval() + AssetsWeightInfoOf::<T>::approve_transfer())]
		pub fn approve(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())
				// To have the caller pay some fees.
				.map_err(|e| e.with_weight(T::DbWeight::get().reads(1)))?;
			let current_allowance = AssetsOf::<T>::allowance(id.clone(), &who, &spender);
			let spender = T::Lookup::unlookup(spender);
			let id: AssetIdParameterOf<T> = id.into();
			// If the new value is equal to the current allowance, do nothing.
			if value == current_allowance {
				return Ok(().into());
			}
			// If the new value is greater than the current allowance, approve the difference
			// because `approve_transfer` works additively (see pallet-assets).
			if value > current_allowance {
				AssetsOf::<T>::approve_transfer(
					origin,
					id,
					spender,
					value.saturating_sub(current_allowance),
				)
				.map_err(|e| {
					e.with_weight(
						T::DbWeight::get().reads(1) + AssetsWeightInfoOf::<T>::approve_transfer(),
					)
				})?;
			} else {
				// If the new value is less than the current allowance, cancel the approval and set the new value
				AssetsOf::<T>::cancel_approval(origin.clone(), id.clone(), spender.clone())
					.map_err(|e| {
						e.with_weight(
							T::DbWeight::get().reads(1)
								+ AssetsWeightInfoOf::<T>::cancel_approval(),
						)
					})?;
				AssetsOf::<T>::approve_transfer(origin, id, spender, value)?;
			}
			Ok(().into())
		}

		/// Increases the allowance of a spender.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		/// * `spender` - The account that is allowed to spend the tokens.
		/// * `value` - The number of tokens to increase the allowance by.
		#[pallet::call_index(6)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::approve_transfer())]
		pub fn increase_allowance(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let spender = T::Lookup::unlookup(spender);
			AssetsOf::<T>::approve_transfer(origin, id.into(), spender, value)
		}
	}

	impl<T: Config> Pallet<T> {
		/// Returns the total token supply for a given asset ID.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		pub fn total_supply(id: AssetIdOf<T>) -> BalanceOf<T> {
			AssetsOf::<T>::total_supply(id)
		}

		/// Returns the account balance for the specified `owner` for a given asset ID. Returns `0` if
		/// the account is non-existent.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		/// * `owner` - The account whose balance is being queried.
		pub fn balance_of(id: AssetIdOf<T>, owner: &AccountIdOf<T>) -> BalanceOf<T> {
			AssetsOf::<T>::balance(id, owner)
		}

		/// Returns the amount which `spender` is still allowed to withdraw from `owner` for a given
		/// asset ID. Returns `0` if no allowance has been set.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		/// * `owner` - The account that owns the tokens.
		/// * `spender` - The account that is allowed to spend the tokens.
		pub fn allowance(
			id: AssetIdOf<T>,
			owner: &AccountIdOf<T>,
			spender: &AccountIdOf<T>,
		) -> BalanceOf<T> {
			AssetsOf::<T>::allowance(id, owner, spender)
		}

		/// Returns the token name for a given asset ID.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		pub fn token_name(id: AssetIdOf<T>) -> Vec<u8> {
			<AssetsOf<T> as MetadataInspect<AccountIdOf<T>>>::name(id)
		}

		/// Returns the token symbol for a given asset ID.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		pub fn token_symbol(id: AssetIdOf<T>) -> Vec<u8> {
			<AssetsOf<T> as MetadataInspect<AccountIdOf<T>>>::symbol(id)
		}

		/// Returns the token decimals for a given asset ID.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		pub fn token_decimals(id: AssetIdOf<T>) -> u8 {
			<AssetsOf<T> as MetadataInspect<AccountIdOf<T>>>::decimals(id)
		}
	}
}

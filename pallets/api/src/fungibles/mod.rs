/// The fungibles pallet serves as a wrapper around the pallet_assets, offering a streamlined
/// interface for interacting with fungible assets. The goal is to provide a simplified,
/// consistent API that adheres to standards in the smart contract space.
pub use pallet::*;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo, WithPostDispatchInfo},
		pallet_prelude::*,
		traits::fungibles::{
			approvals::Inspect as ApprovalInspect, metadata::Inspect as MetadataInspect, Inspect,
		},
	};
	use frame_system::pallet_prelude::*;
	use pallet_assets::WeightInfo;
	use sp_runtime::{
		traits::{StaticLookup, Zero},
		Saturating,
	};
	use sp_std::vec::Vec;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type AssetIdOf<T> = <pallet_assets::Pallet<T, AssetsInstanceOf<T>> as Inspect<
		<T as frame_system::Config>::AccountId,
	>>::AssetId;
	type AssetIdParameterOf<T> =
		<T as pallet_assets::Config<AssetsInstanceOf<T>>>::AssetIdParameter;
	type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
	type Assets<T> = pallet_assets::Pallet<T, AssetsInstanceOf<T>>;
	type AssetsInstanceOf<T> = <T as Config>::AssetsInstance;
	type AssetsWeightInfo<T> = <T as pallet_assets::Config<AssetsInstanceOf<T>>>::WeightInfo;
	pub(crate) type BalanceOf<T> = <pallet_assets::Pallet<T, AssetsInstanceOf<T>> as Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	/// The required input for state queries through the fungibles api.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum FungiblesKey<T: Config> {
		#[codec(index = 0)]
		TotalSupply(AssetIdOf<T>),
		#[codec(index = 1)]
		BalanceOf(AssetIdOf<T>, AccountIdOf<T>),
		#[codec(index = 2)]
		Allowance(AssetIdOf<T>, AccountIdOf<T>, AccountIdOf<T>),
		#[codec(index = 3)]
		TokenName(AssetIdOf<T>),
		#[codec(index = 4)]
		TokenSymbol(AssetIdOf<T>),
		#[codec(index = 5)]
		TokenDecimals(AssetIdOf<T>),
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config<Self::AssetsInstance> {
		type AssetsInstance;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfers `value` amount of tokens from the caller's account to account `to`, with
		/// additional `data` in unspecified format.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		/// * `to` - The recipient account.
		/// * `value` - The number of tokens to transfer.
		///
		/// # Returns
		/// Returns `Ok(())` if successful, or an error if the transfer fails.
		#[pallet::call_index(0)]
		#[pallet::weight(AssetsWeightInfo::<T>::transfer_keep_alive())]
		pub fn transfer(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			target: AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let target = T::Lookup::unlookup(target);
			Assets::<T>::transfer_keep_alive(origin, id.into(), target, amount)
		}

		/// Transfers `value` amount of tokens from the delegated account approved by the `owner` to
		/// account `to`, with additional `data` in unspecified format.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		/// * `owner` - The account which previously approved for a transfer of at least `amount`
		///   and
		/// from which the asset balance will be withdrawn.
		/// * `to` - The recipient account.
		/// * `value` - The number of tokens to transfer.
		///
		/// # Returns
		/// Returns `Ok(())` if successful, or an error if the transfer fails.
		#[pallet::call_index(1)]
		#[pallet::weight(AssetsWeightInfo::<T>::transfer_approved())]
		pub fn transfer_from(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			owner: AccountIdOf<T>,
			target: AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let owner = T::Lookup::unlookup(owner);
			let target = T::Lookup::unlookup(target);
			Assets::<T>::transfer_approved(origin, id.into(), owner, target, amount)
		}

		/// Approves an account to spend a specified number of tokens on behalf of the caller.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		/// * `spender` - The account that is allowed to spend the tokens.
		/// * `value` - The number of tokens to approve.
		///
		/// # Returns
		/// Returns `Ok(())` if successful, or an error if the approval fails.
		#[pallet::call_index(2)]
		#[pallet::weight(T::DbWeight::get().reads(2) + AssetsWeightInfo::<T>::cancel_approval() + AssetsWeightInfo::<T>::approve_transfer())]
		pub fn approve(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			mut value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(T::DbWeight::get().reads(1)))?;
			let current_allowance = Assets::<T>::allowance(id.clone(), &who, &spender);
			let spender = T::Lookup::unlookup(spender);
			let id: AssetIdParameterOf<T> = id.into();
			// If the new value is equal to the current allowance, do nothing.
			if value == current_allowance {
				return Ok(().into());
			}
			// If the new value is greater than the current allowance, approve the difference.
			if value > current_allowance {
				value.saturating_reduce(current_allowance);
				Assets::<T>::approve_transfer(origin, id, spender, value).map_err(|e| {
					e.with_weight(
						T::DbWeight::get().reads(2) + AssetsWeightInfo::<T>::approve_transfer(),
					)
				})?;
			} else {
				// If the new value is less than the current allowance, cancel the approval and set
				// the new value
				Self::do_set_allowance(origin, id, spender, value)?;
			}

			Ok(().into())
		}

		/// Increases the allowance of a spender.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		/// * `spender` - The account that is allowed to spend the tokens.
		/// * `value` - The number of tokens to increase the allowance by.
		///
		/// # Returns
		/// Returns `Ok(())` if successful, or an error if the operation fails.
		#[pallet::call_index(3)]
		#[pallet::weight(AssetsWeightInfo::<T>::approve_transfer())]
		pub fn increase_allowance(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let spender = T::Lookup::unlookup(spender);
			Assets::<T>::approve_transfer(origin, id.into(), spender, value)
		}

		/// Decreases the allowance of a spender.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		/// * `spender` - The account that is allowed to spend the tokens.
		/// * `value` - The number of tokens to decrease the allowance by.
		///
		/// # Returns
		/// Returns `Ok(())` if successful, or an error if the operation fails.
		#[pallet::call_index(4)]
		#[pallet::weight(T::DbWeight::get().reads(2) + AssetsWeightInfo::<T>::cancel_approval() + AssetsWeightInfo::<T>::approve_transfer())]
		pub fn decrease_allowance(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(T::DbWeight::get().reads(1)))?;
			let mut current_allowance = Assets::<T>::allowance(id.clone(), &who, &spender);
			let spender = T::Lookup::unlookup(spender);
			let id: AssetIdParameterOf<T> = id.into();

			current_allowance.saturating_reduce(value);
			Self::do_set_allowance(origin, id, spender, current_allowance)?;
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Returns the total token supply for a given asset ID.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		///
		/// # Returns
		/// The total supply of the token, or an error if the operation fails.
		pub fn total_supply(id: AssetIdOf<T>) -> BalanceOf<T> {
			Assets::<T>::total_supply(id)
		}

		/// Returns the account balance for the specified `owner` for a given asset ID. Returns `0`
		/// if the account is non-existent.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		/// * `owner` - The account whose balance is being queried.
		///
		/// # Returns
		/// The balance of the specified account, or an error if the operation fails.
		pub fn balance_of(id: AssetIdOf<T>, owner: &AccountIdOf<T>) -> BalanceOf<T> {
			Assets::<T>::balance(id, owner)
		}

		/// Returns the amount which `spender` is still allowed to withdraw from `owner` for a given
		/// asset ID. Returns `0` if no allowance has been set.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		/// * `owner` - The account that owns the tokens.
		/// * `spender` - The account that is allowed to spend the tokens.
		///
		/// # Returns
		/// The remaining allowance, or an error if the operation fails.
		pub fn allowance(
			id: AssetIdOf<T>,
			owner: &AccountIdOf<T>,
			spender: &AccountIdOf<T>,
		) -> BalanceOf<T> {
			Assets::<T>::allowance(id, owner, spender)
		}

		/// Returns the token name for a given asset ID.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		///
		/// # Returns
		/// The name of the token as a byte vector, or an error if the operation fails.
		pub fn token_name(id: AssetIdOf<T>) -> Vec<u8> {
			<Assets<T> as MetadataInspect<AccountIdOf<T>>>::name(id)
		}

		/// Returns the token symbol for a given asset ID.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		///
		/// # Returns
		///  The symbol of the token as a byte vector, or an error if the operation fails.
		pub fn token_symbol(id: AssetIdOf<T>) -> Vec<u8> {
			<Assets<T> as MetadataInspect<AccountIdOf<T>>>::symbol(id)
		}

		/// Returns the token decimals for a given asset ID.
		///
		/// # Arguments
		/// * `id` - The ID of the asset.
		///
		/// # Returns
		///  The number of decimals of the token as a byte vector, or an error if the operation
		/// fails.
		pub fn token_decimals(id: AssetIdOf<T>) -> u8 {
			<Assets<T> as MetadataInspect<AccountIdOf<T>>>::decimals(id)
		}

		/// Set the allowance `value` of the `spender` delegated by `origin`
		pub(crate) fn do_set_allowance(
			origin: OriginFor<T>,
			id: AssetIdParameterOf<T>,
			spender: AccountIdLookupOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			Assets::<T>::cancel_approval(origin.clone(), id.clone(), spender.clone()).map_err(
				|e| {
					e.with_weight(
						T::DbWeight::get().reads(2) + AssetsWeightInfo::<T>::cancel_approval(),
					)
				},
			)?;
			if value > Zero::zero() {
				Assets::<T>::approve_transfer(origin, id, spender, value)?;
			}
			Ok(().into())
		}
	}
}

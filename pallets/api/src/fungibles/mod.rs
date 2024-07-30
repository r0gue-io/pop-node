/// The fungibles pallet serves as a wrapper around the pallet_assets, offering a streamlined
/// interface for interacting with fungible assets. The goal is to provide a simplified, consistent
/// API that adheres to standards in the smart contract space.

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod tests;
pub mod weights;

use frame_support::traits::fungibles::{metadata::Inspect as MetadataInspect, Inspect};
pub use pallet::*;
use pallet_assets::WeightInfo as AssetsWeightInfoTrait;
use weights::WeightInfo;

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
	use sp_runtime::{
		traits::{StaticLookup, Zero},
		Saturating,
	};
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
		BalanceOf {
			/// The asset ID.
			id: AssetIdOf<T>,
			/// The account ID of the owner.
			owner: AccountIdOf<T>,
		},
		/// Allowance for a spender approved by an owner, for a given asset ID.
		#[codec(index = 2)]
		Allowance {
			/// The asset ID.
			id: AssetIdOf<T>,
			/// The account ID of the owner.
			owner: AccountIdOf<T>,
			/// The account ID of the spender.
			spender: AccountIdOf<T>,
		},
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
		/// Weight information for dispatchables in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfers `value` amount of tokens from the caller's account to account `to`, with additional
		/// `data` in unspecified format.
		///
		/// # Parameters
		/// - `id` - The ID of the asset.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
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
		/// - `id` - The ID of the asset.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to approve.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 1))]
		pub fn approve(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let weight = |approve: u32, cancel: u32| -> Weight {
				<T as Config>::WeightInfo::approve(cancel, approve)
			};
			let who = ensure_signed(origin.clone()).map_err(|e| e.with_weight(weight(0, 0)))?;
			let current_allowance = AssetsOf::<T>::allowance(id.clone(), &who, &spender);
			let spender = T::Lookup::unlookup(spender);
			let id: AssetIdParameterOf<T> = id.into();

			// If the new value is equal to the current allowance, do nothing.
			let return_weight = if value == current_allowance {
				weight(0, 0)
			}
			// If the new value is greater than the current allowance, approve the difference
			// because `approve_transfer` works additively (see `pallet-assets`).
			else if value > current_allowance {
				AssetsOf::<T>::approve_transfer(
					origin,
					id,
					spender,
					value.saturating_sub(current_allowance),
				)
				.map_err(|e| e.with_weight(weight(1, 0)))?;
				weight(1, 0)
			} else {
				// If the new value is less than the current allowance, cancel the approval and set the new value
				AssetsOf::<T>::cancel_approval(origin.clone(), id.clone(), spender.clone())
					.map_err(|e| e.with_weight(weight(0, 1)))?;
				if value.is_zero() {
					return Ok(Some(weight(0, 1)).into());
				}
				AssetsOf::<T>::approve_transfer(origin, id, spender, value)?;
				weight(1, 1)
			};
			Ok(Some(return_weight).into())
		}

		/// Increases the allowance of a spender.
		///
		/// # Parameters
		/// - `id` - The ID of the asset.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to increase the allowance by.
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

		/// Create a new token with a given asset ID.
		///
		/// # Parameters
		/// - `id` - The ID of the asset.
		/// - `admin` - The account that will administer the asset.
		/// - `min_balance` - The minimum balance required for accounts holding this asset.
		#[pallet::call_index(11)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::create())]
		pub fn create(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			admin: AccountIdOf<T>,
			min_balance: BalanceOf<T>,
		) -> DispatchResult {
			let admin = T::Lookup::unlookup(admin);
			AssetsOf::<T>::create(origin, id.into(), admin, min_balance)
		}

		/// Create a new token with a given asset ID.
		///
		/// # Parameters
		/// - `id` - The ID of the asset.
		/// - `admin` - The account that will administer the asset.
		/// - `min_balance` - The minimum balance required for accounts holding this asset.
		#[pallet::call_index(12)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::start_destroy())]
		pub fn start_destroy(origin: OriginFor<T>, id: AssetIdOf<T>) -> DispatchResult {
			AssetsOf::<T>::start_destroy(origin, id.into())
		}

		/// Set the metadata for a token with a given asset ID.
		///
		/// # Parameters
		/// - `id`: The identifier of the asset to update.
		/// - `name`: The user friendly name of this asset. Limited in length by
		///   `pallet_assets::Config::StringLimit`.
		/// - `symbol`: The exchange symbol for this asset. Limited in length by
		///   `pallet_assets::Config::StringLimit`.
		/// - `decimals`: The number of decimals this asset uses to represent one unit.
		#[pallet::call_index(16)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::set_metadata(name.len() as u32, symbol.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			AssetsOf::<T>::set_metadata(origin, id.into(), name, symbol, decimals)
		}

		#[pallet::call_index(17)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::clear_metadata())]
		pub fn clear_metadata(origin: OriginFor<T>, id: AssetIdOf<T>) -> DispatchResult {
			AssetsOf::<T>::clear_metadata(origin, id.into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Reads fungible asset state based on the provided value.
		///
		/// This function matches the value to determine the type of state query and returns the
		/// encoded result.
		///
		/// # Parameter
		/// - `value` - An instance of `Read<T>`, which specifies the type of state query and
		/// 		  the associated parameters.
		pub fn read_state(value: Read<T>) -> Vec<u8> {
			use Read::*;

			match value {
				TotalSupply(id) => AssetsOf::<T>::total_supply(id).encode(),
				BalanceOf { id, owner } => AssetsOf::<T>::balance(id, owner).encode(),
				Allowance { id, owner, spender } => {
					AssetsOf::<T>::allowance(id, &owner, &spender).encode()
				},
				TokenName(id) => {
					<AssetsOf<T> as MetadataInspect<AccountIdOf<T>>>::name(id).encode()
				},
				TokenSymbol(id) => {
					<AssetsOf<T> as MetadataInspect<AccountIdOf<T>>>::symbol(id).encode()
				},
				TokenDecimals(id) => {
					<AssetsOf<T> as MetadataInspect<AccountIdOf<T>>>::decimals(id).encode()
				},
			}
		}
	}
}

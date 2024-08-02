/// The fungibles pallet serves as a wrapper around the pallet_assets, offering a streamlined
/// interface for interacting with fungible assets. The goal is to provide a simplified, consistent
/// API that adheres to standards in the smart contract space.

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod tests;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
pub use benchmarking::ArgumentsFactory;

use frame_support::traits::{
	fungible::{Inspect as NativeInspect, Mutate as NativeMutate},
	fungibles::{
		metadata::Inspect as AssetsMetadataInspect, Balanced as AssetsBalanced,
		Inspect as AssetsInspect, Mutate as AssetsMutate,
	},
	tokens::Preservation::Preserve,
};
pub use pallet::*;
use pallet_assets::WeightInfo as AssetsWeightInfoTrait;
use weights::WeightInfo;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type AssetIdOf<T> = <pallet_assets::Pallet<T, AssetsInstanceOf<T>> as AssetsInspect<
	<T as frame_system::Config>::AccountId,
>>::AssetId;
type AssetIdParameterOf<T> = <T as pallet_assets::Config<AssetsInstanceOf<T>>>::AssetIdParameter;
type AssetsOf<T> = pallet_assets::Pallet<T, AssetsInstanceOf<T>>;
type AssetsInstanceOf<T> = <T as Config>::AssetsInstance;
type AssetsWeightInfoOf<T> = <T as pallet_assets::Config<AssetsInstanceOf<T>>>::WeightInfo;
type BalanceOf<T> = <pallet_assets::Pallet<T, AssetsInstanceOf<T>> as AssetsInspect<
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
		traits::{Convert, StaticLookup, Zero},
		Either, Saturating,
	};
	use sp_std::vec::Vec;

	/// State reads for the fungibles api with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config> {
		/// Total token supply for a given asset ID.
		#[codec(index = 0)]
		TotalSupply(T::AssetKind),
		/// Account balance for a given asset ID.
		#[codec(index = 1)]
		BalanceOf {
			/// The asset ID.
			asset: T::AssetKind,
			/// The account ID of the owner.
			owner: AccountIdOf<T>,
		},
		/// Allowance for a spender approved by an owner, for a given asset ID.
		#[codec(index = 2)]
		Allowance {
			/// The asset ID.
			asset: T::AssetKind,
			/// The account ID of the owner.
			owner: AccountIdOf<T>,
			/// The account ID of the spender.
			spender: AccountIdOf<T>,
		},
		/// Token name for a given asset ID.
		#[codec(index = 8)]
		TokenName(T::AssetKind),
		/// Token symbol for a given asset ID.
		#[codec(index = 9)]
		TokenSymbol(T::AssetKind),
		/// Token decimals for a given asset ID.
		#[codec(index = 10)]
		TokenDecimals(T::AssetKind),
	}

	#[pallet::error]
	pub enum Error<T> {
		// The method is not supported for the asset class
		UnsupportedMethod,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config<Self::AssetsInstance> {
		/// Registry of assets utilized for dynamic experience between Native Token and Asset
		type Assets: AssetsInspect<Self::AccountId, AssetId = Self::AssetKind, Balance = Self::Balance>
			+ AssetsMutate<Self::AccountId>
			+ AssetsBalanced<Self::AccountId>;

		/// Type of asset class, sourced from [`Config::Assets`], utilized to identify between `Native` and `Asset`
		type AssetKind: Parameter + MaxEncodedLen;

		/// The criteria to identify the class of the asset
		type AssetCriteria: Convert<Self::AssetKind, Either<(), Self::AssetId>>;

		/// The instance of pallet assets it is tightly coupled to.
		type AssetsInstance;

		/// Type to access the Balances Pallet.
		type NativeBalance: NativeInspect<Self::AccountId> + NativeMutate<Self::AccountId>;

		/// Weight information for dispatchables in this pallet.
		type WeightInfo: WeightInfo;

		/// Helper type for benchmarks.
		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: ArgumentsFactory<Self::AssetKind>;
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
			asset: T::AssetKind,
			target: AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			T::Assets::transfer(asset, &sender, &target, amount, Preserve)?;
			Ok(())
		}

		/// Transfers `value` amount of tokens from the delegated account approved by the `owner` to
		/// account `to`, with additional `data` in unspecified format.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		/// * `owner` - The account from which the asset balance will be withdrawn.
		/// * `to` - The recipient account.
		/// * `value` - The number of tokens to transfer.
		#[pallet::call_index(4)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::transfer_approved())]
		pub fn transfer_from(
			origin: OriginFor<T>,
			asset: T::AssetKind,
			owner: AccountIdOf<T>,
			target: AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			match T::AssetCriteria::convert(asset) {
				Either::Right(id) => {
					let owner = T::Lookup::unlookup(owner);
					let target = T::Lookup::unlookup(target);
					AssetsOf::<T>::transfer_approved(origin, id.into(), owner, target, amount)
				},
				Either::Left(_) => Err(Error::<T>::UnsupportedMethod.into()),
			}
		}

		/// Approves an account to spend a specified number of tokens on behalf of the caller.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		/// * `spender` - The account that is allowed to spend the tokens.
		/// * `value` - The number of tokens to approve.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 1))]
		pub fn approve(
			origin: OriginFor<T>,
			asset: T::AssetKind,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			match T::AssetCriteria::convert(asset) {
				Either::Right(id) => Self::do_approve_asset(origin, id, spender, value),
				Either::Left(_) => Err(Error::<T>::UnsupportedMethod.into()),
			}
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
			asset: T::AssetKind,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			match T::AssetCriteria::convert(asset) {
				Either::Right(id) => {
					let spender = T::Lookup::unlookup(spender);
					AssetsOf::<T>::approve_transfer(origin, id.into(), spender, value)
				},
				Either::Left(_) => Err(Error::<T>::UnsupportedMethod.into()),
			}
		}

		/// Decreases the allowance of a spender.
		///
		/// # Parameters
		/// * `id` - The ID of the asset.
		/// * `spender` - The account that is allowed to spend the tokens.
		/// * `value` - The number of tokens to decrease the allowance by.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 1))]
		pub fn decrease_allowance(
			origin: OriginFor<T>,
			asset: T::AssetKind,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			match T::AssetCriteria::convert(asset) {
				Either::Right(id) => Self::do_decrease_allowance(origin, id, spender, value),
				Either::Left(_) => Err(Error::<T>::UnsupportedMethod.into()),
			}
		}
	}

	impl<T: Config> Pallet<T> {
		fn do_approve_asset(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 0)))?;
			let current_allowance = AssetsOf::<T>::allowance(id.clone(), &who, &spender);
			let spender = T::Lookup::unlookup(spender);
			let id: AssetIdParameterOf<T> = id.into();

			// If the new value is equal to the current allowance, do nothing.
			let return_weight = if value == current_allowance {
				Self::weight_approve(0, 0)
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
				.map_err(|e| e.with_weight(Self::weight_approve(1, 0)))?;
				Self::weight_approve(1, 0)
			} else {
				// If the new value is less than the current allowance, cancel the approval and set the new value
				AssetsOf::<T>::cancel_approval(origin.clone(), id.clone(), spender.clone())
					.map_err(|e| e.with_weight(Self::weight_approve(0, 1)))?;
				if value.is_zero() {
					return Ok(Some(Self::weight_approve(0, 1)).into());
				}
				AssetsOf::<T>::approve_transfer(origin, id, spender, value)?;
				Self::weight_approve(1, 1)
			};
			Ok(Some(return_weight).into())
		}

		fn do_decrease_allowance(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 0)))?;
			let mut current_allowance = AssetsOf::<T>::allowance(id.clone(), &who, &spender);
			let spender = T::Lookup::unlookup(spender);
			let id: AssetIdParameterOf<T> = id.into();

			if value.is_zero() {
				return Ok(Some(Self::weight_approve(0, 0)).into());
			}

			current_allowance.saturating_reduce(value);
			// Cancel the aproval and set the new value if `current_allowance` is more than zero.
			AssetsOf::<T>::cancel_approval(origin.clone(), id.clone(), spender.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 1)))?;

			if current_allowance.is_zero() {
				return Ok(Some(Self::weight_approve(0, 1)).into());
			}
			AssetsOf::<T>::approve_transfer(origin, id, spender, current_allowance)?;
			Ok(().into())
		}

		/// Reads fungible asset state based on the provided value.
		///
		/// This function matches the value to determine the type of state query and returns the
		/// encoded result.
		///
		/// # Parameter
		/// * `value` - An instance of `Read<T>`, which specifies the type of state query and
		/// 		  the associated parameters.
		pub fn read_state(value: Read<T>) -> Vec<u8> {
			use Read::*;

			match value {
				TotalSupply(asset) => T::Assets::total_issuance(asset).encode(),
				BalanceOf { asset, owner } => T::Assets::total_balance(asset, &owner).encode(),
				Allowance { asset, owner, spender } => match T::AssetCriteria::convert(asset) {
					Either::Left(_) => todo!(),
					Either::Right(id) => AssetsOf::<T>::allowance(id, &owner, &spender).encode(),
				},
				TokenName(asset) => match T::AssetCriteria::convert(asset) {
					Either::Left(_) => todo!(),
					Either::Right(id) => {
						<AssetsOf<T> as AssetsMetadataInspect<AccountIdOf<T>>>::name(id).encode()
					},
				},
				TokenSymbol(asset) => match T::AssetCriteria::convert(asset) {
					Either::Left(_) => todo!(),
					Either::Right(id) => {
						<AssetsOf<T> as AssetsMetadataInspect<AccountIdOf<T>>>::symbol(id).encode()
					},
				},
				TokenDecimals(asset) => match T::AssetCriteria::convert(asset) {
					Either::Left(_) => todo!(),
					Either::Right(id) => {
						<AssetsOf<T> as AssetsMetadataInspect<AccountIdOf<T>>>::decimals(id)
							.encode()
					},
				},
			}
		}

		pub fn weight_approve(approve: u32, cancel: u32) -> Weight {
			<T as Config>::WeightInfo::approve(cancel, approve)
		}
	}
}

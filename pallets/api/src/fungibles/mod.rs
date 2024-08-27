//! The fungibles pallet offers a streamlined interface for interacting with fungible assets. The
//! goal is to provide a simplified, consistent API that adheres to standards in the smart contract
//! space.

use frame_support::traits::fungibles::{metadata::Inspect as MetadataInspect, Inspect};
pub use pallet::*;
use pallet_assets::WeightInfo as AssetsWeightInfoTrait;
use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod tests;
pub mod weights;

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
	use core::cmp::Ordering::*;
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

	/// State reads for the fungibles API with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config> {
		/// Total token supply for a specified asset.
		#[codec(index = 0)]
		TotalSupply(AssetIdOf<T>),
		/// Account balance for a specified `asset` and `owner`.
		#[codec(index = 1)]
		BalanceOf {
			/// The asset.
			asset: AssetIdOf<T>,
			/// The owner of the asset.
			owner: AccountIdOf<T>,
		},
		/// Allowance for a `spender` approved by an `owner`, for a specified `asset`.
		#[codec(index = 2)]
		Allowance {
			/// The asset.
			asset: AssetIdOf<T>,
			/// The owner of the asset.
			owner: AccountIdOf<T>,
			/// The spender with an allowance.
			spender: AccountIdOf<T>,
		},
		/// Name of the specified asset.
		#[codec(index = 8)]
		TokenName(AssetIdOf<T>),
		/// Symbol for the specified asset.
		#[codec(index = 9)]
		TokenSymbol(AssetIdOf<T>),
		/// Decimals for the specified asset.
		#[codec(index = 10)]
		TokenDecimals(AssetIdOf<T>),
		/// Check if a specified asset exists.
		#[codec(index = 18)]
		AssetExists(AssetIdOf<T>),
	}

	/// Results of state reads for the fungibles API.
	#[derive(Debug)]
	pub enum ReadResult<T: Config> {
		/// Total token supply for a specified asset.
		TotalSupply(BalanceOf<T>),
		/// Account balance for a specified `asset` and `owner`.
		BalanceOf(BalanceOf<T>),
		/// Allowance for a `spender` approved by an `owner`, for a specified `asset`.
		Allowance(BalanceOf<T>),
		/// Name of the specified asset.
		TokenName(Vec<u8>),
		/// Symbol for the specified asset.
		TokenSymbol(Vec<u8>),
		/// Decimals for the specified asset.
		TokenDecimals(u8),
		/// Whether the specified asset exists.
		AssetExists(bool),
	}

	impl<T: Config> ReadResult<T> {
		/// Encodes the result.
		pub fn encode(&self) -> Vec<u8> {
			use ReadResult::*;
			match self {
				TotalSupply(result) => result.encode(),
				BalanceOf(result) => result.encode(),
				Allowance(result) => result.encode(),
				TokenName(result) => result.encode(),
				TokenSymbol(result) => result.encode(),
				TokenDecimals(result) => result.encode(),
				AssetExists(result) => result.encode(),
			}
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config<Self::AssetsInstance> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The instance of pallet assets it is tightly coupled to.
		type AssetsInstance;
		/// Weight information for dispatchables in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when allowance by `owner` to `spender` changes.
		Approval {
			/// The asset.
			asset: AssetIdOf<T>,
			/// The owner providing the allowance.
			owner: AccountIdOf<T>,
			/// The beneficiary of the allowance.
			spender: AccountIdOf<T>,
			/// The new allowance amount.
			value: BalanceOf<T>,
		},
		/// Event emitted when an asset transfer occurs.
		Transfer {
			/// The asset.
			asset: AssetIdOf<T>,
			/// The source of the transfer. `None` when minting.
			from: Option<AccountIdOf<T>>,
			/// The recipient of the transfer. `None` when burning.
			to: Option<AccountIdOf<T>>,
			/// The amount transferred (or minted/burned).
			value: BalanceOf<T>,
		},
		/// Event emitted when an asset is created.
		Create {
			/// The asset identifier.
			id: AssetIdOf<T>,
			/// The creator of the asset.
			creator: AccountIdOf<T>,
			/// The administrator of the asset.
			admin: AccountIdOf<T>,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfers `value` amount of tokens from the caller's account to account `to`.
		///
		/// # Parameters
		/// - `asset` - The asset to transfer.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[pallet::call_index(3)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::transfer_keep_alive())]
		pub fn transfer(
			origin: OriginFor<T>,
			asset: AssetIdOf<T>,
			to: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			AssetsOf::<T>::transfer_keep_alive(
				origin,
				asset.clone().into(),
				T::Lookup::unlookup(to.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { asset, from: Some(from), to: Some(to), value });
			Ok(())
		}

		/// Transfers `value` amount tokens on behalf of `from` to account `to`.
		///
		/// # Parameters
		/// - `asset` - The asset to transfer.
		/// - `from` - The account from which the asset balance will be withdrawn.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[pallet::call_index(4)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::transfer_approved())]
		pub fn transfer_from(
			origin: OriginFor<T>,
			asset: AssetIdOf<T>,
			from: AccountIdOf<T>,
			to: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::transfer_approved(
				origin,
				asset.clone().into(),
				T::Lookup::unlookup(from.clone()),
				T::Lookup::unlookup(to.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { asset, from: Some(from), to: Some(to), value });
			Ok(())
		}

		/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
		///
		/// # Parameters
		/// - `asset` - The asset to approve.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to approve.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 1))]
		pub fn approve(
			origin: OriginFor<T>,
			asset: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 0)))?;
			let current_allowance = AssetsOf::<T>::allowance(asset.clone(), &owner, &spender);

			let weight = match value.cmp(&current_allowance) {
				// If the new value is equal to the current allowance, do nothing.
				Equal => Self::weight_approve(0, 0),
				// If the new value is greater than the current allowance, approve the difference
				// because `approve_transfer` works additively (see `pallet-assets`).
				Greater => {
					AssetsOf::<T>::approve_transfer(
						origin,
						asset.clone().into(),
						T::Lookup::unlookup(spender.clone()),
						value.saturating_sub(current_allowance),
					)
					.map_err(|e| e.with_weight(Self::weight_approve(1, 0)))?;
					Self::weight_approve(1, 0)
				},
				// If the new value is less than the current allowance, cancel the approval and
				// set the new value.
				Less => {
					let asset_param: AssetIdParameterOf<T> = asset.clone().into();
					let spender_source = T::Lookup::unlookup(spender.clone());
					AssetsOf::<T>::cancel_approval(
						origin.clone(),
						asset_param.clone(),
						spender_source.clone(),
					)
					.map_err(|e| e.with_weight(Self::weight_approve(0, 1)))?;
					if value.is_zero() {
						Self::weight_approve(0, 1)
					} else {
						AssetsOf::<T>::approve_transfer(
							origin,
							asset_param,
							spender_source,
							value,
						)?;
						Self::weight_approve(1, 1)
					}
				},
			};
			Self::deposit_event(Event::Approval { asset, owner, spender, value });
			Ok(Some(weight).into())
		}

		/// Increases the allowance of `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `asset` - The asset to have an allowance increased.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to increase the allowance by.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 0))]
		pub fn increase_allowance(
			origin: OriginFor<T>,
			asset: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 0)))?;
			AssetsOf::<T>::approve_transfer(
				origin,
				asset.clone().into(),
				T::Lookup::unlookup(spender.clone()),
				value,
			)
			.map_err(|e| e.with_weight(AssetsWeightInfoOf::<T>::approve_transfer()))?;
			let value = AssetsOf::<T>::allowance(asset.clone(), &owner, &spender);
			Self::deposit_event(Event::Approval { asset, owner, spender, value });
			Ok(().into())
		}

		/// Decreases the allowance of a `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `asset` - The asset to have an allowance decreased.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to decrease the allowance by.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 1))]
		pub fn decrease_allowance(
			origin: OriginFor<T>,
			asset: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 0)))?;
			if value.is_zero() {
				return Ok(Some(Self::weight_approve(0, 0)).into());
			}
			let current_allowance = AssetsOf::<T>::allowance(asset.clone(), &owner, &spender);
			let spender_source = T::Lookup::unlookup(spender.clone());
			let asset_param: AssetIdParameterOf<T> = asset.clone().into();

			// Cancel the approval and set the new value if `new_allowance` is more than zero.
			AssetsOf::<T>::cancel_approval(
				origin.clone(),
				asset_param.clone(),
				spender_source.clone(),
			)
			.map_err(|e| e.with_weight(Self::weight_approve(0, 1)))?;
			let new_allowance = current_allowance.saturating_sub(value);
			let weight = if new_allowance.is_zero() {
				Self::weight_approve(0, 1)
			} else {
				AssetsOf::<T>::approve_transfer(
					origin,
					asset_param,
					spender_source,
					new_allowance,
				)?;
				Self::weight_approve(1, 1)
			};
			Self::deposit_event(Event::Approval { asset, owner, spender, value: new_allowance });
			Ok(Some(weight).into())
		}

		/// Create a new token with a given identifier.
		///
		/// # Parameters
		/// - `id` - The identifier of the asset.
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
			let creator = ensure_signed(origin.clone())?;
			AssetsOf::<T>::create(
				origin,
				id.clone().into(),
				T::Lookup::unlookup(admin.clone()),
				min_balance,
			)?;
			Self::deposit_event(Event::Create { id, creator, admin });
			Ok(())
		}

		/// Start the process of destroying a token.
		///
		/// # Parameters
		/// - `asset` - The asset to be destroyed.
		#[pallet::call_index(12)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::start_destroy())]
		pub fn start_destroy(origin: OriginFor<T>, asset: AssetIdOf<T>) -> DispatchResult {
			AssetsOf::<T>::start_destroy(origin, asset.into())
		}

		/// Set the metadata for a token.
		///
		/// # Parameters
		/// - `asset`: The asset to update.
		/// - `name`: The user friendly name of this asset.
		/// - `symbol`: The exchange symbol for this asset.
		/// - `decimals`: The number of decimals this asset uses to represent one unit.
		#[pallet::call_index(16)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::set_metadata(name.len() as u32, symbol.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			asset: AssetIdOf<T>,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			AssetsOf::<T>::set_metadata(origin, asset.into(), name, symbol, decimals)
		}

		/// Clear the metadata for a token.
		///
		/// # Parameters
		/// - `asset` - The asset to update.
		#[pallet::call_index(17)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::clear_metadata())]
		pub fn clear_metadata(origin: OriginFor<T>, asset: AssetIdOf<T>) -> DispatchResult {
			AssetsOf::<T>::clear_metadata(origin, asset.into())
		}

		/// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
		///
		/// # Parameters
		/// - `asset` - The asset to mint.
		/// - `account` - The account to be credited with the created tokens.
		/// - `value` - The number of tokens to mint.
		#[pallet::call_index(19)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			asset: AssetIdOf<T>,
			account: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::mint(
				origin,
				asset.clone().into(),
				T::Lookup::unlookup(account.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { asset, from: None, to: Some(account), value });
			Ok(())
		}

		/// Destroys `value` amount of tokens from `account`, reducing the total supply.
		///
		/// # Parameters
		/// - `asset` - the asset to burn.
		/// - `account` - The account from which the tokens will be destroyed.
		/// - `value` - The number of tokens to destroy.
		#[pallet::call_index(20)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			asset: AssetIdOf<T>,
			account: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::burn(
				origin,
				asset.clone().into(),
				T::Lookup::unlookup(account.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { asset, from: Some(account), to: None, value });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn weight_approve(approve: u32, cancel: u32) -> Weight {
			<T as Config>::WeightInfo::approve(cancel, approve)
		}
	}

	impl<T: Config> crate::Read for Pallet<T> {
		/// The type of read requested.
		type Read = Read<T>;
		/// The type or result returned.
		type Result = ReadResult<T>;

		/// Determines the weight of the requested read, used to charge the appropriate weight before the read is performed.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn weight(_request: &Self::Read) -> Weight {
			// TODO: match on request and return benchmarked weight
			T::DbWeight::get().reads(1_u64)
		}

		/// Performs the requested read and returns the result.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn read(request: Self::Read) -> Self::Result {
			use Read::*;
			match request {
				TotalSupply(asset) => ReadResult::TotalSupply(AssetsOf::<T>::total_supply(asset)),
				BalanceOf { asset, owner } => {
					ReadResult::BalanceOf(AssetsOf::<T>::balance(asset, owner))
				},
				Allowance { asset, owner, spender } => {
					ReadResult::Allowance(AssetsOf::<T>::allowance(asset, &owner, &spender))
				},
				TokenName(asset) => ReadResult::TokenName(<AssetsOf<T> as MetadataInspect<
					AccountIdOf<T>,
				>>::name(asset)),
				TokenSymbol(asset) => ReadResult::TokenSymbol(<AssetsOf<T> as MetadataInspect<
					AccountIdOf<T>,
				>>::symbol(asset)),
				TokenDecimals(asset) => ReadResult::TokenDecimals(
					<AssetsOf<T> as MetadataInspect<AccountIdOf<T>>>::decimals(asset),
				),
				AssetExists(asset) => ReadResult::AssetExists(AssetsOf::<T>::asset_exists(asset)),
			}
		}
	}
}

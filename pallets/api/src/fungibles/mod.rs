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
		/// Check if token with a given asset ID exists.
		#[codec(index = 18)]
		AssetExists(AssetIdOf<T>),
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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when allowance by `owner` to `spender` changes.
		Approval {
			/// Token ID.
			id: AssetIdOf<T>,
			/// Account providing allowance.
			owner: AccountIdOf<T>,
			/// Allowance beneficiary.
			spender: AccountIdOf<T>,
			/// New allowance amount.
			value: BalanceOf<T>,
		},
		/// Event emitted when transfer of tokens occurs.
		Transfer {
			/// Token ID.
			id: AssetIdOf<T>,
			/// Transfer sender. `None` in case of minting new tokens.
			from: Option<AccountIdOf<T>>,
			/// Transfer recipient. `None` in case of burning tokens.
			to: Option<AccountIdOf<T>>,
			/// Amount of tokens transferred (or minted/burned).
			value: BalanceOf<T>,
		},
		/// Event emitted when a token is created.
		Create {
			/// Token ID.
			id: AssetIdOf<T>,
			/// Owner of the token created.
			owner: AccountIdOf<T>,
			/// Admin of the token created.
			admin: AccountIdOf<T>,
		},
		/// Event emitted when a token is in the process of being destroyed.
		StartDestroy { id: AssetIdOf<T> },
		/// Event emitted when new metadata is set for a token.
		SetMetadata {
			/// Token ID.
			id: AssetIdOf<T>,
			/// Token name.
			name: Vec<u8>,
			/// Token symbol.
			symbol: Vec<u8>,
			/// Token decimals.
			decimals: u8,
		},
		/// Event emitted when metadata is cleared for a token.
		ClearMetadata { id: AssetIdOf<T> },
	}

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
			to: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::transfer_keep_alive(
				origin.clone(),
				id.clone().into(),
				T::Lookup::unlookup(to.clone()),
				value,
			)?;
			let from = ensure_signed(origin)?;
			Self::deposit_event(Event::Transfer { id, from: Some(from), to: Some(to), value });
			Ok(())
		}

		/// Transfers `value` amount tokens on behalf of `from` to account `to` with additional `data`
		/// in unspecified format.
		///
		/// # Parameters
		/// - `id` - The ID of the asset.
		/// - `from` - The account from which the asset balance will be withdrawn.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[pallet::call_index(4)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::transfer_approved())]
		pub fn transfer_from(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			from: AccountIdOf<T>,
			to: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::transfer_approved(
				origin,
				id.clone().into(),
				T::Lookup::unlookup(from.clone()),
				T::Lookup::unlookup(to.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { id, from: Some(from), to: Some(to), value });
			Ok(())
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
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 0)))?;
			let current_allowance = AssetsOf::<T>::allowance(id.clone(), &owner, &spender);
			let spender_unlookup = T::Lookup::unlookup(spender.clone());
			let id_param: AssetIdParameterOf<T> = id.clone().into();

			let return_weight = match value.cmp(&current_allowance) {
				// If the new value is equal to the current allowance, do nothing.
				Equal => Self::weight_approve(0, 0),
				// If the new value is greater than the current allowance, approve the difference
				// because `approve_transfer` works additively (see `pallet-assets`).
				Greater => {
					AssetsOf::<T>::approve_transfer(
						origin,
						id_param,
						spender_unlookup,
						value.saturating_sub(current_allowance),
					)
					.map_err(|e| e.with_weight(Self::weight_approve(1, 0)))?;
					Self::weight_approve(1, 0)
				},
				// If the new value is less than the current allowance, cancel the approval and
				// set the new value.
				Less => {
					AssetsOf::<T>::cancel_approval(
						origin.clone(),
						id_param.clone(),
						spender_unlookup.clone(),
					)
					.map_err(|e| e.with_weight(Self::weight_approve(0, 1)))?;
					if value.is_zero() {
						Self::weight_approve(0, 1)
					} else {
						AssetsOf::<T>::approve_transfer(origin, id_param, spender_unlookup, value)?;
						Self::weight_approve(1, 1)
					}
				},
			};
			Self::deposit_event(Event::Approval { id, owner, spender, value });
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
			let owner = ensure_signed(origin.clone())?;
			AssetsOf::<T>::approve_transfer(
				origin,
				id.clone().into(),
				T::Lookup::unlookup(spender.clone()),
				value,
			)?;
			let value = AssetsOf::<T>::allowance(id.clone(), &owner, &spender);
			Self::deposit_event(Event::Approval { id, owner, spender, value });
			Ok(())
		}

		/// Decreases the allowance of a spender.
		///
		/// # Parameters
		/// - `id` - The ID of the asset.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to decrease the allowance by.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 1))]
		pub fn decrease_allowance(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 0)))?;
			let current_allowance = AssetsOf::<T>::allowance(id.clone(), &owner, &spender);
			let spender_unlookup = T::Lookup::unlookup(spender.clone());
			let id_param: AssetIdParameterOf<T> = id.clone().into();

			if value.is_zero() {
				return Ok(Some(Self::weight_approve(0, 0)).into());
			}
			// Cancel the approval and set the new value if `new_allowance` is more than zero.
			AssetsOf::<T>::cancel_approval(
				origin.clone(),
				id_param.clone(),
				spender_unlookup.clone(),
			)
			.map_err(|e| e.with_weight(Self::weight_approve(0, 1)))?;
			let new_allowance = current_allowance.saturating_sub(value);
			let weight = if new_allowance.is_zero() {
				Self::weight_approve(0, 1)
			} else {
				AssetsOf::<T>::approve_transfer(origin, id_param, spender_unlookup, new_allowance)?;
				Self::weight_approve(1, 1)
			};
			Self::deposit_event(Event::Approval { id, owner, spender, value: new_allowance });
			Ok(Some(weight).into())
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
			AssetsOf::<T>::create(
				origin.clone(),
				id.clone().into(),
				T::Lookup::unlookup(admin.clone()),
				min_balance,
			)?;
			let owner = ensure_signed(origin)?;
			Self::deposit_event(Event::Create { id, owner, admin });
			Ok(())
		}

		/// Start the process of destroying a token with a given asset ID.
		///
		/// # Parameters
		/// - `id` - The ID of the asset.
		#[pallet::call_index(12)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::start_destroy())]
		pub fn start_destroy(origin: OriginFor<T>, id: AssetIdOf<T>) -> DispatchResult {
			AssetsOf::<T>::start_destroy(origin, id.clone().into())?;
			Self::deposit_event(Event::StartDestroy { id });
			Ok(())
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
			AssetsOf::<T>::set_metadata(
				origin,
				id.clone().into(),
				name.clone(),
				symbol.clone(),
				decimals,
			)?;
			Self::deposit_event(Event::SetMetadata { id, name, symbol, decimals });
			Ok(())
		}

		/// Clear the metadata for a token with a given asset ID.
		///
		/// # Parameters
		/// - `id` - The ID of the asset.
		#[pallet::call_index(17)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::clear_metadata())]
		pub fn clear_metadata(origin: OriginFor<T>, id: AssetIdOf<T>) -> DispatchResult {
			AssetsOf::<T>::clear_metadata(origin, id.clone().into())?;
			Self::deposit_event(Event::ClearMetadata { id });
			Ok(())
		}

		/// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
		///
		/// # Parameters
		/// - `id` - The ID of the asset.
		/// - `account` - The account to be credited with the created tokens.
		/// - `value` - The number of tokens to mint.
		#[pallet::call_index(19)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			account: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::mint(
				origin,
				id.clone().into(),
				T::Lookup::unlookup(account.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { id, from: None, to: Some(account), value });
			Ok(())
		}

		/// Destroys `value` amount of tokens from `account`, reducing the total supply.
		///
		/// # Parameters
		/// - `id` - The ID of the asset.
		/// - `account` - The account from which the tokens will be destroyed.
		/// - `value` - The number of tokens to destroy.
		#[pallet::call_index(20)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			id: AssetIdOf<T>,
			account: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::burn(
				origin,
				id.clone().into(),
				T::Lookup::unlookup(account.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { id, from: Some(account), to: None, value });
			Ok(())
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
		///   the associated parameters.
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
				AssetExists(id) => AssetsOf::<T>::asset_exists(id).encode(),
			}
		}

		pub fn weight_approve(approve: u32, cancel: u32) -> Weight {
			<T as Config>::WeightInfo::approve(cancel, approve)
		}
	}
}

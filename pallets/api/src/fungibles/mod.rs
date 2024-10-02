//! The fungibles pallet offers a streamlined interface for interacting with fungible tokens. The
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
type TokenIdOf<T> = <AssetsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type TokenIdParameterOf<T> = <T as pallet_assets::Config<AssetsInstanceOf<T>>>::AssetIdParameter;
type AssetsOf<T> = pallet_assets::Pallet<T, AssetsInstanceOf<T>>;
type AssetsErrorOf<T> = pallet_assets::Error<T, AssetsInstanceOf<T>>;
type AssetsInstanceOf<T> = <T as Config>::AssetsInstance;
type AssetsWeightInfoOf<T> = <T as pallet_assets::Config<AssetsInstanceOf<T>>>::WeightInfo;
type BalanceOf<T> = <AssetsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type WeightOf<T> = <T as Config>::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use core::cmp::Ordering::*;

	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo, WithPostDispatchInfo},
		pallet_prelude::*,
		traits::fungibles::approvals::Inspect as ApprovalInspect,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{CheckedSub, StaticLookup, Zero},
		Saturating,
	};
	use sp_std::vec::Vec;

	use super::*;

	/// State reads for the fungibles API with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config> {
		/// Total token supply for a specified token.
		#[codec(index = 0)]
		TotalSupply(TokenIdOf<T>),
		/// Account balance for a specified `token` and `owner`.
		#[codec(index = 1)]
		BalanceOf {
			/// The token.
			token: TokenIdOf<T>,
			/// The owner of the token.
			owner: AccountIdOf<T>,
		},
		/// Allowance for a `spender` approved by an `owner`, for a specified `token`.
		#[codec(index = 2)]
		Allowance {
			/// The token.
			token: TokenIdOf<T>,
			/// The owner of the token.
			owner: AccountIdOf<T>,
			/// The spender with an allowance.
			spender: AccountIdOf<T>,
		},
		/// Name of the specified token.
		#[codec(index = 8)]
		TokenName(TokenIdOf<T>),
		/// Symbol for the specified token.
		#[codec(index = 9)]
		TokenSymbol(TokenIdOf<T>),
		/// Decimals for the specified token.
		#[codec(index = 10)]
		TokenDecimals(TokenIdOf<T>),
		/// Whether a specified token exists.
		#[codec(index = 18)]
		TokenExists(TokenIdOf<T>),
	}

	/// Results of state reads for the fungibles API.
	#[derive(Debug)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	pub enum ReadResult<T: Config> {
		/// Total token supply for a specified token.
		TotalSupply(BalanceOf<T>),
		/// Account balance for a specified token and owner.
		BalanceOf(BalanceOf<T>),
		/// Allowance for a spender approved by an owner, for a specified token.
		Allowance(BalanceOf<T>),
		/// Name of the specified token.
		TokenName(Vec<u8>),
		/// Symbol for the specified token.
		TokenSymbol(Vec<u8>),
		/// Decimals for the specified token.
		TokenDecimals(u8),
		/// Whether the specified token exists.
		TokenExists(bool),
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
				TokenExists(result) => result.encode(),
			}
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config<Self::AssetsInstance> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The instance of pallet-assets.
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
		// Differing style: event name abides by the PSP22 standard.
		Approval {
			/// The token.
			token: TokenIdOf<T>,
			/// The owner providing the allowance.
			owner: AccountIdOf<T>,
			/// The beneficiary of the allowance.
			spender: AccountIdOf<T>,
			/// The new allowance amount.
			value: BalanceOf<T>,
		},
		/// Event emitted when a token transfer occurs.
		// Differing style: event name abides by the PSP22 standard.
		Transfer {
			/// The token.
			token: TokenIdOf<T>,
			/// The source of the transfer. `None` when minting.
			from: Option<AccountIdOf<T>>,
			/// The recipient of the transfer. `None` when burning.
			to: Option<AccountIdOf<T>>,
			/// The amount transferred (or minted/burned).
			value: BalanceOf<T>,
		},
		/// Event emitted when a token is created.
		Created {
			/// The token identifier.
			id: TokenIdOf<T>,
			/// The creator of the token.
			creator: AccountIdOf<T>,
			/// The administrator of the token.
			admin: AccountIdOf<T>,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfers `value` amount of tokens from the caller's account to account `to`.
		///
		/// # Parameters
		/// - `token` - The token to transfer.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[pallet::call_index(3)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::transfer_keep_alive())]
		pub fn transfer(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			to: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			AssetsOf::<T>::transfer_keep_alive(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(to.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { token, from: Some(from), to: Some(to), value });
			Ok(())
		}

		/// Transfers `value` amount tokens on behalf of `from` to account `to` with additional
		/// `data` in unspecified format.
		///
		/// # Parameters
		/// - `token` - The token to transfer.
		/// - `from` - The account from which the token balance will be withdrawn.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[pallet::call_index(4)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::transfer_approved())]
		pub fn transfer_from(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			from: AccountIdOf<T>,
			to: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::transfer_approved(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(from.clone()),
				T::Lookup::unlookup(to.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { token, from: Some(from), to: Some(to), value });
			Ok(())
		}

		/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
		///
		/// # Parameters
		/// - `token` - The token to approve.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to approve.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 1))]
		pub fn approve(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(WeightOf::<T>::approve(0, 0)))?;
			let current_allowance = AssetsOf::<T>::allowance(token.clone(), &owner, &spender);

			let weight = match value.cmp(&current_allowance) {
				// If the new value is equal to the current allowance, do nothing.
				Equal => WeightOf::<T>::approve(0, 0),
				// If the new value is greater than the current allowance, approve the difference
				// because `approve_transfer` works additively (see `pallet-assets`).
				Greater => {
					AssetsOf::<T>::approve_transfer(
						origin,
						token.clone().into(),
						T::Lookup::unlookup(spender.clone()),
						value.saturating_sub(current_allowance),
					)
					.map_err(|e| e.with_weight(WeightOf::<T>::approve(1, 0)))?;
					WeightOf::<T>::approve(1, 0)
				},
				// If the new value is less than the current allowance, cancel the approval and
				// set the new value.
				Less => {
					let token_param: TokenIdParameterOf<T> = token.clone().into();
					let spender_source = T::Lookup::unlookup(spender.clone());
					AssetsOf::<T>::cancel_approval(
						origin.clone(),
						token_param.clone(),
						spender_source.clone(),
					)
					.map_err(|e| e.with_weight(WeightOf::<T>::approve(0, 1)))?;
					if value.is_zero() {
						WeightOf::<T>::approve(0, 1)
					} else {
						AssetsOf::<T>::approve_transfer(
							origin,
							token_param,
							spender_source,
							value,
						)?;
						WeightOf::<T>::approve(1, 1)
					}
				},
			};
			Self::deposit_event(Event::Approval { token, owner, spender, value });
			Ok(Some(weight).into())
		}

		/// Increases the allowance of `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `token` - The token to have an allowance increased.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to increase the allowance by.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 0))]
		pub fn increase_allowance(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(WeightOf::<T>::approve(0, 0)))?;
			AssetsOf::<T>::approve_transfer(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(spender.clone()),
				value,
			)
			.map_err(|e| e.with_weight(AssetsWeightInfoOf::<T>::approve_transfer()))?;
			let value = AssetsOf::<T>::allowance(token.clone(), &owner, &spender);
			Self::deposit_event(Event::Approval { token, owner, spender, value });
			Ok(().into())
		}

		/// Decreases the allowance of `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `token` - The token to have an allowance decreased.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to decrease the allowance by.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 1))]
		pub fn decrease_allowance(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(WeightOf::<T>::approve(0, 0)))?;
			if value.is_zero() {
				return Ok(Some(WeightOf::<T>::approve(0, 0)).into());
			}
			let current_allowance = AssetsOf::<T>::allowance(token.clone(), &owner, &spender);
			let spender_source = T::Lookup::unlookup(spender.clone());
			let token_param: TokenIdParameterOf<T> = token.clone().into();

			// Cancel the approval and approve `new_allowance` if difference is more than zero.
			let new_allowance =
				current_allowance.checked_sub(&value).ok_or(AssetsErrorOf::<T>::Unapproved)?;
			AssetsOf::<T>::cancel_approval(
				origin.clone(),
				token_param.clone(),
				spender_source.clone(),
			)
			.map_err(|e| e.with_weight(WeightOf::<T>::approve(0, 1)))?;
			let weight = if new_allowance.is_zero() {
				WeightOf::<T>::approve(0, 1)
			} else {
				AssetsOf::<T>::approve_transfer(
					origin,
					token_param,
					spender_source,
					new_allowance,
				)?;
				WeightOf::<T>::approve(1, 1)
			};
			Self::deposit_event(Event::Approval { token, owner, spender, value: new_allowance });
			Ok(Some(weight).into())
		}

		/// Create a new token with a given identifier.
		///
		/// # Parameters
		/// - `id` - The identifier of the token.
		/// - `admin` - The account that will administer the token.
		/// - `min_balance` - The minimum balance required for accounts holding this token.
		#[pallet::call_index(11)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::create())]
		pub fn create(
			origin: OriginFor<T>,
			id: TokenIdOf<T>,
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
			Self::deposit_event(Event::Created { id, creator, admin });
			Ok(())
		}

		/// Start the process of destroying a token.
		///
		/// # Parameters
		/// - `token` - The token to be destroyed.
		// See `pallet-assets` documentation for more information. Related dispatchables are:
		// - `destroy_accounts`
		// - `destroy_approvals`
		// - `finish_destroy`
		#[pallet::call_index(12)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::start_destroy())]
		pub fn start_destroy(origin: OriginFor<T>, token: TokenIdOf<T>) -> DispatchResult {
			AssetsOf::<T>::start_destroy(origin, token.into())
		}

		/// Set the metadata for a token.
		///
		/// # Parameters
		/// - `token`: The token to update.
		/// - `name`: The user friendly name of this token.
		/// - `symbol`: The exchange symbol for this token.
		/// - `decimals`: The number of decimals this token uses to represent one unit.
		#[pallet::call_index(16)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::set_metadata(name.len() as u32, symbol.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			AssetsOf::<T>::set_metadata(origin, token.into(), name, symbol, decimals)
		}

		/// Clear the metadata for a token.
		///
		/// # Parameters
		/// - `token` - The token to update.
		#[pallet::call_index(17)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::clear_metadata())]
		pub fn clear_metadata(origin: OriginFor<T>, token: TokenIdOf<T>) -> DispatchResult {
			AssetsOf::<T>::clear_metadata(origin, token.into())
		}

		/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
		/// supply.
		///
		/// # Parameters
		/// - `token` - The token to mint.
		/// - `account` - The account to be credited with the created tokens.
		/// - `value` - The number of tokens to mint.
		#[pallet::call_index(19)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			account: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::mint(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(account.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { token, from: None, to: Some(account), value });
			Ok(())
		}

		/// Destroys `value` amount of tokens from `account`, reducing the total supply.
		///
		/// # Parameters
		/// - `token` - the token to burn.
		/// - `account` - The account from which the tokens will be destroyed.
		/// - `value` - The number of tokens to destroy.
		#[pallet::call_index(20)]
		#[pallet::weight(<T as Config>::WeightInfo::balance_of() + AssetsWeightInfoOf::<T>::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			account: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let current_balance = AssetsOf::<T>::balance(token.clone(), &account);
			if current_balance < value {
				return Err(AssetsErrorOf::<T>::BalanceLow
					.with_weight(<T as Config>::WeightInfo::balance_of()));
			}
			AssetsOf::<T>::burn(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(account.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { token, from: Some(account), to: None, value });
			Ok(().into())
		}
	}

	impl<T: Config> crate::Read for Pallet<T> {
		/// The type of read requested.
		type Read = Read<T>;
		/// The type or result returned.
		type Result = ReadResult<T>;

		/// Determines the weight of the requested read, used to charge the appropriate weight
		/// before the read is performed.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn weight(request: &Self::Read) -> Weight {
			use Read::*;
			match request {
				TotalSupply(_) => <T as Config>::WeightInfo::total_supply(),
				BalanceOf { .. } => <T as Config>::WeightInfo::balance_of(),
				Allowance { .. } => <T as Config>::WeightInfo::allowance(),
				TokenName(_) => <T as Config>::WeightInfo::token_name(),
				TokenSymbol(_) => <T as Config>::WeightInfo::token_symbol(),
				TokenDecimals(_) => <T as Config>::WeightInfo::token_decimals(),
				TokenExists(_) => <T as Config>::WeightInfo::token_exists(),
			}
		}

		/// Performs the requested read and returns the result.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn read(request: Self::Read) -> Self::Result {
			use Read::*;
			match request {
				TotalSupply(token) => ReadResult::TotalSupply(AssetsOf::<T>::total_supply(token)),
				BalanceOf { token, owner } =>
					ReadResult::BalanceOf(AssetsOf::<T>::balance(token, owner)),
				Allowance { token, owner, spender } =>
					ReadResult::Allowance(AssetsOf::<T>::allowance(token, &owner, &spender)),
				TokenName(token) => ReadResult::TokenName(<AssetsOf<T> as MetadataInspect<
					AccountIdOf<T>,
				>>::name(token)),
				TokenSymbol(token) => ReadResult::TokenSymbol(<AssetsOf<T> as MetadataInspect<
					AccountIdOf<T>,
				>>::symbol(token)),
				TokenDecimals(token) => ReadResult::TokenDecimals(
					<AssetsOf<T> as MetadataInspect<AccountIdOf<T>>>::decimals(token),
				),
				TokenExists(token) => ReadResult::TokenExists(AssetsOf::<T>::asset_exists(token)),
			}
		}
	}
}

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
type AssetsInstanceOf<T, I = ()> = <T as Config<I>>::AssetsInstance;
type AssetsOf<T, I = ()> = pallet_assets::Pallet<T, AssetsInstanceOf<T, I>>;
type TokenIdOf<T, I = ()> = <AssetsOf<T, I> as Inspect<AccountIdOf<T>>>::AssetId;
type TokenIdParameterOf<T, I = ()> =
	<T as pallet_assets::Config<AssetsInstanceOf<T, I>>>::AssetIdParameter;
type AssetsErrorOf<T, I = ()> = pallet_assets::Error<T, AssetsInstanceOf<T, I>>;
type AssetsWeightInfoOf<T, I = ()> =
	<T as pallet_assets::Config<AssetsInstanceOf<T, I>>>::WeightInfo;
type BalanceOf<T, I = ()> = <AssetsOf<T, I> as Inspect<AccountIdOf<T>>>::Balance;
type WeightOf<T, I = ()> = <T as Config<I>>::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use core::{cmp::Ordering::*, marker::PhantomData};

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

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config<I: 'static = ()>:
		frame_system::Config + pallet_assets::Config<Self::AssetsInstance>
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The instance of pallet-assets.
		type AssetsInstance;
		/// Weight information for dispatchables in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// Event emitted when allowance by `owner` to `spender` changes.
		// Differing style: event name abides by the PSP22 standard.
		Approval {
			/// The token.
			token: TokenIdOf<T, I>,
			/// The owner providing the allowance.
			owner: AccountIdOf<T>,
			/// The beneficiary of the allowance.
			spender: AccountIdOf<T>,
			/// The new allowance amount.
			value: BalanceOf<T, I>,
		},
		/// Event emitted when a token transfer occurs.
		// Differing style: event name abides by the PSP22 standard.
		Transfer {
			/// The token.
			token: TokenIdOf<T, I>,
			/// The source of the transfer. `None` when minting.
			from: Option<AccountIdOf<T>>,
			/// The recipient of the transfer. `None` when burning.
			to: Option<AccountIdOf<T>>,
			/// The amount transferred (or minted/burned).
			value: BalanceOf<T, I>,
		},
		/// Event emitted when a token is created.
		Created {
			/// The token identifier.
			id: TokenIdOf<T, I>,
			/// The creator of the token.
			creator: AccountIdOf<T>,
			/// The administrator of the token.
			admin: AccountIdOf<T>,
		},
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Transfers `value` amount of tokens from the caller's account to account `to`.
		///
		/// # Parameters
		/// - `token` - The token to transfer.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[pallet::call_index(3)]
		#[pallet::weight(AssetsWeightInfoOf::<T, I>::transfer_keep_alive())]
		pub fn transfer(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
			to: AccountIdOf<T>,
			value: BalanceOf<T, I>,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			AssetsOf::<T, I>::transfer_keep_alive(
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
		#[pallet::weight(AssetsWeightInfoOf::<T, I>::transfer_approved())]
		pub fn transfer_from(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
			from: AccountIdOf<T>,
			to: AccountIdOf<T>,
			value: BalanceOf<T, I>,
		) -> DispatchResult {
			AssetsOf::<T, I>::transfer_approved(
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
		#[pallet::weight(<T as Config<I>>::WeightInfo::approve(1, 1))]
		pub fn approve(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T, I>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(WeightOf::<T, I>::approve(0, 0)))?;
			let current_allowance = AssetsOf::<T, I>::allowance(token.clone(), &owner, &spender);

			let weight = match value.cmp(&current_allowance) {
				// If the new value is equal to the current allowance, do nothing.
				Equal => WeightOf::<T, I>::approve(0, 0),
				// If the new value is greater than the current allowance, approve the difference
				// because `approve_transfer` works additively (see `pallet-assets`).
				Greater => {
					AssetsOf::<T, I>::approve_transfer(
						origin,
						token.clone().into(),
						T::Lookup::unlookup(spender.clone()),
						value.saturating_sub(current_allowance),
					)
					.map_err(|e| e.with_weight(WeightOf::<T, I>::approve(1, 0)))?;
					WeightOf::<T, I>::approve(1, 0)
				},
				// If the new value is less than the current allowance, cancel the approval and
				// set the new value.
				Less => {
					let token_param: TokenIdParameterOf<T, I> = token.clone().into();
					let spender_source = T::Lookup::unlookup(spender.clone());
					AssetsOf::<T, I>::cancel_approval(
						origin.clone(),
						token_param.clone(),
						spender_source.clone(),
					)
					.map_err(|e| e.with_weight(WeightOf::<T, I>::approve(0, 1)))?;
					if value.is_zero() {
						WeightOf::<T, I>::approve(0, 1)
					} else {
						AssetsOf::<T, I>::approve_transfer(
							origin,
							token_param,
							spender_source,
							value,
						)?;
						WeightOf::<T, I>::approve(1, 1)
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
		#[pallet::weight(<T as Config<I>>::WeightInfo::approve(1, 0))]
		pub fn increase_allowance(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T, I>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(WeightOf::<T, I>::approve(0, 0)))?;
			AssetsOf::<T, I>::approve_transfer(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(spender.clone()),
				value,
			)
			.map_err(|e| e.with_weight(AssetsWeightInfoOf::<T, I>::approve_transfer()))?;
			let value = AssetsOf::<T, I>::allowance(token.clone(), &owner, &spender);
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
		#[pallet::weight(<T as Config<I>>::WeightInfo::approve(1, 1))]
		pub fn decrease_allowance(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T, I>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(WeightOf::<T, I>::approve(0, 0)))?;
			if value.is_zero() {
				return Ok(Some(WeightOf::<T, I>::approve(0, 0)).into());
			}
			let current_allowance = AssetsOf::<T, I>::allowance(token.clone(), &owner, &spender);
			let spender_source = T::Lookup::unlookup(spender.clone());
			let token_param: TokenIdParameterOf<T, I> = token.clone().into();

			// Cancel the approval and approve `new_allowance` if difference is more than zero.
			let new_allowance =
				current_allowance.checked_sub(&value).ok_or(AssetsErrorOf::<T, I>::Unapproved)?;
			AssetsOf::<T, I>::cancel_approval(
				origin.clone(),
				token_param.clone(),
				spender_source.clone(),
			)
			.map_err(|e| e.with_weight(WeightOf::<T, I>::approve(0, 1)))?;
			let weight = if new_allowance.is_zero() {
				WeightOf::<T, I>::approve(0, 1)
			} else {
				AssetsOf::<T, I>::approve_transfer(
					origin,
					token_param,
					spender_source,
					new_allowance,
				)?;
				WeightOf::<T, I>::approve(1, 1)
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
		#[pallet::weight(AssetsWeightInfoOf::<T, I>::create())]
		pub fn create(
			origin: OriginFor<T>,
			id: TokenIdOf<T, I>,
			admin: AccountIdOf<T>,
			min_balance: BalanceOf<T, I>,
		) -> DispatchResult {
			let creator = ensure_signed(origin.clone())?;
			AssetsOf::<T, I>::create(
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
		#[pallet::weight(AssetsWeightInfoOf::<T, I>::start_destroy())]
		pub fn start_destroy(origin: OriginFor<T>, token: TokenIdOf<T, I>) -> DispatchResult {
			AssetsOf::<T, I>::start_destroy(origin, token.into())
		}

		/// Set the metadata for a token.
		///
		/// # Parameters
		/// - `token`: The token to update.
		/// - `name`: The user friendly name of this token.
		/// - `symbol`: The exchange symbol for this token.
		/// - `decimals`: The number of decimals this token uses to represent one unit.
		#[pallet::call_index(16)]
		#[pallet::weight(AssetsWeightInfoOf::<T, I>::set_metadata(name.len() as u32, symbol.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			AssetsOf::<T, I>::set_metadata(origin, token.into(), name, symbol, decimals)
		}

		/// Clear the metadata for a token.
		///
		/// # Parameters
		/// - `token` - The token to update.
		#[pallet::call_index(17)]
		#[pallet::weight(AssetsWeightInfoOf::<T, I>::clear_metadata())]
		pub fn clear_metadata(origin: OriginFor<T>, token: TokenIdOf<T, I>) -> DispatchResult {
			AssetsOf::<T, I>::clear_metadata(origin, token.into())
		}

		/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
		/// supply.
		///
		/// # Parameters
		/// - `token` - The token to mint.
		/// - `account` - The account to be credited with the created tokens.
		/// - `value` - The number of tokens to mint.
		#[pallet::call_index(19)]
		#[pallet::weight(AssetsWeightInfoOf::<T, I>::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
			account: AccountIdOf<T>,
			value: BalanceOf<T, I>,
		) -> DispatchResult {
			AssetsOf::<T, I>::mint(
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
		#[pallet::weight(<T as Config<I>>::WeightInfo::balance_of() + AssetsWeightInfoOf::<T, I>::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
			account: AccountIdOf<T>,
			value: BalanceOf<T, I>,
		) -> DispatchResultWithPostInfo {
			let current_balance = AssetsOf::<T, I>::balance(token.clone(), &account);
			if current_balance < value {
				return Err(AssetsErrorOf::<T, I>::BalanceLow
					.with_weight(<T as Config<I>>::WeightInfo::balance_of()));
			}
			AssetsOf::<T, I>::burn(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(account.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { token, from: Some(account), to: None, value });
			Ok(().into())
		}
	}

	/// State reads for the fungibles API with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config<I>, I: 'static = ()> {
		/// Total token supply for a specified token.
		#[codec(index = 0)]
		TotalSupply(TokenIdOf<T, I>),
		/// Account balance for a specified `token` and `owner`.
		#[codec(index = 1)]
		BalanceOf {
			/// The token.
			token: TokenIdOf<T, I>,
			/// The owner of the token.
			owner: AccountIdOf<T>,
		},
		/// Allowance for a `spender` approved by an `owner`, for a specified `token`.
		#[codec(index = 2)]
		Allowance {
			/// The token.
			token: TokenIdOf<T, I>,
			/// The owner of the token.
			owner: AccountIdOf<T>,
			/// The spender with an allowance.
			spender: AccountIdOf<T>,
		},
		/// Name of the specified token.
		#[codec(index = 8)]
		TokenName(TokenIdOf<T, I>),
		/// Symbol for the specified token.
		#[codec(index = 9)]
		TokenSymbol(TokenIdOf<T, I>),
		/// Decimals for the specified token.
		#[codec(index = 10)]
		TokenDecimals(TokenIdOf<T, I>),
		/// Whether a specified token exists.
		#[codec(index = 18)]
		TokenExists(TokenIdOf<T, I>),
	}

	/// Results of state reads for the fungibles API.
	#[derive(Debug)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	pub enum ReadResult<T: Config<I>, I: 'static = ()> {
		/// Total token supply for a specified token.
		TotalSupply(BalanceOf<T, I>),
		/// Account balance for a specified token and owner.
		BalanceOf(BalanceOf<T, I>),
		/// Allowance for a spender approved by an owner, for a specified token.
		Allowance(BalanceOf<T, I>),
		/// Name of the specified token, if available.
		TokenName(Option<Vec<u8>>),
		/// Symbol for the specified token, if available.
		TokenSymbol(Option<Vec<u8>>),
		/// Decimals for the specified token.
		TokenDecimals(u8),
		/// Whether the specified token exists.
		TokenExists(bool),
	}

	impl<T: Config<I>, I: 'static> ReadResult<T, I> {
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

	impl<T: Config<I>, I: 'static> crate::Read for Pallet<T, I> {
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
				TotalSupply(_) => <T as Config<I>>::WeightInfo::total_supply(),
				BalanceOf { .. } => <T as Config<I>>::WeightInfo::balance_of(),
				Allowance { .. } => <T as Config<I>>::WeightInfo::allowance(),
				TokenName(_) => <T as Config<I>>::WeightInfo::token_name(),
				TokenSymbol(_) => <T as Config<I>>::WeightInfo::token_symbol(),
				TokenDecimals(_) => <T as Config<I>>::WeightInfo::token_decimals(),
				TokenExists(_) => <T as Config<I>>::WeightInfo::token_exists(),
			}
		}

		/// Performs the requested read and returns the result.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn read(request: Self::Read) -> Self::Result {
			use Read::*;
			match request {
				TotalSupply(token) =>
					ReadResult::TotalSupply(AssetsOf::<T, I>::total_supply(token)),
				BalanceOf { token, owner } =>
					ReadResult::BalanceOf(AssetsOf::<T, I>::balance(token, owner)),
				Allowance { token, owner, spender } =>
					ReadResult::Allowance(AssetsOf::<T, I>::allowance(token, &owner, &spender)),
				TokenName(token) => ReadResult::TokenName(
					Some(<AssetsOf<T, I> as MetadataInspect<AccountIdOf<T>>>::name(token))
						.filter(|v| !v.is_empty()),
				),
				TokenSymbol(token) => ReadResult::TokenSymbol(
					Some(<AssetsOf<T, I> as MetadataInspect<AccountIdOf<T>>>::symbol(token))
						.filter(|v| !v.is_empty()),
				),
				TokenDecimals(token) => ReadResult::TokenDecimals(
					<AssetsOf<T, I> as MetadataInspect<AccountIdOf<T>>>::decimals(token),
				),
				TokenExists(token) =>
					ReadResult::TokenExists(AssetsOf::<T, I>::asset_exists(token)),
			}
		}
	}
}

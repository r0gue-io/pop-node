#![cfg_attr(not(feature = "std"), no_std)]

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
	use super::*;
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
	use core::marker::PhantomData;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	/// The pallet is instantiable via the generic parameter `I` (defaulting to `()`),
	/// and the associated type `AssetsInstance` determines the pallet-assets instance to use.
	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_assets::Config<Self::AssetsInstance> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
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
		/// Event emitted when an allowance change occurs.
		Approval {
			token: TokenIdOf<T, I>,
			owner: AccountIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T, I>,
		},
		/// Event emitted when a token transfer occurs.
		Transfer {
			token: TokenIdOf<T, I>,
			from: Option<AccountIdOf<T>>,
			to: Option<AccountIdOf<T>>,
			value: BalanceOf<T, I>,
		},
		/// Event emitted when a token is created.
		Created {
			id: TokenIdOf<T, I>,
			creator: AccountIdOf<T>,
			admin: AccountIdOf<T>,
		},
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Transfers `value` tokens from the caller's account to account `to`.
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
			Self::deposit_event(Event::Transfer {
				token,
				from: Some(from),
				to: Some(to),
				value,
			});
			Ok(())
		}

		/// Transfers tokens on behalf of `from` to account `to`.
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
			Self::deposit_event(Event::Transfer {
				token,
				from: Some(from),
				to: Some(to),
				value,
			});
			Ok(())
		}

		/// Approves `spender` to spend `value` tokens on behalf of the caller.
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
			let current_allowance =
				AssetsOf::<T, I>::allowance(token.clone(), &owner, &spender);

			let weight = match value.cmp(&current_allowance) {
				Equal => WeightOf::<T, I>::approve(0, 0),
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
			Self::deposit_event(Event::Approval {
				token,
				owner,
				spender,
				value,
			});
			Ok(Some(weight).into())
		}

		/// Increases the allowance of `spender` by `value` tokens.
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
			Self::deposit_event(Event::Approval {
				token,
				owner,
				spender,
				value,
			});
			Ok(().into())
		}

		/// Decreases the allowance of `spender` by `value` tokens.
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
			let current_allowance =
				AssetsOf::<T, I>::allowance(token.clone(), &owner, &spender);
			let spender_source = T::Lookup::unlookup(spender.clone());
			let token_param: TokenIdParameterOf<T, I> = token.clone().into();

			let new_allowance = current_allowance
				.checked_sub(&value)
				.ok_or(AssetsErrorOf::<T, I>::Unapproved)?;
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
			Self::deposit_event(Event::Approval {
				token,
				owner,
				spender,
				value: new_allowance,
			});
			Ok(Some(weight).into())
		}

		/// Create a new token.
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
			Self::deposit_event(Event::Created {
				id,
				creator,
				admin,
			});
			Ok(())
		}

		/// Begin destroying a token.
		#[pallet::call_index(12)]
		#[pallet::weight(AssetsWeightInfoOf::<T, I>::start_destroy())]
		pub fn start_destroy(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
		) -> DispatchResult {
			AssetsOf::<T, I>::start_destroy(origin, token.into())
		}

		/// Set the metadata for a token.
		#[pallet::call_index(16)]
		#[pallet::weight(AssetsWeightInfoOf::<T, I>::set_metadata(name.len() as u32, symbol.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			AssetsOf::<T, I>::set_metadata(
				origin,
				token.into(),
				name,
				symbol,
				decimals,
			)
		}

		/// Clear the metadata for a token.
		#[pallet::call_index(17)]
		#[pallet::weight(AssetsWeightInfoOf::<T, I>::clear_metadata())]
		pub fn clear_metadata(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
		) -> DispatchResult {
			AssetsOf::<T, I>::clear_metadata(origin, token.into())
		}

		/// Mint new tokens.
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
			Self::deposit_event(Event::Transfer {
				token,
				from: None,
				to: Some(account),
				value,
			});
			Ok(())
		}

		/// Burn tokens.
		#[pallet::call_index(20)]
		#[pallet::weight(<T as Config<I>>::WeightInfo::balance_of() + AssetsWeightInfoOf::<T, I>::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			token: TokenIdOf<T, I>,
			account: AccountIdOf<T>,
			value: BalanceOf<T, I>,
		) -> DispatchResultWithPostInfo {
			let current_balance =
				AssetsOf::<T, I>::balance(token.clone(), &account);
			if current_balance < value {
				return Err(
					AssetsErrorOf::<T, I>::BalanceLow
						.with_weight(<T as Config<I>>::WeightInfo::balance_of())
				);
			}
			AssetsOf::<T, I>::burn(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(account.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer {
				token,
				from: Some(account),
				to: None,
				value,
			});
			Ok(().into())
		}
	}

	/// State-read requests for the fungibles API.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config<I>, I: 'static = ()> {
		#[codec(index = 0)]
		TotalSupply(TokenIdOf<T, I>),
		#[codec(index = 1)]
		BalanceOf {
			token: TokenIdOf<T, I>,
			owner: AccountIdOf<T>,
		},
		#[codec(index = 2)]
		Allowance {
			token: TokenIdOf<T, I>,
			owner: AccountIdOf<T>,
			spender: AccountIdOf<T>,
		},
		#[codec(index = 8)]
		TokenName(TokenIdOf<T, I>),
		#[codec(index = 9)]
		TokenSymbol(TokenIdOf<T, I>),
		#[codec(index = 10)]
		TokenDecimals(TokenIdOf<T, I>),
		#[codec(index = 18)]
		TokenExists(TokenIdOf<T, I>),
	}

	/// Results of state-read requests.
	#[derive(Debug)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	pub enum ReadResult<T: Config<I>, I: 'static = ()> {
		TotalSupply(BalanceOf<T, I>),
		BalanceOf(BalanceOf<T, I>),
		Allowance(BalanceOf<T, I>),
		TokenName(Option<Vec<u8>>),
		TokenSymbol(Option<Vec<u8>>),
		TokenDecimals(u8),
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
		type Read = Read<T, I>;
		type Result = ReadResult<T, I>;

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

		fn read(request: Self::Read) -> Self::Result {
			use Read::*;
			match request {
				TotalSupply(token) => ReadResult::TotalSupply(AssetsOf::<T, I>::total_supply(token)),
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
				TokenExists(token) => ReadResult::TokenExists(AssetsOf::<T, I>::asset_exists(token)),
			}
		}
	}
}

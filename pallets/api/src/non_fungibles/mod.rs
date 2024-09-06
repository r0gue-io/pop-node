//! The non-fungibles pallet offers a streamlined interface for interacting with non-fungible assets. The
//! goal is to provide a simplified, consistent API that adheres to standards in the smart contract
//! space.

use frame_support::traits::nonfungibles_v2::InspectEnumerable;
pub use pallet::*;
use pallet_nfts::WeightInfo;
use sp_runtime::traits::StaticLookup;

#[cfg(test)]
mod tests;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use types::{AccountIdOf, CollectionIdOf, ItemDetails, ItemIdOf, NftsOf, NftsWeightInfoOf};

	/// State reads for the fungibles API with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config> {
		/// Returns the owner of an item.
		#[codec(index = 0)]
		OwnerOf { collection: CollectionIdOf<T>, item: ItemIdOf<T> },
		/// Returns the owner of a collection.
		#[codec(index = 1)]
		CollectionOwner(CollectionIdOf<T>),
		/// Number of items existing in a concrete collection.
		#[codec(index = 2)]
		TotalSupply(CollectionIdOf<T>),
		/// Returns the total number of items in the collection owned by the account.
		#[codec(index = 3)]
		BalanceOf { collection: CollectionIdOf<T>, owner: AccountIdOf<T> },
		/// Returns the details of a collection.
		#[codec(index = 4)]
		Collection(CollectionIdOf<T>),
		/// Returns the details of an item.
		#[codec(index = 5)]
		Item { collection: CollectionIdOf<T>, item: ItemIdOf<T> },
		/// Whether a spender is allowed to transfer an item or items from owner.
		#[codec(index = 6)]
		Allowance { spender: AccountIdOf<T>, collection: CollectionIdOf<T>, item: ItemIdOf<T> },
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nfts::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when allowance by `owner` to `spender` canceled.
		CancelApproval {
			/// The collection ID.
			collection: CollectionIdOf<T>,
			/// the item ID.
			item: ItemIdOf<T>,
			/// The beneficiary of the allowance.
			spender: AccountIdOf<T>,
		},
		/// Event emitted when allowance by `owner` to `spender` changes.
		Approval {
			/// The collection ID.
			collection: CollectionIdOf<T>,
			/// the item ID.
			item: ItemIdOf<T>,
			/// The owner providing the allowance.
			owner: AccountIdOf<T>,
			/// The beneficiary of the allowance.
			spender: AccountIdOf<T>,
		},
		/// Event emitted when new item is minted to the account.
		Mint {
			/// The owner of the item.
			to: AccountIdOf<T>,
			/// The collection ID.
			collection: CollectionIdOf<T>,
			/// the item ID.
			item: ItemIdOf<T>,
		},
		/// Event emitted when item is burned.
		Burn {
			/// The collection ID.
			collection: CollectionIdOf<T>,
			/// the item ID.
			item: ItemIdOf<T>,
		},
		/// Event emitted when an item transfer occurs.
		Transfer {
			/// The collection ID.
			collection: CollectionIdOf<T>,
			/// the item ID.
			item: ItemIdOf<T>,
			/// The source of the transfer.
			from: AccountIdOf<T>,
			/// The recipient of the transfer.
			to: AccountIdOf<T>,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new non-fungible token to the collection.
		#[pallet::call_index(0)]
		#[pallet::weight(NftsWeightInfoOf::<T>::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			to: AccountIdOf<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			NftsOf::<T>::mint(origin, collection, item, T::Lookup::unlookup(to.clone()), None)?;
			Self::deposit_event(Event::Mint { to, collection, item });
			Ok(())
		}

		/// Destroy a new non-fungible token to the collection.
		#[pallet::call_index(1)]
		#[pallet::weight(NftsWeightInfoOf::<T>::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			NftsOf::<T>::burn(origin, collection, item)?;
			Self::deposit_event(Event::Burn { collection, item });
			Ok(())
		}

		/// Transfer a token from one account to the another account.
		#[pallet::call_index(2)]
		#[pallet::weight(NftsWeightInfoOf::<T>::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			to: AccountIdOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			NftsOf::<T>::transfer(origin, collection, item, T::Lookup::unlookup(to.clone()))?;
			Self::deposit_event(Event::Transfer { from, to, collection, item });
			Ok(())
		}

		/// Delegate a permission to perform actions on the collection item to an account.
		#[pallet::call_index(3)]
		#[pallet::weight(NftsWeightInfoOf::<T>::approve_transfer())]
		pub fn approve(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			spender: AccountIdOf<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin.clone())?;
			NftsOf::<T>::approve_transfer(
				origin,
				collection,
				item,
				T::Lookup::unlookup(spender.clone()),
				None,
			)?;
			Self::deposit_event(Event::Approval { collection, item, spender, owner });
			Ok(())
		}

		/// Cancel one of the transfer approvals for a specific item.
		#[pallet::call_index(4)]
		#[pallet::weight(NftsWeightInfoOf::<T>::cancel_approval())]
		pub fn cancel_approval(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			spender: AccountIdOf<T>,
		) -> DispatchResult {
			NftsOf::<T>::cancel_approval(
				origin,
				collection,
				item,
				T::Lookup::unlookup(spender.clone()),
			)?;
			Self::deposit_event(Event::CancelApproval { collection, item, spender });
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
				OwnerOf { collection, item } => NftsOf::<T>::owner(collection, item).encode(),
				CollectionOwner(collection) => NftsOf::<T>::collection_owner(collection).encode(),
				TotalSupply(collection) => (NftsOf::<T>::items(&collection).count() as u8).encode(),
				Collection(collection) => pallet_nfts::Collection::<T>::get(&collection).encode(),
				Item { collection, item } => {
					pallet_nfts::Item::<T>::get(&collection, &item).encode()
				},
				Allowance { collection, item, spender } => {
					Self::allowance(collection, item, spender).encode()
				},
				BalanceOf { collection, owner } => {
					(NftsOf::<T>::owned_in_collection(&collection, &owner).count() as u8).encode()
				},
			}
		}

		/// Check if the `spender` is approved to transfer the collection item
		pub(crate) fn allowance(
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			spender: AccountIdOf<T>,
		) -> bool {
			let data = pallet_nfts::Item::<T>::get(&collection, &item).encode();
			if let Ok(detail) = ItemDetails::<T>::decode(&mut data.as_slice()) {
				return detail.approvals.contains_key(&spender);
			}
			false
		}
	}
}

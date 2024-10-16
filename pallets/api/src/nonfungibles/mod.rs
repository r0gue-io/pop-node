//! The non-fungibles pallet offers a streamlined interface for interacting with non-fungible
//! assets. The goal is to provide a simplified, consistent API that adheres to standards in the
//! smart contract space.

pub use pallet::*;
use pallet_nfts::WeightInfo;
use sp_runtime::traits::StaticLookup;

#[cfg(test)]
mod tests;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use types::{
		AccountIdOf, CollectionDetailsFor, CollectionIdOf, ItemDetailsFor, ItemIdOf, NftsOf,
		NftsWeightInfoOf,
	};

	use super::*;

	/// State reads for the non-fungibles API with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
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
		Allowance {
			collection: CollectionIdOf<T>,
			owner: AccountIdOf<T>,
			operator: AccountIdOf<T>,
			item: Option<ItemIdOf<T>>,
		},
	}

	/// Results of state reads for the non-fungibles API.
	#[derive(Debug)]
	#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
	pub enum ReadResult<T: Config> {
		OwnerOf(Option<AccountIdOf<T>>),
		CollectionOwner(Option<AccountIdOf<T>>),
		TotalSupply(u128),
		BalanceOf(u32),
		Collection(Option<CollectionDetailsFor<T>>),
		Item(Option<ItemDetailsFor<T>>),
		Allowance(bool),
	}

	impl<T: Config> ReadResult<T> {
		/// Encodes the result.
		pub fn encode(&self) -> Vec<u8> {
			use ReadResult::*;
			match self {
				OwnerOf(result) => result.encode(),
				CollectionOwner(result) => result.encode(),
				TotalSupply(result) => result.encode(),
				BalanceOf(result) => result.encode(),
				Collection(result) => result.encode(),
				Item(result) => result.encode(),
				Allowance(result) => result.encode(),
			}
		}
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
		/// Event emitted when allowance by `owner` to `operator` changes.
		Approval {
			/// The collection ID.
			collection: CollectionIdOf<T>,
			/// The owner providing the allowance.
			owner: AccountIdOf<T>,
			/// The beneficiary of the allowance.
			operator: AccountIdOf<T>,
			/// The item which is (dis)approved. `None` for all owner's items.
			item: Option<ItemIdOf<T>>,
			/// Whether allowance is set or removed.
			approved: bool,
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

		#[pallet::call_index(3)]
		#[pallet::weight(NftsWeightInfoOf::<T>::approve_transfer())]
		pub fn approve(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: Option<ItemIdOf<T>>,
			operator: AccountIdOf<T>,
			approved: bool,
		) -> DispatchResult {
			let owner = ensure_signed(origin.clone())?;
			if approved {
				NftsOf::<T>::approve_transfer(
					origin,
					collection,
					item,
					T::Lookup::unlookup(operator.clone()),
					None,
				)?;
			} else {
				NftsOf::<T>::cancel_approval(
					origin,
					collection,
					item,
					T::Lookup::unlookup(operator.clone()),
				)?;
			}
			Self::deposit_event(Event::Approval { collection, item, operator, owner, approved });
			Ok(())
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
		fn weight(_request: &Self::Read) -> Weight {
			Default::default()
		}

		/// Performs the requested read and returns the result.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn read(value: Self::Read) -> Self::Result {
			use Read::*;
			match value {
				OwnerOf { collection, item } =>
					ReadResult::OwnerOf(NftsOf::<T>::owner(collection, item)),
				CollectionOwner(collection) =>
					ReadResult::CollectionOwner(NftsOf::<T>::collection_owner(collection)),
				TotalSupply(collection) => ReadResult::TotalSupply(
					NftsOf::<T>::collection_items(collection).unwrap_or_default().into(),
				),
				Collection(collection) =>
					ReadResult::Collection(pallet_nfts::Collection::<T>::get(collection)),
				Item { collection, item } =>
					ReadResult::Item(pallet_nfts::Item::<T>::get(collection, item)),
				Allowance { collection, owner, operator, item } => ReadResult::Allowance(
					NftsOf::<T>::check_allowance(&collection, &item, &owner, &operator).is_ok(),
				),
				BalanceOf { collection, owner } => ReadResult::BalanceOf(
					pallet_nfts::AccountBalance::<T>::get((collection, owner)),
				),
			}
		}
	}
}

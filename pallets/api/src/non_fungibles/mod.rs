//! The fungibles pallet offers a streamlined interface for interacting with fungible assets. The
//! goal is to provide a simplified, consistent API that adheres to standards in the smart contract
//! space.

use frame_support::traits::nonfungibles_v2::{Inspect, InspectEnumerable};
pub use pallet::*;
use pallet_nfts::WeightInfo;
use sp_runtime::traits::StaticLookup;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type NonFungiblesInstanceOf<T> = <T as Config>::NonFungiblesInstance;
type NonFungiblesOf<T> = pallet_nfts::Pallet<T, NonFungiblesInstanceOf<T>>;
type NonFungiblesWeightInfoOf<T> =
	<T as pallet_nfts::Config<NonFungiblesInstanceOf<T>>>::WeightInfo;
type CollectionIdOf<T> = <pallet_nfts::Pallet<T, NonFungiblesInstanceOf<T>> as Inspect<
	<T as frame_system::Config>::AccountId,
>>::CollectionId;
type ItemIdOf<T> = <pallet_nfts::Pallet<T, NonFungiblesInstanceOf<T>> as Inspect<
	<T as frame_system::Config>::AccountId,
>>::ItemId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

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
		/// Whether an operator is allowed to transfer an item or items from owner.
		#[codec(index = 2)]
		Allowance {
			operator: AccountIdOf<T>,
			collection: CollectionIdOf<T>,
			maybe_item: Option<ItemIdOf<T>>,
		},
		/// Number of items existing in a concrete collection.
		#[codec(index = 3)]
		TotalSupply(CollectionIdOf<T>),
		/// Returns the details of a collection.
		#[codec(index = 4)]
		Collection(CollectionIdOf<T>),
		/// Returns the details of an item.
		#[codec(index = 5)]
		Item { collection: CollectionIdOf<T>, item: ItemIdOf<T> },
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_nfts::Config<Self::NonFungiblesInstance>
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The instance of pallet nfts it is tightly coupled to.
		type NonFungiblesInstance;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new non-fungible token to the collection.
		#[pallet::call_index(0)]
		#[pallet::weight(NonFungiblesWeightInfoOf::<T>::mint())]
		pub fn mint_to(
			origin: OriginFor<T>,
			to: AccountIdOf<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			NonFungiblesOf::<T>::mint(
				origin,
				collection,
				item,
				T::Lookup::unlookup(to.clone()),
				None,
			)?;
			Ok(())
		}

		/// Destroy a new non-fungible token to the collection.
		#[pallet::call_index(1)]
		#[pallet::weight(NonFungiblesWeightInfoOf::<T>::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
		) -> DispatchResult {
			NonFungiblesOf::<T>::burn(origin, collection, item)?;
			Ok(())
		}

		/// Transfer a token from one account to the another account.
		#[pallet::call_index(2)]
		#[pallet::weight(NonFungiblesWeightInfoOf::<T>::transfer())]
		pub fn transfer(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			dest: AccountIdOf<T>,
		) -> DispatchResult {
			NonFungiblesOf::<T>::transfer(
				origin,
				collection,
				item,
				T::Lookup::unlookup(dest.clone()),
			)?;
			Ok(())
		}

		/// Delegate a permission to perform actions on the collection item to an account.
		#[pallet::call_index(3)]
		#[pallet::weight(NonFungiblesWeightInfoOf::<T>::approve_transfer())]
		pub fn approve(
			origin: OriginFor<T>,
			collection: CollectionIdOf<T>,
			item: ItemIdOf<T>,
			delegate: AccountIdOf<T>,
		) -> DispatchResult {
			NonFungiblesOf::<T>::approve_transfer(
				origin,
				collection,
				item,
				T::Lookup::unlookup(delegate.clone()),
				None,
			)?;
			Ok(())
		}
	}

	/// Cancel one of the transfer approvals for a specific item.
	#[pallet::call_index(4)]
	#[pallet::weight(NonFungiblesWeightInfoOf::<T>::cancel_approval())]
	pub fn cancel_approval(
		origin: OriginFor<T>,
		collection: CollectionIdOf<T>,
		item: ItemIdOf<T>,
		delegate: AccountIdOf<T>,
	) -> DispatchResult {
		NonFungiblesOf::<T>::cancel_approval(
			origin,
			collection,
			item,
			T::Lookup::unlookup(delegate.clone()),
		)?;
		Ok(())
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
				OwnerOf { collection, item } => {
					NonFungiblesOf::<T>::owner(collection, item).encode()
				},
				CollectionOwner(collection) => {
					NonFungiblesOf::<T>::collection_owner(collection).encode()
				},
				TotalSupply(collection) => {
					(NonFungiblesOf::<T>::items(&collection).count() as u8).encode()
				},
				Collection(collection) => {
					pallet_nfts::Collection::<T, T::NonFungiblesInstance>::get(&collection).encode()
				},
				Item { collection, item } => {
					pallet_nfts::Item::<T, T::NonFungiblesInstance>::get(&collection, &item)
						.encode()
				},
				// TODO: approvals field of the nft item is set to be private in the pallet_nfts
				Allowance { operator, collection, maybe_item } => todo!(),
			}
		}
	}
}

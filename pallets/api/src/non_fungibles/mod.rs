//! The fungibles pallet offers a streamlined interface for interacting with fungible assets. The
//! goal is to provide a simplified, consistent API that adheres to standards in the smart contract
//! space.

pub use pallet::*;

use frame_support::traits::nonfungibles_v2::Inspect;
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
		#[codec(index = 0)]
		Dummy(AccountIdOf<T>),
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
		#[pallet::weight(Weight::default())]
		pub fn approve(_origin: OriginFor<T>) -> DispatchResult {
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
			vec![]
		}
	}
}

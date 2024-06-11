#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResultWithPostInfo, pallet_prelude::*, sp_runtime::Saturating,
};
use frame_system::pallet_prelude::*;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type QueryId = u64;

#[derive(MaxEncodedLen, Clone, Encode, Decode, Debug, Eq, PartialEq, Ord, PartialOrd, TypeInfo)]
pub enum QueryType {
	ISMP(QueryId),
	XCM(QueryId),
}

// TODO: adjust once XCM and ISMP are integrated
#[derive(Debug, PartialEq)]
pub struct Response {
	data: Vec<u8>,
}

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet]
pub mod pallet {
	pub use super::{AccountIdOf, QueryId, QueryType, *};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: crate::weights::WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// A global query ID counter. Used for mapping queries to the proper query module (ISMP, XCM, etc.)
	pub(super) type GlobalQueryCounter<T: Config> = StorageValue<_, QueryId, ValueQuery>;

	#[pallet::storage]
	pub(super) type Queries<T> =
		StorageMap<_, Blake2_128Concat, QueryId, (QueryType, AccountIdOf<T>), OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Only requester address may remove query from storage.
		BadOrigin,
		// TODO: proper errors
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn register_query(origin: OriginFor<T>, module_query_id: QueryType) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// get the current global query index, then increment
			let query_id = GlobalQueryCounter::<T>::mutate(|q| {
				let r = *q;
				q.saturating_inc();
				r
			});

			Queries::<T>::insert(query_id, (module_query_id, who));

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(1,1))]
		pub fn poll_query(origin: OriginFor<T>, query_id: QueryId) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let module_query = Queries::<T>::get(query_id).ok_or(Error::<T>::NoneValue)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn route_query(query: QueryType) -> Response {
		match query {
			QueryType::ISMP(query_id) => {
				// route to ISMP
				Response { data: vec![0] }
			},
			QueryType::XCM(query_id) => {
				// route to XCM
				Response { data: vec![1] }
			},
		}
	}

	fn take_response(origin: OriginFor<T>, query_id: QueryId) -> Result<Response, DispatchError> {
		let who = ensure_signed(origin)?;
		let query = Queries::<T>::try_mutate(
			query_id,
			|maybe_query| -> Result<(QueryType, AccountIdOf<T>), DispatchError> {
				let query = maybe_query.take().ok_or(Error::<T>::NoneValue)?;
				if query.1.ne(&who) {
					return Err(Error::<T>::BadOrigin.into());
				}

				Ok(query)
			},
		)?;
		let response = Self::route_query(query.0);
		Ok(response)
	}
}

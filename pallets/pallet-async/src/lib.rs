#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	dispatch::DispatchResultWithPostInfo, pallet_prelude::*, sp_runtime::Saturating,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use sp_runtime::traits::{Header, HeaderProvider, Zero};
use xcm::latest::QueryId;
use xcm_executor::traits::{QueryHandler, QueryResponseStatus};

use parachains_common::{AccountId, AuraId, Balance, Block, Hash, Signature};

#[cfg(test)]
mod mock;
#[cfg(test)]
#[cfg(test)]
mod tests;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(MaxEncodedLen, Clone, Encode, Decode, Debug, Eq, PartialEq, Ord, PartialOrd, TypeInfo)]
pub enum QueryType<QueryId> {
	Ismp,
	Xcm(QueryId),
}

// TODO: should be available to contracts
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
pub enum ResponseStatus {
	Ready,
	Pending,
	NotFound,
	Error,
}

// TODO: adjust once XCM and ISMP are integrated
#[derive(Debug, PartialEq)]
pub enum Response<BlockNumber> {
	Ismp(u8), // TODO: u8 is placeholder
	Xcm(QueryResponseStatus<BlockNumber>),
}

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet]
pub mod pallet {
	use super::{AccountIdOf, QueryId, QueryType, *};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type XcmQueryHandler: QueryHandler;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: weights::WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	/// A global query ID counter. Used for mapping queries to the proper query module (ISMP, XCM, etc.)
	pub(super) type GlobalQueryCounter<T: Config> = StorageValue<_, QueryId, ValueQuery>;

	#[pallet::storage]
	pub(super) type Queries<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		QueryId,
		(QueryType<<T::XcmQueryHandler as QueryHandler>::QueryId>, AccountIdOf<T>),
		OptionQuery,
	>;

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
		pub fn register_query(
			origin: OriginFor<T>,
			module_query_id: QueryType<<T::XcmQueryHandler as QueryHandler>::QueryId>,
		) -> DispatchResult {
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
	fn route_query(
		query: &QueryType<<T::XcmQueryHandler as QueryHandler>::QueryId>,
	) -> Response<<T::XcmQueryHandler as QueryHandler>::BlockNumber> {
		match query {
			QueryType::Ismp => {
				// route to ISMP
				Response::Ismp(0)
			},
			QueryType::Xcm(query_id) => {
				// route to XCM
				// TODO: can probably pull from XCM storage directly before using `take_response`
				// using the getter query()
				let response: QueryResponseStatus<
					<T::XcmQueryHandler as QueryHandler>::BlockNumber,
				> = T::XcmQueryHandler::take_response(*query_id);

				Response::Xcm(response)
			},
		}
	}

	fn take_response(
		origin: OriginFor<T>,
		query_id: QueryId,
	) -> Result<Response<<T::XcmQueryHandler as QueryHandler>::BlockNumber>, DispatchError> {
		let who = ensure_signed(origin)?;
		Ok(Queries::<T>::try_mutate(
			query_id,
			|maybe_query| -> Result<
				Response<<T::XcmQueryHandler as QueryHandler>::BlockNumber>,
				DispatchError,
			> {
				let query = maybe_query.as_mut().ok_or(Error::<T>::NoneValue)?;
				// only requesting origin can remove from storage
				if query.1.ne(&who) {
					return Err(Error::<T>::BadOrigin.into());
				}

				let response = Self::route_query(&query.0);
				// if response is Ready, then remove from storage
				match &response {
					Response::Ismp(_) => {},
					Response::Xcm(xcm_response) => match xcm_response {
						// set maybe_query to None, removing it from storage
						QueryResponseStatus::Ready { .. } => *maybe_query = None,
						_ => {},
					},
				}
				Ok(response)
			},
		)?)
	}
}

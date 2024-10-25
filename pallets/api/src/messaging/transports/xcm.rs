use frame_support::pallet_prelude::Weight;
use xcm::latest::{Response, XcmContext};
pub(crate) use xcm::{
	latest::{Location, QueryId},
	VersionedLocation, VersionedResponse,
};
use xcm_executor::traits::OnResponse;
pub(crate) use xcm_executor::traits::QueryHandler;

use crate::messaging::{
	pallet::{Messages, XcmQueries},
	Config, Event, Pallet,
};

impl<T: Config> OnResponse for Pallet<T> {
	// todo: check origin and querier
	fn expecting_response(_origin: &Location, query_id: u64, _querier: Option<&Location>) -> bool {
		// todo: weight?
		XcmQueries::<T>::contains_key(query_id)
	}

	// todo: check origin and querier
	fn on_response(
		_origin: &Location,
		query_id: u64,
		_querier: Option<&Location>,
		response: Response,
		_max_weight: Weight,
		_context: &XcmContext,
	) -> Weight {
		let (origin, id) = XcmQueries::<T>::get(query_id).unwrap(); // TODO: handle error

		Messages::<T>::mutate(&origin, &id, |message| {
			let Some(super::super::Message::XcmQuery { query_id, deposit }) = message else {
				panic!() // TODO: handle error
			};
			*message = Some(super::super::Message::XcmResponse {
				deposit: *deposit,
				query_id: *query_id,
				// TODO: remove this in favour of using the data stored in the xcm-pallet until
				// taken.
				response: VersionedResponse::from(response),
			});
		});
		Pallet::<T>::deposit_event(Event::<T>::XcmResponseReceived { dest: origin, id });
		// todo: weight
		Weight::zero()
	}
}

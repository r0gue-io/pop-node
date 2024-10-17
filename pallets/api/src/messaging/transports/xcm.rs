use codec::Encode;
use frame_support::pallet_prelude::Weight;
use sp_runtime::BoundedVec;
pub(crate) use xcm::{
	latest::{Location, QueryId},
	VersionedLocation,
};
use xcm::{
	latest::{Response, XcmContext},
	VersionedResponse,
};
use xcm_executor::traits::OnResponse;
pub(crate) use xcm_executor::traits::QueryHandler;

use crate::messaging::{
	pallet::{Requests, Responses, XcmRequests},
	Config, Event, Pallet, Status,
};

impl<T: Config> OnResponse for Pallet<T> {
	// todo: check origin and querier
	fn expecting_response(_origin: &Location, query_id: u64, _querier: Option<&Location>) -> bool {
		// todo: weight?
		XcmRequests::<T>::contains_key(query_id)
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
		let (requestor, id) = XcmRequests::<T>::get(query_id).unwrap(); // TODO: handle error

		// TODO: remove this in favour of using the data stored in the xcm-pallet until
		// taken.
		let response: BoundedVec<u8, T::MaxResponseLen> =
			VersionedResponse::from(response).encode().try_into().unwrap(); // TODO: handle error
		Requests::<T>::mutate(&requestor, &id, |v| {
			let Some((status, ..)) = v else { panic!() }; // TODO: handle error
			*status = Status::Complete;
		});
		Responses::<T>::insert(&requestor, &id, response);
		Pallet::<T>::deposit_event(Event::<T>::ResponseReceived { requestor, id });
		// todo: weight
		Weight::zero()
	}
}

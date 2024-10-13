use ::xcm::latest::{Junction::AccountId32, Location, Response, XcmContext};
use codec::Encode;
use frame_support::pallet_prelude::Weight;
use sp_runtime::BoundedVec;
use xcm::VersionedResponse;
use xcm_executor::traits::OnResponse;

use super::pallet::{Config, Event, Pallet, Requests, Responses};
use crate::cross_chain::Status;

impl<T: Config<AccountId: From<[u8; 32]>>> OnResponse for Pallet<T> {
	fn expecting_response(_origin: &Location, query_id: u64, querier: Option<&Location>) -> bool {
		// todo: weight?
		match querier.map(|l| l.unpack()) {
			Some((0, [AccountId32 { id, .. }])) => {
				// todo: check origin
				let id: T::AccountId = id.clone().into();
				Requests::<T>::contains_key(id, query_id)
			},
			_ => false,
		}
	}

	fn on_response(
		_origin: &Location,
		query_id: u64,
		querier: Option<&Location>,
		response: Response,
		_max_weight: Weight,
		_context: &XcmContext,
	) -> Weight {
		match querier.map(|l| l.unpack()) {
			Some((0, [AccountId32 { id, .. }])) => {
				let id = (id.clone().into(), query_id);
				// Store values for later retrieval
				let response: BoundedVec<u8, T::MaxResponseLen> =
					VersionedResponse::from(response).encode().try_into().unwrap(); // TODO: handle error
				Requests::<T>::mutate(&id.0, &id.1, |v| {
					let Some((status, ..)) = v else { panic!() }; // TODO: handle error
					*status = Status::Complete;
				});
				Responses::<T>::insert(&id.0, &id.1, response);
				Pallet::<T>::deposit_event(Event::<T>::ResponseReceived { id });
				// todo: weight
				Weight::zero()
			},
			_ => Weight::zero(),
		}
	}
}

pub use ink::{
	env::call::Selector,
	prelude::vec::Vec,
	xcm::prelude::{
		Junction, Junctions, Location, MaybeErrorCode, NetworkId, QueryId, Response,
		VersionedLocation, VersionedResponse, VersionedXcm, Weight, XcmContext, XcmHash,
	},
};
use ink::{
	env::{account_id, xcm_execute, xcm_send, DefaultEnvironment},
	scale::Encode,
};

use super::*;

/// Note: usage of a callback requires implementation of the [OnResponse] trait.
#[inline]
pub fn new_query(
	id: MessageId,
	responder: Location,
	timeout: BlockNumber,
	callback: Option<Callback>,
) -> Result<Option<QueryId>> {
	build_dispatch(XCM_NEW_QUERY)
		.input::<(MessageId, Location, BlockNumber, Option<Callback>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, responder, timeout, callback))?;

	build_read_state(QUERY_ID)
		.input::<(AccountId, MessageId)>()
		.output::<Result<Option<QueryId>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(account_id::<DefaultEnvironment>(), id))
}

/// Execute an XCM message locally, using the contract's address as the origin.
pub fn execute<Call: Encode>(msg: &VersionedXcm<Call>) -> ink::env::Result<()> {
	xcm_execute::<DefaultEnvironment, _>(msg)
}

/// Send an XCM message, using the contract's address as the origin.
pub fn send<Call: Encode>(
	dest: &VersionedLocation,
	msg: &VersionedXcm<Call>,
) -> ink::env::Result<XcmHash> {
	xcm_send::<DefaultEnvironment, _>(dest, msg)
}

#[ink::trait_definition]
pub trait OnResponse {
	// pop-api::messaging::xcm::OnResponse::on_response
	#[ink(message, selector = 0x641b0b03)]
	fn on_response(&mut self, id: MessageId, response: Response) -> Result<()>;
}

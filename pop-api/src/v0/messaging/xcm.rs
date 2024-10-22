pub use ink::xcm::prelude::{
	Junction, Junctions, Location, MaybeErrorCode, QueryId, Response, VersionedLocation,
	VersionedResponse, VersionedXcm, XcmContext, XcmHash,
};
use ink::{
	env::{account_id, xcm_execute, xcm_send, DefaultEnvironment},
	scale::Encode,
};

use super::*;
use crate::primitives::to_account_id;

#[inline]
pub fn new_query(
	id: RequestId,
	responder: VersionedLocation,
	timeout: BlockNumber,
) -> Result<Option<QueryId>> {
	build_dispatch(XCM_NEW_QUERY)
		.input::<(RequestId, VersionedLocation, BlockNumber)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, responder, timeout))?;

	build_read_state(QUERY_ID)
		.input::<(AccountId, RequestId)>()
		.output::<Result<Option<QueryId>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(to_account_id(&account_id::<DefaultEnvironment>()), id))
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

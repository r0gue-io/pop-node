use ink::env::{account_id, DefaultEnvironment};
pub use ink::xcm::prelude::{
	Junction, Junctions, Location, MaybeErrorCode, QueryId, Response, VersionedLocation,
	VersionedResponse, XcmContext, XcmHash,
};

use super::*;

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
		.call(&(account_id::<DefaultEnvironment>(), id))
}

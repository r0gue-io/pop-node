use ink::prelude::vec::Vec;

use crate::{
	primitives::{AccountId, Balance, BlockNumber},
	ChainExtensionMethodApi, Result, StatusCode,
};

/// APIs for messaging using the Interoperable State Machine Protocol (ISMP).
pub mod ismp;
/// APIs for messaging using Polkadot's Cross-Consensus Messaging (XCM).
pub mod xcm;

pub(crate) const API: u8 = 151;
// Dispatchables
pub(super) const _REQUEST: u8 = 0;
pub(super) const ISMP_GET: u8 = 1;
pub(super) const ISMP_POST: u8 = 2;
pub(super) const XCM_NEW_QUERY: u8 = 3;
pub(super) const REMOVE: u8 = 4;
// Reads
pub(super) const POLL: u8 = 0;
pub(super) const GET: u8 = 1;
pub(super) const QUERY_ID: u8 = 2;

pub type RequestId = u64;

fn build_dispatch(dispatchable: u8) -> ChainExtensionMethodApi {
	crate::v0::build_dispatch(API, dispatchable)
}

fn build_read_state(state_query: u8) -> ChainExtensionMethodApi {
	crate::v0::build_read_state(API, state_query)
}

#[inline]
pub fn poll(id: (AccountId, RequestId)) -> Result<Option<Status>> {
	build_read_state(POLL)
		.input::<(AccountId, RequestId)>()
		.output::<Result<Option<Status>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&id)
}

#[inline]
pub fn get(id: (AccountId, RequestId)) -> Result<Option<Vec<u8>>> {
	build_read_state(GET)
		.input::<(AccountId, RequestId)>()
		.output::<Result<Option<Vec<u8>>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&id)
}

#[inline]
pub fn remove(requests: Vec<RequestId>) -> Result<()> {
	build_dispatch(REMOVE)
		.input::<Vec<RequestId>>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&requests)
}

#[derive(PartialEq)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Status {
	Pending,
	TimedOut,
	Complete,
}

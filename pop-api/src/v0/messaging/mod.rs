use ink::prelude::vec::Vec;

use crate::{
	constants::MESSAGING,
	messaging::xcm::Weight,
	primitives::{AccountId, Balance, BlockNumber},
	ChainExtensionMethodApi, Result, StatusCode,
};

/// APIs for messaging using the Interoperable State Machine Protocol (ISMP).
pub mod ismp;
/// APIs for messaging using Polkadot's Cross-Consensus Messaging (XCM).
pub mod xcm;

// Dispatchables
pub(super) const _REQUEST: u8 = 0;
pub(super) const ISMP_GET: u8 = 1;
pub(super) const ISMP_POST: u8 = 2;
pub(super) const XCM_NEW_QUERY: u8 = 3;
pub(super) const _XCM_RESPONSE: u8 = 4;
pub(super) const REMOVE: u8 = 5;
// Reads
pub(super) const POLL: u8 = 0;
pub(super) const GET: u8 = 1;
pub(super) const QUERY_ID: u8 = 2;

pub type MessageId = u64;

fn build_dispatch(dispatchable: u8) -> ChainExtensionMethodApi {
	crate::v0::build_dispatch(MESSAGING, dispatchable)
}

fn build_read_state(state_query: u8) -> ChainExtensionMethodApi {
	crate::v0::build_read_state(MESSAGING, state_query)
}

#[inline]
pub fn poll(id: (AccountId, MessageId)) -> Result<Option<Status>> {
	build_read_state(POLL)
		.input::<(AccountId, MessageId)>()
		.output::<Result<Option<Status>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&id)
}

#[inline]
pub fn get(id: (AccountId, MessageId)) -> Result<Option<Vec<u8>>> {
	build_read_state(GET)
		.input::<(AccountId, MessageId)>()
		.output::<Result<Option<Vec<u8>>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&id)
}

#[inline]
pub fn remove(requests: Vec<MessageId>) -> Result<()> {
	build_dispatch(REMOVE)
		.input::<Vec<MessageId>>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&requests)
}

#[ink::scale_derive(Decode, Encode, TypeInfo)]
pub struct Callback {
	selector: [u8; 4],
	weight: Weight,
}

impl Callback {
	pub fn to(selector: u32, weight: Weight) -> Self {
		Self { selector: selector.to_be_bytes(), weight }
	}
}

#[derive(PartialEq)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Status {
	Pending,
	TimedOut,
	Complete,
}

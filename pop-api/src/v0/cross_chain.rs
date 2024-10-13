use ink::prelude::vec::Vec;

use crate::{
	primitives::{AccountId, Balance},
	ChainExtensionMethodApi, Result, StatusCode,
};

pub(crate) const API: u8 = 151;
// Dispatchables
pub(super) const REQUEST: u8 = 0;
pub(super) const REMOVE: u8 = 1;
// Reads
pub(super) const POLL: u8 = 0;
pub(super) const GET: u8 = 1;

pub type RequestId = u64;

fn build_dispatch(dispatchable: u8) -> ChainExtensionMethodApi {
	crate::v0::build_dispatch(API, dispatchable)
}

fn build_read_state(state_query: u8) -> ChainExtensionMethodApi {
	crate::v0::build_read_state(API, state_query)
}

#[inline]
pub fn request(request: Request) -> Result<()> {
	build_dispatch(REQUEST)
		.input::<Request>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&request)
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
pub fn remove(id: RequestId) -> Result<()> {
	build_dispatch(REMOVE)
		.input::<RequestId>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&id)
}

// TODO: eliminate enum
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Request {
	Ismp { id: RequestId, request: ismp::Request, fee: Balance },
	Xcm { id: RequestId },
}

pub mod ismp {
	use super::*;

	// TODO: eliminate enum
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub enum Request {
		Get { para: u32, height: u32, timeout: u64, context: Vec<u8>, keys: Vec<Vec<u8>> },
		Post { para: u32, timeout: u64, data: Vec<u8> },
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

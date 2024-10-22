//! The `nonfungibles` module provides an API for interacting and managing nonfungible tokens.
//!
//! The API includes the following interfaces:
//! 1. PSP-34
//! 2. PSP-34 Metadata
//! 3. PSP-22 Mintable & Burnable

use constants::*;
pub use errors::*;
pub use events::*;
use ink::prelude::vec::Vec;
pub use traits::*;
pub use types::*;

use crate::{
	primitives::{AccountId, Balance, TokenId},
	ChainExtensionMethodApi, Result, StatusCode,
};

pub mod errors;
pub mod events;
pub mod traits;
pub mod types;

/// Returns the total item supply for a specified collection.
///
/// # Parameters
/// - `collection` - The collection.
#[inline]
pub fn total_supply(collection: CollectionId) -> Result<Balance> {
	build_read_state(TOTAL_SUPPLY)
		.input::<TokenId>()
		.output::<Result<Balance>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(token))
}

/// Returns the account balance for a specified `token` and `owner`. Returns `0` if
/// the account is non-existent.
///
/// # Parameters
/// - `token` - The token.
/// - `owner` - The account whose balance is being queried.
#[inline]
pub fn balance_of(token: TokenId, owner: AccountId) -> Result<Balance> {
	build_read_state(BALANCE_OF)
		.input::<(TokenId, AccountId)>()
		.output::<Result<Balance>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(token, owner))
}

mod constants {
	/// 1. PSP-34
	pub(super) const TOTAL_SUPPLY: u8 = 0;
	pub(super) const BALANCE_OF: u8 = 1;
	pub(super) const ALLOWANCE: u8 = 2;
	pub(super) const TRANSFER: u8 = 3;
	pub(super) const APPROVE: u8 = 4;
	pub(super) const OWNER_OF: u8 = 5;

	/// 2. PSP-34 Metadata
	pub(super) const GET_ATTRIBUTE: u8 = 6;

	/// 3. Management
	pub(super) const CREATE: u8 = 7;
	pub(super) const DESTROY: u8 = 8;
	pub(super) const COLLECTION: u8 = 9;
	pub(super) const ITEM: u8 = 10;
	pub(super) const NEXT_COLLECTION_ID: u8 = 11;
	pub(super) const SET_ATTRIBUTE: u8 = 12;
	pub(super) const CLEAR_ATTRIBUTE: u8 = 13;
	pub(super) const SET_METADATA: u8 = 14;
	pub(super) const CLEAR_METADATA: u8 = 15;
	pub(super) const APPROVE_ITEM_ATTRIBUTE: u8 = 16;
	pub(super) const CANCEL_ITEM_ATTRIBUTES_APPROVAL: u8 = 17;
	pub(super) const SET_MAX_SUPPLY: u8 = 18;

	/// 4. PSP-34 Mintable & Burnable
	pub(super) const MINT: u8 = 19;
	pub(super) const BURN: u8 = 20;
}

// Helper method to build a dispatch call.
//
// Parameters:
// - 'dispatchable': The index of the dispatchable function within the module.
fn build_dispatch(dispatchable: u8) -> ChainExtensionMethodApi {
	crate::v0::build_dispatch(FUNGIBLES, dispatchable)
}

// Helper method to build a call to read state.
//
// Parameters:
// - 'state_query': The index of the runtime state query.
fn build_read_state(state_query: u8) -> ChainExtensionMethodApi {
	crate::v0::build_read_state(FUNGIBLES, state_query)
}

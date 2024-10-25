//! The `fungibles` module provides an API for interacting and managing fungible tokens.
//!
//! The API includes the following interfaces:
//! 1. PSP-22
//! 2. PSP-22 Metadata
//! 3. Management
//! 4. PSP-22 Mintable & Burnable

use constants::*;
pub use errors::*;
pub use traits::*;

use crate::{
	constants::SPONSORSHIPS, primitives::AccountId, ChainExtensionMethodApi, Result, StatusCode,
};

pub mod errors;
pub mod traits;

/// Registers a new sponsorship relation between the caller and an account.
///
/// # Parameters
/// - `beneficiary` - The account to be sponsored.
/// - `amount`: How much `beneficiary` is sponsored for.
#[inline]
pub fn sponsor_account(beneficiary: AccountId, amount: Balance) -> Result<()> {
	build_dispatch(SPONSOR_ACCOUNT)
		.input::<(AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(beneficiary))
}

/// Removes the sponsorship relation between caller and the given account if it exists.
///
/// # Parameters
/// - `beneficiary` - The account which will no longer be sponsored by caller.
#[inline]
pub fn remove_sponsorship_for(beneficiary: AccountId) -> Result<()> {
	build_dispatch(REMOVE_SPONSORSHIP)
		.input::<AccountId>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(account))
}

/// Set the value of an existing sponsorship to a given amount.
///
/// Parameters
/// - `beneficiary`: Account of the beneficiary.
/// - `new_amount`: The new amount for the sponsorship.
#[inline]
pub fn set_sponsorship_amount(beneficiary: AccountId, amount: Balance) -> Result<()> {
	build_dispatch(SET_SPONSORSHIP_AMOUNT)
		.input::<(AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(beneficiary))
}

mod constants {
	///
	pub(super) const SPONSOR_ACCOUNT: u8 = 0;
	pub(super) const REMOVE_SPONSORSHIP: u8 = 1;
	pub(super) const SET_SPONSORSHIP_AMOUNT: u8 =2;
}

// Helper method to build a dispatch call.
//
// Parameters:
// - 'dispatchable': The index of the dispatchable function within the module.
fn build_dispatch(dispatchable: u8) -> ChainExtensionMethodApi {
	crate::v0::build_dispatch(SPONSORSHIPS, dispatchable)
}

// Helper method to build a call to read state.
//
// Parameters:
// - 'state_query': The index of the runtime state query.
fn build_read_state(state_query: u8) -> ChainExtensionMethodApi {
	crate::v0::build_read_state(SPONSORSHIPS, state_query)
}

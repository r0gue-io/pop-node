//! The `fungibles` module provides an API for interacting and managing fungible tokens.
//!
//! The API includes the following interfaces:
//! 1. PSP-22
//! 2. PSP-22 Metadata
//! 3. Management
//! 4. PSP-22 Mintable & Burnable

use constants::*;
pub use errors::*;
pub use events::*;
use ink::prelude::vec::Vec;
pub use management::*;
pub use metadata::*;
pub use traits::*;

use crate::{
	constants::{ASSETS, BALANCES, FUNGIBLES},
	primitives::{AccountId, Balance, TokenId},
	ChainExtensionMethodApi, Result, StatusCode,
};

pub mod errors;
pub mod events;
pub mod traits;

/// Returns the total token supply for a specified token.
///
/// # Parameters
/// - `token` - The token.
#[inline]
pub fn total_supply(token: TokenId) -> Result<Balance> {
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

/// Returns the allowance for a `spender` approved by an `owner`, for a specified `token`. Returns
/// `0` if no allowance has been set.
///
/// # Parameters
/// - `token` - The token.
/// - `owner` - The account that owns the tokens.
/// - `spender` - The account that is allowed to spend the tokens.
#[inline]
pub fn allowance(token: TokenId, owner: AccountId, spender: AccountId) -> Result<Balance> {
	build_read_state(ALLOWANCE)
		.input::<(TokenId, AccountId, AccountId)>()
		.output::<Result<Balance>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(token, owner, spender))
}

/// Transfers `value` amount of tokens from the caller's account to account `to`.
///
/// # Parameters
/// - `token` - The token to transfer.
/// - `to` - The recipient account.
/// - `value` - The number of tokens to transfer.
#[inline]
pub fn transfer(token: TokenId, to: AccountId, value: Balance) -> Result<()> {
	build_dispatch(TRANSFER)
		.input::<(TokenId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(token, to, value))
}

/// Transfers `value` amount tokens on behalf of `from` to account `to`.
///
/// # Parameters
/// - `token` - The token to transfer.
/// - `from` - The account from which the token balance will be withdrawn.
/// - `to` - The recipient account.
/// - `value` - The number of tokens to transfer.
#[inline]
pub fn transfer_from(token: TokenId, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
	build_dispatch(TRANSFER_FROM)
		.input::<(TokenId, AccountId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(token, from, to, value))
}

/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
///
/// # Parameters
/// - `token` - The token to approve.
/// - `spender` - The account that is allowed to spend the tokens.
/// - `value` - The number of tokens to approve.
#[inline]
pub fn approve(token: TokenId, spender: AccountId, value: Balance) -> Result<()> {
	build_dispatch(APPROVE)
		.input::<(TokenId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(token, spender, value))
}

/// Increases the allowance of `spender` by `value` amount of tokens.
///
/// # Parameters
/// - `token` - The token to have an allowance increased.
/// - `spender` - The account that is allowed to spend the tokens.
/// - `value` - The number of tokens to increase the allowance by.
#[inline]
pub fn increase_allowance(token: TokenId, spender: AccountId, value: Balance) -> Result<()> {
	build_dispatch(INCREASE_ALLOWANCE)
		.input::<(TokenId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(token, spender, value))
}

/// Decreases the allowance of `spender` by `value` amount of tokens.
///
/// # Parameters
/// - `token` - The token to have an allowance decreased.
/// - `spender` - The account that is allowed to spend the tokens.
/// - `value` - The number of tokens to decrease the allowance by.
#[inline]
pub fn decrease_allowance(token: TokenId, spender: AccountId, value: Balance) -> Result<()> {
	build_dispatch(DECREASE_ALLOWANCE)
		.input::<(TokenId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(token, spender, value))
}

/// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
///
/// # Parameters
/// - `token` - The token to mint.
/// - `account` - The account to be credited with the created tokens.
/// - `value` - The number of tokens to mint.
#[inline]
pub fn mint(token: TokenId, account: AccountId, value: Balance) -> Result<()> {
	build_dispatch(MINT)
		.input::<(TokenId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(token, account, value))
}

/// Destroys `value` amount of tokens from `account`, reducing the total supply.
///
/// # Parameters
/// - `token` - the token to burn.
/// - `account` - The account from which the tokens will be destroyed.
/// - `value` - The number of tokens to destroy.
#[inline]
pub fn burn(token: TokenId, account: AccountId, value: Balance) -> Result<()> {
	build_dispatch(BURN)
		.input::<(TokenId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(token, account, value))
}

/// The PSP-22 compliant interface for querying metadata.
pub mod metadata {
	use super::*;

	/// Returns the name of the specified token.
	///
	/// # Parameters
	/// - `token` - The token.
	#[inline]
	pub fn token_name(token: TokenId) -> Result<Vec<u8>> {
		build_read_state(TOKEN_NAME)
			.input::<TokenId>()
			.output::<Result<Vec<u8>>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(token))
	}

	/// Returns the symbol for the specified token.
	///
	/// # Parameters
	/// - `token` - The token.
	#[inline]
	pub fn token_symbol(token: TokenId) -> Result<Vec<u8>> {
		build_read_state(TOKEN_SYMBOL)
			.input::<TokenId>()
			.output::<Result<Vec<u8>>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(token))
	}

	/// Returns the decimals for the specified token.
	///
	/// # Parameters
	/// - `token` - The token.
	#[inline]
	pub fn token_decimals(token: TokenId) -> Result<u8> {
		build_read_state(TOKEN_DECIMALS)
			.input::<TokenId>()
			.output::<Result<u8>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(token))
	}
}

/// The interface for creating, managing and destroying fungible tokens.
pub mod management {
	use super::*;

	/// Create a new token with a given identifier.
	///
	/// # Parameters
	/// - `id` - The identifier of the token.
	/// - `admin` - The account that will administer the token.
	/// - `min_balance` - The minimum balance required for accounts holding this token.
	#[inline]
	pub fn create(id: TokenId, admin: AccountId, min_balance: Balance) -> Result<()> {
		build_dispatch(CREATE)
			.input::<(TokenId, AccountId, Balance)>()
			.output::<Result<()>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(id, admin, min_balance))
	}

	/// Start the process of destroying a token.
	///
	/// # Parameters
	/// - `token` - The token to be destroyed.
	#[inline]
	pub fn start_destroy(token: TokenId) -> Result<()> {
		build_dispatch(START_DESTROY)
			.input::<TokenId>()
			.output::<Result<()>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(token))
	}

	/// Set the metadata for a token.
	///
	/// # Parameters
	/// - `token`: The token to update.
	/// - `name`: The user friendly name of this token.
	/// - `symbol`: The exchange symbol for this token.
	/// - `decimals`: The number of decimals this token uses to represent one unit.
	#[inline]
	pub fn set_metadata(
		token: TokenId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> Result<()> {
		build_dispatch(SET_METADATA)
			.input::<(TokenId, Vec<u8>, Vec<u8>, u8)>()
			.output::<Result<()>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(token, name, symbol, decimals))
	}

	/// Clear the metadata for a token.
	///
	/// # Parameters
	/// - `token` - The token to update
	#[inline]
	pub fn clear_metadata(token: TokenId) -> Result<()> {
		build_dispatch(CLEAR_METADATA)
			.input::<TokenId>()
			.output::<Result<()>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(token))
	}

	/// Checks if a specified token exists.
	///
	/// # Parameters
	/// - `token` - The token.
	#[inline]
	pub fn token_exists(token: TokenId) -> Result<bool> {
		build_read_state(TOKEN_EXISTS)
			.input::<TokenId>()
			.output::<Result<bool>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(token))
	}
}

mod constants {
	/// 1. PSP-22
	pub(super) const TOTAL_SUPPLY: u8 = 0;
	pub(super) const BALANCE_OF: u8 = 1;
	pub(super) const ALLOWANCE: u8 = 2;
	pub(super) const TRANSFER: u8 = 3;
	pub(super) const TRANSFER_FROM: u8 = 4;
	pub(super) const APPROVE: u8 = 5;
	pub(super) const INCREASE_ALLOWANCE: u8 = 6;
	pub(super) const DECREASE_ALLOWANCE: u8 = 7;

	/// 2. PSP-22 Metadata
	pub(super) const TOKEN_NAME: u8 = 8;
	pub(super) const TOKEN_SYMBOL: u8 = 9;
	pub(super) const TOKEN_DECIMALS: u8 = 10;

	/// 3. Management
	pub(super) const CREATE: u8 = 11;
	pub(super) const START_DESTROY: u8 = 12;
	pub(super) const SET_METADATA: u8 = 16;
	pub(super) const CLEAR_METADATA: u8 = 17;
	pub(super) const TOKEN_EXISTS: u8 = 18;

	/// 4. PSP-22 Mintable & Burnable
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

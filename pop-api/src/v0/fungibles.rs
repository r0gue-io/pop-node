//! The `fungibles` module provides an API for interacting and managing fungible tokens on Pop Network.
//!
//! The API includes the following interfaces:
//! 1. PSP-22
//! 2. PSP-22 Metadata
//! 3. Management
//! 4. PSP-22 Mintable & Burnable

use crate::{
	constants::{ASSETS, BALANCES, FUNGIBLES},
	primitives::{AccountId, Balance, TokenId},
	Result, StatusCode,
};
use constants::*;
use ink::{env::chain_extension::ChainExtensionMethod, prelude::vec::Vec};
pub use management::*;
pub use metadata::*;

// Helper method to build a dispatch call.
//
// Parameters:
// - 'dispatchable': The index of the dispatchable function within the module.
fn build_dispatch(dispatchable: u8) -> ChainExtensionMethod<(), (), (), false> {
	crate::v0::build_dispatch(FUNGIBLES, dispatchable)
}

// Helper method to build a call to read state.
//
// Parameters:
// - 'state_query': The index of the runtime state query.
fn build_read_state(state_query: u8) -> ChainExtensionMethod<(), (), (), false> {
	crate::v0::build_read_state(FUNGIBLES, state_query)
}

mod constants {
	/// 1. PSP-22 Interface:
	pub(super) const TOTAL_SUPPLY: u8 = 0;
	pub(super) const BALANCE_OF: u8 = 1;
	pub(super) const ALLOWANCE: u8 = 2;
	pub(super) const TRANSFER: u8 = 3;
	pub(super) const TRANSFER_FROM: u8 = 4;
	pub(super) const APPROVE: u8 = 5;
	pub(super) const INCREASE_ALLOWANCE: u8 = 6;
	pub(super) const DECREASE_ALLOWANCE: u8 = 7;

	/// 2. PSP-22 Metadata Interface:
	pub(super) const TOKEN_NAME: u8 = 8;
	pub(super) const TOKEN_SYMBOL: u8 = 9;
	pub(super) const TOKEN_DECIMALS: u8 = 10;

	/// 3. Asset Management:
	pub(super) const CREATE: u8 = 11;
	pub(super) const START_DESTROY: u8 = 12;
	pub(super) const SET_METADATA: u8 = 16;
	pub(super) const CLEAR_METADATA: u8 = 17;
	pub(super) const TOKEN_EXISTS: u8 = 18;

	/// 4. PSP-22 Mintable & Burnable interface:
	pub(super) const MINT: u8 = 19;
	pub(super) const BURN: u8 = 20;
}

/// A set of events for use in smart contracts interacting with the fungibles API.
///
/// The `Transfer` and `Approval` events conform to the PSP-22 standard. The other events
/// (`Create`, `StartDestroy`, `SetMetadata`, `ClearMetadata`) are provided for convenience.
///
/// These events are not emitted by the API itself but can be used in your contracts to
/// track token operations. Be mindful of the costs associated with emitting events.
///
/// For more details, refer to [ink! events](https://use.ink/basics/events).
pub mod events {
	use super::*;

	/// Event emitted when allowance by `owner` to `spender` changes.
	#[ink::event]
	pub struct Approval {
		/// The owner providing the allowance.
		#[ink(topic)]
		pub owner: AccountId,
		/// The beneficiary of the allowance.
		#[ink(topic)]
		pub spender: AccountId,
		/// The new allowance amount.
		pub value: u128,
	}

	/// Event emitted when transfer of tokens occurs.
	#[ink::event]
	pub struct Transfer {
		/// The source of the transfer. `None` when minting.
		#[ink(topic)]
		pub from: Option<AccountId>,
		/// The recipient of the transfer. `None` when burning.
		#[ink(topic)]
		pub to: Option<AccountId>,
		/// The amount transferred (or minted/burned).
		pub value: u128,
	}

	/// Event emitted when an token is created.
	#[ink::event]
	pub struct Create {
		/// The token identifier.
		#[ink(topic)]
		pub id: TokenId,
		/// The creator of the token.
		#[ink(topic)]
		pub creator: AccountId,
		/// The administrator of the token.
		#[ink(topic)]
		pub admin: AccountId,
	}

	/// Event emitted when a token is in the process of being destroyed.
	#[ink::event]
	pub struct StartDestroy {
		/// The token.
		#[ink(topic)]
		pub token: TokenId,
	}

	/// Event emitted when new metadata is set for a token.
	#[ink::event]
	pub struct SetMetadata {
		/// The token.
		#[ink(topic)]
		pub token: TokenId,
		/// The name of the token.
		#[ink(topic)]
		pub name: Vec<u8>,
		/// The symbol of the token.
		#[ink(topic)]
		pub symbol: Vec<u8>,
		/// The decimals of the token.
		pub decimals: u8,
	}

	/// Event emitted when metadata is cleared for a token.
	#[ink::event]
	pub struct ClearMetadata {
		/// The token.
		#[ink(topic)]
		pub token: TokenId,
	}
}

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

/// The PSP-22 Metadata interface for querying metadata.
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

/// Represents various errors related to fungible tokens in the Pop API.
///
/// The `FungiblesError` provides a detailed and specific set of error types that can occur when
/// interacting with fungible tokens through the Pop API. Each variant signifies a particular error
/// condition, facilitating precise error handling and debugging.
///
/// It is designed to be lightweight, including only the essential errors relevant to fungible token
/// operations. The `Other` variant serves as a catch-all for any unexpected errors. For more
/// detailed debugging, the `Other` variant can be converted into the richer `Error` type defined in
/// the primitives crate.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum FungiblesError {
	/// An unspecified or unknown error occurred.
	Other(StatusCode),
	/// The token is not live; either frozen or being destroyed.
	TokenNotLive,
	/// Not enough allowance to fulfill a request is available.
	InsufficientAllowance,
	/// Not enough balance to fulfill a request is available.
	InsufficientBalance,
	/// The token ID is already taken.
	InUse,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
	/// The account to alter does not exist.
	NoAccount,
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given token ID is unknown.
	Unknown,
	/// No balance for creation of tokens or fees.
	// TODO: Originally `pallet_balances::Error::InsufficientBalance` but collides with the
	//  `InsufficientBalance` error that is used for `pallet_assets::Error::BalanceLow` to adhere to
	//   standard. This deserves a second look.
	NoBalance,
}

impl From<StatusCode> for FungiblesError {
	/// Converts a `StatusCode` to a `FungiblesError`.
	///
	/// This conversion maps a `StatusCode`, returned by the runtime, to a more descriptive
	/// `FungiblesError`. This provides better context and understanding of the error, allowing
	/// developers to handle the most important errors effectively.
	fn from(value: StatusCode) -> Self {
		let encoded = value.0.to_le_bytes();
		match encoded {
			// Balances.
			[_, BALANCES, 2, _] => FungiblesError::NoBalance,
			// Assets.
			[_, ASSETS, 0, _] => FungiblesError::NoAccount,
			[_, ASSETS, 1, _] => FungiblesError::NoPermission,
			[_, ASSETS, 2, _] => FungiblesError::Unknown,
			[_, ASSETS, 3, _] => FungiblesError::InUse,
			[_, ASSETS, 5, _] => FungiblesError::MinBalanceZero,
			[_, ASSETS, 7, _] => FungiblesError::InsufficientAllowance,
			[_, ASSETS, 10, _] => FungiblesError::TokenNotLive,
			_ => FungiblesError::Other(value),
		}
	}
}

#[cfg(test)]
mod tests {
	use ink::scale::{Decode, Encode};

	use super::FungiblesError;
	use crate::{
		constants::{ASSETS, BALANCES},
		primitives::error::{
			ArithmeticError::*,
			Error::{self, *},
			TokenError::*,
			TransactionalError::*,
		},
		StatusCode,
	};

	fn error_into_status_code(error: Error) -> StatusCode {
		let mut encoded_error = error.encode();
		encoded_error.resize(4, 0);
		let value = u32::from_le_bytes(
			encoded_error.try_into().expect("qed, resized to 4 bytes line above"),
		);
		value.into()
	}

	fn into_fungibles_error(error: Error) -> FungiblesError {
		let status_code: StatusCode = error_into_status_code(error);
		status_code.into()
	}

	// If we ever want to change the conversion from bytes to `u32`.
	#[test]
	fn status_code_vs_encoded() {
		assert_eq!(u32::decode(&mut &[3u8, 10, 2, 0][..]).unwrap(), 133635u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 0, 0][..]).unwrap(), 13315u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 1, 0][..]).unwrap(), 78851u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 2, 0][..]).unwrap(), 144387u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 3, 0][..]).unwrap(), 209923u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 5, 0][..]).unwrap(), 340995u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 7, 0][..]).unwrap(), 472067u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 10, 0][..]).unwrap(), 668675u32);
	}

	#[test]
	fn conversion_status_code_into_fungibles_error_works() {
		let other_errors = vec![
			Other { dispatch_error_index: 5, error_index: 5, error: 1 },
			CannotLookup,
			BadOrigin,
			// `ModuleError` other than assets module.
			Module { index: 2, error: 5 },
			ConsumerRemaining,
			NoProviders,
			TooManyConsumers,
			Token(OnlyProvider),
			Arithmetic(Overflow),
			Transactional(NoLayer),
			Exhausted,
			Corruption,
			Unavailable,
			RootNotAllowed,
			UnknownCall,
			DecodingFailed,
		];
		for error in other_errors {
			let status_code: StatusCode = error_into_status_code(error);
			let fungibles_error: FungiblesError = status_code.into();
			assert_eq!(fungibles_error, FungiblesError::Other(status_code))
		}

		assert_eq!(
			into_fungibles_error(Module { index: BALANCES, error: 2 }),
			FungiblesError::NoBalance
		);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 0 }),
			FungiblesError::NoAccount
		);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 1 }),
			FungiblesError::NoPermission
		);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 2 }),
			FungiblesError::Unknown
		);
		assert_eq!(into_fungibles_error(Module { index: ASSETS, error: 3 }), FungiblesError::InUse);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 5 }),
			FungiblesError::MinBalanceZero
		);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 7 }),
			FungiblesError::InsufficientAllowance
		);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 10 }),
			FungiblesError::TokenNotLive
		);
	}
}

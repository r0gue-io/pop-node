//! Traits that can be used by contracts. Including standard compliant traits.

use super::*;
use core::result::Result;
use ink::prelude::string::String;

/// The PSP22 trait.
#[ink::trait_definition]
pub trait PSP22 {
	/// Returns the total token supply.
	#[ink(message)]
	fn total_supply(&self) -> Balance;

	/// Returns the account balance for the specified `owner`.
	///
	/// # Parameters
	/// - `owner` - The account whose balance is being queried.
	#[ink(message)]
	fn balance_of(&self, owner: AccountId) -> Balance;

	/// Returns the allowance for a `spender` approved by an `owner`.
	///
	/// # Parameters
	/// - `owner` - The account that owns the tokens.
	/// - `spender` - The account that is allowed to spend the tokens.
	#[ink(message)]
	fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

	/// Transfers `value` amount of tokens from the caller's account to account `to`
	/// with additional `data` in unspecified format.
	///
	/// # Parameters
	/// - `to` - The recipient account.
	/// - `value` - The number of tokens to transfer.
	/// - `data` - Additional data in unspecified format.
	#[ink(message)]
	fn transfer(&mut self, to: AccountId, value: Balance, data: Vec<u8>) -> Result<(), PSP22Error>;

	/// Transfers `value` tokens on behalf of `from` to the account `to`
	/// with additional `data` in unspecified format.
	///
	/// # Parameters
	/// - `from` - The account from which the token balance will be withdrawn.
	/// - `to` - The recipient account.
	/// - `value` - The number of tokens to transfer.
	/// - `data` - Additional data with unspecified format.
	#[ink(message)]
	fn transfer_from(
		&mut self,
		from: AccountId,
		to: AccountId,
		value: Balance,
		data: Vec<u8>,
	) -> Result<(), PSP22Error>;

	/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
	///
	/// Successive calls of this method overwrite previous values.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to approve.
	#[ink(message)]
	fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error>;

	/// Increases the allowance of `spender` by `value` amount of tokens.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to increase the allowance by.
	#[ink(message)]
	fn increase_allowance(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error>;

	/// Decreases the allowance of `spender` by `value` amount of tokens.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to decrease the allowance by.
	#[ink(message)]
	fn decrease_allowance(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error>;
}

/// The PSP22 Metadata trait.
#[ink::trait_definition]
pub trait PSP22Metadata {
	/// Returns the token name.
	#[ink(message)]
	fn token_name(&self) -> Option<String>;

	/// Returns the token symbol.
	#[ink(message)]
	fn token_symbol(&self) -> Option<String>;

	/// Returns the token decimals.
	#[ink(message)]
	fn token_decimals(&self) -> u8;
}

/// The PSP22 Mintable trait.
#[ink::trait_definition]
pub trait PSP22Mintable {
	/// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
	///
	/// # Parameters
	/// - `account` - The account to be credited with the created tokens.
	/// - `value` - The number of tokens to mint.
	#[ink(message)]
	fn mint(&mut self, account: AccountId, value: Balance) -> Result<(), PSP22Error>;
}

/// The PSP22 Burnable trait.
#[ink::trait_definition]
pub trait PSP22Burnable {
	/// Destroys `value` amount of tokens from `account`, reducing the total supply.
	///
	/// # Parameters
	/// - `account` - The account from which the tokens will be destroyed.
	/// - `value` - The number of tokens to destroy.
	#[ink(message)]
	fn burn(&mut self, account: AccountId, value: Balance) -> Result<(), PSP22Error>;
}

//! Traits that can be used by contracts. Including standard compliant traits.

use super::*;
use core::result::Result;
use ink::prelude::string::String;

/// The PSP22 trait.
#[ink::trait_definition]
pub trait Psp22 {
	/// Returns the total token supply.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#total_supply--balance
	#[ink(message, selector = 0x162df8c2)]
	fn total_supply(&self) -> Balance;

	/// Returns the account balance for the specified `owner`.
	///
	/// # Parameters
	/// - `owner` - The account whose balance is being queried.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#balance_ofowner-accountid--balance
	#[ink(message, selector = 0x6568382f)]
	fn balance_of(&self, owner: AccountId) -> Balance;

	/// Returns the allowance for a `spender` approved by an `owner`.
	///
	/// # Parameters
	/// - `owner` - The account that owns the tokens.
	/// - `spender` - The account that is allowed to spend the tokens.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#allowanceowner-accountid-spender-accountid--balance
	#[ink(message, selector = 0x4d47d921)]
	fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

	/// Transfers `value` amount of tokens from the caller's account to account `to`
	/// with additional `data` in unspecified format.
	///
	/// # Parameters
	/// - `to` - The recipient account.
	/// - `value` - The number of tokens to transfer.
	/// - `data` - Additional data in unspecified format.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#transferto-accountid-value-balance-data-u8--result-psp22error
	#[ink(message, selector = 0xdb20f9f5)]
	fn transfer(&mut self, to: AccountId, value: Balance, data: Vec<u8>) -> Result<(), PSP22Error>;

	/// Transfers `value` tokens on behalf of `from` to the account `to`
	/// with additional `data` in unspecified format.
	///
	/// # Parameters
	/// - `from` - The account from which the token balance will be withdrawn.
	/// - `to` - The recipient account.
	/// - `value` - The number of tokens to transfer.
	/// - `data` - Additional data with unspecified format.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#transfer_fromfrom-accountid-to-accountid-value-balance-data-u8--result-psp22error
	#[ink(message, selector = 0x54b3c76e)]
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
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#approvespender-accountid-value-balance--result-psp22error
	#[ink(message, selector = 0xb20f1bbd)]
	fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error>;

	/// Increases the allowance of `spender` by `value` amount of tokens.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to increase the allowance by.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#increase_allowancespender-accountid-delta_value-balance--result-psp22error
	#[ink(message, selector = 0x96d6b57a)]
	fn increase_allowance(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error>;

	/// Decreases the allowance of `spender` by `value` amount of tokens.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to decrease the allowance by.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#decrease_allowancespender-accountid-delta_value-balance--result-psp22error
	#[ink(message, selector = 0xfecb57d5)]
	fn decrease_allowance(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error>;
}

/// The PSP22 Metadata trait.
#[ink::trait_definition]
pub trait Psp22Metadata {
	/// Returns the token name.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#token_name--option
	#[ink(message, selector = 0x3d261bd4)]
	fn token_name(&self) -> Option<String>;

	/// Returns the token symbol.
	///
	/// The selector is `0x34205be5` (first 4 bytes of `blake2b_256("PSP22Metadata::token_symbol")`).
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#token_symbol--option
	#[ink(message, selector = 0x34205be5)]
	fn token_symbol(&self) -> Option<String>;

	/// Returns the token decimals.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#token_decimals--u8
	#[ink(message, selector = 0x7271b782)]
	fn token_decimals(&self) -> u8;
}

/// The PSP22 Mintable trait.
#[ink::trait_definition]
pub trait Psp22Mintable {
	/// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
	///
	/// # Parameters
	/// - `account` - The account to be credited with the created tokens.
	/// - `value` - The number of tokens to mint.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#token_decimals--u8
	#[ink(message, selector = 0xfc3c75d4)]
	fn mint(&mut self, account: AccountId, value: Balance) -> Result<(), PSP22Error>;
}

/// The PSP22 Burnable trait.
#[ink::trait_definition]
pub trait Psp22Burnable {
	/// Destroys `value` amount of tokens from `account`, reducing the total supply.
	///
	/// # Parameters
	/// - `account` - The account from which the tokens will be destroyed.
	/// - `value` - The number of tokens to destroy.
	///
	/// # Reference
	/// https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md#token_decimals--u8
	#[ink(message, selector = 0x7a9da510)]
	fn burn(&mut self, account: AccountId, value: Balance) -> Result<(), PSP22Error>;
}

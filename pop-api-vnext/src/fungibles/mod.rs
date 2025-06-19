use alloy_sol_types::{
	sol_data::{Address as SolAddress, Uint},
	SolError, SolType,
};
pub use erc20::{Approval, Transfer};
pub use errors::*;
use ink::{
	abi::Sol,
	contract_ref,
	prelude::{string::String, vec::Vec},
	primitives::sol::SolTypeEncode,
	U256,
};

pub use super::revert;
use super::*;

/// APIs for fungible tokens conforming to the ERC20 standard.
pub mod erc20;
pub mod errors;
pub mod events;

const PRECOMPILE: u16 = 100;

pub type TokenId = u32;

/// The fungibles precompile offers a streamlined interface for interacting with fungible
/// tokens. The goal is to provide a simplified, consistent API that adheres to standards in
/// the smart contract space.
#[ink::trait_definition]
pub trait Fungibles {
	/// Transfers `value` amount of tokens from the caller's account to account `to`.
	///
	/// # Parameters
	/// - `token` - The token to transfer.
	/// - `to` - The recipient account.
	/// - `value` - The number of tokens to transfer.
	#[ink(message)]
	fn transfer(&self, token: TokenId, to: Address, value: U256);

	/// Transfers `value` amount tokens on behalf of `from` to account `to`.
	///
	/// # Parameters
	/// - `token` - The token to transfer.
	/// - `from` - The account from which the token balance will be withdrawn.
	/// - `to` - The recipient account.
	/// - `value` - The number of tokens to transfer.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn transferFrom(&self, token: TokenId, from: Address, to: Address, value: U256);

	/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
	///
	/// # Parameters
	/// - `token` - The token to approve.
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to approve.
	#[ink(message)]
	fn approve(&self, token: TokenId, spender: Address, value: U256);

	/// Increases the allowance of `spender` by `value` amount of tokens.
	///
	/// # Parameters
	/// - `token` - The token to have an allowance increased.
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to increase the allowance by.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn increaseAllowance(&self, token: TokenId, spender: Address, value: U256) -> U256;

	/// Decreases the allowance of `spender` by `value` amount of tokens.
	///
	/// # Parameters
	/// - `token` - The token to have an allowance decreased.
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to decrease the allowance by.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn decreaseAllowance(&self, token: TokenId, spender: Address, value: U256) -> U256;

	/// Create a new token with an automatically generated identifier.
	///
	/// # Parameters
	/// - `admin` - The account that will administer the token.
	/// - `min_balance` - The minimum balance required for accounts holding this token.
	///
	/// NOTE: The minimum balance must be non-zero.
	#[ink(message)]
	fn create(&self, admin: Address, min_balance: U256) -> TokenId;

	/// Start the process of destroying a token.
	///
	/// # Parameters
	/// - `token` - The token to be destroyed.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn startDestroy(&self, token: TokenId);

	/// Set the metadata for a token.
	///
	/// # Parameters
	/// - `token`: The token to update.
	/// - `name`: The user friendly name of this token.
	/// - `symbol`: The exchange symbol for this token.
	/// - `decimals`: The number of decimals this token uses to represent one unit.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn setMetadata(&self, token: TokenId, name: String, symbol: String, decimals: u8);

	/// Clear the metadata for a token.
	///
	/// # Parameters
	/// - `token` - The token to update.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn clearMetadata(&self, token: TokenId);

	/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
	/// supply.
	///
	/// # Parameters
	/// - `token` - The token to mint.
	/// - `account` - The account to be credited with the created tokens.
	/// - `value` - The number of tokens to mint.
	#[ink(message)]
	fn mint(&self, token: TokenId, account: Address, value: U256);

	/// Destroys `value` amount of tokens from `account`, reducing the total supply.
	///
	/// # Parameters
	/// - `token` - The token to burn.
	/// - `account` - The account from which the tokens will be destroyed.
	/// - `value` - The number of tokens to destroy.
	#[ink(message)]
	fn burn(&self, token: TokenId, account: Address, value: U256);

	/// Total token supply for a specified token.
	///
	/// # Parameters
	/// - `token` - The token.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn totalSupply(&self, token: TokenId) -> U256;

	/// Account balance for a specified `token` and `owner`.
	///
	/// # Parameters
	/// - `token` - The token.
	/// - `owner` - The owner of the token.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn balanceOf(&self, token: TokenId, owner: Address) -> U256;

	/// Allowance for a `spender` approved by an `owner`, for a specified `token`.
	///
	/// # Parameters
	/// - `token` - The token.
	/// - `owner` - The owner of the token.
	/// - `spender` - The spender with an allowance.
	#[ink(message)]
	fn allowance(&self, token: TokenId, owner: Address, spender: Address) -> U256;

	/// Name of the specified token.
	///
	/// # Parameters
	/// - `token` - The token.
	#[ink(message)]
	fn name(&self, token: TokenId) -> String;

	/// Symbol for the specified token.
	///
	/// # Parameters
	/// - `token` - The token.
	#[ink(message)]
	fn symbol(&self, token: TokenId) -> String;

	/// Decimals for the specified token.
	///
	/// # Parameters
	/// - `token` - The token.
	#[ink(message)]
	fn decimals(&self, token: TokenId) -> u8;

	/// Whether the specified token exists.
	///
	/// # Parameters
	/// - `token` - The token.
	#[ink(message)]
	fn exists(&self, token: TokenId) -> bool;
}

/// Allowance for a `spender` approved by an `owner`, for a specified `token`.
///
/// # Parameters
/// - `token` - The token.
/// - `owner` - The owner of the token.
/// - `spender` - The spender with an allowance.
#[inline]
pub fn allowance(token: TokenId, owner: Address, spender: Address) -> U256 {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	precompile.allowance(token, owner, spender)
}

/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
///
/// # Parameters
/// - `token` - The token to approve.
/// - `spender` - The account that is allowed to spend the tokens.
/// - `value` - The number of tokens to approve.
#[inline]
pub fn approve(token: TokenId, spender: Address, value: U256) -> Result<(), Error> {
	ensure!(spender != Address::zero(), ZeroRecipientAddress);
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	Ok(precompile.approve(token, spender, value))
}

/// Account balance for a specified `token` and `owner`.
///
/// # Parameters
/// - `token` - The token.
/// - `owner` - The owner of the token.
#[inline]
pub fn balance_of(token: TokenId, owner: Address) -> U256 {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	precompile.balanceOf(token, owner)
}

/// Destroys `value` amount of tokens from `account`, reducing the total supply.
///
/// # Parameters
/// - `token` - The token to burn.
/// - `account` - The account from which the tokens will be destroyed.
/// - `value` - The number of tokens to destroy.
#[inline]
pub fn burn(token: TokenId, account: Address, value: U256) -> Result<(), Error> {
	ensure!(account != Address::zero(), ZeroSenderAddress);
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	Ok(precompile.burn(token, account, value))
}

/// Clear the metadata for a token.
///
/// # Parameters
/// - `token` - The token to update.
#[inline]
pub fn clear_metadata(token: TokenId) {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	precompile.clearMetadata(token)
}

/// Create a new token with an automatically generated identifier.
///
/// # Parameters
/// - `admin` - The account that will administer the token.
/// - `min_balance` - The minimum balance required for accounts holding this token.
///
/// NOTE: The minimum balance must be non-zero.
#[inline]
pub fn create(admin: Address, min_balance: U256) -> Result<TokenId, Error> {
	ensure!(admin != Address::zero(), ZeroAdminAddress);
	ensure!(min_balance != U256::zero(), MinBalanceZero);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	Ok(precompile.create(admin, min_balance))
}

/// Decimals for the specified token.
///
/// # Parameters
/// - `token` - The token.
#[inline]
pub fn decimals(token: TokenId) -> u8 {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	precompile.decimals(token)
}

/// Decreases the allowance of `spender` by `value` amount of tokens.
///
/// # Parameters
/// - `token` - The token to have an allowance decreased.
/// - `spender` - The account that is allowed to spend the tokens.
/// - `value` - The number of tokens to decrease the allowance by.
pub fn decrease_allowance(token: TokenId, spender: Address, value: U256) -> Result<U256, Error> {
	ensure!(spender != Address::zero(), ZeroRecipientAddress);
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	Ok(precompile.decreaseAllowance(token, spender, value))
}

/// Whether the specified token exists.
///
/// # Parameters
/// - `token` - The token.
#[inline]
pub fn exists(token: TokenId) -> bool {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	precompile.exists(token)
}

/// Increases the allowance of `spender` by `value` amount of tokens.
///
/// # Parameters
/// - `token` - The token to have an allowance increased.
/// - `spender` - The account that is allowed to spend the tokens.
/// - `value` - The number of tokens to increase the allowance by.
pub fn increase_allowance(token: TokenId, spender: Address, value: U256) -> Result<U256, Error> {
	ensure!(spender != Address::zero(), ZeroRecipientAddress);
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	Ok(precompile.increaseAllowance(token, spender, value))
}

/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
/// supply.
///
/// # Parameters
/// - `token` - The token to mint.
/// - `account` - The account to be credited with the created tokens.
/// - `value` - The number of tokens to mint.
#[inline]
pub fn mint(token: TokenId, account: Address, value: U256) -> Result<(), Error> {
	ensure!(account != Address::zero(), ZeroRecipientAddress);
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	Ok(precompile.mint(token, account, value))
}

/// Name of the specified token.
///
/// # Parameters
/// - `token` - The token.
#[inline]
pub fn name(token: TokenId) -> String {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	precompile.name(token)
}

/// Set the metadata for a token.
///
/// # Parameters
/// - `token`: The token to update.
/// - `name`: The user friendly name of this token.
/// - `symbol`: The exchange symbol for this token.
/// - `decimals`: The number of decimals this token uses to represent one unit.
#[inline]
pub fn set_metadata(token: TokenId, name: String, symbol: String, decimals: u8) {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	precompile.setMetadata(token, name, symbol, decimals)
}

/// Start the process of destroying a token.
///
/// # Parameters
/// - `token` - The token to be destroyed.
#[inline]
pub fn start_destroy(token: TokenId) {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	precompile.startDestroy(token)
}

/// Symbol for the specified token.
///
/// # Parameters
/// - `token` - The token.
#[inline]
pub fn symbol(token: TokenId) -> String {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	precompile.symbol(token)
}

/// Total token supply for a specified token.
///
/// # Parameters
/// - `token` - The token.
#[inline]
pub fn total_supply(token: TokenId) -> U256 {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	precompile.totalSupply(token)
}

/// Transfers `value` amount of tokens from the caller's account to account `to`.
///
/// # Parameters
/// - `token` - The token to transfer.
/// - `to` - The recipient account.
/// - `value` - The number of tokens to transfer.
#[inline]
pub fn transfer(token: TokenId, to: Address, value: U256) -> Result<(), Error> {
	ensure!(to != Address::zero(), ZeroRecipientAddress);
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	Ok(precompile.transfer(token, to, value))
}

/// Transfers `value` amount tokens on behalf of `from` to account `to`.
///
/// # Parameters
/// - `token` - The token to transfer.
/// - `from` - The account from which the token balance will be withdrawn.
/// - `to` - The recipient account.
/// - `value` - The number of tokens to transfer.
#[inline]
pub fn transfer_from(token: TokenId, from: Address, to: Address, value: U256) -> Result<(), Error> {
	ensure!(from != Address::zero(), ZeroSenderAddress);
	ensure!(to != Address::zero(), ZeroRecipientAddress);
	ensure!(to != from, InvalidRecipient(to));
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop, Sol) = address.into();
	Ok(precompile.transferFrom(token, from, to, value))
}

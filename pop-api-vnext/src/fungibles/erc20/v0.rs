pub use errors::{Error, Error::*};
pub use events::*;

use super::*;

/// Standard ERC-20 errors. See https://eips.ethereum.org/EIPS/eip-6093 for more details.
mod errors;
mod events;

// Precompile index within the runtime
const PRECOMPILE: u16 = 2;

/// Interface of the ERC-20 standard.
#[ink::trait_definition]
pub trait Erc20 {
	/// Returns the value of tokens in existence.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn totalSupply(&self) -> U256;

	/// Returns the value of tokens owned by `account`.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn balanceOf(&self, account: Address) -> U256;

	/// Moves a `value` amount of tokens from the caller's account to `to`.
	///
	/// Returns a boolean value indicating whether the operation succeeded.
	///
	/// Emits a [`Transfer`] event.
	#[ink(message)]
	fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Error>;

	/// Returns the remaining number of tokens that `spender` will be allowed to spend
	/// on behalf of `owner` through [`transfer_from`]. This is zero by default.
	///
	/// This value changes when `approve` or `[`transfer_from`] are called.
	#[ink(message)]
	fn allowance(&self, owner: Address, spender: Address) -> U256;

	/// Sets a `value` amount of tokens as the allowance of `spender` over the caller's
	/// tokens.
	///
	/// Returns a boolean value indicating whether the operation succeeded.
	///
	/// Emits an [`Approval`] event.
	#[ink(message)]
	fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Error>;

	/// Moves a `value` amount of tokens from `from` to `to` using the allowance mechanism.
	/// `value` is then deducted from the caller's allowance.
	///
	/// Returns a boolean value indicating whether the operation succeeded.
	///
	/// Emits a [`Transfer`] event.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn transferFrom(&mut self, from: Address, to: Address, value: U256) -> Result<bool, Error>;
}

/// Returns the value of tokens in existence.
#[inline]
pub fn total_supply(token: TokenId) -> U256 {
	let address = prefixed_address(PRECOMPILE, token);
	let precompile: contract_ref!(Erc20, Pop, Sol) = address.into();
	precompile.totalSupply()
}

/// Returns the value of tokens owned by `account`.
#[inline]
pub fn balance_of(token: TokenId, account: Address) -> U256 {
	let address = prefixed_address(PRECOMPILE, token);
	let precompile: contract_ref!(Erc20, Pop, Sol) = address.into();
	precompile.balanceOf(account)
}

/// Moves a `value` amount of tokens from the caller's account to `to`.
///
/// Returns a boolean value indicating whether the operation succeeded.
///
/// Emits a [`Transfer`] event.
#[inline]
pub fn transfer(token: TokenId, to: Address, value: U256) -> Result<bool, Error> {
	ensure!(to != Address::zero(), ERC20InvalidReceiver(to));
	ensure!(value != U256::zero(), ERC20InsufficientValue);

	let address = prefixed_address(PRECOMPILE, token);
	let mut precompile: contract_ref!(Erc20, Pop, Sol) = address.into();
	precompile.transfer(to, value)
}

/// Returns the remaining number of tokens that `spender` will be allowed to spend
/// on behalf of `owner` through [`transfer_from`]. This is zero by default.
///
/// This value changes when `approve` or [`transfer_from`] are called.
#[inline]
pub fn allowance(token: TokenId, owner: Address, spender: Address) -> U256 {
	let address = prefixed_address(PRECOMPILE, token);
	let precompile: contract_ref!(Erc20, Pop, Sol) = address.into();
	precompile.allowance(owner, spender)
}

/// Sets a `value` amount of tokens as the allowance of `spender` over the caller's
/// tokens.
///
/// Returns a boolean value indicating whether the operation succeeded.
///
/// Emits an [`Approval`] event.
#[inline]
pub fn approve(token: TokenId, spender: Address, value: U256) -> Result<bool, Error> {
	ensure!(spender != Address::zero(), ERC20InvalidSpender(spender));
	ensure!(value != U256::zero(), ERC20InsufficientValue);

	let address = prefixed_address(PRECOMPILE, token);
	let mut precompile: contract_ref!(Erc20, Pop, Sol) = address.into();
	precompile.approve(spender, value)
}

/// Moves a `value` amount of tokens from `from` to `to` using the allowance mechanism.
/// `value` is then deducted from the caller's allowance.
///
/// Returns a boolean value indicating whether the operation succeeded.
///
/// Emits a [`Transfer`] event.
#[inline]
pub fn transfer_from(
	token: TokenId,
	from: Address,
	to: Address,
	value: U256,
) -> Result<bool, Error> {
	ensure!(from != Address::zero(), ERC20InvalidSender(from));
	ensure!(to != Address::zero() && to != from, ERC20InvalidReceiver(to));
	ensure!(value != U256::zero(), ERC20InsufficientValue);

	let address = prefixed_address(PRECOMPILE, token);
	let mut precompile: contract_ref!(Erc20, Pop, Sol) = address.into();
	precompile.transferFrom(from, to, value)
}

/// Extensions to the ERC-20 standard.
pub mod extensions {
	use super::*;

	/// Interface for the optional metadata functions from the ERC-20 standard.
	#[ink::trait_definition]
	pub trait Erc20Metadata {
		/// Returns the name of the token.
		#[ink(message)]
		fn name(&self) -> String;

		/// Returns the symbol of the token.
		#[ink(message)]
		fn symbol(&self) -> String;

		/// Returns the decimals places of the token.
		#[ink(message)]
		fn decimals(&self) -> u8;
	}

	/// Returns the name of the token.
	#[inline]
	pub fn name(token: TokenId) -> String {
		let address = prefixed_address(PRECOMPILE, token);
		let precompile: contract_ref!(Erc20Metadata, Pop, Sol) = address.into();
		precompile.name()
	}

	/// Returns the symbol of the token.
	#[inline]
	pub fn symbol(token: TokenId) -> String {
		let address = prefixed_address(PRECOMPILE, token);
		let precompile: contract_ref!(Erc20Metadata, Pop, Sol) = address.into();
		precompile.symbol()
	}

	/// Returns the decimals places of the token.
	#[inline]
	pub fn decimals(token: TokenId) -> u8 {
		let address = prefixed_address(PRECOMPILE, token);
		let precompile: contract_ref!(Erc20Metadata, Pop, Sol) = address.into();
		precompile.decimals()
	}
}

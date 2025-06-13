use alloy_sol_types::{
	sol_data::{Address as SolAddress, Uint},
	SolError, SolType,
};
pub use erc20::{Approval, Transfer};
pub use errors::*;
use ink::{
	contract_ref,
	prelude::{string::String, vec::Vec},
	primitives::sol::SolTypeEncode,
	U256,
};

pub use super::revert;
use super::*;

/// APIs for fungible tokens conforming to the ERC20 standard.
pub mod erc20;

const PRECOMPILE: u16 = 100;

pub type TokenId = u32;

/// The fungibles precompile offers a streamlined interface for interacting with fungible
/// tokens. The goal is to provide a simplified, consistent API that adheres to standards in
/// the smart contract space.
#[ink::trait_definition]
pub trait Fungibles {
	/// Create a new token with an automatically generated identifier.
	#[ink(message)]
	fn create(&self, admin: Address, min_balance: U256) -> TokenId;

	/// Set the metadata for a token.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn setMetadata(&self, id: TokenId, name: String, symbol: String, decimals: u8);

	/// Clear the metadata for a token.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn clearMetadata(&self, id: TokenId);

	/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
	/// supply.
	#[ink(message)]
	fn mint(&self, id: TokenId, account: Address, value: U256);

	/// Transfers `value` amount of tokens from the caller's account to account `to`.
	#[ink(message)]
	fn transfer(&self, id: TokenId, to: Address, value: U256);

	/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
	#[ink(message)]
	fn approve(&self, id: TokenId, spender: Address, value: U256);

	/// Transfers `value` amount tokens on behalf of `from` to account `to`.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn transferFrom(&self, id: TokenId, from: Address, to: Address, value: U256);

	/// Destroys `value` amount of tokens from `account`, reducing the total supply.
	#[ink(message)]
	fn burn(&self, id: TokenId, address: Address, value: U256);

	/// Start the process of destroying a token.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn startDestroy(&self, id: TokenId);

	///  Whether a specified token exists.
	#[ink(message)]
	fn exists(&self, id: TokenId) -> bool;
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
	let precompile: contract_ref!(Fungibles, Pop) = address.into();
	Ok(precompile.create(admin, min_balance))
}

/// Set the metadata for a token.
#[inline]
pub fn set_metadata(id: TokenId, name: String, symbol: String, decimals: u8) {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop) = address.into();
	precompile.setMetadata(id, name, symbol, decimals)
}

/// Clear the metadata for a token.
#[inline]
pub fn clear_metadata(id: TokenId) {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop) = address.into();
	precompile.clearMetadata(id)
}

/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
/// supply.
#[inline]
pub fn mint(id: TokenId, account: Address, value: U256) -> Result<(), Error> {
	ensure!(account != Address::zero(), ZeroRecipientAddress);
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop) = address.into();
	Ok(precompile.mint(id, account, value))
}

/// Transfers `value` amount of tokens from the caller's account to account `to`.
#[inline]
pub fn transfer(id: TokenId, to: Address, value: U256) -> Result<(), Error> {
	ensure!(to != Address::zero(), ZeroRecipientAddress);
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop) = address.into();
	Ok(precompile.transfer(id, to, value))
}

/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
#[inline]
pub fn approve(id: TokenId, spender: Address, value: U256) -> Result<(), Error> {
	ensure!(spender != Address::zero(), ZeroRecipientAddress);
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop) = address.into();
	Ok(precompile.approve(id, spender, value))
}

/// Transfers `value` amount tokens on behalf of `from` to account `to`.
#[inline]
pub fn transfer_from(id: TokenId, from: Address, to: Address, value: U256) -> Result<(), Error> {
	ensure!(from != Address::zero(), ZeroSenderAddress);
	ensure!(to != Address::zero(), ZeroRecipientAddress);
	ensure!(to != from, InvalidRecipient(to));
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop) = address.into();
	Ok(precompile.transferFrom(id, from, to, value))
}

/// Destroys `value` amount of tokens from `account`, reducing the total supply.
#[inline]
pub fn burn(id: TokenId, account: Address, value: U256) -> Result<(), Error> {
	ensure!(account != Address::zero(), ZeroSenderAddress);
	ensure!(value != U256::zero(), ZeroValue);

	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop) = address.into();
	Ok(precompile.burn(id, account, value))
}

/// Start the process of destroying a token.
#[inline]
pub fn start_destroy(id: TokenId) {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop) = address.into();
	precompile.startDestroy(id)
}

/// Whether a specified token exists.
#[inline]
pub fn exists(id: TokenId) -> bool {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Fungibles, Pop) = address.into();
	precompile.exists(id)
}

/// Returns the total token supply.
#[inline]
pub fn total_supply(id: TokenId) -> U256 {
	erc20::total_supply(id)
}

/// Returns the value of tokens owned by `account`.
#[inline]
pub fn balance_of(id: TokenId, account: Address) -> U256 {
	erc20::balance_of(id, account)
}

/// Returns the remaining number of tokens that `spender` will be allowed to spend
/// on behalf of `owner` through [`transfer_from`]. This is zero by default.
///
/// This value changes when `approve` or [`transfer_from`] are called.
#[inline]
pub fn allowance(id: TokenId, owner: Address, spender: Address) -> U256 {
	erc20::allowance(id, owner, spender)
}

/// Returns the name of the token.
#[inline]
pub fn name(id: TokenId) -> String {
	erc20::extensions::name(id)
}

/// Returns the symbol of the token.
#[inline]
pub fn symbol(id: TokenId) -> String {
	erc20::extensions::symbol(id)
}

/// Returns the decimals places of the token.
#[inline]
pub fn decimals(id: TokenId) -> u8 {
	erc20::extensions::decimals(id)
}

// NOTE: subject to change based on ink!'s support for solidity custom errors.
pub enum Error {
	/// The token recipient is invalid.
	InvalidRecipient(InvalidRecipient),
	/// The minimum balance should be non-zero.
	MinBalanceZero(MinBalanceZero),
	/// The signing account has no permission to do the operation.
	NoPermission(NoPermission),
	/// The `admin` address cannot be the zero address.
	ZeroAdminAddress(ZeroAdminAddress),
	/// The recipient cannot be the zero address.
	ZeroRecipientAddress(ZeroRecipientAddress),
	/// The sender cannot be the zero address.
	ZeroSenderAddress(ZeroSenderAddress),
	/// The specified `value` cannot be zero.
	ZeroValue(ZeroValue),
}

impl<'a> SolEncode<'a> for Error {
	type SolType = ();

	fn encode(&'a self) -> Vec<u8> {
		use Error::*;
		match self {
			InvalidRecipient(e) => e.abi_encode(),
			MinBalanceZero(e) => e.abi_encode(),
			NoPermission(e) => e.abi_encode(),
			ZeroAdminAddress(e) => e.abi_encode(),
			ZeroRecipientAddress(e) => e.abi_encode(),
			ZeroSenderAddress(e) => e.abi_encode(),
			ZeroValue(e) => e.abi_encode(),
		}
	}

	fn to_sol_type(&'a self) -> Self::SolType {
		()
	}
}

mod errors {
	use super::*;

	/// The token recipient is invalid.
	pub struct InvalidRecipient(pub Address);
	impl SolError for InvalidRecipient {
		type Parameters<'a> = (SolAddress,);
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [23, 133, 139, 190];
		const SIGNATURE: &'static str = "InvalidRecipient(address)";

		#[inline]
		fn new<'a>(tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self(Address::from(*tuple.0 .0))
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			(self.0.to_sol_type().tokenize(),)
		}
	}
	impl<'a> SolEncode<'a> for InvalidRecipient {
		type SolType = (&'a Address,);

		#[inline]
		fn encode(&'a self) -> Vec<u8> {
			self.abi_encode()
		}

		#[inline]
		fn to_sol_type(&'a self) -> Self::SolType {
			(&self.0,)
		}
	}
	impl From<InvalidRecipient> for Error {
		fn from(value: InvalidRecipient) -> Self {
			Self::InvalidRecipient(value)
		}
	}

	/// The minimum balance should be non-zero.
	pub struct MinBalanceZero;
	impl SolError for MinBalanceZero {
		type Parameters<'a> = ();
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [95, 21, 97, 139];
		const SIGNATURE: &'static str = "MinBalanceZero()";

		#[inline]
		fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			()
		}
	}
	impl From<MinBalanceZero> for Error {
		fn from(value: MinBalanceZero) -> Self {
			Self::MinBalanceZero(value)
		}
	}

	/// The signing account has no permission to do the operation.
	pub struct NoPermission;
	impl SolError for NoPermission {
		type Parameters<'a> = ();
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [157, 123, 54, 157];
		const SIGNATURE: &'static str = "NoPermission()";

		#[inline]
		fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			()
		}
	}
	impl<'a> SolEncode<'a> for NoPermission {
		type SolType = ();

		fn encode(&'a self) -> Vec<u8> {
			self.abi_encode()
		}

		fn to_sol_type(&'a self) -> Self::SolType {
			()
		}
	}
	impl From<NoPermission> for Error {
		fn from(value: NoPermission) -> Self {
			Self::NoPermission(value)
		}
	}

	/// The `admin` address cannot be the zero address.
	pub struct ZeroAdminAddress;
	impl SolError for ZeroAdminAddress {
		type Parameters<'a> = ();
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [62, 243, 155, 129];
		const SIGNATURE: &'static str = "ZeroAdminAddress()";

		#[inline]
		fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			()
		}
	}
	impl From<ZeroAdminAddress> for Error {
		fn from(value: ZeroAdminAddress) -> Self {
			Self::ZeroAdminAddress(value)
		}
	}

	/// The recipient cannot be the zero address.
	pub struct ZeroRecipientAddress;
	impl SolError for ZeroRecipientAddress {
		type Parameters<'a> = ();
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [206, 239, 152, 87];
		const SIGNATURE: &'static str = "ZeroRecipientAddress()";

		#[inline]
		fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			()
		}
	}
	impl From<ZeroRecipientAddress> for Error {
		fn from(value: ZeroRecipientAddress) -> Self {
			Self::ZeroRecipientAddress(value)
		}
	}

	/// The sender cannot be the zero address.
	pub struct ZeroSenderAddress;
	impl SolError for ZeroSenderAddress {
		type Parameters<'a> = ();
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [255, 54, 43, 196];
		const SIGNATURE: &'static str = "ZeroSenderAddress()";

		#[inline]
		fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			()
		}
	}
	impl From<ZeroSenderAddress> for Error {
		fn from(value: ZeroSenderAddress) -> Self {
			Self::ZeroSenderAddress(value)
		}
	}

	/// The `value` should be non-zero.
	pub struct ZeroValue;
	impl SolError for ZeroValue {
		type Parameters<'a> = ();
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [124, 148, 110, 215];
		const SIGNATURE: &'static str = "ZeroValue()";

		#[inline]
		fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			()
		}
	}
	impl From<ZeroValue> for Error {
		fn from(value: ZeroValue) -> Self {
			Self::ZeroValue(value)
		}
	}

	#[test]
	fn error_encoding_works() {
		for (result, expected) in [
			(
				InvalidRecipient([255u8; 20].into()).abi_encode(),
				"17858bbe000000000000000000000000ffffffffffffffffffffffffffffffffffffffff",
			),
			(MinBalanceZero.abi_encode(), "5f15618b"),
			(NoPermission.abi_encode(), "9d7b369d"),
			(ZeroAdminAddress.abi_encode(), "3ef39b81"),
			(ZeroRecipientAddress.abi_encode(), "ceef9857"),
			(ZeroSenderAddress.abi_encode(), "ff362bc4"),
			(ZeroValue.abi_encode(), "7c946ed7"),
		] {
			assert_eq!(hex::encode(result), expected)
		}
	}
}

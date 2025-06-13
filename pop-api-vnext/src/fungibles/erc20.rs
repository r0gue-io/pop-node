pub use errors::*;

use super::{
	contract_ref, prefixed_address, Address, Pop, SolAddress, SolEncode, SolError, SolType,
	SolTypeEncode, String, TokenId, Uint, Vec, U256,
};
use crate::ensure;

const PRECOMPILE: u16 = 101;

/// Interface of the ERC-20 standard.
#[ink::trait_definition]
pub trait Erc20 {
	/// Returns the value of tokens in existence.
	#[ink(message)]
	#[allow(non_snake_case)] // Required to ensure message name results in correct sol selector
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
	fn transfer(&mut self, to: Address, value: U256) -> bool;

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
	fn approve(&mut self, spender: Address, value: U256) -> bool;

	/// Moves a `value` amount of tokens from `from` to `to` using the allowance mechanism.
	/// `value` is then deducted from the caller's allowance.
	///
	/// Returns a boolean value indicating whether the operation succeeded.
	///
	/// Emits a [`Transfer`] event.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn transferFrom(&mut self, from: Address, to: Address, value: U256) -> bool;
}

/// Emitted when the allowance of a `spender` for an `owner` is set by a call to
/// [`approve`]. `value` is the new allowance.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Approval {
	/// The owner providing the allowance.
	#[ink(topic)]
	pub owner: Address,
	/// The beneficiary of the allowance.
	#[ink(topic)]
	pub spender: Address,
	/// The new allowance amount.
	pub value: U256,
}

/// Emitted when `value` tokens are moved from one account (`from`) to another (`to`).
///
/// Note that `value` may be zero.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Transfer {
	/// The source of the transfer. The zero address when minting.
	#[ink(topic)]
	pub from: Address,
	/// The recipient of the transfer. The zero address when burning.
	#[ink(topic)]
	pub to: Address,
	/// The amount transferred (or minted/burned).
	pub value: U256,
}

/// Returns the value of tokens in existence.
#[inline]
pub fn total_supply(id: TokenId) -> U256 {
	let address = prefixed_address(PRECOMPILE, id);
	let precompile: contract_ref!(Erc20, Pop) = address.into();
	precompile.totalSupply()

	// let selector = 0x18160ddd_u32.to_be_bytes().into();
	// build_call_solidity::<Pop>()
	// 	.call(address)
	// 	.exec_input(ExecutionInput::new(selector))
	// 	.returns::<U256>()
	// 	.invoke()
}

/// Returns the value of tokens owned by `account`.
#[inline]
pub fn balance_of(id: TokenId, account: Address) -> U256 {
	let address = prefixed_address(PRECOMPILE, id);
	let precompile: contract_ref!(Erc20, Pop) = address.into();
	precompile.balanceOf(account)

	// let selector = 0x70a08231_u32.to_be_bytes().into();
	// build_call_solidity::<Pop>()
	// 	.call(address)
	// 	.exec_input(ExecutionInput::new(selector).push_arg(account))
	// 	.returns::<U256>()
	// 	.invoke()
}

/// Returns the value of tokens owned by `account`.
#[inline]
pub fn transfer(id: TokenId, to: Address, value: U256) -> Result<bool, Error> {
	ensure!(to != Address::zero(), ERC20InvalidSender(to));
	ensure!(value != U256::zero(), ERC20InsufficientValue);

	let address = prefixed_address(PRECOMPILE, id);
	let mut precompile: contract_ref!(Erc20, Pop) = address.into();
	Ok(precompile.transfer(to, value))

	// let selector = 0xa9059cbb_u32.to_be_bytes().into();
	// let result = build_call_solidity::<Pop>()
	// 	.call(address)
	// 	.exec_input(ExecutionInput::new(selector).push_arg(to).push_arg(value))
	// 	.returns::<bool>()
	// 	.invoke();
	// Ok(result)
}

/// Returns the remaining number of tokens that `spender` will be allowed to spend
/// on behalf of `owner` through [`transfer_from`]. This is zero by default.
///
/// This value changes when `approve` or [`transfer_from`] are called.
#[inline]
pub fn allowance(id: TokenId, owner: Address, spender: Address) -> U256 {
	let address = prefixed_address(PRECOMPILE, id);
	let precompile: contract_ref!(Erc20, Pop) = address.into();
	precompile.allowance(owner, spender)

	// let selector = 0xdd62ed3e_u32.to_be_bytes().into();
	// build_call_solidity::<Pop>()
	// 	.call(address)
	// 	.exec_input(ExecutionInput::new(selector).push_arg(owner).push_arg(spender))
	// 	.returns::<U256>()
	// 	.invoke()
}

/// Sets a `value` amount of tokens as the allowance of `spender` over the caller's
/// tokens.
///
/// Returns a boolean value indicating whether the operation succeeded.
///
/// Emits an [`Approval`] event.
#[inline]
pub fn approve(id: TokenId, spender: Address, value: U256) -> Result<bool, Error> {
	ensure!(spender != Address::zero(), ERC20InvalidApprover(spender));
	ensure!(value != U256::zero(), ERC20InsufficientValue);

	let address = prefixed_address(PRECOMPILE, id);
	let mut precompile: contract_ref!(Erc20, Pop) = address.into();
	Ok(precompile.approve(spender, value))

	// let selector = 0x095ea7b3_u32.to_be_bytes().into();
	// Ok(build_call_solidity::<Pop>()
	// 	.call(address)
	// 	.exec_input(ExecutionInput::new(selector).push_arg(spender).push_arg(value))
	// 	.returns::<bool>()
	// 	.invoke())
}

/// Moves a `value` amount of tokens from `from` to `to` using the allowance mechanism.
/// `value` is then deducted from the caller's allowance.
///
/// Returns a boolean value indicating whether the operation succeeded.
///
/// Emits a [`Transfer`] event.
#[inline]
pub fn transfer_from(id: TokenId, from: Address, to: Address, value: U256) -> Result<bool, Error> {
	ensure!(from != Address::zero(), ERC20InvalidSender(from));
	ensure!(to != Address::zero() && to != from, ERC20InvalidReceiver(to));
	ensure!(value != U256::zero(), ERC20InsufficientValue);

	let address = prefixed_address(PRECOMPILE, id);
	let mut precompile: contract_ref!(Erc20, Pop) = address.into();
	Ok(precompile.transferFrom(from, to, value))

	// let selector = 0x23b872dd_u32.to_be_bytes().into();
	// Ok(build_call_solidity::<Pop>()
	// 	.call(address)
	// 	.exec_input(ExecutionInput::new(selector).push_arg(from).push_arg(to).push_arg(value))
	// 	.returns::<bool>()
	// 	.invoke())
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
	pub fn name(id: TokenId) -> String {
		let address = prefixed_address(PRECOMPILE, id);
		let precompile: contract_ref!(Erc20Metadata, Pop) = address.into();
		precompile.name()

		// let selector = 0x06fdde03_u32.to_be_bytes().into();
		// build_call_solidity::<Pop>()
		// 	.call(address)
		// 	.exec_input(ExecutionInput::new(selector))
		// 	.returns::<String>()
		// 	.invoke()
	}

	/// Returns the symbol of the token.
	#[inline]
	pub fn symbol(id: TokenId) -> String {
		let address = prefixed_address(PRECOMPILE, id);
		let precompile: contract_ref!(Erc20Metadata, Pop) = address.into();
		precompile.symbol()

		// let selector = 0x95d89b41_u32.to_be_bytes().into();
		// build_call_solidity::<Pop>()
		// 	.call(address)
		// 	.exec_input(ExecutionInput::new(selector))
		// 	.returns::<String>()
		// 	.invoke()
	}

	/// Returns the decimals places of the token.
	#[inline]
	pub fn decimals(id: TokenId) -> u8 {
		let address = prefixed_address(PRECOMPILE, id);
		let precompile: contract_ref!(Erc20Metadata, Pop) = address.into();
		precompile.decimals()

		// let selector = 0x313ce567_u32.to_be_bytes().into();
		// build_call_solidity::<Pop>()
		// 	.call(address)
		// 	.exec_input(ExecutionInput::new(selector))
		// 	.returns::<u8>()
		// 	.invoke()
	}
}

// NOTE: subject to change based on ink!'s support for solidity custom errors.
pub enum Error {
	/// Indicates a failure with the `spender`’s `allowance`.
	InsufficientAllowance(ERC20InsufficientAllowance),
	/// Indicates an error related to the current `balance` of a `sender`.
	InsufficientBalance(ERC20InsufficientBalance),
	/// Indicates an error related to a specified `value`.
	InsufficientValue(ERC20InsufficientValue),
	/// Indicates a failure with the `approver` of a token to be approved.
	InvalidApprover(ERC20InvalidApprover),
	/// Indicates a failure with the token `receiver`.
	InvalidReceiver(ERC20InvalidReceiver),
	/// Indicates a failure with the token `sender`.
	InvalidSender(ERC20InvalidSender),
}

impl<'a> SolEncode<'a> for Error {
	type SolType = ();

	fn encode(&'a self) -> Vec<u8> {
		use Error::*;
		match self {
			InsufficientAllowance(e) => e.abi_encode(),
			InsufficientBalance(e) => e.abi_encode(),
			InsufficientValue(e) => e.abi_encode(),
			InvalidApprover(e) => e.abi_encode(),
			InvalidReceiver(e) => e.abi_encode(),
			InvalidSender(e) => e.abi_encode(),
		}
	}

	fn to_sol_type(&'a self) -> Self::SolType {
		()
	}
}

/// Standard ERC-20 errors.
// See https://eips.ethereum.org/EIPS/eip-6093 for more details.
mod errors {
	use super::*;

	/// Indicates a failure with the `spender`’s `allowance`.
	pub struct ERC20InsufficientAllowance(pub Address, pub U256, pub U256);
	impl SolError for ERC20InsufficientAllowance {
		type Parameters<'a> = (SolAddress, Uint<256>, Uint<256>);
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [251, 143, 65, 178];
		const SIGNATURE: &'static str = "ERC20InsufficientAllowance(address,uint256,uint256)";

		#[inline]
		fn new<'a>(tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self(
				Address::from(*tuple.0 .0),
				U256::from_little_endian(tuple.1.as_le_slice()),
				U256::from_little_endian(tuple.2.as_le_slice()),
			)
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			(
				self.0.to_sol_type().tokenize(),
				self.1.to_sol_type().tokenize(),
				self.2.to_sol_type().tokenize(),
			)
		}
	}

	/// Indicates an error related to the current `balance` of a `sender`.
	pub struct ERC20InsufficientBalance(pub Address, pub U256, pub U256);
	impl SolError for ERC20InsufficientBalance {
		type Parameters<'a> = (SolAddress, Uint<256>, Uint<256>);
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [228, 80, 211, 140];
		const SIGNATURE: &'static str = "ERC20InsufficientBalance(address,uint256,uint256)";

		#[inline]
		fn new<'a>(tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self(
				Address::from(*tuple.0 .0),
				U256::from_little_endian(tuple.1.as_le_slice()),
				U256::from_little_endian(tuple.2.as_le_slice()),
			)
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			(
				self.0.to_sol_type().tokenize(),
				self.1.to_sol_type().tokenize(),
				self.2.to_sol_type().tokenize(),
			)
		}
	}

	/// Indicates an error related to a specified `value`.
	pub struct ERC20InsufficientValue;
	impl SolError for ERC20InsufficientValue {
		type Parameters<'a> = ();
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [191, 254, 152, 173];
		const SIGNATURE: &'static str = "ERC20InsufficientValue()";

		#[inline]
		fn new<'a>(_tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			()
		}
	}
	impl From<ERC20InsufficientValue> for Error {
		fn from(value: ERC20InsufficientValue) -> Self {
			Self::InsufficientValue(value)
		}
	}

	/// Indicates a failure with the `approver` of a token to be approved.
	pub struct ERC20InvalidApprover(pub Address);
	impl SolError for ERC20InvalidApprover {
		type Parameters<'a> = (SolAddress,);
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [230, 2, 223, 5];
		const SIGNATURE: &'static str = "ERC20InvalidApprover(address)";

		#[inline]
		fn new<'a>(tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self((*tuple.0 .0).into())
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			(self.0.to_sol_type().tokenize(),)
		}
	}
	impl From<ERC20InvalidApprover> for Error {
		fn from(value: ERC20InvalidApprover) -> Self {
			Self::InvalidApprover(value)
		}
	}

	/// Indicates a failure with the token `sender`.
	pub struct ERC20InvalidSender(pub Address);
	impl SolError for ERC20InvalidSender {
		type Parameters<'a> = (SolAddress,);
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [150, 198, 253, 30];
		const SIGNATURE: &'static str = "ERC20InvalidSender(address)";

		#[inline]
		fn new<'a>(tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self((*tuple.0 .0).into())
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			(self.0.to_sol_type().tokenize(),)
		}
	}
	impl From<ERC20InvalidSender> for Error {
		fn from(value: ERC20InvalidSender) -> Self {
			Self::InvalidSender(value)
		}
	}

	/// Indicates a failure with the token `receiver`.
	pub struct ERC20InvalidReceiver(pub Address);
	impl SolError for ERC20InvalidReceiver {
		type Parameters<'a> = (SolAddress,);
		type Token<'a> = <Self::Parameters<'a> as SolType>::Token<'a>;

		const SELECTOR: [u8; 4] = [236u8, 68u8, 47u8, 5u8];
		const SIGNATURE: &'static str = "ERC20InvalidReceiver(address)";

		#[inline]
		fn new<'a>(tuple: <Self::Parameters<'a> as SolType>::RustType) -> Self {
			Self((*tuple.0 .0).into())
		}

		#[inline]
		fn tokenize(&self) -> Self::Token<'_> {
			(self.0.to_sol_type().tokenize(),)
		}
	}

	impl<'a> SolEncode<'a> for ERC20InvalidReceiver {
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
	impl From<ERC20InvalidReceiver> for Error {
		fn from(value: ERC20InvalidReceiver) -> Self {
			Self::InvalidReceiver(value)
		}
	}

	#[test]
	fn error_encoding_works() {
		for (result, expected) in [
		    (
				ERC20InsufficientAllowance([255u8; 20].into(), U256::MAX, U256::MAX).abi_encode(),
				"fb8f41b2000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
			),
			(
			    ERC20InsufficientBalance([255u8; 20].into(), U256::MAX, U256::MAX).abi_encode(),
				"e450d38c000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
			),
			(ERC20InsufficientValue.abi_encode(),"bffe98ad"),
			(
			    ERC20InvalidApprover([255u8; 20].into()).abi_encode(),
				"e602df05000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"
			),
			(
			    ERC20InvalidReceiver([255u8; 20].into()).abi_encode(),
				"ec442f05000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"
			),
			(
			    ERC20InvalidSender([255u8; 20].into()).abi_encode(),
				"96c6fd1e000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"
			),
		] {
		    assert_eq!(hex::encode(result), expected)
		}
	}
}

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
	env::{
		call::{build_call_solidity, ExecutionInput},
		DefaultEnvironment,
	},
	prelude::vec::Vec,
	Address, U256,
};

/// APIs for fungible tokens.
#[cfg(feature = "fungibles")]
pub mod fungibles {
	use super::*;

	const PRECOMPILE: u16 = 100;

	pub type TokenId = u32;

	/// Create a new token with an automatically generated identifier.
	#[inline]
	pub fn create(admin: Address, min_balance: U256) -> TokenId {
		let address = fixed_address(PRECOMPILE);

		// GOAL:
		// let precompile: ink::contract_ref!(Fungibles, DefaultEnvironment) = address.into();
		// precompile.create(admin, min_balance)

		// WORKAROUND:
		let selector = 0x0ecaea73_u32.to_be_bytes().into();
		build_call_solidity::<DefaultEnvironment>()
			.call(address)
			.exec_input(ExecutionInput::new(selector).push_arg(admin).push_arg(min_balance))
			.returns::<TokenId>()
			.invoke()
	}

	/// Whether a specified token exists.
	#[inline]
	pub fn exists(id: TokenId) -> bool {
		let address = fixed_address(PRECOMPILE);

		// GOAL:
		// let precompile: ink::contract_ref!(Fungibles, DefaultEnvironment) = address.into();
		// precompile.exists(id)

		// WORKAROUND:
		let selector = 0x13c369ed_u32.to_be_bytes().into();
		build_call_solidity::<DefaultEnvironment>()
			.call(address)
			.exec_input(ExecutionInput::new(selector).push_arg(id))
			.returns::<bool>()
			.invoke()
	}

	/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
	/// supply.
	#[inline]
	pub fn mint(id: TokenId, account: Address, value: U256) {
		let address = fixed_address(PRECOMPILE);

		// GOAL:
		// let precompile: ink::contract_ref!(Fungibles, DefaultEnvironment) = address.into();
		// precompile.mint(id, address, value)

		// WORKAROUND:
		let selector = 0x79d9b87b_u32.to_be_bytes().into();
		build_call_solidity::<DefaultEnvironment>()
			.call(address)
			.exec_input(
				ExecutionInput::new(selector).push_arg(id).push_arg(account).push_arg(value),
			)
			.returns::<()>()
			.invoke()
	}

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
		fn set_metadata(&self, id: TokenId, name: Vec<u8>, symbol: Vec<u8>, decimals: u8);

		/// Clear the metadata for a token.
		#[ink(message)]
		fn clear_metadata(&self, id: TokenId);

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
		fn transfer_from(&self, id: TokenId, from: Address, to: Address, value: U256);

		/// Destroys `value` amount of tokens from `account`, reducing the total supply.
		#[ink(message)]
		fn burn(&self, id: TokenId, address: Address, value: U256);

		/// Start the process of destroying a token.
		#[ink(message)]
		fn start_destroy(&self, id: TokenId) -> bool;

		///  Whether a specified token exists.
		#[ink(message)]
		fn exists(&self, id: TokenId) -> bool;
	}

	/// APIs for fungible tokens conforming to the ERC20 standard.
	pub mod erc20 {
		use super::*;

		const PRECOMPILE: u16 = 101;

		/// Returns the value of tokens in existence.
		#[inline]
		pub fn total_supply(id: TokenId) -> U256 {
			let address = prefixed_address(PRECOMPILE, id);
			let selector = 0x18160ddd_u32.to_be_bytes().into();
			build_call_solidity::<DefaultEnvironment>()
				.call(address)
				.exec_input(ExecutionInput::new(selector))
				.returns::<U256>()
				.invoke()
		}

		/// Returns the value of tokens owned by `account`.
		#[inline]
		pub fn balance_of(id: TokenId, account: Address) -> U256 {
			let address = prefixed_address(PRECOMPILE, id);
			let selector = 0x70a08231_u32.to_be_bytes().into();
			build_call_solidity::<DefaultEnvironment>()
				.call(address)
				.exec_input(ExecutionInput::new(selector).push_arg(account))
				.returns::<U256>()
				.invoke()
		}

		/// Returns the remaining number of tokens that `spender` will be allowed to spend
		/// on behalf of `owner` through [`transfer_from`]. This is zero by default.
		///
		/// This value changes when `approve` or [`transfer_from`] are called.
		#[inline]
		pub fn allowance(id: TokenId, owner: Address, spender: Address) -> U256 {
			let address = prefixed_address(PRECOMPILE, id);
			let selector = 0xdd62ed3e_u32.to_be_bytes().into();
			build_call_solidity::<DefaultEnvironment>()
				.call(address)
				.exec_input(ExecutionInput::new(selector).push_arg(owner).push_arg(spender))
				.returns::<U256>()
				.invoke()
		}

		/// Returns the value of tokens owned by `account`.
		#[inline]
		pub fn transfer(id: TokenId, to: Address, value: U256) -> bool {
			let address = prefixed_address(PRECOMPILE, id);
			let selector = 0x70a08231_u32.to_be_bytes().into();
			build_call_solidity::<DefaultEnvironment>()
				.call(address)
				.exec_input(ExecutionInput::new(selector).push_arg(to).push_arg(value))
				.returns::<bool>()
				.invoke()
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

			/// Returns the remaining number of tokens that `spender` will be allowed to spend
			/// on behalf of `owner` through [`transfer_from`]. This is zero by default.
			///
			/// This value changes when `approve` or `[`transfer_from`] are called.
			#[ink(message)]
			fn allowance(&self, owner: Address, spender: Address) -> U256;

			/// Moves a `value` amount of tokens from the caller's account to `to`. Returns a
			/// boolean value indicating whether the operation succeeded. Emits a {Transfer}
			/// event.
			#[ink(message)]
			fn transfer(&self, to: Address, value: U256) -> bool;
		}
	}
}

fn fixed_address(n: u16) -> Address {
	let shifted = (n as u32) << 16;

	let suffix = shifted.to_be_bytes();
	let mut address = [0u8; 20];
	let mut i = 16;
	while i < address.len() {
		address[i] = suffix[i - 16];
		i = i + 1;
	}
	address.into()
}

fn prefixed_address(n: u16, prefix: u32) -> Address {
	let mut address = fixed_address(n);
	address.0[..4].copy_from_slice(&prefix.to_be_bytes());
	address
}

#[test]
fn fixed_address_works() {
	assert_eq!(hex::encode(fixed_address(100)), "0000000000000000000000000000000000640000")
}

#[test]
fn prefixed_address_works() {
	assert_eq!(
		hex::encode(prefixed_address(101, u32::MAX)),
		"ffffffff00000000000000000000000000650000"
	);
}

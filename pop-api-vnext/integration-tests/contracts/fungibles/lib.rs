#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{prelude::string::String, U256};
use pop_api::fungibles::{self as api, events::*, Fungibles, TokenId};

#[ink::contract]
pub mod fungibles {

	use pop_api::revert;

	use super::*;

	#[ink(storage)]
	pub struct Fungible;

	impl Fungible {
		#[ink(constructor, payable)]
		#[allow(clippy::new_without_default)]
		pub fn new() -> Self {
			Self {}
		}
	}

	impl Fungibles for Fungible {
		#[ink(message)]
		fn transfer(&self, token: TokenId, to: Address, value: U256) {
			if let Err(error) = api::transfer(token, to, value) {
				revert(&error)
			}
			self.env().emit_event(Transfer { from: self.env().address(), to, value });
		}

		#[ink(message)]
		fn transferFrom(&self, token: TokenId, from: Address, to: Address, value: U256) {
			if let Err(error) = api::transfer_from(token, from, to, value) {
				revert(&error)
			}
			self.env().emit_event(Transfer { from, to, value });
		}

		#[ink(message)]
		fn approve(&self, token: TokenId, spender: Address, value: U256) {
			if let Err(error) = api::approve(token, spender, value) {
				revert(&error)
			}
			self.env().emit_event(Approval { owner: self.env().address(), spender, value });
		}

		#[ink(message)]
		fn increaseAllowance(&self, token: TokenId, spender: Address, value: U256) -> U256 {
			match api::increase_allowance(token, spender, value) {
				Ok(allowance) => allowance,
				Err(error) => revert(&error),
			}
		}

		#[ink(message)]
		fn decreaseAllowance(&self, token: TokenId, spender: Address, value: U256) -> U256 {
			match api::decrease_allowance(token, spender, value) {
				Ok(allowance) => allowance,
				Err(error) => revert(&error),
			}
		}

		#[ink(message)]
		fn create(&self, admin: Address, min_balance: U256) -> TokenId {
			match api::create(admin, min_balance) {
				Ok(token) => {
					self.env().emit_event(Created {
						id: token,
						creator: self.env().address(),
						admin,
					});
					token
				},
				Err(error) => revert(&error),
			}
		}

		#[ink(message)]
		fn startDestroy(&self, token: TokenId) {
			api::start_destroy(token);
			self.env().emit_event(DestroyStarted { token });
		}

		#[ink(message)]
		fn setMetadata(&self, token: TokenId, name: String, symbol: String, decimals: u8) {
			api::set_metadata(token, name, symbol, decimals);
		}

		#[ink(message)]
		fn clearMetadata(&self, token: TokenId) {
			api::clear_metadata(token);
		}

		#[ink(message)]
		fn mint(&self, token: TokenId, account: Address, value: U256) {
			if let Err(error) = api::mint(token, account, value) {
				revert(&error)
			}
		}

		#[ink(message)]
		fn burn(&self, token: TokenId, account: Address, value: U256) {
			if let Err(error) = api::burn(token, account, value) {
				revert(&error)
			}
		}

		#[ink(message)]
		fn totalSupply(&self, token: TokenId) -> U256 {
			api::total_supply(token)
		}

		#[ink(message)]
		fn balanceOf(&self, token: TokenId, owner: Address) -> U256 {
			api::balance_of(token, owner)
		}

		#[ink(message)]
		fn allowance(&self, token: TokenId, owner: Address, spender: Address) -> U256 {
			api::allowance(token, owner, spender)
		}

		#[ink(message)]
		fn name(&self, token: TokenId) -> String {
			api::name(token)
		}

		#[ink(message)]
		fn symbol(&self, token: TokenId) -> String {
			api::symbol(token)
		}

		#[ink(message)]
		fn decimals(&self, token: TokenId) -> u8 {
			api::decimals(token)
		}

		#[ink(message)]
		fn exists(&self, token: TokenId) -> bool {
			api::exists(token)
		}
	}
}

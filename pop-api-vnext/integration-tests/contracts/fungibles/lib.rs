#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{prelude::string::String, U256};
use pop_api::fungibles::{self as api, *};

#[ink::contract]
pub mod fungibles {

	use super::*;

	#[ink(storage)]
	pub struct Fungible;

	impl Fungible {
		#[ink(constructor, default, payable)]
		#[allow(clippy::new_without_default)]
		pub fn new() -> Self {
			Self {}
		}

		#[ink(constructor, payable)]
		pub fn create(min_balance: U256) -> Result<Self, Error> {
			let contract = Self {};
			// Account of the contract which will be set to the owner of the fungible token.
			let owner = contract.env().address();
			let id = api::create(owner, min_balance)?;
			contract.env().emit_event(Created {
				id,
				creator: contract.env().caller(),
				admin: owner,
			});
			Ok(contract)
		}
	}

	impl Fungibles for Fungible {
		#[ink(message)]
		fn transfer(&self, token: TokenId, to: Address, value: U256) -> Result<(), Error> {
			api::transfer(token, to, value)?;
			self.env().emit_event(Transfer { from: self.env().address(), to, value });
			Ok(())
		}

		#[ink(message)]
		fn transferFrom(
			&self,
			token: TokenId,
			from: Address,
			to: Address,
			value: U256,
		) -> Result<(), Error> {
			api::transfer_from(token, from, to, value)?;
			self.env().emit_event(Transfer { from, to, value });
			Ok(())
		}

		#[ink(message)]
		fn approve(&self, token: TokenId, spender: Address, value: U256) -> Result<(), Error> {
			api::approve(token, spender, value)?;
			self.env().emit_event(Approval { owner: self.env().address(), spender, value });
			Ok(())
		}

		#[ink(message)]
		fn increaseAllowance(
			&self,
			token: TokenId,
			spender: Address,
			value: U256,
		) -> Result<U256, Error> {
			api::increase_allowance(token, spender, value)
		}

		#[ink(message)]
		fn decreaseAllowance(
			&self,
			token: TokenId,
			spender: Address,
			value: U256,
		) -> Result<U256, Error> {
			api::decrease_allowance(token, spender, value)
		}

		#[ink(message)]
		fn create(&self, admin: Address, min_balance: U256) -> Result<TokenId, Error> {
			let token = api::create(admin, min_balance)?;
			self.env()
				.emit_event(Created { id: token, creator: self.env().address(), admin });
			Ok(token)
		}

		#[ink(message)]
		fn startDestroy(&self, token: TokenId) {
			api::start_destroy(token);
			self.env().emit_event(DestroyStarted { token });
		}

		#[ink(message)]
		fn setMetadata(&self, token: TokenId, name: String, symbol: String, decimals: u8) {
			api::set_metadata(token, name.clone(), symbol.clone(), decimals);
			self.env().emit_event(MetadataSet { token, name, symbol, decimals });
		}

		#[ink(message)]
		fn clearMetadata(&self, token: TokenId) {
			api::clear_metadata(token);
			self.env().emit_event(MetadataCleared { token });
		}

		#[ink(message)]
		fn mint(&self, token: TokenId, account: Address, value: U256) -> Result<(), Error> {
			api::mint(token, account, value)
		}

		#[ink(message)]
		fn burn(&self, token: TokenId, account: Address, value: U256) -> Result<(), Error> {
			api::burn(token, account, value)
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

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{prelude::string::String, U256};
use pop_api::{
	fungibles::{
		self as api,
		erc20::{extensions::Erc20Metadata, Erc20},
		*,
	},
	revert,
};

// NOTE: requires `cargo-contract` built from `master`

#[ink::contract]
pub mod fungibles {
	use super::*;

	#[ink(storage)]
	pub struct Fungible {
		id: TokenId,
		owner: Address,
	}

	impl Fungible {
		/// Instantiate the contract and create a new token. The token identifier will be stored
		/// in contract's storage.
		///
		/// # Parameters
		/// - `name` - The name of the token.
		/// - `symbol` - The symbol of the token.
		/// - `min_balance` - The minimum balance required for accounts holding this token.
		/// - `decimals` - The number of decimals.
		///
		/// NOTE: The minimum balance must be non-zero.
		// The `min_balance` ensures accounts hold a minimum amount of tokens, preventing tiny,
		// inactive balances from bloating the blockchain state and slowing down the network.
		#[ink(constructor, payable)]
		#[allow(clippy::new_without_default)]
		pub fn new(name: String, symbol: String, min_balance: U256, decimals: u8) -> Self {
			let mut instance = Self { id: 0, owner: Self::env().caller() };
			match api::create(instance.env().address(), min_balance) {
				Ok(id) => instance.id = id,
				Err(error) => revert(&error),
			}
			api::set_metadata(instance.id, name, symbol, decimals);
			instance
		}

		/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
		/// supply.
		///
		/// # Parameters
		/// - `account` - The account to be credited with the created tokens.
		/// - `value` - The number of tokens to mint.
		#[ink(message)]
		pub fn mint(&mut self, account: Address, value: U256) {
			if let Err(error) = self.ensure_owner() {
				// TODO: Workaround until ink supports Error to Solidity custom error conversion; https://github.com/use-ink/ink/issues/2404
				revert(&error)
			}

			if let Err(error) = api::mint(self.id, account, value) {
				revert(&error)
			}
			self.env().emit_event(Transfer { from: Address::zero(), to: account, value });
		}

		/// Increases the allowance of `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to increase the allowance by.
		#[ink(message)]
		pub fn increase_allowance(&mut self, spender: Address, value: U256) {
			if let Err(e) = self.ensure_owner() {
				revert(&e)
			}
			let contract = self.env().address();

			// Validate recipient.
			if spender == contract {
				revert(&InvalidRecipient(spender))
			}
			if let Err(error) = api::increase_allowance(self.id, spender, value) {
				revert(&error)
			}
			let allowance = self.allowance(contract, spender);
			self.env().emit_event(Approval { owner: contract, spender, value: allowance });
		}

		/// Decreases the allowance of `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to decrease the allowance by.
		#[ink(message)]
		pub fn decrease_allowance(&mut self, spender: Address, value: U256) {
			if let Err(e) = self.ensure_owner() {
				revert(&e)
			}
			let contract = self.env().address();

			// Validate recipient.
			if spender == contract {
				revert(&InvalidRecipient(spender))
			}
			if let Err(error) = api::decrease_allowance(self.id, spender, value) {
				revert(&error)
			}
			let value = self.allowance(contract, spender);
			self.env().emit_event(Approval { owner: contract, spender, value });
		}

		/// Destroys `value` amount of tokens from `account`, reducing the total supply.
		///
		/// # Parameters
		/// - `account` - The account from which the tokens will be destroyed.
		/// - `value` - The number of tokens to destroy.
		#[ink(message)]
		pub fn burn(&mut self, account: Address, value: U256) {
			if let Err(e) = self.ensure_owner() {
				revert(&e)
			}

			if let Err(error) = api::burn(self.id, account, value) {
				revert(&error)
			}
			self.env().emit_event(Transfer { from: account, to: Address::zero(), value });
		}

		/// Transfer the ownership of the contract to another account.
		///
		/// # Parameters
		/// - `owner` - New owner account.
		///
		/// NOTE: the specified owner account is not checked, allowing the zero address to be
		/// specified if desired..
		#[ink(message)]
		pub fn transfer_ownership(&mut self, owner: Address) {
			if let Err(e) = self.ensure_owner() {
				revert(&e)
			}
			self.owner = owner;
		}

		/// Check if the caller is the owner of the contract.
		fn ensure_owner(&self) -> Result<(), NoPermission> {
			if self.owner != self.env().caller() {
				return Err(NoPermission);
			}
			Ok(())
		}
	}

	impl Erc20 for Fungible {
		/// Returns the total token supply.
		#[ink(message)]
		fn totalSupply(&self) -> U256 {
			api::total_supply(self.id)
		}

		/// Returns the account balance for the specified `owner`.
		///
		/// # Parameters
		/// - `owner` - The account whose balance is being queried.
		#[ink(message)]
		fn balanceOf(&self, owner: Address) -> U256 {
			api::balance_of(self.id, owner)
		}

		/// Returns the allowance for a `spender` approved by an `owner`.
		///
		/// # Parameters
		/// - `owner` - The account that owns the tokens.
		/// - `spender` - The account that is allowed to spend the tokens.
		#[ink(message)]
		fn allowance(&self, owner: Address, spender: Address) -> U256 {
			api::allowance(self.id, owner, spender)
		}

		/// Transfers `value` amount of tokens from the contract to account `to` with
		/// additional `data` in unspecified format.
		///
		/// # Parameters
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[ink(message)]
		fn transfer(&mut self, to: Address, value: U256) -> bool {
			if let Err(error) = self.ensure_owner() {
				revert(&error)
			}
			let contract = self.env().address();

			// Validate recipient.
			if to == contract {
				revert(&InvalidRecipient(to))
			}

			if let Err(error) = api::transfer(self.id, to, value) {
				revert(&error)
			}
			self.env().emit_event(Transfer { from: contract, to, value });
			true
		}

		/// Transfers `value` tokens on behalf of `from` to the account `to`. Contract must be
		/// pre-approved by `from`.
		///
		/// # Parameters
		/// - `from` - The account from which the token balance will be withdrawn.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[ink(message)]
		fn transferFrom(&mut self, from: Address, to: Address, value: U256) -> bool {
			if let Err(e) = self.ensure_owner() {
				revert(&e)
			}
			let contract = self.env().address();

			// A successful transfer reduces the allowance from `from` to the contract and triggers
			// an `Approval` event with the updated allowance amount.
			if let Err(error) = api::transfer_from(self.id, from, to, value) {
				revert(&error)
			}
			self.env().emit_event(Transfer { from: contract, to, value });
			self.env().emit_event(Approval {
				owner: from,
				spender: contract,
				value: self.allowance(from, contract),
			});
			true
		}

		/// Approves `spender` to spend `value` amount of tokens on behalf of the contract.
		///
		/// Successive calls of this method overwrite previous values.
		///
		/// # Parameters
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to approve.
		#[ink(message)]
		fn approve(&mut self, spender: Address, value: U256) -> bool {
			if let Err(e) = self.ensure_owner() {
				revert(&e)
			}
			let contract = self.env().address();

			// Validate recipient.
			if spender == contract {
				revert(&InvalidRecipient(spender))
			}
			if let Err(error) = api::approve(self.id, spender, value) {
				revert(&error)
			}
			self.env().emit_event(Approval { owner: contract, spender, value });
			true
		}
	}

	impl Erc20Metadata for Fungible {
		/// Returns the token name.
		#[ink(message)]
		fn name(&self) -> String {
			api::name(self.id)
		}

		/// Returns the token symbol.
		#[ink(message)]
		fn symbol(&self) -> String {
			api::symbol(self.id)
		}

		/// Returns the token decimals.
		#[ink(message)]
		fn decimals(&self) -> u8 {
			api::decimals(self.id)
		}
	}
}

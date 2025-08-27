#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{prelude::string::String, U256};
use pop_api::{
	ensure,
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
		pub fn new(
			name: String,
			symbol: String,
			min_balance: U256,
			decimals: u8,
		) -> Result<Self, Error> {
			let mut instance = Self { id: 0, owner: Self::env().caller() };
			instance.id = api::create(instance.env().address(), min_balance)?;
			api::set_metadata(instance.id, name, symbol, decimals)?;
			Ok(instance)
		}

		/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
		/// supply.
		///
		/// # Parameters
		/// - `account` - The account to be credited with the created tokens.
		/// - `value` - The number of tokens to mint.
		#[ink(message)]
		pub fn mint(&mut self, account: Address, value: U256) -> Result<(), Error> {
			self.ensure_owner()?;
			api::mint(self.id, account, value)?;
			self.env().emit_event(Transfer { from: Address::zero(), to: account, value });
			Ok(())
		}

		/// Increases the allowance of `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to increase the allowance by.
		#[ink(message)]
		pub fn increase_allowance(&mut self, spender: Address, value: U256) -> Result<(), Error> {
			self.ensure_owner()?;
			let contract = self.env().address();

			// Validate recipient.
			ensure!(spender != contract, InvalidRecipient(spender));
			api::increase_allowance(self.id, spender, value)?;
			let allowance = self.allowance(contract, spender);
			self.env().emit_event(Approval { owner: contract, spender, value: allowance });
			Ok(())
		}

		/// Decreases the allowance of `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to decrease the allowance by.
		#[ink(message)]
		pub fn decrease_allowance(&mut self, spender: Address, value: U256) -> Result<(), Error> {
			self.ensure_owner()?;
			let contract = self.env().address();

			// Validate recipient.
			ensure!(spender != contract, InvalidRecipient(spender));
			api::decrease_allowance(self.id, spender, value)?;
			let value = self.allowance(contract, spender);
			self.env().emit_event(Approval { owner: contract, spender, value });
			Ok(())
		}

		/// Destroys `value` amount of tokens from `account`, reducing the total supply.
		///
		/// # Parameters
		/// - `account` - The account from which the tokens will be destroyed.
		/// - `value` - The number of tokens to destroy.
		#[ink(message)]
		pub fn burn(&mut self, account: Address, value: U256) -> Result<(), Error> {
			self.ensure_owner()?;
			api::burn(self.id, account, value)?;
			self.env().emit_event(Transfer { from: account, to: Address::zero(), value });
			Ok(())
		}

		/// Transfer the ownership of the contract to another account.
		///
		/// # Parameters
		/// - `owner` - New owner account.
		///
		/// NOTE: the specified owner account is not checked, allowing the zero address to be
		/// specified if desired..
		#[ink(message)]
		pub fn transfer_ownership(&mut self, owner: Address) -> Result<(), Error> {
			self.ensure_owner()?;
			self.owner = owner;
			Ok(())
		}

		/// Check if the caller is the owner of the contract.
		fn ensure_owner(&self) -> Result<(), Error> {
			ensure!(self.env().caller() == self.owner, NoPermission);
			Ok(())
		}
	}

	impl Erc20 for Fungible {
		/// Returns the total token supply.
		#[ink(message)]
		fn totalSupply(&self) -> U256 {
			erc20::total_supply(self.id)
		}

		/// Returns the account balance for the specified `owner`.
		///
		/// # Parameters
		/// - `owner` - The account whose balance is being queried.
		#[ink(message)]
		fn balanceOf(&self, owner: Address) -> U256 {
			erc20::balance_of(self.id, owner)
		}

		/// Returns the allowance for a `spender` approved by an `owner`.
		///
		/// # Parameters
		/// - `owner` - The account that owns the tokens.
		/// - `spender` - The account that is allowed to spend the tokens.
		#[ink(message)]
		fn allowance(&self, owner: Address, spender: Address) -> U256 {
			erc20::allowance(self.id, owner, spender)
		}

		/// Transfers `value` amount of tokens from the contract to account `to` with
		/// additional `data` in unspecified format.
		///
		/// # Parameters
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[ink(message)]
		fn transfer(&mut self, to: Address, value: U256) -> Result<bool, erc20::Error> {
			if let Err(error) = self.ensure_owner() {
				revert(&error)
			}
			let contract = self.env().address();

			// Validate recipient.
			if to == contract {
				revert(&InvalidRecipient(to))
			}

			erc20::transfer(self.id, to, value)?;
			self.env().emit_event(Transfer { from: contract, to, value });
			Ok(true)
		}

		/// Transfers `value` tokens on behalf of `from` to the account `to`. Contract must be
		/// pre-approved by `from`.
		///
		/// # Parameters
		/// - `from` - The account from which the token balance will be withdrawn.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[ink(message)]
		fn transferFrom(
			&mut self,
			from: Address,
			to: Address,
			value: U256,
		) -> Result<bool, erc20::Error> {
			if let Err(error) = self.ensure_owner() {
				revert(&error)
			}
			let contract = self.env().address();

			// A successful transfer reduces the allowance from `from` to the contract and triggers
			// an `Approval` event with the updated allowance amount.
			erc20::transfer_from(self.id, from, to, value)?;
			self.env().emit_event(Transfer { from: contract, to, value });
			self.env().emit_event(Approval {
				owner: from,
				spender: contract,
				value: self.allowance(from, contract),
			});
			Ok(true)
		}

		/// Approves `spender` to spend `value` amount of tokens on behalf of the contract.
		///
		/// Successive calls of this method overwrite previous values.
		///
		/// # Parameters
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to approve.
		#[ink(message)]
		fn approve(&mut self, spender: Address, value: U256) -> Result<bool, erc20::Error> {
			if let Err(error) = self.ensure_owner() {
				revert(&error)
			}
			let contract = self.env().address();

			// Validate recipient.
			if spender == contract {
				revert(&InvalidRecipient(spender));
			}
			erc20::approve(self.id, spender, value)?;
			self.env().emit_event(Approval { owner: contract, spender, value });
			Ok(true)
		}
	}

	impl Erc20Metadata for Fungible {
		/// Returns the token name.
		#[ink(message)]
		fn name(&self) -> String {
			erc20::extensions::name(self.id)
		}

		/// Returns the token symbol.
		#[ink(message)]
		fn symbol(&self) -> String {
			erc20::extensions::symbol(self.id)
		}

		/// Returns the token decimals.
		#[ink(message)]
		fn decimals(&self) -> u8 {
			erc20::extensions::decimals(self.id)
		}
	}
}

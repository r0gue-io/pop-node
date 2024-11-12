#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::{string::String, vec::Vec};
use pop_api::{
	primitives::TokenId,
	v0::fungibles::{
		self as api,
		events::{Approval, Created, Transfer},
		traits::{Psp22, Psp22Burnable, Psp22Metadata, Psp22Mintable},
		Psp22Error,
	},
};

#[cfg(test)]
mod tests;

#[ink::contract]
mod fungibles {
	use super::*;

	#[ink(storage)]
	pub struct Fungible {
		id: TokenId,
		owner: AccountId,
	}

	impl Fungible {
		/// Instantiate the contract and create a new token. The token identifier will be stored
		/// in contract's storage.
		///
		/// # Parameters
		/// * - `id` - The identifier of the token.
		/// * - `min_balance` - The minimum balance required for accounts holding this token.
		// The `min_balance` ensures accounts hold a minimum amount of tokens, preventing tiny,
		// inactive balances from bloating the blockchain state and slowing down the network.
		#[ink(constructor, payable)]
		pub fn new(id: TokenId, min_balance: Balance) -> Result<Self, Psp22Error> {
			let instance = Self { id, owner: Self::env().caller() };
			let contract_id = instance.env().account_id();
			api::create(id, contract_id, min_balance).map_err(Psp22Error::from)?;
			instance
				.env()
				.emit_event(Created { id, creator: contract_id, admin: contract_id });
			Ok(instance)
		}
	}

	impl Psp22 for Fungible {
		/// Returns the total token supply.
		#[ink(message)]
		fn total_supply(&self) -> Balance {
			api::total_supply(self.id).unwrap_or_default()
		}

		/// Returns the account balance for the specified `owner`.
		///
		/// # Parameters
		/// - `owner` - The account whose balance is being queried.
		#[ink(message)]
		fn balance_of(&self, owner: AccountId) -> Balance {
			api::balance_of(self.id, owner).unwrap_or_default()
		}

		/// Returns the allowance for a `spender` approved by an `owner`.
		///
		/// # Parameters
		/// - `owner` - The account that owns the tokens.
		/// - `spender` - The account that is allowed to spend the tokens.
		#[ink(message)]
		fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
			api::allowance(self.id, owner, spender).unwrap_or_default()
		}

		/// Transfers `value` amount of tokens from the contract to account `to` with
		/// additional `data` in unspecified format. Contract must be pre-approved by `from`.
		///
		/// # Parameters
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		/// - `data` - Additional data in unspecified format.
		#[ink(message)]
		fn transfer(
			&mut self,
			to: AccountId,
			value: Balance,
			_data: Vec<u8>,
		) -> Result<(), Psp22Error> {
			self.ensure_owner()?;
			let contract = self.env().account_id();

			// No-op if the contract and `to` is the same address or `value` is zero.
			if contract == to || value == 0 {
				return Ok(());
			}
			api::transfer(self.id, to, value).map_err(Psp22Error::from)?;
			self.env().emit_event(Transfer { from: Some(contract), to: Some(to), value });
			Ok(())
		}

		/// Transfers `value` tokens on behalf of `from` to the account `to`
		/// with additional `data` in unspecified format. Contract must be pre-approved by `from`.
		///
		/// # Parameters
		/// - `from` - The account from which the token balance will be withdrawn.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		/// - `data` - Additional data with unspecified format.
		#[ink(message)]
		fn transfer_from(
			&mut self,
			from: AccountId,
			to: AccountId,
			value: Balance,
			_data: Vec<u8>,
		) -> Result<(), Psp22Error> {
			self.ensure_owner()?;
			let contract = self.env().account_id();

			// No-op if `from` and `to` is the same address or `value` is zero.
			if from == to || value == 0 {
				return Ok(());
			}
			// A successful transfer reduces the allowance from `from` to the contract and triggers
			// an `Approval` event with the updated allowance amount.
			api::transfer_from(self.id, from, to, value).map_err(Psp22Error::from)?;
			self.env().emit_event(Transfer { from: Some(contract), to: Some(to), value });
			self.env().emit_event(Approval {
				owner: from,
				spender: contract,
				value: self.allowance(from, contract),
			});
			Ok(())
		}

		/// Approves `spender` to spend `value` amount of tokens on behalf of the contract.
		///
		/// Successive calls of this method overwrite previous values.
		///
		/// # Parameters
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to approve.
		#[ink(message)]
		fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), Psp22Error> {
			self.ensure_owner()?;
			let contract = self.env().account_id();

			// No-op if the contract and `spender` is the same address.
			if contract == spender {
				return Ok(());
			}
			api::approve(self.id, spender, value).map_err(Psp22Error::from)?;
			self.env().emit_event(Approval { owner: contract, spender, value });
			Ok(())
		}

		/// Increases the allowance of `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to increase the allowance by.
		#[ink(message)]
		fn increase_allowance(
			&mut self,
			spender: AccountId,
			value: Balance,
		) -> Result<(), Psp22Error> {
			self.ensure_owner()?;
			let contract = self.env().account_id();

			// No-op if the contract and `spender` is the same address or `value` is zero.
			if contract == spender || value == 0 {
				return Ok(());
			}
			api::increase_allowance(self.id, spender, value).map_err(Psp22Error::from)?;
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
		fn decrease_allowance(
			&mut self,
			spender: AccountId,
			value: Balance,
		) -> Result<(), Psp22Error> {
			self.ensure_owner()?;
			let contract = self.env().account_id();

			// No-op if the contract and `spender` is the same address or `value` is zero.
			if contract == spender || value == 0 {
				return Ok(());
			}
			api::decrease_allowance(self.id, spender, value).map_err(Psp22Error::from)?;
			let value = self.allowance(contract, spender);
			self.env().emit_event(Approval { owner: contract, spender, value });
			Ok(())
		}
	}

	impl Psp22Metadata for Fungible {
		/// Returns the token name.
		#[ink(message)]
		fn token_name(&self) -> Option<String> {
			api::token_name(self.id)
				.unwrap_or_default()
				.and_then(|v| String::from_utf8(v).ok())
		}

		/// Returns the token symbol.
		#[ink(message)]
		fn token_symbol(&self) -> Option<String> {
			api::token_symbol(self.id)
				.unwrap_or_default()
				.and_then(|v| String::from_utf8(v).ok())
		}

		/// Returns the token decimals.
		#[ink(message)]
		fn token_decimals(&self) -> u8 {
			api::token_decimals(self.id).unwrap_or_default()
		}
	}

	impl Psp22Mintable for Fungible {
		/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
		/// supply.
		///
		/// # Parameters
		/// - `account` - The account to be credited with the created tokens.
		/// - `value` - The number of tokens to mint.
		#[ink(message)]
		fn mint(&mut self, account: AccountId, value: Balance) -> Result<(), Psp22Error> {
			self.ensure_owner()?;
			// No-op if `value` is zero.
			if value == 0 {
				return Ok(());
			}
			api::mint(self.id, account, value).map_err(Psp22Error::from)?;
			self.env().emit_event(Transfer { from: None, to: Some(account), value });
			Ok(())
		}
	}

	impl Psp22Burnable for Fungible {
		/// Destroys `value` amount of tokens from `account`, reducing the total supply.
		///
		/// # Parameters
		/// - `account` - The account from which the tokens will be destroyed.
		/// - `value` - The number of tokens to destroy.
		#[ink(message)]
		fn burn(&mut self, account: AccountId, value: Balance) -> Result<(), Psp22Error> {
			self.ensure_owner()?;
			// No-op if `value` is zero.
			if value == 0 {
				return Ok(());
			}
			api::burn(self.id, account, value).map_err(Psp22Error::from)?;
			self.env().emit_event(Transfer { from: Some(account), to: None, value });
			Ok(())
		}
	}

	impl Fungible {
		/// Check if the caller is the owner of the contract.
		fn ensure_owner(&self) -> Result<(), Psp22Error> {
			if self.owner != self.env().caller() {
				return Err(Psp22Error::Custom(String::from("Not the owner")));
			}
			Ok(())
		}

		/// Transfer the ownership of the contract to another account.
		///
		/// # Parameters
		/// - `owner` - New owner account.
		#[ink(message)]
		pub fn transfer_ownership(&mut self, owner: AccountId) -> Result<(), Psp22Error> {
			self.ensure_owner()?;
			self.owner = owner;
			Ok(())
		}
	}
}

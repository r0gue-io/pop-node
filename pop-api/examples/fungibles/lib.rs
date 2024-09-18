#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::{string::String, vec::Vec};
use pop_api::{
	primitives::TokenId,
	v0::fungibles::{
		self as api,
		events::Created,
		interfaces::{Psp22, Psp22Burnable, Psp22Metadata, Psp22Mintable},
		PSP22Error, PSP22Result,
	},
	StatusCode,
};

#[cfg(test)]
mod tests;

#[ink::contract]
mod fungibles {
	use super::*;

	#[ink(storage)]
	pub struct Fungibles {
		id: TokenId,
	}

	impl Fungibles {
		/// Instantiate the contract and wrap around an existing token.
		///
		/// # Parameters
		/// * - `token` - The token.
		#[ink(constructor, payable)]
		pub fn new_existing(id: TokenId) -> PSP22Result<Self> {
			// Make sure token exists.
			if !api::token_exists(id).unwrap_or_default() {
				return Err(PSP22Error::Custom(String::from("Unknown")));
			}
			let contract = Self { id };
			Ok(contract)
		}

		/// Instantiate the contract and create a new token. The token identifier will be stored
		/// in contract's storage.
		///
		/// # Parameters
		/// * - `id` - The identifier of the token.
		/// * - `admin` - The account that will administer the token.
		/// * - `min_balance` - The minimum balance required for accounts holding this token.
		#[ink(constructor, payable)]
		pub fn new(id: TokenId, admin: AccountId, min_balance: Balance) -> PSP22Result<Self> {
			// TODO: should be nicer conversion possible.
			api::create(id, admin, min_balance).map_err(|e| PSP22Error::from(e))?;
			let contract = Self { id };
			contract
				.env()
				.emit_event(Created { id, creator: contract.env().account_id(), admin });
			Ok(contract)
		}

		#[ink(message)]
		pub fn set_metadata(
			&self,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> PSP22Result<()> {
			api::set_metadata(self.id, name, symbol, decimals).map_err(|e| PSP22Error::from(e))
		}

		#[ink(message)]
		pub fn token_exists(&self) -> PSP22Result<bool> {
			api::token_exists(self.id).map_err(|e| PSP22Error::from(e))
		}
	}

	impl Psp22 for Fungibles {
		#[ink(message)]
		fn total_supply(&self) -> Balance {
			api::total_supply(self.id).unwrap_or_default()
		}

		#[ink(message)]
		fn balance_of(&self, owner: AccountId) -> Balance {
			api::balance_of(self.id, owner).unwrap_or_default()
		}

		#[ink(message)]
		fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
			api::allowance(self.id, owner, spender).unwrap_or_default()
		}

		#[ink(message)]
		fn transfer(&mut self, to: AccountId, value: Balance, _data: Vec<u8>) -> PSP22Result<()> {
			api::transfer(self.id, to, value).map_err(|e| PSP22Error::from(e))
		}

		#[ink(message)]
		fn transfer_from(
			&mut self,
			from: AccountId,
			to: AccountId,
			value: Balance,
			_data: Vec<u8>,
		) -> PSP22Result<()> {
			api::transfer_from(self.id, from, to, value).map_err(|e| PSP22Error::from(e))
		}

		#[ink(message)]
		fn approve(&mut self, spender: AccountId, value: Balance) -> PSP22Result<()> {
			api::approve(self.id, spender, value).map_err(|e| PSP22Error::from(e))
		}

		#[ink(message)]
		fn increase_allowance(&mut self, spender: AccountId, value: Balance) -> PSP22Result<()> {
			api::increase_allowance(self.id, spender, value).map_err(|e| PSP22Error::from(e))
		}

		#[ink(message)]
		fn decrease_allowance(&mut self, spender: AccountId, value: Balance) -> PSP22Result<()> {
			api::decrease_allowance(self.id, spender, value).map_err(|e| PSP22Error::from(e))
		}
	}

	impl Psp22Metadata for Fungibles {
		#[ink(message)]
		fn token_name(&self) -> Option<Vec<u8>> {
			if let Ok(value) = api::token_name(self.id) {
				return Some(value);
			}
			None
		}

		#[ink(message)]
		fn token_symbol(&self) -> Option<Vec<u8>> {
			if let Ok(value) = api::token_symbol(self.id) {
				return Some(value);
			}
			None
		}

		#[ink(message)]
		fn token_decimals(&self) -> u8 {
			api::token_decimals(self.id).unwrap_or_default()
		}
	}

	impl Psp22Mintable for Fungibles {
		#[ink(message)]
		fn mint(&mut self, account: AccountId, amount: Balance) -> PSP22Result<()> {
			api::mint(self.id, account, amount).map_err(|e| PSP22Error::from(e))
		}
	}

	impl Psp22Burnable for Fungibles {
		#[ink(message)]
		fn burn(&mut self, account: AccountId, amount: Balance) -> PSP22Result<()> {
			api::burn(self.id, account, amount).map_err(|e| PSP22Error::from(e))
		}
	}
}

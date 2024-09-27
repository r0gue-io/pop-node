#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::{string::String, vec::Vec};
use pop_api::{
	primitives::TokenId,
	v0::fungibles::{
		self as api,
		events::{Approval, Created, Transfer},
		traits::{PSP22Burnable, PSP22Metadata, PSP22Mintable, PSP22},
		PSP22Error,
	},
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
		pub fn existing(id: TokenId) -> Result<Self, PSP22Error> {
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
		pub fn new(id: TokenId, min_balance: Balance) -> Result<Self, PSP22Error> {
			let contract = Self { id };
			let contract_id = contract.env().account_id();
			api::create(id, contract_id, min_balance).map_err(PSP22Error::from)?;
			contract
				.env()
				.emit_event(Created { id, creator: contract_id, admin: contract_id });
			Ok(contract)
		}
	}

	impl PSP22 for Fungibles {
		/// Returns the total token supply.
		#[ink(message)]
		fn total_supply(&self) -> Balance {
			api::total_supply(self.id).unwrap_or_default()
		}

		/// Returns the account balance for the specified `owner`
		#[ink(message)]
		fn balance_of(&self, owner: AccountId) -> Balance {
			api::balance_of(self.id, owner).unwrap_or_default()
		}

		/// Returns the amount which `spender` is still allowed to withdraw from `owner`
		#[ink(message)]
		fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
			api::allowance(self.id, owner, spender).unwrap_or_default()
		}

		/// Transfers `value` amount of tokens from the caller's account to account `to`
		/// with additional `data` in unspecified format.
		#[ink(message)]
		fn transfer(
			&mut self,
			to: AccountId,
			value: Balance,
			_data: Vec<u8>,
		) -> Result<(), PSP22Error> {
			let caller = self.env().caller();
			// No-op if the caller and `to` is the same address or `value` is zero, returns success
			// and no events are emitted.
			if caller == to || value == 0 {
				return Ok(());
			}

			api::transfer(self.id, to, value).map_err(PSP22Error::from)?;
			self.env().emit_event(Transfer { from: Some(caller), to: Some(to), value });
			Ok(())
		}

		/// Transfers `value` tokens on the behalf of `from` to the account `to`
		/// with additional `data` in unspecified format.
		#[ink(message)]
		fn transfer_from(
			&mut self,
			from: AccountId,
			to: AccountId,
			value: Balance,
			_data: Vec<u8>,
		) -> Result<(), PSP22Error> {
			let caller = self.env().caller();
			// No-op if `from` and `to` is the same address or `value` is zero, returns success and
			// no events are emitted.
			if from == to || value == 0 {
				return Ok(());
			}

			// If `from` and the caller are different addresses, a successful transfer results
			// in decreased allowance by `from` to the caller and an `Approval` event with
			// the new allowance amount is emitted.
			api::transfer_from(self.id, from, to, value).map_err(PSP22Error::from)?;
			// Emit events.
			self.env().emit_event(Transfer { from: Some(caller), to: Some(to), value });
			self.env().emit_event(Approval {
				owner: from,
				spender: caller,
				value: self.allowance(from, caller),
			});
			Ok(())
		}

		/// Allows `spender` to withdraw from the caller's account multiple times, up to
		/// the total amount of `value`.
		#[ink(message)]
		fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error> {
			let caller = self.env().caller();
			// No-op if the caller and `spender` is the same address, returns success and no events
			// are emitted.
			if caller == spender {
				return Ok(());
			}

			api::approve(self.id, spender, value).map_err(PSP22Error::from)?;
			self.env().emit_event(Approval { owner: caller, spender, value });
			Ok(())
		}

		/// Increases by `value` the allowance granted to `spender` by the caller.
		#[ink(message)]
		fn increase_allowance(
			&mut self,
			spender: AccountId,
			value: Balance,
		) -> Result<(), PSP22Error> {
			let caller = self.env().caller();
			// No-op if the caller and `spender` is the same address or `value` is zero, returns
			// success and no events are emitted.
			if caller == spender || value == 0 {
				return Ok(());
			}

			api::increase_allowance(self.id, spender, value).map_err(PSP22Error::from)?;
			let allowance = self.allowance(caller, spender);
			self.env().emit_event(Approval { owner: caller, spender, value: allowance });
			Ok(())
		}

		/// Decreases by `value` the allowance granted to `spender` by the caller.
		#[ink(message)]
		fn decrease_allowance(
			&mut self,
			spender: AccountId,
			value: Balance,
		) -> Result<(), PSP22Error> {
			let caller = self.env().caller();
			// No-op if the caller and `spender` is the same address or `value` is zero, returns
			// success and no events are emitted.
			if caller == spender || value == 0 {
				return Ok(());
			}

			api::decrease_allowance(self.id, spender, value).map_err(PSP22Error::from)?;
			let allowance = self.allowance(caller, spender);
			self.env().emit_event(Approval { owner: caller, spender, value: allowance });
			Ok(())
		}
	}

	impl PSP22Metadata for Fungibles {
		/// Returns the token name.
		#[ink(message)]
		fn token_name(&self) -> Option<String> {
			api::token_name(self.id)
				.ok()
				.filter(|v| !v.is_empty())
				.and_then(|v| String::from_utf8(v).ok())
		}

		/// Returns the token symbol.
		#[ink(message)]
		fn token_symbol(&self) -> Option<String> {
			api::token_symbol(self.id)
				.ok()
				.filter(|v| !v.is_empty())
				.and_then(|v| String::from_utf8(v).ok())
		}

		/// Returns the token decimals.
		#[ink(message)]
		fn token_decimals(&self) -> u8 {
			api::token_decimals(self.id).unwrap_or_default()
		}
	}

	impl PSP22Mintable for Fungibles {
		/// Mints `value` tokens to the senders account.
		#[ink(message)]
		fn mint(&mut self, account: AccountId, value: Balance) -> Result<(), PSP22Error> {
			if value == 0 {
				return Ok(());
			}
			api::mint(self.id, account, value).map_err(PSP22Error::from)?;
			self.env().emit_event(Transfer { from: None, to: Some(account), value });
			Ok(())
		}
	}

	impl PSP22Burnable for Fungibles {
		/// Burns `value` tokens from the senders account.
		#[ink(message)]
		fn burn(&mut self, account: AccountId, value: Balance) -> Result<(), PSP22Error> {
			if value == 0 {
				return Ok(());
			}
			let balance = self.balance_of(account);
			if balance < value {
				return Err(PSP22Error::InsufficientBalance);
			}
			api::burn(self.id, account, value).map_err(PSP22Error::from)?;
			self.env().emit_event(Transfer { from: Some(account), to: None, value });
			Ok(())
		}
	}
}

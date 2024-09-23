#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::{string::String, vec::Vec};
use pop_api::{
	fungibles::{self as api},
	primitives::TokenId,
	v0::fungibles::{
		self as api,
		events::{Approval, Created, Transfer},
		traits::{Psp22, Psp22Burnable, Psp22Metadata, Psp22Mintable},
		PSP22Error,
	},
};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod utils;

#[ink::contract]
mod fungibles {
	use super::*;

	#[ink(storage)]
	pub struct Fungibles {
		id: TokenId,
	}

	impl Fungibles {
		fn emit_created_event(&mut self, id: u32, creator: AccountId, admin: AccountId) {
			self.env().emit_event(Created { id, creator, admin });
		}

		fn emit_transfer_event(
			&mut self,
			from: Option<AccountId>,
			to: Option<AccountId>,
			value: Balance,
		) {
			self.env().emit_event(Transfer { from, to, value });
		}

		fn emit_approval_event(&mut self, owner: AccountId, spender: AccountId, value: Balance) {
			self.env().emit_event(Approval { owner, spender, value });
		}

		/// Instantiate the contract and wrap around an existing token.
		///
		/// # Parameters
		/// * - `token` - The token.
		#[ink(constructor, payable)]
		pub fn new_existing(id: TokenId) -> Result<Self, PSP22Error> {
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
		pub fn new(
			id: TokenId,
			// TODO: If admin is different than the contract address, `NoPermission` thrown for mint, burn
			// _admin: AccountId,
			min_balance: Balance,
		) -> Result<Self, PSP22Error> {
			let mut contract = Self { id };
			let contract_id = contract.env().account_id();
			api::create(id, contract_id, min_balance).map_err(PSP22Error::from)?;
			contract.emit_created_event(id, contract_id, contract_id);
			Ok(contract)
		}
	}

	impl Psp22 for Fungibles {
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

			// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
			if value > self.balance_of(caller) {
				return Err(PSP22Error::InsufficientBalance);
			}

			api::transfer(self.id, to, value).map_err(PSP22Error::from)?;
			self.emit_transfer_event(Some(caller), Some(to), value);
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

			// Reverts with `InsufficientBalance` if the `value` exceeds the balance of the account
			// `from`.
			let allowance = self.allowance(from, caller);
			if allowance < value {
				return Err(PSP22Error::InsufficientAllowance);
			}

			// Reverts with `InsufficientAllowance` if `from` and the caller are different addresses
			// and the `value` exceeds the allowance granted by `from` to the caller.
			let from_balance = self.balance_of(from);
			if from_balance < value {
				return Err(PSP22Error::InsufficientBalance);
			}

			// If `from` and the caller are different addresses, a successful transfer results
			// in decreased allowance by `from` to the caller and an `Approval` event with
			// the new allowance amount is emitted.
			api::transfer_from(self.id, from, to, value).map_err(PSP22Error::from)?;
			// Emit events.
			self.emit_transfer_event(Some(caller), Some(to), value);
			let allowance = self.allowance(from, to).saturating_sub(value);
			self.emit_approval_event(from, caller, allowance);
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
			self.emit_approval_event(caller, spender, value);
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

			let allowance = self.allowance(caller, spender);
			api::increase_allowance(self.id, spender, value).map_err(PSP22Error::from)?;
			self.emit_approval_event(caller, spender, allowance.saturating_add(value));
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
			// Reverts with `InsufficientAllowance` if `spender` and the caller are different
			// addresses and the `value` exceeds the allowance granted by the caller to
			// `spender`.
			let allowance = self.allowance(caller, spender);
			if allowance < value {
				return Err(PSP22Error::InsufficientAllowance);
			}

			api::decrease_allowance(self.id, spender, value).map_err(PSP22Error::from)?;
			self.emit_approval_event(caller, spender, allowance.saturating_sub(value));
			Ok(())
		}
	}

	impl Psp22Metadata for Fungibles {
		/// Returns the token name.
		#[ink(message)]
		fn token_name(&self) -> Option<String> {
			api::token_name(self.id).ok().and_then(|v| String::from_utf8(v).ok())
		}

		/// Returns the token symbol.
		#[ink(message)]
		fn token_symbol(&self) -> Option<String> {
			api::token_symbol(self.id).ok().and_then(|v| String::from_utf8(v).ok())
		}

		/// Returns the token decimals.
		#[ink(message)]
		fn token_decimals(&self) -> u8 {
			api::token_decimals(self.id).unwrap_or_default()
		}
	}

	impl Psp22Mintable for Fungibles {
		/// Mints `value` tokens to the senders account.
		#[ink(message)]
		fn mint(&mut self, account: AccountId, value: Balance) -> Result<(), PSP22Error> {
			if value == 0 {
				return Ok(());
			}
			api::mint(self.id, account, value).map_err(PSP22Error::from)?;
			self.emit_transfer_event(None, Some(account), value);
			Ok(())
		}
	}

	impl Psp22Burnable for Fungibles {
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
			self.emit_transfer_event(Some(account), None, value);
			Ok(())
		}
	}
}

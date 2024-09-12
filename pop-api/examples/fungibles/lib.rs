#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use pop_api::{
	primitives::TokenId,
	v0::fungibles::{self as api},
	StatusCode,
};

#[cfg(test)]
mod tests;

type PopApiResult<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod fungibles {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Fungibles;

	impl Fungibles {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			Default::default()
		}

		#[ink(message)]
		pub fn mint(
			&mut self,
			token: TokenId,
			account: AccountId,
			amount: Balance,
		) -> PopApiResult<()> {
			api::mint(token, account, amount)
		}

		#[ink(message)]
		pub fn burn(
			&mut self,
			token: TokenId,
			account: AccountId,
			amount: Balance,
		) -> PopApiResult<()> {
			api::burn(token, account, amount)
		}

		#[ink(message)]
		pub fn transfer(&mut self, id: TokenId, to: AccountId, value: Balance) -> PopApiResult<()> {
			api::transfer(id, to, value)
		}

		#[ink(message)]
		pub fn transfer_from(
			&mut self,
			id: TokenId,
			from: AccountId,
			to: AccountId,
			value: Balance,
			// In the PSP-22 standard a `[u8]`, but the size needs to be known at compile time.
			_data: Vec<u8>,
		) -> PopApiResult<()> {
			api::transfer_from(id, from, to, value)
		}

		#[ink(message)]
		pub fn approve(
			&mut self,
			id: TokenId,
			spender: AccountId,
			value: Balance,
		) -> PopApiResult<()> {
			api::approve(id, spender, value)
		}

		#[ink(message)]
		pub fn increase_allowance(
			&mut self,
			id: TokenId,
			spender: AccountId,
			value: Balance,
		) -> PopApiResult<()> {
			api::increase_allowance(id, spender, value)
		}

		#[ink(message)]
		pub fn decrease_allowance(
			&mut self,
			id: TokenId,
			spender: AccountId,
			value: Balance,
		) -> PopApiResult<()> {
			api::decrease_allowance(id, spender, value)
		}

		#[ink(message, payable)]
		pub fn create(
			&self,
			id: TokenId,
			admin: AccountId,
			min_balance: Balance,
		) -> PopApiResult<()> {
			api::create(id, admin, min_balance)?;
			self.env().emit_event(api::events::Create { id, creator: admin, admin });
			Ok(())
		}

		#[ink(message)]
		pub fn set_metadata(
			&self,
			id: TokenId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> PopApiResult<()> {
			api::set_metadata(id, name, symbol, decimals)
		}

		#[ink(message)]
		pub fn total_supply(&self, id: TokenId) -> PopApiResult<Balance> {
			api::total_supply(id)
		}

		#[ink(message)]
		pub fn balance_of(&self, id: TokenId, owner: AccountId) -> PopApiResult<Balance> {
			api::balance_of(id, owner)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			id: TokenId,
			owner: AccountId,
			spender: AccountId,
		) -> PopApiResult<Balance> {
			api::allowance(id, owner, spender)
		}

		#[ink(message)]
		pub fn token_name(&self, id: TokenId) -> PopApiResult<Vec<u8>> {
			api::token_name(id)
		}

		#[ink(message)]
		pub fn token_symbol(&self, id: TokenId) -> PopApiResult<Vec<u8>> {
			api::token_symbol(id)
		}

		#[ink(message)]
		pub fn token_decimals(&self, id: TokenId) -> PopApiResult<u8> {
			api::token_decimals(id)
		}

		#[ink(message)]
		pub fn token_exists(&self, id: TokenId) -> PopApiResult<bool> {
			api::token_exists(id)
		}
	}
}

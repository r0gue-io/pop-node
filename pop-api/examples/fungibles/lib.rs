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
		pub fn transfer(&mut self, token: TokenId, to: AccountId, value: Balance) -> PopApiResult<()> {
			api::transfer(token, to, value)
		}

		#[ink(message)]
		pub fn transfer_from(
			&mut self,
			token: TokenId,
			from: AccountId,
			to: AccountId,
			value: Balance,
			_data: Vec<u8>,
		) -> PopApiResult<()> {
			api::transfer_from(token, from, to, value)
		}

		#[ink(message)]
		pub fn approve(
			&mut self,
			token: TokenId,
			spender: AccountId,
			value: Balance,
		) -> PopApiResult<()> {
			api::approve(token, spender, value)
		}

		#[ink(message)]
		pub fn increase_allowance(
			&mut self,
			token: TokenId,
			spender: AccountId,
			value: Balance,
		) -> PopApiResult<()> {
			api::increase_allowance(token, spender, value)
		}

		#[ink(message)]
		pub fn decrease_allowance(
			&mut self,
			token: TokenId,
			spender: AccountId,
			value: Balance,
		) -> PopApiResult<()> {
			api::decrease_allowance(token, spender, value)
		}

		#[ink(message, payable)]
		pub fn create(
			&self,
			id: TokenId,
			admin: AccountId,
			min_balance: Balance,
		) -> PopApiResult<()> {
			api::create(id, admin, min_balance)?;
			self.env().emit_event(api::events::Created { id, creator: admin, admin });
			Ok(())
		}

		#[ink(message)]
		pub fn set_metadata(
			&self,
			token: TokenId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> PopApiResult<()> {
			api::set_metadata(token, name, symbol, decimals)
		}

		#[ink(message)]
		pub fn total_supply(&self, token: TokenId) -> PopApiResult<Balance> {
			api::total_supply(token)
		}

		#[ink(message)]
		pub fn balance_of(&self, token: TokenId, owner: AccountId) -> PopApiResult<Balance> {
			api::balance_of(token, owner)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			token: TokenId,
			owner: AccountId,
			spender: AccountId,
		) -> PopApiResult<Balance> {
			api::allowance(token, owner, spender)
		}

		#[ink(message)]
		pub fn token_name(&self, token: TokenId) -> PopApiResult<Vec<u8>> {
			api::token_name(token)
		}

		#[ink(message)]
		pub fn token_symbol(&self, token: TokenId) -> PopApiResult<Vec<u8>> {
			api::token_symbol(token)
		}

		#[ink(message)]
		pub fn token_decimals(&self, token: TokenId) -> PopApiResult<u8> {
			api::token_decimals(token)
		}

		#[ink(message)]
		pub fn token_exists(&self, token: TokenId) -> PopApiResult<bool> {
			api::token_exists(token)
		}
	}
}

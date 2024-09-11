#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use pop_api::{
	assets::fungibles::{self as api},
	primitives::TokenId,
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod fungibles {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Fungibles;

	impl Fungibles {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("PopApiFungiblesExample::new");
			Default::default()
		}

		#[ink(message)]
		pub fn total_supply(&self, id: TokenId) -> Result<Balance> {
			api::total_supply(id)
		}

		#[ink(message)]
		pub fn balance_of(&self, id: TokenId, owner: AccountId) -> Result<Balance> {
			api::balance_of(id, owner)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			id: TokenId,
			owner: AccountId,
			spender: AccountId,
		) -> Result<Balance> {
			api::allowance(id, owner, spender)
		}

		#[ink(message)]
		pub fn transfer(&mut self, id: TokenId, to: AccountId, value: Balance) -> Result<()> {
			api::transfer(id, to, value)
		}

		#[ink(message)]
		pub fn transfer_from(
			&mut self,
			id: TokenId,
			from: AccountId,
			to: AccountId,
			value: Balance,
			_data: Vec<u8>,
		) -> Result<()> {
			api::transfer_from(id, from, to, value)
		}

		#[ink(message)]
		pub fn approve(&mut self, id: TokenId, spender: AccountId, value: Balance) -> Result<()> {
			api::approve(id, spender, value)
		}

		#[ink(message)]
		pub fn increase_allowance(
			&mut self,
			id: TokenId,
			spender: AccountId,
			value: Balance,
		) -> Result<()> {
			api::increase_allowance(id, spender, value)
		}

		#[ink(message)]
		pub fn decrease_allowance(
			&mut self,
			id: TokenId,
			spender: AccountId,
			value: Balance,
		) -> Result<()> {
			api::decrease_allowance(id, spender, value)
		}

		#[ink(message)]
		pub fn token_name(&self, id: TokenId) -> Result<Vec<u8>> {
			api::token_name(id)
		}

		#[ink(message)]
		pub fn token_symbol(&self, id: TokenId) -> Result<Vec<u8>> {
			api::token_symbol(id)
		}

		#[ink(message)]
		pub fn token_decimals(&self, id: TokenId) -> Result<u8> {
			api::token_decimals(id)
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {}
	}
}

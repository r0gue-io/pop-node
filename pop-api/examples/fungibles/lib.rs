#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use pop_api::{
	fungibles::{self as api},
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
			Default::default()
		}

		#[ink(message)]
		pub fn total_supply(&self, token: TokenId) -> Result<Balance> {
			api::total_supply(token)
		}

		#[ink(message)]
		pub fn balance_of(&self, token: TokenId, owner: AccountId) -> Result<Balance> {
			api::balance_of(token, owner)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			token: TokenId,
			owner: AccountId,
			spender: AccountId,
		) -> Result<Balance> {
			api::allowance(token, owner, spender)
		}

		#[ink(message)]
		pub fn transfer(&mut self, token: TokenId, to: AccountId, value: Balance) -> Result<()> {
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
		) -> Result<()> {
			api::transfer_from(token, from, to, value)
		}

		#[ink(message)]
		pub fn approve(
			&mut self,
			token: TokenId,
			spender: AccountId,
			value: Balance,
		) -> Result<()> {
			api::approve(token, spender, value)
		}

		#[ink(message)]
		pub fn increase_allowance(
			&mut self,
			token: TokenId,
			spender: AccountId,
			value: Balance,
		) -> Result<()> {
			api::increase_allowance(token, spender, value)
		}

		#[ink(message)]
		pub fn decrease_allowance(
			&mut self,
			token: TokenId,
			spender: AccountId,
			value: Balance,
		) -> Result<()> {
			api::decrease_allowance(token, spender, value)
		}

		#[ink(message)]
		pub fn token_name(&self, token: TokenId) -> Result<Vec<u8>> {
			api::token_name(token)
		}

		#[ink(message)]
		pub fn token_symbol(&self, token: TokenId) -> Result<Vec<u8>> {
			api::token_symbol(token)
		}

		#[ink(message)]
		pub fn token_decimals(&self, token: TokenId) -> Result<u8> {
			api::token_decimals(token)
		}
	}
}

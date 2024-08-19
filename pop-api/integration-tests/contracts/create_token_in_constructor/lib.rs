#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::{
	fungibles::{self as api},
	primitives::TokenId,
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod create_token_in_constructor {
	use super::*;

	#[ink(storage)]
	pub struct Fungible {
		id: TokenId,
	}

	impl Fungible {
		#[ink(constructor, payable)]
		pub fn new(id: TokenId, min_balance: Balance) -> Result<Self> {
			let contract = Self { id };
			// AccountId of the contract which will be set to the owner of the fungible token.
			let owner = contract.env().account_id();
			api::create(id, owner, min_balance)?;
			Ok(contract)
		}

		#[ink(message)]
		pub fn token_exists(&self) -> Result<bool> {
			api::token_exists(self.id)
		}
	}
}

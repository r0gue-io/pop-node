#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::{
	fungibles::{self as api, events::Created},
	primitives::TokenId,
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod create_token_in_constructor {
	use super::*;

	#[ink(storage)]
	pub struct Fungible {
		token: TokenId,
	}

	impl Fungible {
		#[ink(constructor, payable)]
		pub fn new(id: TokenId, min_balance: Balance) -> Result<Self> {
			let contract = Self { token: id };
			// AccountId of the contract which will be set to the owner of the fungible token.
			let owner = contract.env().account_id();
			api::create(id, owner, min_balance)?;
			contract.env().emit_event(Created { id, creator: owner, admin: owner });
			Ok(contract)
		}

		#[ink(message, payable)]
		pub fn token_exists(&self) -> Result<bool> {
			api::token_exists(self.token)
		}
	}
}

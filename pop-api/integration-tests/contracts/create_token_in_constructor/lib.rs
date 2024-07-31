#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use pop_api::{
	assets::fungibles::{self as api},
	primitives::AssetId,
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod create_token_in_constructor {
	use super::*;

	#[ink(storage)]
	pub struct Fungible {
		id: AssetId,
	}

	impl Fungible {
		#[ink(constructor, payable)]
		pub fn new(id: AssetId, min_balance: Balance) -> Result<Self> {
			let contract = Self { id };
			// AccountId of the contract which will be set to the owner of the fungible token.
			let owner = contract.env().account_id();
			api::create(id, owner, min_balance)?;
			Ok(contract)
		}

		#[ink(message)]
		pub fn asset_exists(&self) -> Result<bool> {
			api::asset_exists(self.id)
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			PopApiFungiblesExample::new();
		}
	}
}

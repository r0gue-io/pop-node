#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::{
	nonfungibles::{
		self as api,
		types::{CollectionConfig, CollectionId},
	},
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod create_collection_in_constructor {
	use super::*;

	#[ink(storage)]
	pub struct Nonfungible {
		collection: CollectionId,
	}

	impl Nonfungible {
		#[ink(constructor, payable)]
		pub fn new(admin: AccountId, config: CollectionConfig) -> Result<Self> {
			let id = api::next_collection_id()?;
			let contract = Self { collection: id };
			api::create(admin, config)?;
			Ok(contract)
		}

		#[ink(message)]
		pub fn total_supply(&self) -> Result<u128> {
			api::total_supply(self.collection)
		}
	}
}

#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Local Fungibles:
/// 1. PSP-22 Interface
/// 2. PSP-22 Metadata Interface
/// 3. Asset Management
///
use ink::prelude::vec::Vec;
use pop_api::{
	assets::fungibles::{self as api},
	primitives::AssetId,
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

		/// 1. PSP-22 Interface:
		/// - total_supply
		/// - balance_of
		/// - allowance
		/// - transfer
		/// - transfer_from
		/// - approve
		/// - increase_allowance
		/// - decrease_allowance

		#[ink(message)]
		pub fn total_supply(&self, id: AssetId) -> Result<Balance> {
			api::total_supply(id)
		}

		#[ink(message)]
		pub fn balance_of(&self, id: AssetId, owner: AccountId) -> Result<Balance> {
			api::balance_of(id, owner)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			id: AssetId,
			owner: AccountId,
			spender: AccountId,
		) -> Result<Balance> {
			api::allowance(id, owner, spender)
		}

		#[ink(message)]
		pub fn transfer(&mut self, id: AssetId, to: AccountId, value: Balance) -> Result<()> {
			api::transfer(id, to, value)
		}

		#[ink(message)]
		pub fn transfer_from(
			&mut self,
			id: AssetId,
			from: AccountId,
			to: AccountId,
			value: Balance,
			// In the PSP-22 standard a `[u8]`, but the size needs to be known at compile time.
			_data: Vec<u8>,
		) -> Result<()> {
			api::transfer_from(id, from, to, value)
		}

		#[ink(message)]
		pub fn approve(&mut self, id: AssetId, spender: AccountId, value: Balance) -> Result<()> {
			api::approve(id, spender, value)
		}

		#[ink(message)]
		pub fn increase_allowance(
			&mut self,
			id: AssetId,
			spender: AccountId,
			value: Balance,
		) -> Result<()> {
			api::increase_allowance(id, spender, value)
		}

		#[ink(message)]
		pub fn decrease_allowance(
			&mut self,
			id: AssetId,
			spender: AccountId,
			value: Balance,
		) -> Result<()> {
			api::decrease_allowance(id, spender, value)
		}

		/// 2. PSP-22 Metadata Interface:
		/// - token_name
		/// - token_symbol
		/// - token_decimals

		#[ink(message)]
		pub fn token_name(&self, id: AssetId) -> Result<Vec<u8>> {
			api::token_name(id)
		}

		#[ink(message)]
		pub fn token_symbol(&self, id: AssetId) -> Result<Vec<u8>> {
			api::token_symbol(id)
		}

		#[ink(message)]
		pub fn token_decimals(&self, id: AssetId) -> Result<u8> {
			api::token_decimals(id)
		}

		/// 3. Asset Management:
		/// - create
		/// - start_destroy
		/// - set_metadata
		/// - clear_metadata
		/// - asset_exists

		#[ink(message)]
		pub fn create(
			&mut self,
			id: AssetId,
			admin: AccountId,
			min_balance: Balance,
		) -> Result<()> {
			api::create(id, admin, min_balance)
		}

		#[ink(message)]
		pub fn start_destroy(&mut self, id: AssetId) -> Result<()> {
			api::start_destroy(id)
		}

		#[ink(message)]
		pub fn set_metadata(
			&mut self,
			id: AssetId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> Result<()> {
			api::set_metadata(id, name, symbol, decimals)
		}

		#[ink(message)]
		pub fn clear_metadata(&self, id: AssetId) -> Result<()> {
			api::clear_metadata(id)
		}

		#[ink(message)]
		pub fn asset_exists(&self, id: AssetId) -> Result<bool> {
			api::asset_exists(id)
		}

		/// 4. PSP-22 Mintable & Burnable Interface:
		/// - mint
		/// - burn

		#[ink(message)]
		pub fn mint(&mut self, id: AssetId, account: AccountId, amount: Balance) -> Result<()> {
			api::mint(id, account, amount)
		}

		#[ink(message)]
		pub fn burn(&mut self, id: AssetId, account: AccountId, amount: Balance) -> Result<()> {
			api::burn(id, account, amount)
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

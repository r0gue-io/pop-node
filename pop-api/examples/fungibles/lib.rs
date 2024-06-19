#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Local Fungibles:
/// 1. PSP-22 Interface
/// 2. PSP-22 Metadata Interface
/// 3. Asset Management
///
use ink::prelude::vec::Vec;
use pop_api::{
	assets::use_cases::fungibles as api,
	primitives::{AccountId as AccountId32, AssetId},
	PopApiError,
};

pub type Result<T> = core::result::Result<T, PopApiError>;

#[ink::contract(env = pop_api::Environment)]
mod fungibles {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Fungibles;

	impl Fungibles {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("PopApiAssetsExample::new");
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
		pub fn balance_of(&self, id: AssetId, owner: AccountId32) -> Result<Balance> {
			api::balance_of(id, owner)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			id: AssetId,
			owner: AccountId32,
			spender: AccountId32,
		) -> Result<Balance> {
			api::allowance(id, owner, spender)
		}

		#[ink(message)]
		pub fn transfer(&self, id: AssetId, to: AccountId32, value: Balance) -> Result<()> {
			ink::env::debug_println!(
				"PopApiAssetsExample::transfer: id: {:?}, to: {:?} value: {:?}",
				id,
				to,
				value,
			);

			let result = api::transfer(id, to, value);
			ink::env::debug_println!("Result: {:?}", result);
			result
		}

		#[ink(message)]
		pub fn transfer_from(
			&self,
			id: AssetId,
			from: Option<AccountId32>,
			to: Option<AccountId32>,
			value: Balance,
			// Size needs to be known at compile time or ink's `Vec`
			data: Vec<u8>,
		) -> Result<()> {
			ink::env::debug_println!(
				"PopApiAssetsExample::transfer_from: id: {:?}, from: {:?}, to: {:?} value: {:?}",
				id,
				from,
				to,
				value,
			);

			let result = api::transfer_from(id, from, to, value, &data);
			ink::env::debug_println!("Result: {:?}", result);
			result
		}

		/// 2. PSP-22 Metadata Interface:
		/// - token_name
		/// - token_symbol
		/// - token_decimals

		/// 3. Asset Management:
		/// - create
		/// - start_destroy
		/// - destroy_accounts
		/// - destroy_approvals
		/// - finish_destroy
		/// - set_metadata
		/// - clear_metadata

		#[ink(message)]
		pub fn create(&self, id: AssetId, admin: AccountId32, min_balance: Balance) -> Result<()> {
			ink::env::debug_println!(
				"PopApiAssetsExample::create: id: {:?} admin: {:?} min_balance: {:?}",
				id,
				admin,
				min_balance,
			);
			let result = api::create(id, admin, min_balance);
			ink::env::debug_println!("Result: {:?}", result);
			result
		}

		#[ink(message)]
		pub fn set_metadata(
			&self,
			id: AssetId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> Result<()> {
			ink::env::debug_println!(
				"PopApiAssetsExample::set_metadata: id: {:?} name: {:?} symbol: {:?}, decimals: {:?}",
				id,
				name,
				symbol,
				decimals,
			);
			let result = api::set_metadata(id, name, symbol, decimals);
			ink::env::debug_println!("Result: {:?}", result);
			result
		}

		#[ink(message)]
		pub fn asset_exists(&self, id: AssetId) -> Result<bool> {
			api::asset_exists(id)
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			PopApiAssetsExample::new();
		}
	}
}
#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Local Fungibles:
/// 1. PSP-22 Interface
/// 2. PSP-22 Metadata Interface
/// 3. Asset Management
///
use ink::prelude::vec::Vec;
use pop_api::{
	assets::fungibles::{self as api},
	error::StatusCode,
	primitives::{AccountId as AccountId32, AssetId},
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract(env = pop_api::Environment)]
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
			// api::total_supply(id).map_err(|e| e.into())
			api::total_supply(id)
		}

		#[ink(message)]
		pub fn balance_of(&self, id: AssetId, owner: AccountId32) -> Result<Balance> {
			// api::balance_of(id, owner).map_err(|e| e.into())
			api::balance_of(id, owner)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			id: AssetId,
			owner: AccountId32,
			spender: AccountId32,
		) -> Result<Balance> {
			// api::allowance(id, owner, spender).map_err(|e| e.into())
			api::allowance(id, owner, spender)
		}

		#[ink(message)]
		pub fn transfer(&self, id: AssetId, to: AccountId32, value: Balance) -> Result<()> {
			ink::env::debug_println!(
				"PopApiFungiblesExample::transfer: id: {:?}, to: {:?} value: {:?}",
				id,
				to,
				value,
			);

			let result = api::transfer(id, to, value);
			ink::env::debug_println!("Result: {:?}", result);
			// result.map_err(|e| e.into())
			result
		}

		#[ink(message)]
		pub fn transfer_from(
			&self,
			id: AssetId,
			from: AccountId32,
			to: AccountId32,
			value: Balance,
			// In the standard a `[u8]`, but the size needs to be known at compile time.
			data: Vec<u8>,
		) -> Result<()> {
			ink::env::debug_println!(
				"PopApiFungiblesExample::transfer_from: id: {:?}, from: {:?}, to: {:?} value: {:?}",
				id,
				from,
				to,
				value,
			);

			let result = api::transfer_from(id, from, to, value, &data);
			ink::env::debug_println!("Result: {:?}", result);
			// result.map_err(|e| e.into())
			result
		}

		#[ink(message)]
		pub fn approve(&self, id: AssetId, spender: AccountId32, value: Balance) -> Result<()> {
			ink::env::debug_println!(
				"PopApiFungiblesExample::approve: id: {:?}, spender {:?}, value: {:?}",
				id,
				spender,
				value,
			);

			let result = api::approve(id, spender, value);
			ink::env::debug_println!("Result: {:?}", result);
			// result.map_err(|e| e.into())
			result
		}

		#[ink(message)]
		pub fn increase_allowance(
			&self,
			id: AssetId,
			spender: AccountId32,
			value: Balance,
		) -> Result<()> {
			ink::env::debug_println!(
				"PopApiFungiblesExample::increase_allowance: id: {:?}, spender {:?}, value: {:?}",
				id,
				spender,
				value,
			);

			let result = api::increase_allowance(id, spender, value);
			ink::env::debug_println!("Result: {:?}", result);
			// result.map_err(|e| e.into())
			result
		}

		#[ink(message)]
		pub fn decrease_allowance(
			&self,
			id: AssetId,
			spender: AccountId32,
			value: Balance,
		) -> Result<()> {
			ink::env::debug_println!(
				"PopApiFungiblesExample::decrease_allowance: id: {:?}, spender {:?}, value: {:?}",
				id,
				spender,
				value,
			);

			let result = api::decrease_allowance(id, spender, value);
			ink::env::debug_println!("Result: {:?}", result);
			// result.map_err(|e| e.into())
			result
		}

		/// 2. PSP-22 Metadata Interface:
		/// - token_name
		/// - token_symbol
		/// - token_decimals

		#[ink(message)]
		pub fn token_name(&self, id: AssetId) -> Result<Vec<u8>> {
			// api::token_name(id).map_err(|e| e.into())
			api::token_name(id)
		}

		#[ink(message)]
		pub fn token_symbol(&self, id: AssetId) -> Result<Vec<u8>> {
			// api::token_symbol(id).map_err(|e| e.into())
			api::token_symbol(id)
		}

		#[ink(message)]
		pub fn token_decimals(&self, id: AssetId) -> Result<u8> {
			// api::token_decimals(id).map_err(|e| e.into())
			api::token_decimals(id)
		}

		// 3. Asset Management:
		// - create
		// - start_destroy
		// - destroy_accounts
		// - destroy_approvals
		// - finish_destroy
		// - set_metadata
		// - clear_metadata

		// #[ink(message)]
		// pub fn create(&self, id: AssetId, admin: AccountId32, min_balance: Balance) -> Result<()> {
		// 	ink::env::debug_println!(
		// 		"PopApiFungiblesExample::create: id: {:?} admin: {:?} min_balance: {:?}",
		// 		id,
		// 		admin,
		// 		min_balance,
		// 	);
		// 	let result = api::create(id, admin, min_balance);
		// 	ink::env::debug_println!("Result: {:?}", result);
		// 	// result.map_err(|e| e.into())
		// 	result
		// }

		// #[ink(message)]
		// pub fn set_metadata(
		// 	&self,
		// 	id: AssetId,
		// 	name: Vec<u8>,
		// 	symbol: Vec<u8>,
		// 	decimals: u8,
		// ) -> Result<()> {
		// 	ink::env::debug_println!(
		// 		"PopApiFungiblesExample::set_metadata: id: {:?} name: {:?} symbol: {:?}, decimals: {:?}",
		// 		id,
		// 		name,
		// 		symbol,
		// 		decimals,
		// 	);
		// 	let result = api::set_metadata(id, name, symbol, decimals);
		// 	ink::env::debug_println!("Result: {:?}", result);
		// 	// result.map_err(|e| e.into())
		// 	result
		// }
		//
		// #[ink(message)]
		// pub fn asset_exists(&self, id: AssetId) -> Result<bool> {
		// 	// api::asset_exists(id).map_err(|e| e.into())
		// 	api::asset_exists(id)
		// }
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

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use pop_api::{
	primitives::TokenId,
	v0::fungibles::{self as api},
	StatusCode,
};

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
		pub fn total_supply(&self, id: TokenId) -> PopApiResult<Balance> {
			api::total_supply(id)
		}

		#[ink(message)]
		pub fn balance_of(&self, id: TokenId, owner: AccountId) -> PopApiResult<Balance> {
			api::balance_of(id, owner)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			id: TokenId,
			owner: AccountId,
			spender: AccountId,
		) -> PopApiResult<Balance> {
			api::allowance(id, owner, spender)
		}

		#[ink(message)]
		pub fn transfer(&mut self, id: TokenId, to: AccountId, value: Balance) -> PopApiResult<()> {
			api::transfer(id, to, value)
		}

		#[ink(message)]
		pub fn transfer_from(
			&mut self,
			id: TokenId,
			from: AccountId,
			to: AccountId,
			value: Balance,
			// In the PSP-22 standard a `[u8]`, but the size needs to be known at compile time.
			_data: Vec<u8>,
		) -> PopApiResult<()> {
			api::transfer_from(id, from, to, value)
		}

		#[ink(message)]
		pub fn approve(
			&mut self,
			id: TokenId,
			spender: AccountId,
			value: Balance,
		) -> PopApiResult<()> {
			api::approve(id, spender, value)
		}

		#[ink(message)]
		pub fn increase_allowance(
			&mut self,
			id: TokenId,
			spender: AccountId,
			value: Balance,
		) -> PopApiResult<()> {
			api::increase_allowance(id, spender, value)
		}

		#[ink(message)]
		pub fn decrease_allowance(
			&mut self,
			id: TokenId,
			spender: AccountId,
			value: Balance,
		) -> PopApiResult<()> {
			api::decrease_allowance(id, spender, value)
		}

		#[ink(message)]
		pub fn token_name(&self, id: TokenId) -> PopApiResult<Vec<u8>> {
			api::token_name(id)
		}

		#[ink(message)]
		pub fn token_symbol(&self, id: TokenId) -> PopApiResult<Vec<u8>> {
			api::token_symbol(id)
		}

		#[ink(message)]
		pub fn token_decimals(&self, id: TokenId) -> PopApiResult<u8> {
			api::token_decimals(id)
		}

		#[ink(message, payable)]
		pub fn create(
			&self,
			id: TokenId,
			admin: AccountId,
			min_balance: Balance,
		) -> PopApiResult<()> {
			api::create(id, admin, min_balance)
		}

		#[ink(message)]
		pub fn set_metadata(
			&self,
			id: TokenId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> PopApiResult<()> {
			api::set_metadata(id, name, symbol, decimals)
		}

		#[ink(message)]
		pub fn token_exists(&self, id: TokenId) -> PopApiResult<bool> {
			api::token_exists(id)
		}
	}
}

/// We put `drink`-based tests as usual unit tests, into a test module.
#[cfg(test)]
mod tests {
	use drink::session::{Session, NO_ARGS, NO_SALT};
	use pop_sandbox::{ALICE, INIT_VALUE};

	use super::*;

	#[drink::contract_bundle_provider]
	enum BundleProvider {}

	#[drink::test(sandbox = pop_sandbox::PopSandbox)]
	fn deploy_and_call_a_contract(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
		let _ = env_logger::try_init();
		let contract_bundle = BundleProvider::local()?;

		const ASSET_ID: TokenId = 1;

		// Deploy a contract.
		let contract_address =
			session.deploy_bundle(contract_bundle, "new", NO_ARGS, NO_SALT, Some(INIT_VALUE))?;
		// Calling the method in the contract.
		session.call_with_address(
			contract_address.clone(),
			"create",
			&vec![ASSET_ID.to_string(), ALICE.to_string(), 10_000.to_string()],
			None,
		)??;
		// Calling the method in the contract.
		session.call_with_address(
			contract_address.clone(),
			"token_exists",
			&vec![ASSET_ID.to_string()],
			None,
		)??;
		// Check that the token is created successfully.
		let result = session.record().last_call_return_decoded::<PopApiResult<bool>>()??;
		assert_eq!(result, Ok(true));
		Ok(())
	}
}

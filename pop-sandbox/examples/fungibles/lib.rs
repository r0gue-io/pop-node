#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::{
	primitives::TokenId,
	v0::fungibles::{self as api},
	StatusCode,
};

type PopApiResult<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod create_token_in_constructor {
	use super::*;

	#[ink(storage)]
	pub struct Fungible {
		id: TokenId,
	}

	impl Fungible {
		#[ink(constructor, payable)]
		pub fn new(id: TokenId, min_balance: Balance) -> PopApiResult<Self> {
			ink::env::debug_println!("Fungible::call() asset_id={id}, min_balance={min_balance}");
			let contract = Self { id };
			let owner = contract.env().account_id();
			api::create(id, owner, min_balance)?;
			Ok(contract)
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
	use super::*;
	use drink::session::{Session, NO_SALT};
	use pop_sandbox::utils::{call_function, ALICE};

	#[drink::contract_bundle_provider]
	enum BundleProvider {}

	#[drink::test(sandbox = pop_sandbox::PopSandbox)]
	fn deploy_and_call_a_contract(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
		let _ = env_logger::try_init();
		let contract_bundle = BundleProvider::local()?;

		const ASSET_ID: TokenId = 1;
		// Deploy a contract and create a new token with ASSET_ID = 1
		let contract_address = session.deploy_bundle(
			contract_bundle,
			"new",
			&[1.to_string(), 1_000.to_string()],
			NO_SALT,
			Some(100_000_000),
		)?;
		// Calling the method in the contract.
		let session = call_function(
			session,
			&contract_address,
			&ALICE,
			"token_exists".to_string(),
			Some(vec![ASSET_ID.to_string()]),
			None,
		)?;
		// Check that the token is created successfully.
		let result = session.record().last_call_return_decoded::<PopApiResult<bool>>()??;
		assert_eq!(result, Ok(true));
		Ok(())
	}
}

#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Local Fungibles:
/// 1. PSP-22 Interface
/// 2. PSP-22 Metadata Interface
/// 3. Asset Management
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
			ink::env::debug_println!("Fungibles::new");
			Default::default()
		}

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
		pub fn asset_exists(&self, id: AssetId) -> Result<bool> {
			api::asset_exists(id)
		}

		#[ink(message)]
		pub fn mint(&mut self, id: AssetId, account: AccountId, amount: Balance) -> Result<()> {
			api::mint(id, account, amount)
		}

		#[ink(message)]
		pub fn burn(&mut self, id: AssetId, account: AccountId, amount: Balance) -> Result<()> {
			api::burn(id, account, amount)
		}
	}
}

#[cfg(test)]
mod tests {
	use drink::session::{Session, NO_ARGS, NO_SALT};

	#[drink::contract_bundle_provider]
	enum BundleProvider {}

	/// Test that we can call chain extension from ink! contract and get a correct result.
	#[drink::test(sandbox = pop_sandbox::PopDevnetSandbox)]
	fn we_can_test_chain_extension(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
		// Now we get the contract bundle from the `BundleProvider` enum. Since the current crate
		// comes with a contract, we can use the `local` method to get the bundle for it.
		let contract_bundle = BundleProvider::local()?;

		// We can now deploy the contract.
		let _contract_address = session.deploy_bundle(
			// The bundle that we want to deploy.
			contract_bundle,
			// The constructor that we want to call.
			"new",
			// The constructor arguments (as stringish objects).
			&["true"],
			// Salt for the contract address derivation.
			NO_SALT,
			// Initial endowment (the amount of tokens that we want to transfer to the contract).
			None,
		)?;

		Ok(())
	}
}

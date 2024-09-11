#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod create_token_in_constructor {
	use pop_api::{
		primitives::AssetId,
		v0::assets::fungibles::{self as api},
		StatusCode,
	};

	pub type Result<T> = core::result::Result<T, StatusCode>;

	#[ink(storage)]
	pub struct Fungible {
		id: AssetId,
	}

	impl Fungible {
		#[ink(constructor)]
		pub fn new(id: AssetId, min_balance: Balance) -> Result<Self> {
						ink::env::debug_println!("Fungible::call() asset_id={id}, min_balance={min_balance}");
			let contract = Self { id };
			// AccountId of the contract which will be set to the owner of the fungible token.
			let owner = contract.env().account_id();
			// TODO: Calling POP API caused DeploymentReverted
			api::create(id, owner, min_balance)?;
			Ok(contract)
		}

		#[ink(message)]
		pub fn asset_exists(&self) -> Result<bool> {
			api::asset_exists(self.id)
		}
	}
}

/// We put `drink`-based tests as usual unit tests, into a test module.
#[cfg(test)]
mod tests {
	use drink::session::{Session, NO_ARGS, NO_SALT};

	#[drink::contract_bundle_provider]
	enum BundleProvider {}

	#[drink::test(sandbox = pop_sandbox::PopSandbox)]
	fn deploy_and_call_a_contract(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
		let contract_bundle = BundleProvider::local()?;
		let _contract_address = session.deploy_bundle(
			contract_bundle,
			"new",
			&[1.to_string(), 1_000.to_string()],
			NO_SALT,
			None,
		)?;
		Ok(())
	}
}

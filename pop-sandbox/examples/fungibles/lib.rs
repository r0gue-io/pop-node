#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod create_token_in_constructor {
	use pop_api::{
		primitives::TokenId,
		v0::fungibles::{self as api},
		StatusCode,
	};

	pub type Result<T> = core::result::Result<T, StatusCode>;

	#[ink(storage)]
	pub struct Fungible {
		id: TokenId,
	}

	impl Fungible {
		#[ink(constructor)]
		pub fn new(id: TokenId, min_balance: Balance) -> Result<Self> {
			ink::env::debug_println!("Fungible::call() asset_id={id}, min_balance={min_balance}");
			let contract = Self { id };
			let owner = contract.env().account_id();
			api::create(id, owner, min_balance)?;
			Ok(contract)
		}

		#[ink(message)]
		pub fn token_exists(&self) -> Result<bool> {
			api::token_exists(self.id)
		}
	}
}

/// We put `drink`-based tests as usual unit tests, into a test module.
#[cfg(test)]
mod tests {
	use drink::session::{Session, NO_SALT};

	#[drink::contract_bundle_provider]
	enum BundleProvider {}

	#[drink::test(sandbox = pop_sandbox::PopSandbox)]
	fn deploy_and_call_a_contract(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
		let _ = env_logger::try_init();
		let contract_bundle = BundleProvider::local()?;
		// TODO: utility method for deploy_contract
		let _contract_address = session.deploy_bundle(
			contract_bundle,
			"new",
			&[1.to_string(), 1_000.to_string()],
			NO_SALT,
			Some(100_000_000),
		)?;
		Ok(())
	}
}

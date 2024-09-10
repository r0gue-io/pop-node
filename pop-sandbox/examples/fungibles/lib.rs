#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod create_token_in_constructor {
	use pop_api::{
		v0::assets::fungibles::{self as api},
		StatusCode,
	};

	pub type Result<T> = core::result::Result<T, StatusCode>;

	#[ink(storage)]
	pub struct Fungible {
		id: u32,
	}

	impl Fungible {
		// #[ink(constructor, payable)]
		// pub fn new() -> Result<Self> {
		// 	let id = 0;
		// 	let min_balance = 1;
		// 	let contract = Self { id };
		// 	// AccountId of the contract which will be set to the owner of the fungible token.
		// 	let owner = contract.env().account_id();
		// 	api::create(id, owner, min_balance)?;
		// 	Ok(contract)
		// }

		#[ink(constructor)]
		pub fn new() -> Self {
			let contract = Self { id: 0 };
			let owner = contract.env().account_id();
			contract
		}

		// #[ink(constructor)]
		// pub fn new() -> Self {
		// 	let id = 0;
		// 	let min_balance = 1;
		// 	let contract = Self { id };
		// 	// AccountId of the contract which will be set to the owner of the fungible token.
		// 	let owner = contract.env().account_id();
		// 	api::create(id, owner, min_balance).unwrap();
		// 	contract
		// }

		#[ink(message)]
		pub fn asset_exists(&self) -> Result<bool> {
			// api::asset_exists(self.id)
			Ok(true)
		}
	}
}

// #[ink::contract]
// mod create_token_in_constructor {
// 	use super::*;

// 	#[ink(storage)]
// 	pub struct Fungible {
// 		id: AssetId,
// 	}

// 	impl Fungible {
// 		#[ink(constructor)]
// 		pub fn new(id: AssetId, min_balance: Balance) -> Result<Self> {
// 			let contract = Self { id };
// 			// AccountId of the contract which will be set to the owner of the fungible token.
// 			let owner = contract.env().account_id();
// 			api::create(id, owner, min_balance)?;
// 			Ok(contract)
// 		}

// 		#[ink(message)]
// 		pub fn asset_exists(&self) -> Result<bool> {
// 			api::asset_exists(self.id)
// 		}
// 	}
// }

/// We put `drink`-based tests as usual unit tests, into a test module.
#[cfg(test)]
mod tests {
	use drink::session::{Session, NO_SALT, NO_ARGS};

	#[drink::contract_bundle_provider]
	enum BundleProvider {}

	#[drink::test(sandbox = pop_sandbox::PopSandbox)]
	fn deploy_and_call_a_contract(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
		let contract_bundle = BundleProvider::local()?;
		let _contract_address =
			session.deploy_bundle(contract_bundle, "new", NO_ARGS, NO_SALT, None)?;
		Ok(())
	}
}

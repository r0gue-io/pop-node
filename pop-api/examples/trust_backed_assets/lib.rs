#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::assets::trust_backed as trust_backed_assets;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
	TrustBackedAssetsError(trust_backed_assets::Error),
	UnknownAsset,
}

impl From<trust_backed_assets::Error> for ContractError {
	fn from(value: trust_backed_assets::Error) -> Self {
		ContractError::TrustBackedAssetsError(value)
	}
}

#[ink::contract(env = pop_api::Environment)]
mod pop_api_extension_demo {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct PopApiExtensionDemo;

	impl PopApiExtensionDemo {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("Contract::new");
			Default::default()
		}

		#[ink(message)]
		pub fn mint_asset_through_runtime(
			&mut self,
			id: u32,
			beneficiary: AccountId,
			amount: Balance,
		) -> Result<(), ContractError> {
			ink::env::debug_println!(
				"Contract::mint_asset_through_runtime: id: {:?} beneficiary: {:?} amount: {:?}",
				id,
				beneficiary,
				amount
			);

			// Check if asset doesn't exist.
            if !trust_backed_assets::asset_exists(id)? {
				return Err(ContractError::UnknownAsset);
			}

			// Mint asset via pop api.
			trust_backed_assets::mint(id, beneficiary, amount)?;
			ink::env::debug_println!("Contract::mint_asset_through_runtime: asset(s) minted successfully");
			Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			PopApiExtensionDemo::new();
		}
	}
}


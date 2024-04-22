#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::nfts;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
	InvalidCollection,
	ItemAlreadyExists,
	NftsError(nfts::Error),
	NotOwner,
}

impl From<nfts::Error> for ContractError {
	fn from(value: nfts::Error) -> Self {
		ContractError::NftsError(value)
	}
}

#[ink::contract(env = pop_api::Environment)]
mod pop_api_nfts {
	use super::ContractError;

	#[ink(storage)]
	#[derive(Default)]
	pub struct PopApiNfts;

	impl PopApiNfts {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("PopApiNfts::new");
			Default::default()
		}

		#[ink(message)]
		pub fn mint_through_runtime(
			&mut self,
			collection_id: u32,
			item_id: u32,
			receiver: AccountId,
		) -> Result<(), ContractError> {
			ink::env::debug_println!(
				"PopApiNfts::mint_through_runtime: collection_id: {:?} item_id {:?} receiver: {:?}",
				collection_id,
				item_id,
				receiver
			);

			// Check if item already exists (demo purposes only, unnecessary as would expect check in mint call)
			if pop_api::nfts::item(collection_id, item_id)?.is_some() {
				return Err(ContractError::ItemAlreadyExists);
			}

			// mint api
			pop_api::nfts::mint(collection_id, item_id, receiver)?;
			ink::env::debug_println!("PopApiNfts::mint_through_runtime: item minted successfully");

			// check owner
			match pop_api::nfts::owner(collection_id, item_id)? {
				Some(owner) if owner == receiver => {
					ink::env::debug_println!(
						"PopApiNfts::mint_through_runtime success: minted item belongs to receiver"
					);
				},
				_ => {
					return Err(ContractError::NotOwner);
				},
			}

			ink::env::debug_println!("PopApiNfts::mint_through_runtime end");
			Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			PopApiNfts::new();
		}
	}
}

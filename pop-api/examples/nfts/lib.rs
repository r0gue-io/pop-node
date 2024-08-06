// DEPRECATED
#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::nfts::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
	InvalidCollection,
	ItemAlreadyExists,
	NftsError(Error),
	NotOwner,
}

impl From<Error> for ContractError {
	fn from(value: Error) -> Self {
		ContractError::NftsError(value)
	}
}

#[ink::contract(env = pop_api::Environment)]
mod nfts {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Nfts;

	impl Nfts {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("Nfts::new");
			Default::default()
		}

		#[ink(message)]
		pub fn create_nft_collection(&self) -> Result<(), ContractError> {
			ink::env::debug_println!("Nfts::create_nft_collection: collection creation started.");
			let admin = Self::env().caller();
			let item_settings = ItemSettings(BitFlags::from(ItemSetting::Transferable));

			let mint_settings = MintSettings {
				mint_type: MintType::Issuer,
				price: Some(0),
				start_block: Some(0),
				end_block: Some(0),
				default_item_settings: item_settings,
			};

			let config = CollectionConfig {
				settings: CollectionSettings(BitFlags::from(CollectionSetting::TransferableItems)),
				max_supply: None,
				mint_settings,
			};
			pop_api::nfts::create(admin, config)?;
			ink::env::debug_println!(
				"Nfts::create_nft_collection: collection created successfully."
			);
			Ok(())
		}

		#[ink(message)]
		pub fn mint_nft(
			&mut self,
			collection_id: u32,
			item_id: u32,
			receiver: AccountId,
		) -> Result<(), ContractError> {
			ink::env::debug_println!(
				"Nfts::mint: collection_id: {:?} item_id {:?} receiver: {:?}",
				collection_id,
				item_id,
				receiver
			);

			// Check if item already exists (demo purposes only, unnecessary as would expect check in mint call)
			if item(collection_id, item_id)?.is_some() {
				return Err(ContractError::ItemAlreadyExists);
			}

			// mint api
			mint(collection_id, item_id, receiver)?;
			ink::env::debug_println!("Nfts::mint: item minted successfully");

			// check owner
			match owner(collection_id, item_id)? {
				Some(owner) if owner == receiver => {
					ink::env::debug_println!("Nfts::mint success: minted item belongs to receiver");
				},
				_ => {
					return Err(ContractError::NotOwner);
				},
			}

			ink::env::debug_println!("Nfts::mint end");
			Ok(())
		}

		#[ink(message)]
		pub fn read_collection(&self, collection_id: u32) -> Result<(), ContractError> {
			ink::env::debug_println!("Nfts::read_collection: collection_id: {:?}", collection_id);
			let collection = pop_api::nfts::collection(collection_id)?;
			ink::env::debug_println!("Nfts::read_collection: collection: {:?}", collection);
			Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			Nfts::new();
		}
	}
}

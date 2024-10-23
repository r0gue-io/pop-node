#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// 1. PSP-34
/// 2. PSP-34 Metadata
/// 3. Management
/// 4. PSP-34 Mintable & Burnable
use ink::prelude::vec::Vec;
use pop_api::{
	nonfungibles::{
		self as api,
		events::{Approval, AttributeSet, Transfer},
		AttributeNamespace, CancelAttributesApprovalWitness, CollectionDetails, CollectionId,
		CreateCollectionConfig, DestroyWitness, ItemId,
	},
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod nonfungibles {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct NonFungibles;

	impl NonFungibles {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("PopApiNonFungiblesExample::new");
			Default::default()
		}

		/// 1. PSP-34 Interface:
		/// - total_supply
		/// - balance_of
		/// - allowance
		/// - transfer
		/// - approve
		/// - owner_of

		#[ink(message)]
		pub fn total_supply(&self, collection: CollectionId) -> Result<u128> {
			api::total_supply(collection)
		}

		#[ink(message)]
		pub fn balance_of(&self, collection: CollectionId, owner: AccountId) -> Result<u32> {
			api::balance_of(collection, owner)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			collection: CollectionId,
			owner: AccountId,
			operator: AccountId,
			item: Option<ItemId>,
		) -> Result<bool> {
			api::allowance(collection, owner, operator, item)
		}

		#[ink(message)]
		pub fn transfer(
			&mut self,
			collection: CollectionId,
			item: ItemId,
			to: AccountId,
		) -> Result<()> {
			api::transfer(collection, item, to)?;
			self.env().emit_event(Transfer {
				from: Some(self.env().account_id()),
				to: Some(to),
				item,
			});
			Ok(())
		}

		#[ink(message)]
		pub fn approve(
			&mut self,
			collection: CollectionId,
			item: Option<ItemId>,
			operator: AccountId,
			approved: bool,
		) -> Result<()> {
			api::approve(collection, item, operator, approved)?;
			self.env().emit_event(Approval {
				owner: self.env().account_id(),
				operator,
				item,
				approved,
			});
			Ok(())
		}

		#[ink(message)]
		pub fn owner_of(
			&self,
			collection: CollectionId,
			item: ItemId,
		) -> Result<Option<AccountId>> {
			api::owner_of(collection, item)
		}

		/// 2. PSP-34 Metadata Interface:
		/// - get_attribute

		#[ink(message)]
		pub fn get_attribute(
			&self,
			collection: CollectionId,
			item: ItemId,
			namespace: AttributeNamespace,
			key: Vec<u8>,
		) -> Result<Option<Vec<u8>>> {
			api::get_attribute(collection, item, namespace, key)
		}

		/// 3. Asset Management:
		/// - create
		/// - destroy
		/// - collection
		/// - set_attribute
		/// - clear_attribute
		/// - set_metadata
		/// - clear_metadata
		/// - approve_item_attributes
		/// - cancel_item_attributes_approval
		/// - set_max_supply
		/// - item_metadata

		#[ink(message)]
		pub fn create(&mut self, admin: AccountId, config: CreateCollectionConfig) -> Result<()> {
			api::create(admin, config)
		}

		#[ink(message)]
		pub fn destroy(&mut self, collection: CollectionId, witness: DestroyWitness) -> Result<()> {
			api::destroy(collection, witness)
		}

		#[ink(message)]
		pub fn collection(&self, collection: CollectionId) -> Result<Option<CollectionDetails>> {
			api::collection(collection)
		}

		#[ink(message)]
		pub fn set_attribute(
			&mut self,
			collection: CollectionId,
			item: ItemId,
			namespace: AttributeNamespace,
			key: Vec<u8>,
			value: Vec<u8>,
		) -> Result<()> {
			api::set_attribute(collection, item, namespace, key.clone(), value.clone())?;
			self.env().emit_event(AttributeSet { item, key, data: value });
			Ok(())
		}

		#[ink(message)]
		pub fn clear_attribute(
			&mut self,
			collection: CollectionId,
			item: ItemId,
			namespace: AttributeNamespace,
			key: Vec<u8>,
		) -> Result<()> {
			api::clear_attribute(collection, item, namespace, key)
		}

		#[ink(message)]
		pub fn set_metadata(
			&mut self,
			collection: CollectionId,
			item: ItemId,
			data: Vec<u8>,
		) -> Result<()> {
			api::set_metadata(collection, item, data)
		}

		#[ink(message)]
		pub fn clear_metadata(&mut self, collection: CollectionId, item: ItemId) -> Result<()> {
			api::clear_metadata(collection, item)
		}

		#[ink(message)]
		pub fn approve_item_attributes(
			&mut self,
			collection: CollectionId,
			item: ItemId,
			delegate: AccountId,
		) -> Result<()> {
			api::approve_item_attributes(collection, item, delegate)
		}

		#[ink(message)]
		pub fn cancel_item_attributes_approval(
			&mut self,
			collection: CollectionId,
			item: ItemId,
			delegate: AccountId,
			witness: CancelAttributesApprovalWitness,
		) -> Result<()> {
			api::cancel_item_attributes_approval(collection, item, delegate, witness)
		}

		#[ink(message)]
		pub fn set_max_supply(&mut self, collection: CollectionId, max_supply: u32) -> Result<()> {
			api::set_max_supply(collection, max_supply)
		}

		#[ink(message)]
		pub fn item_metadata(
			&mut self,
			collection: CollectionId,
			item: ItemId,
		) -> Result<Option<Vec<u8>>> {
			api::item_metadata(collection, item)
		}

		/// 4. PSP-22 Mintable & Burnable Interface:
		/// - mint
		/// - burn

		#[ink(message)]
		pub fn mint(
			&mut self,
			to: AccountId,
			collection: CollectionId,
			item: ItemId,
			mint_price: Option<u32>,
		) -> Result<()> {
			api::mint(to, collection, item, mint_price)
		}

		#[ink(message)]
		pub fn burn(&mut self, collection: CollectionId, item: ItemId) -> Result<()> {
			api::burn(collection, item)
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			PopApiNonFungiblesExample::new();
		}
	}
}

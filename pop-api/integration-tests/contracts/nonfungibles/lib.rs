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
		AttributeNamespace, CancelAttributesApprovalWitness, CollectionConfig, CollectionId,
		DestroyWitness, ItemId, MintWitness,
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
		/// - balance_of
		/// - owner_of
		/// - allowance
		/// - approve
		/// - transfer
		/// - total_supply

		#[ink(message)]
		pub fn balance_of(&self, collection: CollectionId, owner: AccountId) -> Result<u32> {
			api::balance_of(collection, owner)
		}

		#[ink(message)]
		pub fn owner_of(
			&self,
			collection: CollectionId,
			item: ItemId,
		) -> Result<Option<AccountId>> {
			api::owner_of(collection, item)
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
		pub fn approve(
			&mut self,
			collection: CollectionId,
			operator: AccountId,
			item: Option<ItemId>,
			approved: bool,
		) -> Result<()> {
			api::approve(collection, operator, item, approved)?;
			self.env().emit_event(Approval {
				owner: self.env().account_id(),
				operator,
				item,
				approved,
			});
			Ok(())
		}

		#[ink(message)]
		pub fn transfer(
			&mut self,
			collection: CollectionId,
			to: AccountId,
			item: ItemId,
		) -> Result<()> {
			api::transfer(collection, to, item)?;
			self.env().emit_event(Transfer {
				from: Some(self.env().account_id()),
				to: Some(to),
				item,
			});
			Ok(())
		}

		#[ink(message)]
		pub fn total_supply(&self, collection: CollectionId) -> Result<u128> {
			api::total_supply(collection)
		}

		/// 2. PSP-34 Metadata Interface:
		/// - get_attribute

		#[ink(message)]
		pub fn get_attribute(
			&self,
			collection: CollectionId,
			item: Option<ItemId>,
			namespace: AttributeNamespace,
			key: Vec<u8>,
		) -> Result<Option<Vec<u8>>> {
			api::get_attribute(collection, item, namespace, key)
		}

		/// 3. Asset Management:
		/// - next_collection_id
		/// - item_metadata
		/// - create
		/// - destroy
		/// - set_attribute
		/// - clear_attribute
		/// - set_metadata
		/// - clear_metadata
		/// - set_max_supply
		/// - approve_item_attributes
		/// - cancel_item_attributes_approval
		/// - clear_all_transfer_approvals
		/// - clear_collection_approvals

		#[ink(message)]
		pub fn next_collection_id(&self) -> Result<Option<CollectionId>> {
			api::next_collection_id()
		}

		#[ink(message)]
		pub fn item_metadata(
			&mut self,
			collection: CollectionId,
			item: ItemId,
		) -> Result<Option<Vec<u8>>> {
			api::item_metadata(collection, item)
		}

		#[ink(message)]
		pub fn create(&mut self, admin: AccountId, config: CollectionConfig) -> Result<()> {
			api::create(admin, config)
		}

		#[ink(message)]
		pub fn destroy(&mut self, collection: CollectionId, witness: DestroyWitness) -> Result<()> {
			api::destroy(collection, witness)
		}

		#[ink(message)]
		pub fn set_attribute(
			&mut self,
			collection: CollectionId,
			item: Option<ItemId>,
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
			item: Option<ItemId>,
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
		pub fn set_max_supply(&mut self, collection: CollectionId, max_supply: u32) -> Result<()> {
			api::set_max_supply(collection, max_supply)
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
		pub fn clear_all_transfer_approvals(
			&mut self,
			collection: CollectionId,
			item: ItemId,
		) -> Result<()> {
			api::clear_all_transfer_approvals(collection, item)
		}

		#[ink(message)]
		pub fn clear_collection_approvals(
			&mut self,
			collection: CollectionId,
			limit: u32,
		) -> Result<()> {
			api::clear_collection_approvals(collection, limit)
		}

		/// 4. PSP-34 Mintable & Burnable Interface:
		/// - mint
		/// - burn

		#[ink(message)]
		pub fn mint(
			&mut self,
			to: AccountId,
			collection: CollectionId,
			item: ItemId,
			witness: Option<MintWitness>,
		) -> Result<()> {
			api::mint(to, collection, item, witness)
		}

		#[ink(message)]
		pub fn burn(&mut self, collection: CollectionId, item: ItemId) -> Result<()> {
			api::burn(collection, item)
		}
	}
}

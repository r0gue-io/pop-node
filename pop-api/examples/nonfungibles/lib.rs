use ink::prelude::vec::Vec;
use pop_api::v0::nonfungibles::{
	self as api,
	events::{Approval, Transfer},
	traits::{Psp34, Psp34Enumerable, Psp34Metadata},
	types::{CollectionConfig, CollectionId, ItemId},
	Psp34Error,
};

#[cfg(test)]
mod tests;

#[ink::contract]
mod nonfungibles {
	use super::*;

	#[ink(storage)]
	pub struct NonFungible {
		id: CollectionId,
		owner: AccountId,
	}

	impl NonFungible {
		/// Instantiate the contract and create a new collection. The collection identifier will be
		/// stored in contract's storage.
		///
		/// # Parameters
		/// * - `id` - The identifier of the collection.
		/// * - `config` - The configuration of the collection.
		#[ink(constructor, payable)]
		pub fn new(id: CollectionId, config: CollectionConfig) -> Result<Self, Psp34Error> {
			let instance = Self { id, owner: Self::env().caller() };
			let contract_id = instance.env().account_id();
			api::create(contract_id, config).map_err(Psp34Error::from)?;
			Ok(instance)
		}
	}
	impl Psp34 for NonFungible {
		/// Returns the collection `Id`.
		#[ink(message)]
		fn collection_id(&self) -> ItemId {
			self.id
		}

		// Returns the current total supply of the NFT.
		#[ink(message)]
		fn total_supply(&self) -> u128 {
			api::total_supply(self.id).unwrap_or_default()
		}

		/// Returns the amount of items the owner has within a collection.
		///
		/// # Parameters
		/// - `owner` - The account whose balance is being queried.
		#[ink(message)]
		fn balance_of(&self, owner: AccountId) -> u32 {
			api::balance_of(self.id, owner).unwrap_or_default()
		}

		/// Returns whether the operator is approved by the owner to withdraw `item`. If `item` is
		/// `None`, it returns whether the operator is approved to withdraw all owner's items for
		/// the given collection.
		///
		/// # Parameters
		/// * `owner` - The account that owns the item(s).
		/// * `operator` - the account that is allowed to withdraw the item(s).
		/// * `item` - The item. If `None`, it is regarding all owner's items in collection.
		#[ink(message)]
		fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<ItemId>) -> bool {
			api::allowance(self.id, id, owner, operator).unwrap_or_default()
		}

		/// Transfers an owned or approved item to the specified recipient.
		///
		/// # Parameters
		/// * `to` - The recipient account.
		/// * `item` - The item.
		/// - `data` - Additional data in unspecified format.
		#[ink(message)]
		fn transfer(
			&mut self,
			to: AccountId,
			id: ItemId,
			_data: Vec<u8>,
		) -> Result<(), Psp34Error> {
			let contract = self.env().account_id();
			api::transfer(self.id, id, to).map_err(Psp34Error::from)?;
			self.env().emit_event(Transfer { from: Some(contract), to: Some(to), item: id });
			Ok(())
		}

		/// Approves operator to withdraw item(s) from the contract's account.
		///
		/// # Parameters
		/// * `operator` - The account that is allowed to withdraw the item.
		/// * `item` - Optional item. `None` means all items owned in the specified collection.
		/// * `approved` - Whether the operator is given or removed the right to withdraw the
		///   item(s).
		#[ink(message)]
		fn approve(
			&mut self,
			operator: AccountId,
			id: Option<ItemId>,
			approved: bool,
		) -> Result<(), Psp34Error> {
			let contract = self.env().account_id();
			api::approve(self.id, id, operator, approved).map_err(Psp34Error::from)?;
			let value = self.allowance(contract, operator, id);
			self.env()
				.emit_event(Approval { owner: contract, operator, item: id, approved });
			Ok(())
		}

		/// Returns the owner of an item within a specified collection, if any.
		///
		/// # Parameters
		/// * `item` - The item.
		#[ink(message)]
		fn owner_of(&self, id: ItemId) -> Option<AccountId> {
			api::owner_of(self.id, id).unwrap_or_default()
		}
	}
}

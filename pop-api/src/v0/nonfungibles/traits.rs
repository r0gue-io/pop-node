//! Traits that can be used by contracts. Including standard compliant traits.

use core::result::Result;

use super::*;

/// The PSP34 trait.
#[ink::trait_definition]
pub trait Psp34 {
	/// Returns the collection `Id`.
	#[ink(message, selector = 0xffa27a5f)]
	fn collection_id(&self) -> ItemId;

	// Returns the current total supply of the NFT.
	#[ink(message, selector = 0x628413fe)]
	fn total_supply(&self) -> u128;

	/// Returns the amount of items the owner has within a collection.
	///
	/// # Parameters
	/// - `owner` - The account whose balance is being queried.
	#[ink(message, selector = 0xcde7e55f)]
	fn balance_of(&self, owner: AccountId) -> u32;

	/// Returns whether the operator is approved by the owner to withdraw `item`. If `item` is
	/// `None`, it returns whether the operator is approved to withdraw all owner's items for the
	/// given collection.
	///
	/// # Parameters
	/// - `owner` - The account that owns the item(s).
	/// - `operator` - the account that is allowed to withdraw the item(s).
	/// - `item` - The item. If `None`, it is regarding all owner's items in collection.
	#[ink(message, selector = 0x4790f55a)]
	fn allowance(&self, owner: AccountId, operator: AccountId, id: Option<ItemId>) -> bool;

	/// Transfers an owned or approved item to the specified recipient.
	///
	/// # Parameters
	/// - `to` - The recipient account.
	/// - `item` - The item.
	/// - `data` - Additional data in unspecified format.
	#[ink(message, selector = 0x3128d61b)]
	fn transfer(&mut self, to: AccountId, item: ItemId, data: Vec<u8>) -> Result<(), Psp34Error>;

	/// Approves operator to withdraw item(s) from the contract's account.
	///
	/// # Parameters
	/// - `operator` - The account that is allowed to withdraw the item.
	/// - `item` - Optional item. `None` means all items owned in the specified collection.
	/// - `approved` - Whether the operator is given or removed the right to withdraw the item(s).
	#[ink(message, selector = 0x1932a8b0)]
	fn approve(
		&mut self,
		operator: AccountId,
		item: Option<ItemId>,
		approved: bool,
	) -> Result<(), Psp34Error>;

	/// Returns the owner of an item within a specified collection, if any.
	///
	/// # Parameters
	/// - `item` - The item.
	#[ink(message, selector = 0x1168624d)]
	fn owner_of(&self, item: ItemId) -> Option<AccountId>;
}

/// The PSP34 Metadata trait.
#[ink::trait_definition]
pub trait Psp34Metadata {
	/// Returns the attribute of `item` for the given `key`.
	///
	/// # Parameters
	/// - `item` - The item. If `None` the attributes for the collection are queried.
	/// - `key` - The key of the attribute.
	#[ink(message, selector = 0xf19d48d1)]
	fn get_attribute(&self, item: ItemId, key: Vec<u8>) -> Option<Vec<u8>>;
}

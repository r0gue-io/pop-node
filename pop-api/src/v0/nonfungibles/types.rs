use super::*;
use crate::primitives::AccountId;

pub type ItemId = u32;
pub type CollectionId = u32;
pub(super) type Balance = u32;

/// Information about a collection.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct CollectionDetails {
	/// Collection's owner.
	pub owner: AccountId,
	/// The total balance deposited by the owner for all the storage data associated with this
	/// collection. Used by `destroy`.
	pub owner_deposit: Balance,
	/// The total number of outstanding items of this collection.
	pub items: u32,
	/// The total number of outstanding item metadata of this collection.
	pub item_metadatas: u32,
	/// The total number of outstanding item configs of this collection.
	pub item_configs: u32,
	/// The total number of attributes for this collection.
	pub attributes: u32,
}

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct CreateCollectionConfig {
	pub max_supply: Option<u32>,
	pub mint_type: MintType,
	pub price: Option<Balance>,
	pub start_block: Option<BlockNumber>,
	pub end_block: Option<BlockNumber>,
}

/// Attribute namespaces for non-fungible tokens.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum AttributeNamespace {
	/// An attribute set by collection's owner.
	#[codec(index = 1)]
	CollectionOwner,
	/// An attribute set by item's owner.
	#[codec(index = 2)]
	ItemOwner,
	/// An attribute set by a pre-approved account.
	#[codec(index = 3)]
	Account(AccountId),
}

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum MintType {
	/// Only an `Issuer` could mint items.
	Issuer,
	/// Anyone could mint items.
	Public,
	/// Only holders of items in specified collection could mint new items.
	HolderOf(CollectionId),
}

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct CancelAttributesApprovalWitness {
	/// An amount of attributes previously created by account.
	pub account_attributes: u32,
}

/// Witness data for the destroy transactions.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct DestroyWitness {
	/// The total number of items in this collection that have outstanding item metadata.
	#[codec(compact)]
	pub item_metadatas: u32,
	/// The total number of outstanding item configs of this collection.
	#[codec(compact)]
	pub item_configs: u32,
	/// The total number of attributes for this collection.
	#[codec(compact)]
	pub attributes: u32,
}

/// Witness data for items mint transactions.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct MintWitness {
	/// Provide the id of the item in a required collection.
	pub owned_item: Option<ItemId>,
	/// The price specified in mint settings.
	pub mint_price: Option<Balance>,
}

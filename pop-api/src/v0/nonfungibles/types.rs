use super::*;
use crate::primitives::AccountId;

pub type ItemId = u32;
pub type Collection = u32;

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct CreateCollectionConfig<Price, BlockNumber, CollectionId> {
	pub max_supply: Option<u32>,
	pub mint_type: MintType<CollectionId>,
	pub price: Option<Price>,
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
pub enum MintType<CollectionId> {
	/// Only an `Issuer` could mint items.
	Issuer,
	/// Anyone could mint items.
	Public,
	/// Only holders of items in specified collection could mint new items.
	HolderOf(CollectionId),
}

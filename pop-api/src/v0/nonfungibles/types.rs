//! A set of data types for use in smart contracts interacting with the non-fungibles API.

use enumflags2::{bitflags, BitFlags};

use super::*;
use crate::primitives::AccountId;

type Balance = u32;
/// The identifier of a collection.
pub type CollectionId = u32;
/// The identifier of an item.
pub type ItemId = u32;

/// Witness data for the destroy transactions.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode)]
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
#[ink::scale_derive(Encode)]
pub struct MintWitness {
	/// Provide the id of the item in a required collection.
	pub owned_item: Option<ItemId>,
	/// The price specified in mint settings.
	pub mint_price: Option<Balance>,
}

/// Support for up to 64 user-enabled features on a collection.
#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode)]
pub enum CollectionSetting {
	/// Items in this collection are transferable.
	TransferableItems,
	/// The metadata of this collection can be modified.
	UnlockedMetadata,
	/// Attributes of this collection can be modified.
	UnlockedAttributes,
	/// The supply of this collection can be modified.
	UnlockedMaxSupply,
	/// When this isn't set then the deposit is required to hold the items of this collection.
	DepositRequired,
}

/// Wrapper type for `BitFlags<CollectionSetting>` that implements `Codec`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollectionSettings(pub BitFlags<CollectionSetting>);

impl CollectionSettings {
	/// Enable all features on a collection.
	pub fn all_enabled() -> Self {
		Self(BitFlags::EMPTY)
	}

	/// Provide `settings` bit flags indicate which features are turned off.
	pub fn from_disabled(settings: BitFlags<CollectionSetting>) -> Self {
		Self(settings)
	}
}

impl ink::scale::Encode for CollectionSettings {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}

/// Collection's configuration.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode)]
pub struct CollectionConfig {
	/// Collection's settings.
	pub settings: CollectionSettings,
	/// Collection's max supply.
	pub max_supply: Option<u32>,
	/// Default settings each item will get during the mint.
	pub mint_settings: MintSettings,
}

/// Mint type. Can the NFT be create by anyone, or only the creator of the collection,
/// or only by wallets that already hold an NFT from a certain collection?
/// The ownership of a privately minted NFT is still publicly visible.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode)]
pub enum MintType {
	/// Only an `Issuer` could mint items.
	Issuer,
	/// Anyone could mint items.
	Public,
	/// Only holders of items in specified collection could mint new items.
	HolderOf(CollectionId),
}

/// Holds the information about minting.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode)]
pub struct MintSettings {
	/// Whether anyone can mint or if minters are restricted to some subset.
	pub mint_type: MintType,
	/// An optional price per mint.
	pub price: Option<Balance>,
	/// When the mint starts.
	pub start_block: Option<BlockNumber>,
	/// When the mint ends.
	pub end_block: Option<BlockNumber>,
	/// Default settings each item will get during the mint.
	pub default_item_settings: ItemSettings,
}

/// Attribute namespaces for non-fungible tokens.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode)]
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

/// A witness data to cancel attributes approval operation.
#[derive(Debug)]
#[ink::scale_derive(Encode)]
pub struct CancelAttributesApprovalWitness {
	/// An amount of attributes previously created by account.
	pub account_attributes: u32,
}

/// Support for up to 64 user-enabled features on an item.
#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode)]
pub enum ItemSetting {
	/// This item is transferable.
	Transferable,
	/// The metadata of this item can be modified.
	UnlockedMetadata,
	/// Attributes of this item can be modified.
	UnlockedAttributes,
}

/// Wrapper type for `BitFlags<ItemSetting>` that implements `Codec`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemSettings(pub BitFlags<ItemSetting>);

impl ItemSettings {
	/// Enable all features on an item.
	pub fn all_enabled() -> Self {
		Self(BitFlags::EMPTY)
	}

	/// Provide `settings` bit flags indicate which features are turned off.
	pub fn from_disabled(settings: BitFlags<ItemSetting>) -> Self {
		Self(settings)
	}
}

impl ink::scale::Encode for ItemSettings {
	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.bits().using_encoded(f)
	}
}

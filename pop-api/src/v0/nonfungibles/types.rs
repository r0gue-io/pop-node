use enumflags2::{bitflags, BitFlags};

use super::*;
use crate::{macros::impl_codec_bitflags, primitives::AccountId};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

/// Collection's configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct CollectionConfig {
	/// Collection's settings.
	pub settings: CollectionSettings,
	/// Collection's max supply.
	pub max_supply: Option<u32>,
	/// Default settings each item will get during the mint.
	pub mint_settings: MintSettings,
}

/// Support for up to 64 user-enabled features on a collection.
#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
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
	pub fn from_disabled(settings: BitFlags<CollectionSetting>) -> Self {
		Self(settings)
	}

	#[cfg(feature = "std")]
	pub fn all_enabled() -> Self {
		Self(BitFlags::EMPTY)
	}
}

impl_codec_bitflags!(CollectionSettings, u64, CollectionSetting);

/// Support for up to 64 user-enabled features on an item.
#[bitflags]
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
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
	pub fn all_enabled() -> Self {
		Self(BitFlags::EMPTY)
	}

	#[cfg(feature = "std")]
	pub fn from_disabled(settings: BitFlags<ItemSetting>) -> Self {
		Self(settings)
	}
}

impl_codec_bitflags!(ItemSettings, u64, ItemSetting);

/// Holds the information about minting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
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

#[cfg(feature = "std")]
impl Default for MintSettings {
	fn default() -> Self {
		Self {
			mint_type: MintType::Issuer,
			price: None,
			start_block: None,
			end_block: None,
			default_item_settings: ItemSettings::all_enabled(),
		}
	}
}

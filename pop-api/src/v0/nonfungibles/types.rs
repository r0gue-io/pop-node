//! A set of data types for use in smart contracts interacting with the non-fungibles API.

use enumflags2::{bitflags, BitFlags};

use super::*;
use crate::{
	macros::impl_codec_bitflags,
	primitives::{AccountId, Balance},
};

/// The identifier of a collection.
pub type CollectionId = u32;
/// The identifier of an item.
pub type ItemId = u32;

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
	/// Enable all features on a collection.
	pub fn all_enabled() -> Self {
		Self(BitFlags::EMPTY)
	}

	/// Provide `settings` bit flags indicate which features are turned off.
	pub fn from_disabled(settings: BitFlags<CollectionSetting>) -> Self {
		Self(settings)
	}
}

impl_codec_bitflags!(CollectionSettings, u64, CollectionSetting);

/// Collection's configuration.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct CollectionConfig {
	/// Collection's settings.
	pub settings: CollectionSettings,
	/// Collection's max supply.
	pub max_supply: Option<u32>,
	/// Default settings each item will get during the mint.
	pub mint_settings: MintSettings,
}

/// Mint type. Can the NFT be created by anyone, or only the creator of the collection,
/// or only by wallets that already hold an NFT from a certain collection?
/// The ownership of a privately minted NFT is still publicly visible.
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

/// Holds the information about minting.
#[derive(Debug, PartialEq, Eq)]
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

/// A witness data to cancel attributes approval operation.
#[derive(Debug)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct CancelAttributesApprovalWitness {
	/// An amount of attributes previously created by account.
	pub account_attributes: u32,
}

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
	/// Enable all features on an item.
	pub fn all_enabled() -> Self {
		Self(BitFlags::EMPTY)
	}

	/// Provide `settings` bit flags indicate which features are turned off.
	pub fn from_disabled(settings: BitFlags<ItemSetting>) -> Self {
		Self(settings)
	}
}

impl_codec_bitflags!(ItemSettings, u64, ItemSetting);

#[cfg(test)]
mod tests {
	use scale::Encode;

	use super::*;

	#[test]
	fn ensure_destroy_witness() {
		assert_eq!(
			DestroyWitness {
				item_metadatas: u32::MAX,
				item_configs: u32::MAX,
				attributes: u32::MAX
			}
			.encode(),
			pallet_nfts::DestroyWitness {
				item_metadatas: u32::MAX,
				item_configs: u32::MAX,
				attributes: u32::MAX
			}
			.encode()
		);
	}

	#[test]
	fn ensure_mint_witness() {
		assert_eq!(
			MintWitness { owned_item: Some(ItemId::MAX), mint_price: Some(Balance::MAX) }.encode(),
			pallet_nfts::MintWitness::<ItemId, Balance> {
				owned_item: Some(ItemId::MAX),
				mint_price: Some(Balance::MAX)
			}
			.encode()
		);
	}

	#[test]
	fn ensure_collection_setting() {
		assert_eq!(
			vec![
				CollectionSetting::TransferableItems,
				CollectionSetting::UnlockedMetadata,
				CollectionSetting::UnlockedAttributes,
				CollectionSetting::UnlockedMaxSupply,
				CollectionSetting::DepositRequired,
			]
			.encode(),
			vec![
				pallet_nfts::CollectionSetting::TransferableItems,
				pallet_nfts::CollectionSetting::UnlockedMetadata,
				pallet_nfts::CollectionSetting::UnlockedAttributes,
				pallet_nfts::CollectionSetting::UnlockedMaxSupply,
				pallet_nfts::CollectionSetting::DepositRequired,
			]
			.encode()
		);
	}

	#[test]
	fn ensure_collection_settings() {
		assert_eq!(
			CollectionSettings::all_enabled().encode(),
			pallet_nfts::CollectionSettings::all_enabled().encode()
		);
	}

	#[test]
	fn ensure_collection_config() {
		assert_eq!(
			CollectionConfig {
				settings: CollectionSettings::all_enabled(),
				max_supply: Some(u32::MAX),
				mint_settings: default_mint_settings(),
			}
			.encode(),
			pallet_nfts::CollectionConfig {
				settings: pallet_nfts::CollectionSettings::all_enabled(),
				max_supply: Some(u32::MAX),
				mint_settings: default_pallet_mint_settings(),
			}
			.encode()
		);
	}

	#[test]
	fn ensure_mint_type() {
		assert_eq!(
			vec![MintType::Issuer, MintType::Public, MintType::HolderOf(CollectionId::MAX)]
				.encode(),
			vec![
				pallet_nfts::MintType::Issuer,
				pallet_nfts::MintType::Public,
				pallet_nfts::MintType::HolderOf(CollectionId::MAX)
			]
			.encode()
		);
	}

	#[test]
	fn ensure_mint_settings() {
		assert_eq!(default_mint_settings().encode(), default_pallet_mint_settings().encode());
	}

	#[test]
	fn ensure_attribute_namespace() {
		let account: AccountId = [0; 32].into();
		assert_eq!(
			vec![
				AttributeNamespace::CollectionOwner,
				AttributeNamespace::ItemOwner,
				AttributeNamespace::Account(account),
			]
			.encode(),
			vec![
				pallet_nfts::AttributeNamespace::<AccountId>::CollectionOwner,
				pallet_nfts::AttributeNamespace::<AccountId>::ItemOwner,
				pallet_nfts::AttributeNamespace::<AccountId>::Account(account)
			]
			.encode()
		);
	}

	#[test]
	fn ensure_cancel_attributes_approval_witness() {
		assert_eq!(
			CancelAttributesApprovalWitness { account_attributes: u32::MAX }.encode(),
			pallet_nfts::CancelAttributesApprovalWitness { account_attributes: u32::MAX }.encode(),
		);
	}

	#[test]
	fn ensure_item_setting() {
		assert_eq!(
			vec![
				ItemSetting::Transferable,
				ItemSetting::UnlockedMetadata,
				ItemSetting::UnlockedAttributes,
			]
			.encode(),
			vec![
				pallet_nfts::ItemSetting::Transferable,
				pallet_nfts::ItemSetting::UnlockedMetadata,
				pallet_nfts::ItemSetting::UnlockedAttributes
			]
			.encode()
		);
	}

	#[test]
	fn ensure_item_settings() {
		assert_eq!(
			ItemSettings::all_enabled().encode(),
			pallet_nfts::ItemSettings::all_enabled().encode()
		);
	}

	fn default_mint_settings() -> MintSettings {
		MintSettings {
			mint_type: MintType::Public,
			price: Some(Balance::MAX),
			start_block: Some(BlockNumber::MIN),
			end_block: Some(BlockNumber::MAX),
			default_item_settings: ItemSettings::all_enabled(),
		}
	}

	fn default_pallet_mint_settings(
	) -> pallet_nfts::MintSettings<Balance, BlockNumber, CollectionId> {
		pallet_nfts::MintSettings::<Balance, BlockNumber, CollectionId> {
			mint_type: pallet_nfts::MintType::Public,
			price: Some(Balance::MAX),
			start_block: Some(BlockNumber::MIN),
			end_block: Some(BlockNumber::MAX),
			default_item_settings: pallet_nfts::ItemSettings::all_enabled(),
		}
	}
}

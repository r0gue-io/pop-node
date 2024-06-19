use super::RuntimeCall;
use crate::{PopApiError, *};
use ink::prelude::vec::Vec;
use primitives::{ApprovalsLimit, BoundedBTreeMap, KeyLimit, MultiAddress};
pub use primitives::{CollectionId, ItemId};
use scale::Encode;
pub use types::*;

type Result<T> = core::result::Result<T, PopApiError>;

/// Issue a new collection of non-fungible items
pub fn create(
	admin: impl Into<MultiAddress<AccountId, ()>>,
	config: CollectionConfig,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::Nfts(NftCalls::Create { admin: admin.into(), config }))?)
}

/// Destroy a collection of fungible items.
pub fn destroy(collection: CollectionId) -> Result<()> {
	Ok(dispatch(RuntimeCall::Nfts(NftCalls::Destroy { collection }))?)
}

/// Mint an item of a particular collection.
pub fn mint(
	collection: CollectionId,
	item: ItemId,
	mint_to: impl Into<MultiAddress<AccountId, ()>>,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::Nfts(NftCalls::Mint {
		collection,
		item,
		mint_to: mint_to.into(),
		witness_data: None,
	}))?)
}

/// Destroy a single item.
pub fn burn(collection: CollectionId, item: ItemId) -> Result<()> {
	Ok(dispatch(RuntimeCall::Nfts(NftCalls::Burn { collection, item }))?)
}

/// Move an item from the sender account to another.
pub fn transfer(
	collection: CollectionId,
	item: ItemId,
	dest: impl Into<MultiAddress<AccountId, ()>>,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::Nfts(NftCalls::Transfer { collection, item, dest: dest.into() }))?)
}

/// Re-evaluate the deposits on some items.
pub fn redeposit(collection: CollectionId, items: Vec<ItemId>) -> Result<()> {
	Ok(dispatch(RuntimeCall::Nfts(NftCalls::Redeposit { collection, items }))?)
}

/// Change the Owner of a collection.
pub fn transfer_ownership(
	collection: CollectionId,
	new_owner: impl Into<MultiAddress<AccountId, ()>>,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::Nfts(NftCalls::TransferOwnership {
		collection,
		new_owner: new_owner.into(),
	}))?)
}

/// Set (or reset) the acceptance of ownership for a particular account.
pub fn set_accept_ownership(
	collection: CollectionId,
	maybe_collection: Option<CollectionId>,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::Nfts(NftCalls::SetAcceptOwnership { collection, maybe_collection }))?)
}

/// Set the maximum number of items a collection could have.
pub fn set_collection_max_supply(collection: CollectionId, max_supply: u32) -> Result<()> {
	Ok(dispatch(RuntimeCall::Nfts(NftCalls::SetCollectionMaxSupply { collection, max_supply }))?)
}

/// Update mint settings.
pub fn update_mint_settings(collection: CollectionId, mint_settings: MintSettings) -> Result<()> {
	Ok(dispatch(RuntimeCall::Nfts(NftCalls::UpdateMintSettings { collection, mint_settings }))?)
}

/// Get the owner of the item, if the item exists.
pub fn owner(collection: CollectionId, item: ItemId) -> Result<Option<AccountId>> {
	Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::Owner(collection, item)))?)
}

/// Get the owner of the collection, if the collection exists.
pub fn collection_owner(collection: CollectionId) -> Result<Option<AccountId>> {
	Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::CollectionOwner(collection)))?)
}

/// Get the details of a collection.
pub fn collection(collection: CollectionId) -> Result<Option<CollectionDetails>> {
	Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::Collection(collection)))?)
}

/// Get the details of an item.
pub fn item(collection: CollectionId, item: ItemId) -> Result<Option<ItemDetails>> {
	Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::Item(collection, item)))?)
}

pub mod approvals {
	use super::*;

	/// Approve an item to be transferred by a delegated third-party account.
	pub fn approve_transfer(
		collection: CollectionId,
		item: ItemId,
		delegate: impl Into<MultiAddress<AccountId, ()>>,
		maybe_deadline: Option<BlockNumber>,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::ApproveTransfer {
			collection,
			item,
			delegate: delegate.into(),
			maybe_deadline,
		}))?)
	}

	/// Cancel one of the transfer approvals for a specific item.
	pub fn cancel_approval(
		collection: CollectionId,
		item: ItemId,
		delegate: impl Into<MultiAddress<AccountId, ()>>,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::CancelApproval {
			collection,
			item,
			delegate: delegate.into(),
		}))?)
	}

	/// Cancel all the approvals of a specific item.
	pub fn clear_all_transfer_approvals(collection: CollectionId, item: ItemId) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::ClearAllTransferApprovals { collection, item }))?)
	}
}

pub mod attributes {
	use super::*;

	/// Approve item's attributes to be changed by a delegated third-party account.
	pub fn approve_item_attribute(
		collection: CollectionId,
		item: ItemId,
		delegate: impl Into<MultiAddress<AccountId, ()>>,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::ApproveItemAttributes {
			collection,
			item,
			delegate: delegate.into(),
		}))?)
	}

	/// Cancel the previously provided approval to change item's attributes.
	pub fn cancel_item_attributes_approval(
		collection: CollectionId,
		item: ItemId,
		delegate: impl Into<MultiAddress<AccountId, ()>>,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::CancelItemAttributesApproval {
			collection,
			item,
			delegate: delegate.into(),
		}))?)
	}

	/// Set an attribute for a collection or item.
	pub fn set_attribute(
		collection: CollectionId,
		maybe_item: Option<ItemId>,
		namespace: AttributeNamespace<AccountId>,
		key: BoundedVec<u8, KeyLimit>,
		value: BoundedVec<u8, KeyLimit>,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::SetAttribute {
			collection,
			maybe_item,
			namespace,
			key,
			value,
		}))?)
	}

	/// Clear an attribute for a collection or item.
	pub fn clear_attribute(
		collection: CollectionId,
		maybe_item: Option<ItemId>,
		namespace: AttributeNamespace<AccountId>,
		key: BoundedVec<u8, KeyLimit>,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::ClearAttribute {
			collection,
			maybe_item,
			namespace,
			key,
		}))?)
	}

	/// Get the attribute value of `item` of `collection` corresponding to `key`.
	pub fn attribute(
		collection: CollectionId,
		item: ItemId,
		key: BoundedVec<u8, KeyLimit>,
	) -> Result<Option<Vec<u8>>> {
		Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::Attribute(collection, item, key)))?)
	}

	// /// Get the custom attribute value of `item` of `collection` corresponding to `key`.
	// pub fn custom_attribute(
	// 	account: AccountId,
	// 	collection: CollectionId,
	// 	item: ItemId,
	// 	key: BoundedVec<u8, KeyLimit>,
	// ) -> Result<Option<Vec<u8>>> {
	// 	Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::CustomAttribute(
	// 		account, collection, item, key,
	// 	)))?)
	// }

	/// Get the system attribute value of `item` of `collection` corresponding to `key` if
	/// `item` is `Some`. Otherwise, returns the system attribute value of `collection`
	/// corresponding to `key`.
	pub fn system_attribute(
		collection: CollectionId,
		item: Option<ItemId>,
		key: BoundedVec<u8, KeyLimit>,
	) -> Result<Option<Vec<u8>>> {
		Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::SystemAttribute(collection, item, key)))?)
	}

	/// Get the attribute value of `item` of `collection` corresponding to `key`.
	pub fn collection_attribute(
		collection: CollectionId,
		key: BoundedVec<u8, KeyLimit>,
	) -> Result<Option<Vec<u8>>> {
		Ok(state::read(RuntimeStateKeys::Nfts(NftsKeys::CollectionAttribute(collection, key)))?)
	}
}

pub mod locking {
	use super::*;

	/// Disallows changing the metadata or attributes of the item.
	pub fn lock_item_properties(
		collection: CollectionId,
		item: ItemId,
		lock_metadata: bool,
		lock_attributes: bool,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::LockItemProperties {
			collection,
			item,
			lock_metadata,
			lock_attributes,
		}))?)
	}

	/// Disallow further unprivileged transfer of an item.
	pub fn lock_item_transfer(collection: CollectionId, item: ItemId) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::LockItemTransfer { collection, item }))?)
	}

	/// Re-allow unprivileged transfer of an item.
	pub fn unlock_item_transfer(collection: CollectionId, item: ItemId) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::UnlockItemTransfer { collection, item }))?)
	}

	/// Disallows specified settings for the whole collection.
	pub fn lock_collection(
		collection: CollectionId,
		lock_settings: CollectionSettings,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::LockCollection { collection, lock_settings }))?)
	}
}

pub mod metadata {
	use super::*;

	/// Set the metadata for an item.
	pub fn set_metadata(
		collection: CollectionId,
		item: ItemId,
		data: BoundedVec<u8, StringLimit>,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::SetMetadata { collection, item, data }))?)
	}

	/// Clear the metadata for an item.
	pub fn clear_metadata(collection: CollectionId, item: ItemId) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::ClearMetadata { collection, item }))?)
	}

	/// Set the metadata for a collection.
	pub fn set_collection_metadata(
		collection: CollectionId,
		data: BoundedVec<u8, StringLimit>,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::SetCollectionMetadata { collection, data }))?)
	}

	/// Clear the metadata for a collection.
	pub fn clear_collection_metadata(collection: CollectionId) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::ClearCollectionMetadata { collection }))?)
	}
}

pub mod roles {
	use super::*;

	/// Change the Issuer, Admin and Freezer of a collection.
	pub fn set_team(
		collection: CollectionId,
		issuer: Option<impl Into<MultiAddress<AccountId, ()>>>,
		admin: Option<impl Into<MultiAddress<AccountId, ()>>>,
		freezer: Option<impl Into<MultiAddress<AccountId, ()>>>,
	) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::SetTeam {
			collection,
			issuer: issuer.map(|i| i.into()),
			admin: admin.map(|i| i.into()),
			freezer: freezer.map(|i| i.into()),
		}))?)
	}
}

pub mod trading {
	use super::*;

	/// Allows to pay the tips.
	pub fn pay_tips(tips: BoundedVec<ItemTip, MaxTips>) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::PayTips { tips }))?)
	}

	/// Set (or reset) the price for an item.
	pub fn price(collection: CollectionId, item: ItemId, price: Option<Balance>) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::SetPrice { collection, item, price }))?)
	}

	/// Allows to buy an item if it's up for sale.
	pub fn buy_item(collection: CollectionId, item: ItemId, bid_price: Balance) -> Result<()> {
		Ok(dispatch(RuntimeCall::Nfts(NftCalls::BuyItem { collection, item, bid_price }))?)
	}

	pub mod swaps {
		use super::*;

		/// Register a new atomic swap, declaring an intention to send an `item` in exchange for
		/// `desired_item` from origin to target on the current chain.
		pub fn create_swap(
			offered_collection: CollectionId,
			offered_item: ItemId,
			desired_collection: CollectionId,
			maybe_desired_item: Option<ItemId>,
			maybe_price: Option<PriceWithDirection>,
			duration: BlockNumber,
		) -> Result<()> {
			Ok(dispatch(RuntimeCall::Nfts(NftCalls::CreateSwap {
				offered_collection,
				offered_item,
				desired_collection,
				maybe_desired_item,
				maybe_price,
				duration,
			}))?)
		}

		/// Cancel an atomic swap.
		pub fn cancel_swap(offered_collection: CollectionId, offered_item: ItemId) -> Result<()> {
			Ok(dispatch(RuntimeCall::Nfts(NftCalls::CancelSwap {
				offered_collection,
				offered_item,
			}))?)
		}

		/// Claim an atomic swap.
		pub fn claim_swap(
			send_collection: CollectionId,
			send_item: ItemId,
			receive_collection: CollectionId,
			receive_item: ItemId,
		) -> Result<()> {
			Ok(dispatch(RuntimeCall::Nfts(NftCalls::ClaimSwap {
				send_collection,
				send_item,
				receive_collection,
				receive_item,
			}))?)
		}
	}
}

#[derive(Encode)]
pub(crate) enum NftCalls {
	#[codec(index = 0)]
	Create { admin: MultiAddress<AccountId, ()>, config: CollectionConfig },
	#[codec(index = 2)]
	Destroy { collection: CollectionId },
	#[codec(index = 3)]
	Mint {
		collection: CollectionId,
		item: ItemId,
		mint_to: MultiAddress<AccountId, ()>,
		witness_data: Option<()>,
	},
	#[codec(index = 5)]
	Burn { collection: CollectionId, item: ItemId },
	#[codec(index = 6)]
	Transfer { collection: CollectionId, item: ItemId, dest: MultiAddress<AccountId, ()> },
	#[codec(index = 7)]
	Redeposit { collection: CollectionId, items: Vec<ItemId> },
	#[codec(index = 8)]
	LockItemTransfer { collection: CollectionId, item: ItemId },
	#[codec(index = 9)]
	UnlockItemTransfer { collection: CollectionId, item: ItemId },
	#[codec(index = 10)]
	LockCollection { collection: CollectionId, lock_settings: CollectionSettings },
	#[codec(index = 11)]
	TransferOwnership { collection: CollectionId, new_owner: MultiAddress<AccountId, ()> },
	#[codec(index = 12)]
	SetTeam {
		collection: CollectionId,
		issuer: Option<MultiAddress<AccountId, ()>>,
		admin: Option<MultiAddress<AccountId, ()>>,
		freezer: Option<MultiAddress<AccountId, ()>>,
	},
	#[codec(index = 15)]
	ApproveTransfer {
		collection: CollectionId,
		item: ItemId,
		delegate: MultiAddress<AccountId, ()>,
		maybe_deadline: Option<BlockNumber>,
	},
	#[codec(index = 16)]
	CancelApproval { collection: CollectionId, item: ItemId, delegate: MultiAddress<AccountId, ()> },
	#[codec(index = 17)]
	ClearAllTransferApprovals { collection: CollectionId, item: ItemId },
	#[codec(index = 18)]
	LockItemProperties {
		collection: CollectionId,
		item: ItemId,
		lock_metadata: bool,
		lock_attributes: bool,
	},
	#[codec(index = 19)]
	SetAttribute {
		collection: CollectionId,
		maybe_item: Option<ItemId>,
		namespace: AttributeNamespace<AccountId>,
		key: BoundedVec<u8, KeyLimit>,
		value: BoundedVec<u8, KeyLimit>,
	},
	#[codec(index = 21)]
	ClearAttribute {
		collection: CollectionId,
		maybe_item: Option<ItemId>,
		namespace: AttributeNamespace<AccountId>,
		key: BoundedVec<u8, KeyLimit>,
	},
	#[codec(index = 22)]
	ApproveItemAttributes {
		collection: CollectionId,
		item: ItemId,
		delegate: MultiAddress<AccountId, ()>,
	},
	#[codec(index = 23)]
	CancelItemAttributesApproval {
		collection: CollectionId,
		item: ItemId,
		delegate: MultiAddress<AccountId, ()>,
	},
	#[codec(index = 24)]
	SetMetadata { collection: CollectionId, item: ItemId, data: BoundedVec<u8, StringLimit> },
	#[codec(index = 25)]
	ClearMetadata { collection: CollectionId, item: ItemId },
	#[codec(index = 26)]
	SetCollectionMetadata { collection: CollectionId, data: BoundedVec<u8, StringLimit> },
	#[codec(index = 27)]
	ClearCollectionMetadata { collection: CollectionId },
	#[codec(index = 28)]
	SetAcceptOwnership { collection: CollectionId, maybe_collection: Option<CollectionId> },
	#[codec(index = 29)]
	SetCollectionMaxSupply { collection: CollectionId, max_supply: u32 },
	#[codec(index = 30)]
	UpdateMintSettings { collection: CollectionId, mint_settings: MintSettings },
	#[codec(index = 31)]
	SetPrice { collection: CollectionId, item: ItemId, price: Option<Balance> },
	#[codec(index = 32)]
	BuyItem { collection: CollectionId, item: ItemId, bid_price: Balance },
	#[codec(index = 33)]
	PayTips { tips: BoundedVec<ItemTip, MaxTips> },
	#[codec(index = 34)]
	CreateSwap {
		offered_collection: CollectionId,
		offered_item: ItemId,
		desired_collection: CollectionId,
		maybe_desired_item: Option<ItemId>,
		maybe_price: Option<PriceWithDirection>,
		duration: BlockNumber,
	},
	#[codec(index = 35)]
	CancelSwap { offered_collection: CollectionId, offered_item: ItemId },
	#[codec(index = 36)]
	ClaimSwap {
		send_collection: CollectionId,
		send_item: ItemId,
		receive_collection: CollectionId,
		receive_item: ItemId,
	},
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given item ID is unknown.
	UnknownCollection,
	/// The item ID has already been used for an item.
	AlreadyExists,
	/// The approval had a deadline that expired, so the approval isn't valid anymore.
	ApprovalExpired,
	/// The owner turned out to be different to what was expected.
	WrongOwner,
	/// The witness data given does not match the current state of the chain.
	BadWitness,
	/// Collection ID is already taken.
	CollectionIdInUse,
	/// Items within that collection are non-transferable.
	ItemsNonTransferable,
	/// The provided account is not a delegate.
	NotDelegate,
	/// The delegate turned out to be different to what was expected.
	WrongDelegate,
	/// No approval exists that would allow the transfer.
	Unapproved,
	/// The named owner has not signed ownership acceptance of the collection.
	Unaccepted,
	/// The item is locked (non-transferable).
	ItemLocked,
	/// Item's attributes are locked.
	LockedItemAttributes,
	/// Collection's attributes are locked.
	LockedCollectionAttributes,
	/// Item's metadata is locked.
	LockedItemMetadata,
	/// Collection's metadata is locked.
	LockedCollectionMetadata,
	/// All items have been minted.
	MaxSupplyReached,
	/// The max supply is locked and can't be changed.
	MaxSupplyLocked,
	/// The provided max supply is less than the number of items a collection already has.
	MaxSupplyTooSmall,
	/// The given item ID is unknown.
	UnknownItem,
	/// Swap doesn't exist.
	UnknownSwap,
	/// The given item has no metadata set.
	MetadataNotFound,
	/// The provided attribute can't be found.
	AttributeNotFound,
	/// Item is not for sale.
	NotForSale,
	/// The provided bid is too low.
	BidTooLow,
	/// The item has reached its approval limit.
	ReachedApprovalLimit,
	/// The deadline has already expired.
	DeadlineExpired,
	/// The duration provided should be less than or equal to `MaxDeadlineDuration`.
	WrongDuration,
	/// The method is disabled by system settings.
	MethodDisabled,
	/// The provided setting can't be set.
	WrongSetting,
	/// Item's config already exists and should be equal to the provided one.
	InconsistentItemConfig,
	/// Config for a collection or an item can't be found.
	NoConfig,
	/// Some roles were not cleared.
	RolesNotCleared,
	/// Mint has not started yet.
	MintNotStarted,
	/// Mint has already ended.
	MintEnded,
	/// The provided Item was already used for claiming.
	AlreadyClaimed,
	/// The provided data is incorrect.
	IncorrectData,
	/// The extrinsic was sent by the wrong origin.
	WrongOrigin,
	/// The provided signature is incorrect.
	WrongSignature,
	/// The provided metadata might be too long.
	IncorrectMetadata,
	/// Can't set more attributes per one call.
	MaxAttributesLimitReached,
	/// The provided namespace isn't supported in this call.
	WrongNamespace,
	/// Can't delete non-empty collections.
	CollectionNotEmpty,
	/// The witness data should be provided.
	WitnessRequired,
}

impl TryFrom<u32> for Error {
	type Error = PopApiError;

	fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
		use Error::*;
		match status_code {
			0 => Ok(NoPermission),
			1 => Ok(UnknownCollection),
			2 => Ok(AlreadyExists),
			3 => Ok(ApprovalExpired),
			4 => Ok(WrongOwner),
			5 => Ok(BadWitness),
			6 => Ok(CollectionIdInUse),
			7 => Ok(ItemsNonTransferable),
			8 => Ok(NotDelegate),
			9 => Ok(WrongDelegate),
			10 => Ok(Unapproved),
			11 => Ok(Unaccepted),
			12 => Ok(ItemLocked),
			13 => Ok(LockedItemAttributes),
			14 => Ok(LockedCollectionAttributes),
			15 => Ok(LockedItemMetadata),
			16 => Ok(LockedCollectionMetadata),
			17 => Ok(MaxSupplyReached),
			18 => Ok(MaxSupplyLocked),
			19 => Ok(MaxSupplyTooSmall),
			20 => Ok(UnknownItem),
			21 => Ok(UnknownSwap),
			22 => Ok(MetadataNotFound),
			23 => Ok(AttributeNotFound),
			24 => Ok(NotForSale),
			25 => Ok(BidTooLow),
			26 => Ok(ReachedApprovalLimit),
			27 => Ok(DeadlineExpired),
			28 => Ok(WrongDuration),
			29 => Ok(MethodDisabled),
			30 => Ok(WrongSetting),
			31 => Ok(InconsistentItemConfig),
			32 => Ok(NoConfig),
			33 => Ok(RolesNotCleared),
			34 => Ok(MintNotStarted),
			35 => Ok(MintEnded),
			36 => Ok(AlreadyClaimed),
			37 => Ok(IncorrectData),
			38 => Ok(WrongOrigin),
			39 => Ok(WrongSignature),
			40 => Ok(IncorrectMetadata),
			41 => Ok(MaxAttributesLimitReached),
			42 => Ok(WrongNamespace),
			43 => Ok(CollectionNotEmpty),
			44 => Ok(WitnessRequired),
			_ => todo!(),
		}
	}
}

// impl From<PopApiError> for Error {
// 	fn from(error: PopApiError) -> Self {
// 		match error {
// 			PopApiError::Nfts(e) => e,
// 			_ => panic!("unexpected pallet nfts error. This error is unknown to pallet nfts"),
// 		}
// 	}
// }

// Local implementations of pallet-nfts types
mod types {
	use super::*;
	use crate::{
		primitives::{CollectionId, ItemId},
		Balance, BlockNumber,
	};
	pub use enumflags2::{bitflags, BitFlags};
	use scale::{Decode, EncodeLike, MaxEncodedLen};
	use scale_info::{build::Fields, meta_type, prelude::vec, Path, Type, TypeInfo, TypeParameter};

	/// Attribute namespaces for non-fungible tokens.
	#[derive(Encode)]
	pub enum AttributeNamespace<AccountId> {
		/// An attribute was set by the pallet.
		Pallet,
		/// An attribute was set by collection's owner.
		CollectionOwner,
		/// An attribute was set by item's owner.
		ItemOwner,
		/// An attribute was set by pre-approved account.
		Account(AccountId),
	}

	/// Collection's configuration.
	#[derive(Encode)]
	pub struct CollectionConfig {
		/// Collection's settings.
		pub settings: CollectionSettings,
		/// Collection's max supply.
		pub max_supply: Option<u32>,
		/// Default settings each item will get during the mint.
		pub mint_settings: MintSettings,
	}

	/// Information about a collection.
	#[derive(Decode, Debug, Encode, Eq, PartialEq)]
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

	/// Wrapper type for `BitFlags<CollectionSetting>` that implements `Codec`.
	pub struct CollectionSettings(pub BitFlags<CollectionSetting>);

	impl_codec_bitflags!(CollectionSettings, u64, CollectionSetting);

	/// Support for up to 64 user-enabled features on a collection.
	#[bitflags]
	#[repr(u64)]
	#[derive(Copy, Clone, Encode, TypeInfo)]
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

	/// Information concerning the ownership of a single unique item.
	#[derive(Decode, Debug, Encode, Eq, PartialEq)]
	pub struct ItemDetails {
		/// The owner of this item.
		pub owner: AccountId,
		/// The approved transferrer of this item, if one is set.
		pub approvals: BoundedBTreeMap<AccountId, Option<BlockNumber>, ApprovalsLimit>,
		/// The amount held in the pallet's default account for this item. Free-hold items will
		/// have this as zero.
		pub deposit: Balance,
	}

	/// Support for up to 64 user-enabled features on an item.
	#[bitflags]
	#[repr(u64)]
	#[derive(Copy, Clone, Encode, TypeInfo)]
	pub enum ItemSetting {
		/// This item is transferable.
		Transferable,
		/// The metadata of this item can be modified.
		UnlockedMetadata,
		/// Attributes of this item can be modified.
		UnlockedAttributes,
	}

	/// Wrapper type for `BitFlags<ItemSetting>` that implements `Codec`.
	pub struct ItemSettings(pub BitFlags<ItemSetting>);

	impl_codec_bitflags!(ItemSettings, u64, ItemSetting);

	/// Information about the tip.
	#[derive(Encode)]
	pub struct ItemTip {
		/// The collection of the item.
		pub(super) collection: CollectionId,
		/// An item of which the tip is sent for.
		pub(super) item: ItemId,
		/// A sender of the tip.
		pub(super) receiver: AccountId,
		/// An amount the sender is willing to tip.
		pub(super) amount: Balance,
	}

	/// Holds the information about minting.
	#[derive(Encode)]
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

	/// Mint type. Can the NFT be created by anyone, or only the creator of the collection,
	/// or only by wallets that already hold an NFT from a certain collection?
	/// The ownership of a privately minted NFT is still publicly visible.
	#[derive(Encode)]
	pub enum MintType {
		/// Only an `Issuer` could mint items.
		Issuer,
		/// Anyone could mint items.
		Public,
		/// Only holders of items in specified collection could mint new items.
		HolderOf(CollectionId),
	}

	/// Holds the details about the price.
	#[derive(Encode)]
	pub struct PriceWithDirection {
		/// An amount.
		pub(super) amount: Balance,
		/// A direction (send or receive).
		pub(super) direction: PriceDirection,
	}

	/// Specifies whether the tokens will be sent or received.
	#[derive(Encode)]
	pub enum PriceDirection {
		/// Tokens will be sent.
		Send,
		/// Tokens will be received.
		Receive,
	}

	macro_rules! impl_codec_bitflags {
		($wrapper:ty, $size:ty, $bitflag_enum:ty) => {
			impl MaxEncodedLen for $wrapper {
				fn max_encoded_len() -> usize {
					<$size>::max_encoded_len()
				}
			}
			impl Encode for $wrapper {
				fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
					self.0.bits().using_encoded(f)
				}
			}
			impl EncodeLike for $wrapper {}
			impl Decode for $wrapper {
				fn decode<I: scale::Input>(
					input: &mut I,
				) -> core::result::Result<Self, scale::Error> {
					let field = <$size>::decode(input)?;
					Ok(Self(BitFlags::from_bits(field as $size).map_err(|_| "invalid value")?))
				}
			}

			impl TypeInfo for $wrapper {
				type Identity = Self;

				fn type_info() -> Type {
					Type::builder()
						.path(Path::new("BitFlags", module_path!()))
						.type_params(vec![TypeParameter::new(
							"T",
							Some(meta_type::<$bitflag_enum>()),
						)])
						.composite(
							Fields::unnamed()
								.field(|f| f.ty::<$size>().type_name(stringify!($bitflag_enum))),
						)
				}
			}
		};
	}
	pub(crate) use impl_codec_bitflags;
}

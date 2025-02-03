//! The `nonfungibles` module provides an API for interacting and managing non-fungible tokens.
//!
//! The API includes the following interfaces:
//! 1. PSP-34
//! 2. PSP-34 Metadata
//! 3. Management
//! 4. PSP-34 Mintable & Burnable

use constants::*;
pub use errors::*;
pub use events::*;
use ink::prelude::vec::Vec;
pub use traits::*;
pub use types::*;

use crate::{
	constants::{MODULE_ERROR, NFTS, NONFUNGIBLES},
	primitives::{AccountId, BlockNumber},
	ChainExtensionMethodApi, Result, StatusCode,
};

pub mod errors;
pub mod events;
pub mod traits;
pub mod types;

/// Returns the amount of items the owner has within a collection.
///
/// # Parameters
/// - `collection` - The collection.
/// - `owner` - The account whose balance is being queried.
#[inline]
pub fn balance_of(collection: CollectionId, owner: AccountId) -> Result<u32> {
	build_read_state(BALANCE_OF)
		.input::<(CollectionId, AccountId)>()
		.output::<Result<u32>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, owner))
}

/// Returns the owner of an item within a specified collection, if any.
///
/// # Parameters
/// - `collection` - The collection.
/// - `item` - The item.
#[inline]
pub fn owner_of(collection: CollectionId, item: ItemId) -> Result<Option<AccountId>> {
	build_read_state(OWNER_OF)
		.input::<(CollectionId, ItemId)>()
		.output::<Result<Option<AccountId>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item))
}

/// Returns whether the operator is approved by the owner to withdraw `item`. If `item` is
/// `None`, it returns whether the operator is approved to withdraw all owner's items for the
/// given collection.
///
/// # Parameters
/// - `collection` - The collection.
/// - `owner` - The account that owns the item(s).
/// - `operator` - the account that is allowed to withdraw the item(s).
/// - `item` - The item. If `None`, it is regarding all owner's items in collection.
#[inline]
pub fn allowance(
	collection: CollectionId,
	owner: AccountId,
	operator: AccountId,
	item: Option<ItemId>,
) -> Result<bool> {
	build_read_state(ALLOWANCE)
		.input::<(CollectionId, AccountId, AccountId, Option<ItemId>)>()
		.output::<Result<bool>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, owner, operator, item))
}

/// Approves operator to withdraw item(s) from the owner's account.
///
/// # Parameters
/// - `collection` - The collection.
/// - `operator` - The account that is allowed to withdraw the item.
/// - `item` - Optional item. `None` means all items owned in the specified collection.
/// - `approved` - Whether the operator is given or removed the right to withdraw the item(s).
#[inline]
pub fn approve(
	collection: CollectionId,
	operator: AccountId,
	item: Option<ItemId>,
	approved: bool,
) -> Result<()> {
	build_dispatch(APPROVE)
		.input::<(CollectionId, AccountId, Option<ItemId>, bool, Option<BlockNumber>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, operator, item, approved, None))
}

/// Transfers an owned or approved item to the specified recipient.
///
/// # Parameters
/// - `collection` - The collection.
/// - `to` - The recipient account.
/// - `item` - The item.
#[inline]
pub fn transfer(collection: CollectionId, to: AccountId, item: ItemId) -> Result<()> {
	build_dispatch(TRANSFER)
		.input::<(CollectionId, AccountId, ItemId)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, to, item))
}

/// Returns the total supply of a collection.
///
/// # Parameters
/// - `collection` - The collection.
#[inline]
pub fn total_supply(collection: CollectionId) -> Result<u128> {
	build_read_state(TOTAL_SUPPLY)
		.input::<CollectionId>()
		.output::<Result<u128>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection))
}

/// Returns the attribute of `item` for the given `key`.
///
/// # Parameters
/// - `collection` - The collection.
/// - `item` - The item. If `None` the attributes for the collection are queried.
/// - `namespace` - The attribute's namespace.
/// - `key` - The key of the attribute.
#[inline]
pub fn get_attribute(
	collection: CollectionId,
	item: Option<ItemId>,
	namespace: AttributeNamespace,
	key: Vec<u8>,
) -> Result<Option<Vec<u8>>> {
	build_read_state(GET_ATTRIBUTE)
		.input::<(CollectionId, Option<ItemId>, AttributeNamespace, Vec<u8>)>()
		.output::<Result<Option<Vec<u8>>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, namespace, key))
}

/// Returns the next collection identifier.
#[inline]
pub fn next_collection_id() -> Result<CollectionId> {
	build_read_state(NEXT_COLLECTION_ID)
		.output::<Result<CollectionId>, true>()
		.handle_error_code::<StatusCode>()
		.call(&())
}

/// Returns the metadata of the specified collection `item`.
///
/// # Parameters
/// - `collection` - The collection.
/// - `item` - The item.
#[inline]
pub fn item_metadata(collection: CollectionId, item: ItemId) -> Result<Option<Vec<u8>>> {
	build_read_state(ITEM_METADATA)
		.input::<(CollectionId, ItemId)>()
		.output::<Result<Option<Vec<u8>>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item))
}

/// Creates an NFT collection.
///
/// # Parameters
/// - `admin` - The admin account of the collection.
/// - `config` - Settings and config to be set for the new collection.
#[inline]
pub fn create(admin: AccountId, config: CollectionConfig) -> Result<()> {
	build_dispatch(CREATE)
		.input::<(AccountId, CollectionConfig)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(admin, config))
}

/// Destroy an NFT collection.
///
/// # Parameters
/// - `collection` - The collection to be destroyed.
/// - `witness` - Information on the items minted in the `collection`. This must be
/// correct.
#[inline]
pub fn destroy(collection: CollectionId, witness: DestroyWitness) -> Result<()> {
	build_dispatch(DESTROY)
		.input::<(CollectionId, DestroyWitness)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, witness))
}

/// Set an attribute for a collection or an item.
///
/// Namespace must be provided following the below ruleset:
/// - `CollectionOwner` namespace could be modified by the `collection` Admin only;
/// - `ItemOwner` namespace could be modified by the `item` owner only. `item` should be set in that
///   case;
/// - `Account(AccountId)` namespace could be modified only when the provided account was given a
///   permission to do so;

/// # Parameters
/// - `collection` - The collection.
/// - `item` - The optional item.
/// - `namespace` - The attribute's namespace.
/// - `key` - The key of the attribute.
/// - `value` - The value to which to set the attribute.
#[inline]
pub fn set_attribute(
	collection: CollectionId,
	item: Option<ItemId>,
	namespace: AttributeNamespace,
	key: Vec<u8>,
	value: Vec<u8>,
) -> Result<()> {
	build_dispatch(SET_ATTRIBUTE)
		.input::<(CollectionId, Option<ItemId>, AttributeNamespace, Vec<u8>, Vec<u8>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, namespace, key, value))
}

/// Clear an attribute for a collection or an item.
///
/// # Parameters
/// - `collection` - The collection.
/// - `item` - The optional item.
/// - `namespace` - The attribute's namespace.
/// - `key` - The key of the attribute.
#[inline]
pub fn clear_attribute(
	collection: CollectionId,
	item: Option<ItemId>,
	namespace: AttributeNamespace,
	key: Vec<u8>,
) -> Result<()> {
	build_dispatch(CLEAR_ATTRIBUTE)
		.input::<(CollectionId, Option<ItemId>, AttributeNamespace, Vec<u8>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, namespace, key))
}

/// Set the metadata for an item.
///
/// Caller must be the admin of the collection.
///
/// # Parameters
/// - `collection` - The collection.
/// - `item` - The item. If `None`, set metadata for the collection.
/// - `data` - the metadata.
#[inline]
pub fn set_metadata(collection: CollectionId, item: ItemId, data: Vec<u8>) -> Result<()> {
	build_dispatch(SET_METADATA)
		.input::<(CollectionId, ItemId, Vec<u8>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, data))
}

/// Clear the metadata for an item or collection.
///
/// Caller must be the admin of the collection.
///
/// # Parameters
/// - `collection` - The collection.
/// - `item` - The item.
#[inline]
pub fn clear_metadata(collection: CollectionId, item: ItemId) -> Result<()> {
	build_dispatch(CLEAR_METADATA)
		.input::<(CollectionId, ItemId)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item))
}

/// Set the maximum number of items a collection could have.
///
/// Caller must be the owner of the collection.
///
/// # Parameters
/// - `collection` - The collection.
/// - `max_supply` - The collection's max supply.
#[inline]
pub fn set_max_supply(collection: CollectionId, max_supply: u32) -> Result<()> {
	build_dispatch(SET_MAX_SUPPLY)
		.input::<(CollectionId, u32)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, max_supply))
}

/// Approve item's attributes to be changed by a delegated third-party account.
///
/// Caller must be the owner of the item.
///
/// # Parameters
/// - `collection` - The colleciton.
/// - `item` - The item.
/// - `delegate` - The account to delegate permission to change attributes of the item.
#[inline]
pub fn approve_item_attributes(
	collection: CollectionId,
	item: ItemId,
	delegate: AccountId,
) -> Result<()> {
	build_dispatch(APPROVE_ITEM_ATTRIBUTES)
		.input::<(CollectionId, ItemId, AccountId)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, delegate))
}

/// Cancel the previously provided approval to change item's attributes.
/// All the previously set attributes by the `delegate` will be removed.
///
/// # Parameters
/// - `collection` - The collection.
/// - `item` - The item that holds attributes.
/// - `delegate` - The previously approved account to remove.
/// - `witness` - A witness data to cancel attributes approval operation.
/// The account to delegate permission to change attributes of the item.
#[inline]
pub fn cancel_item_attributes_approval(
	collection: CollectionId,
	item: ItemId,
	delegate: AccountId,
	witness: CancelAttributesApprovalWitness,
) -> Result<()> {
	build_dispatch(CANCEL_ITEM_ATTRIBUTES_APPROVAL)
		.input::<(CollectionId, ItemId, AccountId, CancelAttributesApprovalWitness)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, delegate, witness))
}

/// Cancel all the approvals of a specific item.
///
/// # Parameters
/// - `collection` - The collection.
/// - `item` - The item of the collection of whose approvals will be cleared.
#[inline]
pub fn clear_all_transfer_approvals(collection: CollectionId, item: ItemId) -> Result<()> {
	build_dispatch(CLEAR_ALL_TRANSFER_APPROVALS)
		.input::<(CollectionId, ItemId)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item))
}

/// Cancel approvals to transfer all owner's collection items.
///
/// # Parameters
/// - `collection` - The collection.
/// - `limit` - The amount of collection approvals that will be cleared.
#[inline]
pub fn clear_collection_approvals(collection: CollectionId, limit: u32) -> Result<()> {
	build_dispatch(CLEAR_COLLECTION_APPROVALS)
		.input::<(CollectionId, u32)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, limit))
}

/// Mints an item to the specified address.
///
/// # Parameters
/// - `to` - The recipient account.
/// - `collection` - The collection.
/// - `item` - The ID for the item.
/// - `witness` - The optional witness data for items mint transactions.
#[inline]
pub fn mint(
	to: AccountId,
	collection: CollectionId,
	item: ItemId,
	witness: Option<MintWitness>,
) -> Result<()> {
	build_dispatch(MINT)
		.input::<(AccountId, CollectionId, ItemId, Option<MintWitness>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(to, collection, item, witness))
}

/// Destroys the specified item. Clearing the corresponding approvals.
///
/// # Parameters
/// - `collection` - The colleciton.
/// - `item` - The item.
#[inline]
pub fn burn(collection: CollectionId, item: ItemId) -> Result<()> {
	build_dispatch(BURN)
		.input::<(CollectionId, ItemId)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item))
}

mod constants {
	/// 1. PSP-34
	pub(super) const BALANCE_OF: u8 = 0;
	pub(super) const OWNER_OF: u8 = 1;
	pub(super) const ALLOWANCE: u8 = 2;
	pub(super) const APPROVE: u8 = 3;
	pub(super) const TRANSFER: u8 = 4;
	pub(super) const TOTAL_SUPPLY: u8 = 5;

	/// 2. PSP-34 Metadata
	pub(super) const GET_ATTRIBUTE: u8 = 6;

	/// 3. Management
	/// TODO: Replacement of `Collection` read.
	pub(super) const NEXT_COLLECTION_ID: u8 = 8;
	pub(super) const ITEM_METADATA: u8 = 9;
	pub(super) const CREATE: u8 = 10;
	pub(super) const DESTROY: u8 = 11;
	pub(super) const SET_ATTRIBUTE: u8 = 12;
	pub(super) const CLEAR_ATTRIBUTE: u8 = 13;
	pub(super) const SET_METADATA: u8 = 14;
	pub(super) const CLEAR_METADATA: u8 = 15;
	pub(super) const SET_MAX_SUPPLY: u8 = 16;
	pub(super) const APPROVE_ITEM_ATTRIBUTES: u8 = 17;
	pub(super) const CANCEL_ITEM_ATTRIBUTES_APPROVAL: u8 = 18;
	pub(super) const CLEAR_ALL_TRANSFER_APPROVALS: u8 = 19;
	pub(super) const CLEAR_COLLECTION_APPROVALS: u8 = 20;

	/// 4. PSP-34 Mintable & Burnable
	pub(super) const MINT: u8 = 21;
	pub(super) const BURN: u8 = 22;
}

// Helper method to build a dispatch call.
//
// Parameters:
// - 'dispatchable': The index of the dispatchable function within the module.
fn build_dispatch(dispatchable: u8) -> ChainExtensionMethodApi {
	crate::v0::build_dispatch(NONFUNGIBLES, dispatchable)
}

// Helper method to build a call to read state.
//
// Parameters:
// - 'state_query': The index of the runtime state query.
fn build_read_state(state_query: u8) -> ChainExtensionMethodApi {
	crate::v0::build_read_state(NONFUNGIBLES, state_query)
}

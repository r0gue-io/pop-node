//! The `nonfungibles` module provides an API for interacting and managing nonfungible tokens.
//!
//! The API includes the following interfaces:
//! 1. PSP-34
//! 2. PSP-34 Metadata
//! 3. PSP-22 Mintable & Burnable

use constants::*;
pub use errors::*;
pub use events::*;
use ink::prelude::vec::Vec;
pub use traits::*;
pub use types::*;

use crate::{
	constants::NONFUNGIBLES,
	primitives::{AccountId, BlockNumber},
	ChainExtensionMethodApi, Result, StatusCode,
};

pub mod errors;
pub mod events;
pub mod traits;
pub mod types;

#[inline]
pub fn total_supply(collection: CollectionId) -> Result<u32> {
	build_read_state(TOTAL_SUPPLY)
		.input::<CollectionId>()
		.output::<Result<u32>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection))
}

#[inline]
pub fn balance_of(collection: CollectionId, owner: AccountId) -> Result<u32> {
	build_read_state(BALANCE_OF)
		.input::<(CollectionId, AccountId)>()
		.output::<Result<u32>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, owner))
}

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

#[inline]
pub fn transfer(collection: CollectionId, item: ItemId, to: AccountId) -> Result<()> {
	build_dispatch(TRANSFER)
		.input::<(CollectionId, ItemId, AccountId)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, to))
}

#[inline]
pub fn approve(
	collection: CollectionId,
	item: ItemId,
	operator: AccountId,
	approved: bool,
) -> Result<()> {
	build_read_state(APPROVE)
		.input::<(CollectionId, ItemId, AccountId, bool)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, operator, approved))
}

#[inline]
pub fn owner_of(collection: CollectionId, item: ItemId) -> Result<AccountId> {
	build_read_state(OWNER_OF)
		.input::<(CollectionId, ItemId)>()
		.output::<Result<AccountId>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item))
}

#[inline]
pub fn get_attribute(
	collection: CollectionId,
	item: AccountId,
	namespace: AttributeNamespace,
	key: Vec<u8>,
) -> Result<Vec<u8>> {
	build_read_state(GET_ATTRIBUTE)
		.input::<(CollectionId, AccountId, AttributeNamespace, Vec<u8>)>()
		.output::<Result<Vec<u8>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, namespace, key))
}

#[inline]
pub fn create(admin: AccountId, config: CreateCollectionConfig) -> Result<()> {
	build_read_state(CREATE)
		.input::<(AccountId, CreateCollectionConfig)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(admin, config))
}

#[inline]
pub fn destroy(collection: CollectionId, witness: DestroyWitness) -> Result<()> {
	build_read_state(DESTROY)
		.input::<(CollectionId, DestroyWitness)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, witness))
}

#[inline]
pub fn collection(collection: CollectionId) -> Result<Option<CollectionDetails>> {
	build_read_state(COLLECTION)
		.input::<CollectionId>()
		.output::<Result<Option<CollectionDetails>>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection))
}

// TODO: ItemDetails.
#[inline]
pub fn item(collection: CollectionId, item: ItemId) -> Result<CollectionDetails> {
	build_read_state(ITEM)
		.input::<(CollectionId, ItemId)>()
		.output::<Result<CollectionDetails>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item))
}

#[inline]
pub fn set_attribute(
	collection: CollectionId,
	item: ItemId,
	namespace: AttributeNamespace,
	key: Vec<u8>,
	value: Vec<u8>,
) -> Result<()> {
	build_read_state(SET_ATTRIBUTE)
		.input::<(CollectionId, ItemId, AttributeNamespace, Vec<u8>, Vec<u8>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, namespace, key, value))
}

#[inline]
pub fn clear_attribute(
	collection: CollectionId,
	item: ItemId,
	namespace: AttributeNamespace,
	key: Vec<u8>,
) -> Result<()> {
	build_read_state(CLEAR_ATTRIBUTE)
		.input::<(CollectionId, ItemId, AttributeNamespace, Vec<u8>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, namespace, key))
}

#[inline]
pub fn set_metadata(collection: CollectionId, item: ItemId, data: Vec<u8>) -> Result<()> {
	build_read_state(SET_METADATA)
		.input::<(CollectionId, ItemId, Vec<u8>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, data))
}

#[inline]
pub fn clear_metadata(collection: CollectionId, item: ItemId) -> Result<()> {
	build_read_state(CLEAR_METADATA)
		.input::<(CollectionId, ItemId)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item))
}

#[inline]
pub fn approve_item_attributes(
	collection: CollectionId,
	item: ItemId,
	delegate: AccountId,
) -> Result<()> {
	build_read_state(CLEAR_METADATA)
		.input::<(CollectionId, ItemId, AccountId)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, delegate))
}

#[inline]
pub fn cancel_item_attributes_approval(
	collection: CollectionId,
	item: ItemId,
	delegate: AccountId,
	witness: CancelAttributesApprovalWitness,
) -> Result<()> {
	build_read_state(CLEAR_METADATA)
		.input::<(CollectionId, ItemId, AccountId, CancelAttributesApprovalWitness)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item, delegate, witness))
}

#[inline]
pub fn set_max_supply(collection: CollectionId, max_supply: u32) -> Result<()> {
	build_read_state(SET_MAX_SUPPLY)
		.input::<(CollectionId, u32)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, max_supply))
}

#[inline]
pub fn mint(
	to: AccountId,
	collection: CollectionId,
	item: ItemId,
	mint_price: Option<Balance>,
) -> Result<()> {
	build_read_state(MINT)
		.input::<(AccountId, CollectionId, ItemId, Option<Balance>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(to, collection, item, mint_price))
}

#[inline]
pub fn burn(collection: CollectionId, item: ItemId) -> Result<()> {
	build_read_state(BURN)
		.input::<(CollectionId, ItemId)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(collection, item))
}

mod constants {
	/// 1. PSP-34
	pub(super) const TOTAL_SUPPLY: u8 = 0;
	pub(super) const BALANCE_OF: u8 = 1;
	pub(super) const ALLOWANCE: u8 = 2;
	pub(super) const TRANSFER: u8 = 3;
	pub(super) const APPROVE: u8 = 4;
	pub(super) const OWNER_OF: u8 = 5;

	/// 2. PSP-34 Metadata
	pub(super) const GET_ATTRIBUTE: u8 = 6;

	/// 3. Management
	pub(super) const CREATE: u8 = 7;
	pub(super) const DESTROY: u8 = 8;
	pub(super) const COLLECTION: u8 = 9;
	pub(super) const ITEM: u8 = 10;
	pub(super) const NEXT_COLLECTION_ID: u8 = 11;
	pub(super) const SET_ATTRIBUTE: u8 = 12;
	pub(super) const CLEAR_ATTRIBUTE: u8 = 13;
	pub(super) const SET_METADATA: u8 = 14;
	pub(super) const CLEAR_METADATA: u8 = 15;
	pub(super) const APPROVE_ITEM_ATTRIBUTE: u8 = 16;
	pub(super) const CANCEL_ITEM_ATTRIBUTES_APPROVAL: u8 = 17;
	pub(super) const SET_MAX_SUPPLY: u8 = 18;

	/// 4. PSP-34 Mintable & Burnable
	pub(super) const MINT: u8 = 19;
	pub(super) const BURN: u8 = 20;
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

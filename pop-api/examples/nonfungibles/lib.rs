//! # Non-Fungibles Contract Example
//!
//! This [ink!][ink] contract implements a nonfungible token by leveraging the [Pop Non-Fungibles
//! API][pop-api-nonfungibles].
//!
//! ## Warning
//!
//! This contract is an *example* demonstrating usage of Pop's smart contract API. It is neither
//! audited nor endorsed for production use. Do **not** rely on it to keep anything of value secure.
//!
//! ## Features
//!
//! - Create an NFT collection.
//! - Mint NFTs.
//! - Transfer NFTs owned by the contract.
//! - Burn NFTs owned by the contract.
//! - Query ownership, balances, and total supply.
//! - Destroy the NFT collection and self-destruct the contract.
//!
//! ## Use Cases
//!
//! This contract can serve a variety of purposes where owner-controlled NFT management is
//! essential. Example use cases include:
//!
//! - **DAO Membership NFTs**: A DAO can use this contract to issue NFTs that represent membership
//!   or voting rights. The DAO's admin (or DAO contract itself) can mint or burn these NFTs based
//!   on proposals or membership status changes.
//! - **Event Tickets & Access Passes**: Projects can use this contract to distribute NFTs as
//!   tickets for events or as access passes to gated content or features, with the owner
//!   controlling who receives or revokes them.
//! - **Achievement Badges**: Platforms and games can mint NFTs as proof-of-achievement or user
//!   milestones, with full control over when and to whom they are issued.
//! - **Loyalty or Participation Rewards**: Businesses can issue NFTs as collectibles or recognition
//!   for customer loyalty or community participation, managed centrally by the contract owner.
//!
//! ## Notes
//!
//! - IMPORTANT: The contract must be deployed as **payable** to handle deposits, which are required
//!   for creating collections and minting NFTs. Ensure that sufficient value is therefore specified
//!   when instantiating the contract to cover the collection deposit - e.g., 10 PAS.
//! - Only the original deployer (owner) can call `mint`, `burn`, 'transfer' and `destroy`.
//! - Deposits are returned to the original deployer (owner) when the collection is destroyed.
//!
//! [ink]: https://use.ink
//! [pop-api-nonfungibles]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/src/v0/fungibles

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::string::ToString;
use pop_api::{
	nonfungibles::{
		self as api, events::Transfer, CollectionConfig, CollectionId, CollectionSettings,
		DestroyWitness, ItemId, ItemSettings, MintSettings, MintType, Psp34Error,
	},
	primitives::AccountId,
};

#[cfg(test)]
mod tests;

/// By default, Pop API returns errors as [`pop_api::StatusCode`], which are convertible to
/// [`Psp34Error`]. When using [`Psp34Error`], errors follow the PSP34 standard, making them easier
/// to interpret.
pub type Result<T> = core::result::Result<T, Psp34Error>;

/// Event emitted when a collection is created.
#[ink::event]
#[derive(Debug)]
pub struct Created {
	/// The collection.
	#[ink(topic)]
	id: CollectionId,
	/// The administrator of the collection.
	#[ink(topic)]
	admin: AccountId,
	/// Maximum number of items in the collection.
	max_supply: Option<u32>,
}

/// Event emitted when a collection is destroyed.
#[ink::event]
#[derive(Debug)]
pub struct Destroyed {
	/// The collection.
	#[ink(topic)]
	id: CollectionId,
}

#[ink::contract]
pub mod nonfungibles {
	use super::*;

	/// The contract represents (wraps) a single NFT collection.
	///
	/// Upon instantiation, it creates a new collection; upon termination, it destroys the
	/// collection. It also provides common methods to query collection data, and manage items
	/// through minting, burning, and transferring.
	#[ink(storage)]
	pub struct NonFungibles {
		/// Collection ID.
		id: CollectionId,
		/// Next item ID.
		next_item: ItemId,
		/// Owner of the contract and collection. Set to the contract's instantiator.
		owner: AccountId,
	}

	impl NonFungibles {
		/// Creates a new NFT collection.
		///
		/// On success a [`Created`] event is emitted.
		///
		/// # Arguments
		/// - `max_supply`: Maximum number of items in the collection.
		///
		/// # Notes
		/// - Creating a NFT collection requires a deposit, which is taken from the contract's
		///   balance on instantiation. The value provided at instantiation (via `payable`) must
		///   therefore be sufficient to cover the collection deposit.
		/// - Storage deposits are considered a best practice to mitigate against denial-of-service
		///   attacks and state bloat.
		/// - Destroying the collection later would refund the deposit back to the depositor.
		#[ink(constructor, payable)]
		pub fn new(max_supply: Option<u32>) -> Result<Self> {
			// Get the next available collection ID.
			let id = api::next_collection_id().map_err(Psp34Error::from)?;

			let instance = Self { id, next_item: 0, owner: Self::env().caller() };
			let contract_id = instance.env().account_id();

			// Set mint settings.
			let mint_settings = MintSettings {
				start_block: None,
				end_block: None,
				// Only the collection admin can mint.
				mint_type: MintType::Issuer,
				// Mint price is not set because only the contract can mint.
				price: None,
				default_item_settings: ItemSettings::all_enabled(),
			};

			// Create the collection.
			let config = CollectionConfig {
				settings: CollectionSettings::all_enabled(),
				max_supply,
				mint_settings,
			};
			// Contract is the admin of the collection.
			api::create(contract_id, config).map_err(Psp34Error::from)?;
			instance.env().emit_event(Created { id, admin: contract_id, max_supply });
			Ok(instance)
		}

		/// Returns the ID of the collection.
		#[ink(message)]
		pub fn collection_id(&self) -> CollectionId {
			self.id
		}

		/// Returns the next item ID of the collection.
		#[ink(message)]
		pub fn next_item_id(&self) -> ItemId {
			self.next_item
		}

		/// Returns the number of items owned by an account.
		#[ink(message)]
		pub fn balance_of(&self, owner: AccountId) -> u32 {
			api::balance_of(self.id, owner).unwrap_or_default()
		}

		/// Returns the owner of an item, if any.
		#[ink(message)]
		pub fn owner_of(&self, item: ItemId) -> Option<AccountId> {
			api::owner_of(self.id, item).unwrap_or_default()
		}

		/// Returns the total supply of the collection.
		#[ink(message)]
		pub fn total_supply(&self) -> u128 {
			api::total_supply(self.id).unwrap_or_default()
		}

		/// Mint an item.
		///
		/// On success a [`Transfer`] event is emitted.
		///
		/// # Arguments
		/// - `to`: Account into which the item will be minted.
		///
		/// # Note
		/// The deposit will be taken from the contract and not the specified `to` account. The
		/// contract must have sufficient balance to cover the storage deposit for a new item. The
		/// `mint` function is therefore made `payable` so that the caller can provide this deposit
		/// amount with each call.
		#[ink(message, payable)]
		pub fn mint(&mut self, to: AccountId) -> Result<()> {
			// Only the contract owner can call this method to mint the item, based on the mint
			// settings defined at collection creation.
			self.ensure_owner()?;
			let item = self.next_item;
			api::mint(to, self.id, item, None).map_err(Psp34Error::from)?;
			self.env().emit_event(Transfer { from: None, to: Some(to), item });
			self.next_item = self.next_item.saturating_add(1);
			Ok(())
		}

		/// Destroy a single item.
		///
		/// Item must be owned by the contract, and this method can only be called by the contract
		/// owner.
		///
		/// On success a [`Transfer`] event is emitted.
		///
		/// # Arguments
		/// - `item`: The item to be burned.
		#[ink(message)]
		pub fn burn(&mut self, item: ItemId) -> Result<()> {
			// Only the contract owner can burn items the contract owns.
			self.ensure_owner()?;
			api::burn(self.id, item).map_err(Psp34Error::from)?;
			self.env()
				.emit_event(Transfer { from: Some(self.env().account_id()), to: None, item });
			Ok(())
		}

		/// Move an item from the contract to another account.
		///
		/// Item must be owned by the contract.
		///
		/// On success a [`Transfer`] event is emitted.
		///
		/// # Arguments
		/// - `to`: The account to receive ownership of the item.
		/// - `item`: The item to be transferred.
		#[ink(message)]
		pub fn transfer(&mut self, to: AccountId, item: ItemId) -> Result<()> {
			// Only the contract owner can transfer items the contract owns.
			self.ensure_owner()?;
			api::transfer(self.id, to, item).map_err(Psp34Error::from)?;
			self.env().emit_event(Transfer {
				from: Some(self.env().account_id()),
				to: Some(to),
				item,
			});
			Ok(())
		}

		/// Destroy the collection.
		///
		/// Collection must be managed by the contract and not have any items.
		///
		/// On success a [`Destroyed`] event is emitted.
		///
		/// # Arguments
		/// - `destroy_witness`: The witness data required to destroy the collection.
		///
		/// # Notes
		/// Deposits will be returned to the contract automatically when a collection is destroyed.
		/// All deposits will be returned to the contract instantiator on contract termination.
		#[ink(message)]
		pub fn destroy(&mut self, destroy_witness: DestroyWitness) -> Result<()> {
			// Only the contract owner can destroy the contract/collection.
			self.ensure_owner()?;
			// Destroying the collection returns all deposits to the contract.
			api::destroy(self.id, destroy_witness).map_err(Psp34Error::from)?;
			self.env().emit_event(Destroyed { id: self.id });
			// Then terminating the contract returns all contract's funds to the contract
			// instantiator.
			self.env().terminate_contract(self.owner);
		}

		// Ensure that the caller is the contract owner.
		fn ensure_owner(&self) -> Result<()> {
			if self.env().caller() != self.owner {
				return Err(Psp34Error::Custom("Not the contract owner".to_string()))
			}
			Ok(())
		}
	}
}

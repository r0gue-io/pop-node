//! # Non-Fungibles Contract Example
//!
//! This [ink!][ink] contract implements a nonfungible token by leveraging the [Pop Non-Fungibles
//! API][pop-api-nonfungibles].
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
//! - The contract must be deployed as **payable** to handle deposits, which are required for
//!   creating collections and minting NFTs.
//! - Only the original deployer (owner) can call `mint`, `burn` and `destroy`.
//! - Deposits are returned to the original deployer (owner) when the collection is destroyed.
//!
//! [ink]: https://use.ink
//! [pop-api-nonfungibles]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/src/v0/fungibles

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::string::ToString;
use pop_api::{
	nonfungibles::{
		self as api, events::Transfer, CollectionConfig, CollectionId, CollectionSettings,
		DestroyWitness, ItemId, ItemSettings, MintSettings, MintType, MintWitness, Psp34Error,
	},
	primitives::AccountId,
};

/// By default, Pop API returns errors as [`pop_api::StatusCode`], which are convertible to
/// [`Psp34Error`]. When using [`Psp34Error`], errors follow the PSP34 standard, making them easier
/// to interpret.
pub type Result<T> = core::result::Result<T, Psp34Error>;

/// Event emitted when a collection is created.
#[ink::event]
pub struct Created {
	/// The collection.
	#[ink(topic)]
	id: CollectionId,
	/// The administrator of the collection.
	#[ink(topic)]
	admin: AccountId,
	/// Maximum number of items in the collection.
	max_supply: u32,
}

/// Event emitted when a collection is destroyed.
#[ink::event]
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
	/// Upon instantiation, it creates a new
	/// collection; upon termination, it destroys the collection. It also provides common methods
	/// to query collection data, and manage items through minting, burning, and transferring.
	#[ink(storage)]
	pub struct NonFungibles {
		/// Collection ID.
		id: CollectionId,
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
		/// - Creating a collection requires a deposit, which is taken from the contract's balance
		///   on instantiation. The value provided at instantiation (via `payable`) must therefore
		///   be sufficient to cover the collection deposit.
		/// - Destroying the collection later would refund the deposit back to the depositor.
		#[ink(constructor, payable)]
		pub fn new(max_supply: u32) -> Result<Self> {
			// Get the next available collection ID.
			let id = api::next_collection_id().map_err(Psp34Error::from)?.unwrap_or_default();

			let instance = Self { id, owner: Self::env().caller() };
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
				max_supply: Some(max_supply),
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

		/// Returns the number of items owned by an account.
		#[ink(message)]
		pub fn balance_of(&self, owner: AccountId) -> Result<u32> {
			api::balance_of(self.id, owner).map_err(Psp34Error::from)
		}

		/// Returns the owner of an item, if any.
		#[ink(message)]
		pub fn owner_of(&self, item: ItemId) -> Result<Option<AccountId>> {
			api::owner_of(self.id, item).map_err(Psp34Error::from)
		}

		/// Returns the total supply of the collection.
		#[ink(message)]
		pub fn total_supply(&self) -> Result<u128> {
			api::total_supply(self.id).map_err(Psp34Error::from)
		}

		/// Mint an item.
		///
		/// On success a [`Transfer`] event is emitted.
		///
		/// # Arguments
		/// - `to`: Account into which the item will be minted.
		/// - `item`: An identifier of the new item.
		/// - `witness_data`: Witness data for the mint operation.
		///
		/// # Note
		/// The deposit will be taken from the contract and not the `owner` of the
		/// `item`. The contract must have sufficient balance to cover the storage deposit for a new
		/// item.
		#[ink(message, payable)]
		pub fn mint(&mut self, to: AccountId, item: ItemId, witness: MintWitness) -> Result<()> {
			// Only the contract owner can call this method to mint the item.
			self.ensure_owner()?;
			api::mint(to, self.id, item, Some(witness)).map_err(Psp34Error::from)?;
			self.env().emit_event(Transfer { from: None, to: Some(to), item });
			Ok(())
		}

		/// Destroy a single item. Item must be owned by the contract, and this method can only be
		/// called by the contract itself.
		///
		/// On success a [`Transfer`] event is emitted.
		///
		/// # Arguments
		/// - `item`: The item to be burned.
		#[ink(message)]
		pub fn burn(&mut self, item: ItemId) -> Result<()> {
			// Only the contract owner can burn items from the collection.
			self.ensure_owner()?;
			api::burn(self.id, item).map_err(Psp34Error::from)?;
			self.env()
				.emit_event(Transfer { from: Some(self.env().account_id()), to: None, item });
			Ok(())
		}

		/// Move an item from the contract to another account. Item must be owned by the contract.
		///
		/// On success a [`Transfer`] event is emitted.
		///
		/// # Arguments
		/// - `item`: The item to be transferred.
		/// - `dest`: The account to receive ownership of the item.
		#[ink(message)]
		pub fn transfer(&mut self, to: AccountId, item: ItemId) -> Result<()> {
			self.ensure_owner()?;
			api::transfer(self.id, to, item).map_err(Psp34Error::from)?;
			self.env().emit_event(Transfer {
				from: Some(self.env().account_id()),
				to: Some(to),
				item,
			});
			Ok(())
		}

		/// Terminate a contract and destroy the collection. Collection must be managed by the
		/// contract.
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

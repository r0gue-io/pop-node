#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::string::ToString;
use pop_api::nonfungibles::{
	self as api, events::Transfer, CollectionConfig, CollectionId, CollectionSettings,
	DestroyWitness, ItemId, ItemSettings, MintSettings, MintType, MintWitness, Psp34Error,
};

#[ink::contract]
mod nonfungibles {
	use super::*;

	/// The contract represents a single NFT collection.
	///
	/// Upon instantiation, it creates a new
	/// collection; upon termination, it destroys the collection. It also provides common methods
	/// to query collection data, and manage items through minting, burning, and transferring.
	#[ink(storage)]
	pub struct NonFungibles {
		/// Collection ID
		id: CollectionId,
		/// Owner of the collection
		owner: AccountId,
	}

	impl NonFungibles {
		/// Creates a new NFT collection.
		///
		/// # Arguments
		/// - `max_supply`: Maximum number of items in the collection.
		/// - `price`: Mint price.
		///
		/// # Notes
		/// - Creating a collection requires a deposit, taken from the contract's balance on
		///   instantiation.
		/// - Destroying the collection later would refund the deposit back to the depositor.
		#[ink(constructor, payable)]
		pub fn new(max_supply: u32, price: u128) -> Result<Self, Psp34Error> {
			ink::env::debug_println!("PopApiNonFungiblesExample::new");

			// Get the next available collection ID.
			let id = api::next_collection_id().map_err(Psp34Error::from)?.unwrap_or_default();

			let instance = Self { id, owner: Self::env().caller() };
			let contract_id = instance.env().account_id();

			// Set mint settings: public minting, mint price, all item settings enabled.
			let mint_settings = MintSettings {
				start_block: None,
				end_block: None,
				mint_type: MintType::Public,
				price: Some(price),
				default_item_settings: ItemSettings::all_enabled(),
			};

			// Create the collection.
			api::create(
				// Contract is the admin of the collection.
				contract_id,
				CollectionConfig {
					settings: CollectionSettings::all_enabled(),
					max_supply: Some(max_supply),
					mint_settings,
				},
			)
			.map_err(Psp34Error::from)?;
			Ok(instance)
		}

		#[ink(message)]
		pub fn collection_id(&self) -> CollectionId {
			self.id
		}

		/// Returns the amount of items the owner.
		#[ink(message)]
		pub fn balance_of(&self, owner: AccountId) -> Result<u32, Psp34Error> {
			api::balance_of(self.id, owner).map_err(Psp34Error::from)
		}

		/// Returns the owner of an item, if any.
		#[ink(message)]
		pub fn owner_of(&self, item: ItemId) -> Result<Option<AccountId>, Psp34Error> {
			api::owner_of(self.id, item).map_err(Psp34Error::from)
		}

		/// Returns the total supply of a collection.
		#[ink(message)]
		pub fn total_supply(&self) -> Result<u128, Psp34Error> {
			api::total_supply(self.id).map_err(Psp34Error::from)
		}

		/// Mint an item.
		///
		/// # Arguments
		/// - `to`: Account into which the item will be minted.
		/// - `item`: An identifier of the new item.
		/// - `witness_data`: When the mint type is `HolderOf(collection_id)`, then the owned
		///   item_id from the current collection needs to be provided within the witness data
		///   object. If the mint price is set, then it should be additionally confirmed in the
		///   `witness_data`.
		///
		/// Note: the deposit will be taken from the contract caller and not the `owner` of the
		/// `item`.
		#[ink(message, payable)]
		pub fn mint(
			&mut self,
			to: AccountId,
			item: ItemId,
			witness: Option<MintWitness>,
		) -> Result<(), Psp34Error> {
			api::mint(to, self.id, item, witness).map_err(Psp34Error::from)
		}

		/// Destroy a single item. Item must be owned by the contract.
		///
		/// # Arguments
		/// - `item`: The item to be burned.
		#[ink(message)]
		pub fn burn(&mut self, item: ItemId) -> Result<(), Psp34Error> {
			api::burn(self.id, item).map_err(Psp34Error::from)
		}

		/// Move an item from the contract to another account. Item must be owned by the contract.
		///
		/// # Arguments
		/// - `item`: The item to be transferred.
		/// - `dest`: The account to receive ownership of the item.
		#[ink(message)]
		pub fn transfer(&mut self, to: AccountId, item: ItemId) -> Result<(), Psp34Error> {
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
		/// # Arguments
		/// - `destroy_witness`: The witness data required to destroy the collection.
		///
		/// # Notes
		/// Deposits will be returned to the contract automatically when collection destroyed. On
		/// contract terminated, all deposits will be returned to the contract caller.
		#[ink(message)]
		pub fn destroy(&mut self, destroy_witness: DestroyWitness) -> Result<(), Psp34Error> {
			if self.env().caller() != self.owner {
				return Err(Psp34Error::Custom("Not the contract owner".to_string()))
			}
			api::destroy(self.id, destroy_witness).map_err(Psp34Error::from)?;
			self.env().terminate_contract(self.env().account_id());
		}
	}
}

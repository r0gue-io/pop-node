#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::string::ToString;
use pop_api::{
	nonfungibles::{
		self as api, events::Transfer, CollectionConfig, CollectionId, CollectionSettings,
		DestroyWitness, ItemId, ItemSettings, MintSettings, MintType, MintWitness, Psp34Error,
	},
	primitives::AccountId,
};

/// By default, Pop API returns errors as `StatusCode` and it is convertible to `Psp34Error`.
/// When using `Psp34Error`, errors follow the PSP34 standard, making them easier to interpret.
type Result<T> = core::result::Result<T, Psp34Error>;

/// Event emitted when a collection is created.
#[ink::event]
struct Created {
	/// The collection.
	#[ink(topic)]
	id: CollectionId,
	/// The administrator of the collection.
	#[ink(topic)]
	admin: AccountId,
	/// Maximum number of items in the collection.
	max_supply: u32,
	/// Mint price.
	price: u128,
}

/// Event emitted when a collection is destroyed.
#[ink::event]
struct Destroyed {
	/// The collection.
	#[ink(topic)]
	id: CollectionId,
}

#[ink::contract]
mod nonfungibles {
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
		/// # Arguments
		/// - `max_supply`: Maximum number of items in the collection.
		/// - `price`: Mint price.
		///
		/// # Notes
		/// - Creating a collection requires a deposit, which is taken from the contract's balance
		///   on instantiation. The value provided at instantiation (via `payable`) must therefore
		///   be sufficient to cover the collection deposit.
		/// - Destroying the collection later would refund the deposit back to the depositor.
		#[ink(constructor, payable)]
		pub fn new(max_supply: u32, price: u128) -> Result<Self> {
			// Get the next available collection ID.
			let id = api::next_collection_id().map_err(Psp34Error::from)?.unwrap_or_default();

			let instance = Self { id, owner: Self::env().caller() };
			let contract_id = instance.env().account_id();

			// Set mint settings: public minting, mint price, all item settings enabled.
			let mint_settings = MintSettings {
				start_block: None,
				end_block: None,
				mint_type: MintType::Issuer,
				price: Some(price),
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
			instance.env().emit_event(Created { id, admin: contract_id, max_supply, price });
			Ok(instance)
		}

		/// Returns the ID of the collection.
		#[ink(message)]
		pub fn collection_id(&self) -> CollectionId {
			self.id
		}

		/// Returns the amount of items owned by an account.
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
		/// # Arguments
		/// - `to`: Account into which the item will be minted.
		/// - `item`: An identifier of the new item.
		/// - `witness_data`: If the mint price is set, then it should be additionally confirmed in
		///   the `witness_data`.
		///
		/// Note: the deposit will be taken from the contract and not the `owner` of the
		/// `item`. The value provided (via `payable`) must therefore be sufficient to cover the
		/// deposit and the mint price.
		#[ink(message, payable)]
		pub fn mint(&mut self, to: AccountId, item: ItemId, witness: MintWitness) -> Result<()> {
			api::mint(to, self.id, item, Some(witness)).map_err(Psp34Error::from)?;
			self.env().emit_event(Transfer { from: None, to: Some(to), item });
			Ok(())
		}

		/// Destroy a single item. Item must be owned by the contract and this method can only be
		/// called by the contract itself.
		///
		/// # Arguments
		/// - `item`: The item to be burned.
		fn burn(&mut self, item: ItemId) -> Result<()> {
			api::burn(self.id, item).map_err(Psp34Error::from)?;
			self.env()
				.emit_event(Transfer { from: Some(self.env().account_id()), to: None, item });
			Ok(())
		}

		/// Move an item from the contract to another account. Item must be owned by the contract.
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
		/// # Arguments
		/// - `destroy_witness`: The witness data required to destroy the collection.
		///
		/// # Notes
		/// Deposits will be returned to the contract automatically when collection destroyed. On
		/// contract terminated, all deposits will be returned to the contract instantiator.
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

		fn ensure_owner(&self) -> Result<()> {
			if self.env().caller() != self.owner {
				return Err(Psp34Error::Custom("Not the contract owner".to_string()))
			}
			Ok(())
		}
	}
}

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::storage::Mapping;
use pop_api::{
	nonfungibles::{CollectionId, ItemId},
	StatusCode,
};

pub type Result<T> = core::result::Result<T, Error>;

#[ink::contract]
mod dao {
	use nft_verifier::NftVerifierRef;

	use super::{Error::*, *};

	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	enum Status {
		Pending,
		Used,
	}

	#[ink(storage)]
	pub struct Dao {
		verifier: NftVerifierRef,
		// collection_id: CollectionId,
		// next_item_id: ItemId,
		// used_items: Mapping<ItemId, Status>,
	}

	impl Dao {
		#[ink(constructor, payable)]
		pub fn new(code_hash: Hash) -> Result<Self> {
			let balance = Self::env().balance();
			// api::register(beneficiary)?;
			let verifier = NftVerifierRef::new(1000u32, 0u32)
				.endowment(balance / 2)
				.code_hash(code_hash)
				.salt(0.to_le_bytes())
				.instantiate()?;
			// let config = CreateCollectionConfig {
			//     max_supply: 0,
			//     mint_type: MintType::Issuer,
			//     price: None,
			//     start_block: None,
			//     end_block: None,
			// };
			// let collection_id = api::next_collection_id();
			// api::create(Self::env().account_id(), config)?;
			Self { verifier }
		}

		#[ink(message, payable)]
		pub fn register(&mut self, height: u32, item: ItemId) -> Result<()> {
			let account = self.env().caller();
			self.verifier.verify(height, item)?;
			self.used_items.insert(item, Status::Pending);
			self.env().emit_event(RegistrationRequested { account, item });
		}

		#[ink(message, payable)]
		pub fn complete(&mut self, item: ItemId) -> Result<ItemId> {
			let account = self.env().caller();
			if self.verifier.complete(item)? {
				todo!();
				// self.next_item_id = self.next_item_id.saturating_add(1);
				let item = self.next_item_id;
				// api::mint(account, self.collection_id, item, None)?;
				// api::sponsor_account(account)?;
				self.used_items.insert(item, Status::Used);
			} else {
				return Err(Rejected);
			}
			Ok(item)
		}

		// #[ink(message)]
		// pub fn claim_rewards(&mut self, era: Era) -> Result<()> {
		//     api::claim(era)
		// }
		// fn register() -> Result<()> {
		// 	pop_api::incentives::
		// }
	}

	#[ink::event]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct RegistrationRequested {
		pub account: AccountId,
		pub item: ItemId,
	}

	#[derive(Debug, PartialEq, Eq)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub enum Error {
		StatusCode(u32),
		VerifierError(nft_verifier::Error),
		Rejected,
	}

	impl From<StatusCode> for Error {
		fn from(value: StatusCode) -> Self {
			Error::StatusCode(value.0)
		}
	}

	impl From<nft_verifier::Error> for Error {
		fn from(value: nft_verifier::Error) -> Self {
			Error::VerifierError(value)
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			let dao = Dao::default();
			assert_eq!(dao.get(), false);
		}
	}
}

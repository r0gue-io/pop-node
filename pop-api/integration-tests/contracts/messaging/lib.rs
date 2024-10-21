#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use pop_api::{
	messaging::{
		self as api,
		ismp::{Get, Post},
		xcm::{QueryId, VersionedLocation},
		RequestId, Status,
	},
	primitives::AccountId,
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

// Converts a H160 read as a H256 to a H256 expected by revive
// TODO: remove once ink! updated
fn to_account_id(address: &AccountId) -> AccountId {
	let mut account_id = AccountId::from([0xEE; 32]);
	<AccountId as AsMut<[u8; 32]>>::as_mut(&mut account_id)[..20]
		.copy_from_slice(&<AccountId as AsRef<[u8; 32]>>::as_ref(&address)[..20]);
	account_id
}

#[ink::contract]
mod messaging {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Contract;

	impl Contract {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			Default::default()
		}

		#[ink(message)]
		pub fn ismp_get(&mut self, id: RequestId, request: Get, fee: Balance) -> Result<()> {
			api::ismp::get(id, request, fee)?;
			Ok(())
		}

		#[ink(message)]
		pub fn ismp_post(&mut self, id: RequestId, request: Post, fee: Balance) -> Result<()> {
			api::ismp::post(id, request, fee)?;
			Ok(())
		}

		#[ink(message)]
		pub fn xcm_new_query(
			&mut self,
			id: RequestId,
			responder: VersionedLocation,
			timeout: BlockNumber,
		) -> Result<Option<QueryId>> {
			api::xcm::new_query(id, responder, timeout)
		}

		#[ink(message)]
		pub fn poll(&self, request: RequestId) -> Result<Option<Status>> {
			api::poll((to_account_id(&self.env().account_id()), request))
		}

		#[ink(message)]
		pub fn get(&self, request: RequestId) -> Result<Option<Vec<u8>>> {
			api::get((to_account_id(&self.env().account_id()), request))
		}

		#[ink(message)]
		pub fn remove(&mut self, request: RequestId) -> Result<()> {
			api::remove([request].to_vec())?;
			Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			Contract::new();
		}
	}
}

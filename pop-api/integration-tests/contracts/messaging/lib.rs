#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use pop_api::{
	messaging::{
		self as api,
		ismp::{Get, Post},
		xcm::{Junction, Location, QueryId, VersionedLocation},
		RequestId, Status,
	},
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

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
			// responder: VersionedLocation,
			// Workaround for 'polkavm::interpreter] Store of 4 bytes to 0xfffdefcc failed! (pc =
			// 8904, cycle = 239)' when using VersionedLocation
			responder: Option<u32>,
			timeout: BlockNumber,
		) -> Result<Option<QueryId>> {
			let responder = match responder {
				Some(para) => Location::new(1, [Junction::Parachain(para)]),
				None => Location::parent(),
			}
			.into_versioned();
			api::xcm::new_query(id, responder, timeout)
		}

		#[ink(message)]
		pub fn poll(&self, request: RequestId) -> Result<Option<Status>> {
			api::poll((self.env().account_id(), request))
		}

		#[ink(message)]
		pub fn get(&self, request: RequestId) -> Result<Option<Vec<u8>>> {
			api::get((self.env().account_id(), request))
		}

		#[ink(message)]
		pub fn remove(&mut self, request: RequestId) -> Result<()> {
			api::remove([request].to_vec())?;
			Ok(())
		}

		#[ink(message)]
		pub fn test(&mut self, one: VersionedLocation, two: VersionedLocation) -> Result<()> {
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

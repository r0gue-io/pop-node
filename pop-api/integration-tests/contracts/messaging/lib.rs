#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use pop_api::{
	messaging::{self as api, Request, RequestId, Status},
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
			ink::env::debug_println!("PopApiFungiblesExample::new");
			Default::default()
		}

		#[ink(message)]
		pub fn request(&mut self, request: Request) -> Result<()> {
			api::request(request)?;
			Ok(())
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

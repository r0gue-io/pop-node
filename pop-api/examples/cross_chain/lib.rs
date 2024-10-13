#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use pop_api::{
	cross_chain::{self as api, ismp, Request, RequestId, Status},
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod cross_chain {
	use ink::codegen::Env;

	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct CrossChain {
		para: u32,
		id: RequestId,
	}

	impl CrossChain {
		#[ink(constructor, payable)]
		pub fn new(para: u32) -> Self {
			Self { para, id: 0 }
		}

		#[ink(message)]
		pub fn get(&mut self, key: Vec<u8>, height: u32) -> Result<()> {
			self.id = self.id.saturating_add(1);
			api::request(Request::Ismp {
				id: self.id,
				request: ismp::Request::Get {
					para: self.para,
					height,
					timeout: 0,
					context: Vec::default(),
					keys: Vec::from([key.clone()]),
				},
				fee: 0,
			})?;
			self.env().emit_event(Requested { id: self.id, key, height });
			Ok(())
		}

		#[ink(message)]
		pub fn complete(&mut self, request: RequestId) -> Result<()> {
			if let Ok(Some(status)) = api::poll((self.env().account_id(), request)) {
				if status == Status::Complete {
					let result = api::get((self.env().account_id(), request))?;
					api::remove(request)?;
					self.env().emit_event(Completed { id: request, result });
				}
			}
			Ok(())
		}
	}

	#[ink::event]
	pub struct Requested {
		#[ink(topic)]
		pub id: RequestId,
		pub key: Vec<u8>,
		pub height: u32,
	}

	#[ink::event]
	pub struct Completed {
		#[ink(topic)]
		pub id: RequestId,
		pub result: Option<Vec<u8>>,
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			CrossChain::new(1_000);
		}
	}
}

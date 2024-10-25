#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::{incentives as api, primitives::Era, StatusCode};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod incentives {

	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Contract {}

	impl Contract {
		#[ink(constructor, payable)]
		pub fn new(beneficiary: AccountId) -> Result<Self> {
			api::register(beneficiary)?;
			Ok(Default::default())
		}

		#[ink(message)]
		pub fn claim_rewards(&mut self, era: Era) -> Result<()> {
			api::claim(era)
		}
	}

	#[cfg(test)]
	mod tests {

		use super::*;

		#[ink::test]
		fn default_works() {
			Contracts::new();
		}
	}
}

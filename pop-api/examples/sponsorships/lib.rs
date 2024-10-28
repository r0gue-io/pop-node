#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::{
	sponsorships::{
		self as api,
	},
	StatusCode,
};
pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod sponsorships {
	use super::*;

	#[ink(storage)]
	pub struct Sponsorships {
		value: bool,
		owner: AccountId,
	}

	impl Sponsorships {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("PopApiSponsorshipsExample::new");
			Sponsorships {
				value: false,
				owner: Self::env().caller(),
			}
		}

		#[ink(message, payable)]
		pub fn sign_up(&mut self, user: AccountId) -> Result<()> {
			let caller = Self::env().caller();
			assert!(caller == self.owner, "Caller is not owner");
			api::sponsor_account(user, self.env().transferred_value())?;
			Ok(())
		}

		#[ink(message, payable)]
		pub fn withdraw_sponsorship(&mut self) -> Result<()> {
			let beneficiary = self.env().caller();
			api::remove_sponsorship_for(beneficiary)?;
			Ok(())
		}

		// Execution fees for this contract will be covered by the contract itself
		// for sponsored accounts.
		// This call is just here to test and observe the sponsored flows.
		#[ink(message, payable)]
		pub fn flip(&mut self) -> Result<()> {
			self.value = !self.value;
			Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			PopApiSponsorshipExample::new();
		}
	}
}

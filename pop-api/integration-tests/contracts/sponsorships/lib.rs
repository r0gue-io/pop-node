#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::{
	sponsorships::{
		self as api,
		events::{NewSponsorship, SponsorshipRemoved},
	},
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod sponsorships {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Sponsorships {
		value: bool,
	}

	impl Sponsorships {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("PopApiSponsorshipsExample::new");
			Default::default()
		}

		#[ink(message)]
		pub fn sing_up(&self) -> Result<()> {
			let beneficiary = self.env().caller();
			api::sponsor_account(beneficiary)?;
			self.env()
				.emit_event(NewSponsorship { sponsor: self.env().account_id(), beneficiary });
			Ok(())
		}

		#[ink(message)]
		pub fn withdraw_sponsorship(&self) -> Result<()> {
			let beneficiary = self.env().caller();
			api::remove_sponsorship_for(beneficiary)?;
			self.env().emit_event(SponsorshipRemoved {
				was_sponsor: self.env().account_id(),
				was_beneficiary: beneficiary,
			});
			Ok(())
		}

		#[ink(message)]
		pub fn flip_value(&mut self) -> Result<()> {
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

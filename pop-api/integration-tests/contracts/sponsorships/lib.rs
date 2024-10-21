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

		#[ink(message)]
		pub fn sing_up(&mut self, user: Option<AccountId>) -> Result<()> {
			let caller = self.env().caller();
			assert!(caller == self.owner, "Caller is not owner");
			let beneficiary = user.unwrap_or(caller.clone());
			api::sponsor_account(beneficiary)?;
			self.env()
				.emit_event(NewSponsorship { sponsor: self.env().account_id(), beneficiary });
			Ok(())
		}

		#[ink(message)]
		pub fn withdraw_sponsorship(&mut self) -> Result<()> {
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

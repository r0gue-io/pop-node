#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::{
	sponsorships::{
		self as api,
		events::{NewSponsorship, SponsorshipRemoved},
	},
	StatusCode,
};
use ink::env::{DefaultEnvironment, Environment};

pub type Result<T> = core::result::Result<T, StatusCode>;

type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;

fn to_account_id(address: &AccountId) -> AccountId {
	let mut account_id = AccountId::from([0xEE; 32]);
	<AccountId as AsMut<[u8; 32]>>::as_mut(&mut account_id)[..20]
		.copy_from_slice(&<AccountId as AsRef<[u8; 32]>>::as_ref(&address)[..20]);
	account_id
}

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
		pub fn sing_up(&mut self, user: AccountId) -> Result<()> {
			let caller = Self::env().caller();
			//assert!(caller == self.owner, "Caller is not owner");
			let beneficiary = user;
			api::sponsor_account(caller)?;
			self.env()
				.emit_event(NewSponsorship { sponsor: self.env().account_id(), beneficiary });
			Ok(())
		}

		#[ink(message, payable)]
		pub fn withdraw_sponsorship(&mut self) -> Result<()> {
			let beneficiary = to_account_id(&self.env().caller());
			api::remove_sponsorship_for(beneficiary)?;
			self.env().emit_event(SponsorshipRemoved {
				was_sponsor: self.env().account_id(),
				was_beneficiary: beneficiary,
			});
			Ok(())
		}

		#[ink(message, payable)]
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
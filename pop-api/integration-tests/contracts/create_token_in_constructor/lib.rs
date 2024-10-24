#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{env::Environment, prelude::*};
use pop_api::{
	fungibles::{self as api, events::Created},
	primitives::TokenId,
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;

fn to_account_id(address: &AccountId) -> AccountId {
	let mut account_id = AccountId::from([0xEE; 32]);
	<AccountId as AsMut<[u8; 32]>>::as_mut(&mut account_id)[..20]
		.copy_from_slice(&<AccountId as AsRef<[u8; 32]>>::as_ref(&address)[..20]);
	account_id
}

#[ink::contract]
mod create_token_in_constructor {
	use super::*;

	#[ink(storage)]
	pub struct Fungible {
		token: TokenId,
	}

	impl Fungible {
		#[ink(constructor, payable)]
		pub fn new(id: TokenId, min_balance: Balance) -> Result<Self> {
			Self::env().caller();
			let contract = Self { token: id };
			// AccountId of the contract which will be set to the owner of the fungible token.
			let owner = to_account_id(&contract.env().account_id());
			ink::env::debug_println!("{}", &format!("owner: {:?}", owner));
			api::create(id, owner, min_balance)?;
			contract.env().emit_event(Created { id, creator: owner, admin: owner });
			Ok(contract)
		}

		#[ink(constructor, payable)]
		pub fn new_default() -> Result<Self> {
			let contract = Self { token: 0 };
			// AccountId of the contract which will be set to the owner of the fungible token.
			let owner = contract.env().account_id();
			api::create(0, owner, 1)?;
			contract.env().emit_event(Created { id: 0, creator: owner, admin: owner });
			Ok(contract)
		}

		#[ink(message, payable)]
		pub fn token_exists(&self) -> Result<bool> {
			api::token_exists(self.token)
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn new_works() {
			let contract = Fungible::new(0, 1).unwrap();
			// assert_eq!(contract.token_exists().unwrap(), true);
		}
	}
}

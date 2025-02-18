#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// 1. PSP-22
/// 2. PSP-22 Metadata
/// 3. Management
/// 4. PSP-22 Mintable & Burnable
use ink::prelude::vec::Vec;
use pop_api::{
    fungibles::{
        self as api,
        events::{Approval, Created, DestroyStarted, MetadataCleared, MetadataSet, Transfer},
        TokenId,
    },
    StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod fungibles {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Fungibles;

	impl Fungibles {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("PopApiFungiblesExample::new");
			Default::default()
		}

		/// 1. PSP-22 Interface:
		/// - total_supply
		/// - balance_of
		/// - allowance
		/// - transfer
		/// - transfer_from
		/// - approve
		/// - increase_allowance
		/// - decrease_allowance

		#[ink(message)]
		pub fn total_supply(&self, token: TokenId) -> Result<Balance> {
			api::total_supply(token)
		}

		#[ink(message)]
		pub fn balance_of(&self, token: TokenId, owner: AccountId) -> Result<Balance> {
			api::balance_of(token, owner)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			token: TokenId,
			owner: AccountId,
			spender: AccountId,
		) -> Result<Balance> {
			api::allowance(token, owner, spender)
		}

        #[ink(message)]
        pub fn transfer(&mut self, token: TokenId, to: AccountId, value: Balance) -> Result<()> {
            api::transfer(token.clone(), to, value)?;
            self.env().emit_event(Transfer {
                from: Some(self.env().account_id()),
                to: Some(to),
                value,
            });
            Ok(())
        }

		#[ink(message)]
		pub fn transfer_from(
			&mut self,
			token: TokenId,
			from: AccountId,
			to: AccountId,
			value: Balance,
			// In the PSP-22 standard a `[u8]`, but the size needs to be known at compile time.
			_data: Vec<u8>,
		) -> Result<()> {
			api::transfer_from(token, from, to, value)?;
			self.env().emit_event(Transfer { from: Some(from), to: Some(to), value });
			Ok(())
		}

		#[ink(message)]
		pub fn approve(
			&mut self,
			token: TokenId,
			spender: AccountId,
			value: Balance,
		) -> Result<()> {
			api::approve(token, spender, value)?;
			self.env()
				.emit_event(Approval { owner: self.env().account_id(), spender, value });
			Ok(())
		}

		#[ink(message)]
		pub fn increase_allowance(
			&mut self,
			token: TokenId,
			spender: AccountId,
			value: Balance,
		) -> Result<()> {
			api::increase_allowance(token, spender, value)
		}

		#[ink(message)]
		pub fn decrease_allowance(
			&mut self,
			token: TokenId,
			spender: AccountId,
			value: Balance,
		) -> Result<()> {
			api::decrease_allowance(token, spender, value)
		}

		/// 2. PSP-22 Metadata Interface:
		/// - token_name
		/// - token_symbol
		/// - token_decimals

		#[ink(message)]
		pub fn token_name(&self, token: TokenId) -> Result<Option<Vec<u8>>> {
			api::token_name(token)
		}

		#[ink(message)]
		pub fn token_symbol(&self, token: TokenId) -> Result<Option<Vec<u8>>> {
			api::token_symbol(token)
		}

		#[ink(message)]
		pub fn token_decimals(&self, token: TokenId) -> Result<u8> {
			api::token_decimals(token)
		}

        /// 3. Asset Management:
        /// - create
        /// - start_destroy
        /// - set_metadata
        /// - clear_metadata
        /// - token_exists

        #[ink(message)]
        pub fn create(
            &mut self,
            id: TokenId,
            admin: AccountId,
            min_balance: Balance,
        ) -> Result<()> {
            api::create(id.clone(), admin, min_balance)?;
            self.env().emit_event(Created { id, creator: admin, admin });
            Ok(())
        }

        #[ink(message)]
        pub fn start_destroy(&mut self, token: TokenId) -> Result<()> {
            api::start_destroy(token.clone())?;
            self.env().emit_event(DestroyStarted { token });
            Ok(())
        }

        #[ink(message)]
        pub fn set_metadata(
            &mut self,
            token: TokenId,
            name: Vec<u8>,
            symbol: Vec<u8>,
            decimals: u8,
        ) -> Result<()> {
            api::set_metadata(token.clone(), name.clone(), symbol.clone(), decimals)?;
            self.env().emit_event(MetadataSet { token, name, symbol, decimals });
            Ok(())
        }

        #[ink(message)]
        pub fn clear_metadata(&mut self, token: TokenId) -> Result<()> {
            api::clear_metadata(token.clone())?;
            self.env().emit_event(MetadataCleared { token });
            Ok(())
        }

		#[ink(message)]
		pub fn token_exists(&self, token: TokenId) -> Result<bool> {
			api::token_exists(token)
		}

		/// 4. PSP-22 Mintable & Burnable Interface:
		/// - mint
		/// - burn

		#[ink(message)]
		pub fn mint(&mut self, token: TokenId, account: AccountId, amount: Balance) -> Result<()> {
			api::mint(token, account, amount)
		}

		#[ink(message)]
		pub fn burn(&mut self, token: TokenId, account: AccountId, amount: Balance) -> Result<()> {
			api::burn(token, account, amount)
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			PopApiFungiblesExample::new();
		}
	}
}

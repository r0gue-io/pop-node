#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::prelude::vec::Vec;
use pop_api::{
    v0::{foreign_fungibles::{
        self as api, TokenId,
    }, fungibles::events::Transfer},
    StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod foreign_fungibles {
    use super::*;

    #[ink(storage)]
    #[derive(Default)]
    pub struct ForeignFungibles;

    impl ForeignFungibles {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            ink::env::debug_println!("PopApiFungiblesExample::new");
            Default::default()
        }

        #[ink(message)]
        pub fn balance_of(&self, token: TokenId, owner: AccountId) -> Result<Balance> {
            api::balance_of(token, owner)
        }

        #[ink(message)]
        pub fn transfer(&mut self, token: TokenId, to: AccountId, value: Balance) -> Result<()> {
            api::transfer(token, to, value)?;
            self.env().emit_event(Transfer {
                from: Some(self.env().account_id()),
                to: Some(to),
                value,
            });
            Ok(())
        }
    }
}

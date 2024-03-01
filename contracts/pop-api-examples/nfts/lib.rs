#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::Environment,
    prelude::vec::Vec,
};

use ink::primitives::AccountId;
use sp_runtime::MultiAddress;
use pop_api::impls::pop_network::PopEnv;
use pop_api::impls::pop_network::PopApiError;

#[ink::contract(env = crate::PopEnv)]
mod pop_api_extension_demo {

    use super::PopApiError;
    use scale::Encode;
    use ink::env::Error as EnvError;

    #[ink(storage)]
    #[derive(Default)]
    pub struct PopApiExtensionDemo;

    impl PopApiExtensionDemo {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            ink::env::debug_println!("PopApiExtensionDemo::new");
            Default::default()
        }

        #[ink(message)]
        pub fn mint_through_runtime(
            &mut self,
            collection_id: u32,
            item_id: u32,
            receiver: AccountId,
        ) {
            ink::env::debug_println!("PopApiExtensionDemo::mint_through_runtime: collection_id: {:?} \nitem_id {:?} \nreceiver: {:?}, ", collection_id, item_id, receiver);

            let call = pop_api::impls::pop_network::Nfts::mint(collection_id, item_id, receiver);
            self.env().extension().dispatch(call);

            ink::env::debug_println!("PopApiExtensionDemo::mint_through_runtime end");
            
        }
    }
}
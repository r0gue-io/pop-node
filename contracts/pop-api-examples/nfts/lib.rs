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

    use ink::env::Error as EnvError;

    /// A trivial contract with a single message, that uses `call-runtime` API for
    /// performing native token transfer.
    #[ink(storage)]
    #[derive(Default)]
    pub struct PopApiExtensionDemo;

    // impl From<EnvError> for PopApiError {
    //     fn from(e: EnvError) -> Self {
    //         match e {
    //             EnvError::CallRuntimeFailed => PopApiError::PlaceholderError,
    //             _ => panic!("Unexpected error from `pallet-contracts`."),
    //         }
    //     }
    // }

    impl PopApiExtensionDemo {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            ink::env::debug_println!("PopApiExtensionDemo::new");
            Default::default()
        }

        #[ink(message)]
        pub fn transfer_through_runtime(
            &mut self,
            receiver: AccountId,
            value: Balance,
        ) {
            ink::env::debug_println!("PopApiExtensionDemo::transfer_through_runtime: \nreceiver: {:?}, \nvalue: {:?}", receiver, value);

            // let call = RuntimeCall::Balances(BalancesCall::TransferKeepAlive {
            //         dest: receiver.into(),
            //         value: value,
            //     });
            // self.env().extension().dispatch(call);

            ink::env::debug_println!("PopApiExtensionDemo::transfer_through_runtime end");
            
        }
    }
}
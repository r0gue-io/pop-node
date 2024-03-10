#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(env = pop_api::Environment)]
mod pop_api_extension_demo {
    use pop_api::primitives::storage_keys::{
        ParachainSystemKeys::LastRelayChainBlockNumber, RuntimeStateKeys::ParachainSystem,
    };

    #[ink(event)]
    pub struct RelayBlockNumberRead {
        value: BlockNumber,
    }

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
        pub fn read_relay_block_number(&self) {
            let result =
                pop_api::state::read::<BlockNumber>(ParachainSystem(LastRelayChainBlockNumber));
            ink::env::debug_println!("{:?}", result);
            self.env().emit_event(RelayBlockNumberRead {
                value: result.expect("Failed to read relay block number."),
            });
        }
    }
}

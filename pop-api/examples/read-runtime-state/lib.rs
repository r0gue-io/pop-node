#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract(env = pop_api::Environment)]
mod read_relay_blocknumber {
    use pop_api::primitives::storage_keys::{
        ParachainSystemKeys::LastRelayChainBlockNumber, RuntimeStateKeys::ParachainSystem,
    };

    #[ink(event)]
    pub struct RelayBlockNumberRead {
        value: BlockNumber,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct ReadRelayBlockNumber;

    impl ReadRelayBlockNumber {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            ink::env::debug_println!("ReadRelayBlockNumber::new");
            Default::default()
        }

        #[ink(message)]
        pub fn read_relay_block_number(&self) {
            let result =
                pop_api::state::read::<BlockNumber>(ParachainSystem(LastRelayChainBlockNumber));
            ink::env::debug_println!("Last relay block number read by contract: {:?}", result);
            self.env().emit_event(RelayBlockNumberRead {
                value: result.expect("Failed to read relay block number."),
            });
        }
    }
}

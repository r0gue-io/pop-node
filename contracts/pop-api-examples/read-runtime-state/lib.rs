#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::Environment,
    prelude::vec::Vec,
};

use ink::primitives::AccountId;
use sp_runtime::MultiAddress;
use pop_api::primitives::storage_keys::ParachainSystemKeys;


#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PopApiError {
    TotalSupplyFailed,
}

pub type Result<T> = core::result::Result<T, PopApiError>;

use scale;
impl From<scale::Error> for PopApiError {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

/// This is an example of how an ink! contract may call the Substrate
/// runtime function `RandomnessCollectiveFlip::random_seed`. See the
/// file `runtime/chain-extension-example.rs` for that implementation.
///
/// Here we define the operations to interact with the Substrate runtime.
#[ink::chain_extension]
pub trait PopApi {
    type ErrorCode = PopApiError;

    /// Note: this gives the operation a corresponding `func_id` (1101 in this case),
    /// and the chain-side chain extension will get the `func_id` to do further
    /// operations.

    #[ink(extension = 0xfeca)]
    fn read_state(key: ParachainSystemKeys) -> Result<<ink::env::DefaultEnvironment as Environment>::BlockNumber>;

}

impl ink::env::chain_extension::FromStatusCode for PopApiError {
    fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::TotalSupplyFailed),
            _ => panic!("encountered unknown status code"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum CustomEnvironment {}

impl Environment for CustomEnvironment {
    const MAX_EVENT_TOPICS: usize =
        <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = PopApi;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod pop_api_extension_demo {
    use crate::{
        ParachainSystemKeys,
    };

    use super::PopApiError;

    use ink::env::Error as EnvError;

    #[ink(event)]
    pub struct RelayBlockNumberRead {
        value: BlockNumber
    }


    #[ink(storage)]
    #[derive(Default)]
    pub struct PopApiExtensionDemo;

    impl From<EnvError> for PopApiError {
        fn from(e: EnvError) -> Self {
            match e {
                EnvError::CallRuntimeFailed => PopApiError::TotalSupplyFailed,
                _ => panic!("Unexpected error from `pallet-contracts`."),
            }
        }
    }

    impl PopApiExtensionDemo {
        #[ink(constructor, payable)]
        pub fn new() -> Self {
            ink::env::debug_println!("PopApiExtensionDemo::new");
            Default::default()
        }

        #[ink(message)]
        pub fn read_relay_block_number(
            &self
        ) {
            let result: BlockNumber = self.env().extension().read_state(ParachainSystemKeys::LastRelayChainBlockNumber);
            ink::env::debug_println!("[Contract] Last relay chain block# : {:?}", result);
            self.env().emit_event(
                RelayBlockNumberRead {value: result.expect("Failed to read relay block number.")}
            );
        }
    }
}
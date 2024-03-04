#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::Environment,
    prelude::vec::Vec,
};

use ink::primitives::AccountId;
use sp_runtime::MultiAddress;
use scale::{Encode, Decode};

#[derive(Encode)]
enum RuntimeCall {
    Balances(BalancesCall),
}

#[derive(scale::Encode)]
enum BalancesCall {
    #[codec(index = 3)]
    TransferKeepAlive {
        dest: MultiAddress<AccountId, ()>,
        #[codec(compact)]
        value: u128,
    },
    #[codec(index = 8)]
    ForceSetBalance {
        who: MultiAddress<AccountId, ()>,
        #[codec(compact)]
        new_free: u128,
    },
}

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

#[ink::chain_extension]
pub trait PopApi {
    type ErrorCode = PopApiError;

    #[ink(extension = 0x0)]
    fn dispatch(call: RuntimeCall) -> Result<Vec<u8>>;
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
        BalancesCall,
        RuntimeCall,
    };

    use super::PopApiError;

    use ink::env::Error as EnvError;

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
        pub fn transfer_through_runtime(
            &mut self,
            receiver: AccountId,
            value: Balance,
        ) {
            ink::env::debug_println!("PopApiExtensionDemo::transfer_through_runtime: \nreceiver: {:?}, \nvalue: {:?}", receiver, value);

            let call = RuntimeCall::Balances(BalancesCall::TransferKeepAlive {
                    dest: receiver.into(),
                    value: value,
                });
            self.env().extension().dispatch(call);

            ink::env::debug_println!("PopApiExtensionDemo::transfer_through_runtime end");
            
        }
    }
}
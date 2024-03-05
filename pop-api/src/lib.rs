#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod v0;

use crate::PopApiError::{Nfts, UnknownStatusCode};
use ink::{env::Environment, ChainExtensionInstance};
pub use pop_api_primitives as primitives;
use scale;
use sp_runtime::MultiSignature;
pub use v0::nfts;
use v0::RuntimeCall;

// Id used for identifying non-fungible collections.
pub type CollectionId = u32;

// Id used for identifying non-fungible items.
pub type ItemId = u32;

type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
type Signature = MultiSignature;
type StringLimit = u32;
type KeyLimit = u32;
type MaxTips = u32;

pub type Result<T> = core::result::Result<T, PopApiError>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PopApiError {
    UnknownStatusCode(u32),
    Nfts(nfts::Error),
}

impl ink::env::chain_extension::FromStatusCode for PopApiError {
    fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
        match status_code {
            0 => Ok(()),
            50_000..=50_999 => Err(Nfts((status_code - 50_000).try_into()?)),
            _ => Err(UnknownStatusCode(status_code)),
        }
    }
}

impl From<scale::Error> for PopApiError {
    fn from(_: scale::Error) -> Self {
        panic!("encountered unexpected invalid SCALE encoding")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PopEnv {}

impl Environment for PopEnv {
    const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = PopApi;
}

#[ink::chain_extension]
pub trait PopApi {
    type ErrorCode = PopApiError;

    #[ink(extension = 0)]
    #[allow(private_interfaces)]
    fn dispatch(call: RuntimeCall) -> crate::Result<()>;

    #[ink(extension = 1)]
    #[allow(private_interfaces)]
    fn read_state(call: RuntimeCall) -> crate::Result<Vec<u8>>;
}

fn dispatch(call: RuntimeCall) -> Result<()> {
    <<PopEnv as Environment>::ChainExtension as ChainExtensionInstance>::instantiate()
        .dispatch(call)
}

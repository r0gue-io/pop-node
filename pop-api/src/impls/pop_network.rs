use crate::interfaces::nfts::NftCalls;
use pallet_nfts::CollectionConfig;
use sp_runtime::{MultiAddress, MultiSignature};
use ink::prelude::vec::Vec;
use scale;

use ink::env::Environment;

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


pub struct Pop {
   pub api: Api 
}

pub struct Api {
    nft: Nfts 
}

pub type NftCallsOf = NftCalls<AccountId, Balance, BlockNumber, CollectionId, ItemId, KeyLimit, StringLimit, MaxTips, Signature>;

pub struct Nfts;

impl Nfts {
    pub fn create(&self, admin: MultiAddress<AccountId, ()>, config: CollectionConfig<Balance, BlockNumber, CollectionId>) -> RuntimeCall {
        RuntimeCall::Nfts(NftCallsOf::Create { admin, config })
    }

   pub fn mint(&self, collection: CollectionId, item: ItemId, mint_to: MultiAddress<AccountId, ()>) -> RuntimeCall {
        RuntimeCall::Nfts(NftCallsOf::Mint { collection, item, mint_to })
    }
}

#[derive(scale::Encode)]
pub enum RuntimeCall {
    #[codec(index = 50)]
    Nfts(NftCalls<AccountId, Balance, BlockNumber, CollectionId, ItemId, KeyLimit, StringLimit, MaxTips, Signature>)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PopApiError {
    PlaceholderError,
}

pub type Result<T> = core::result::Result<T, PopApiError>;

#[ink::chain_extension]
pub trait PopApi {
    type ErrorCode = PopApiError;

    #[ink(extension = 0xfecb)]
    fn dispatch(call: RuntimeCall) -> Result<Vec<u8>>;
}

impl ink::env::chain_extension::FromStatusCode for PopApiError {
    fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
        match status_code {
            0 => Ok(()),
            1 => Err(Self::PlaceholderError),
            _ => panic!("encountered unknown status code"),
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
    const MAX_EVENT_TOPICS: usize =
        <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

    type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
    type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
    type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
    type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;
    type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

    type ChainExtension = PopApi;
}
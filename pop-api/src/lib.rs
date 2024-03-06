#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod primitives;
pub mod v0;

use crate::PopApiError::{Balances, Nfts, UnknownStatusCode};
use ink::{prelude::vec::Vec, ChainExtensionInstance};
use primitives::storage_keys::*;
use scale;
pub use sp_runtime::{BoundedVec, MultiAddress, MultiSignature};
use v0::RuntimeCall;
pub use v0::{balances, nfts, state};

// Id used for identifying non-fungible collections.
pub type CollectionId = u32;

// Id used for identifying non-fungible items.
pub type ItemId = u32;

type AccountId = <ink::env::DefaultEnvironment as ink::env::Environment>::AccountId;
type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;
type BlockNumber = <ink::env::DefaultEnvironment as ink::env::Environment>::BlockNumber;
type StringLimit = u32;
type KeyLimit = u32;
type MaxTips = u32;

pub type Result<T> = core::result::Result<T, PopApiError>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PopApiError {
	UnknownStatusCode(u32),
	DecodingFailed,
	Balances(balances::Error),
	Nfts(nfts::Error),
}

impl ink::env::chain_extension::FromStatusCode for PopApiError {
	fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
		match status_code {
			0 => Ok(()),
			10_000..=10_999 => Err(Balances((status_code - 10_000).try_into()?)),
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
pub enum Environment {}

impl ink::env::Environment for Environment {
	const MAX_EVENT_TOPICS: usize =
		<ink::env::DefaultEnvironment as ink::env::Environment>::MAX_EVENT_TOPICS;

	type AccountId = <ink::env::DefaultEnvironment as ink::env::Environment>::AccountId;
	type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;
	type Hash = <ink::env::DefaultEnvironment as ink::env::Environment>::Hash;
	type BlockNumber = <ink::env::DefaultEnvironment as ink::env::Environment>::BlockNumber;
	type Timestamp = <ink::env::DefaultEnvironment as ink::env::Environment>::Timestamp;

	type ChainExtension = PopApi;
}

#[ink::chain_extension]
pub trait PopApi {
	type ErrorCode = PopApiError;

	#[ink(extension = 0)]
	#[allow(private_interfaces)]
	fn dispatch(call: RuntimeCall) -> Result<()>;

	#[ink(extension = 1)]
	#[allow(private_interfaces)]
	fn read_state(key: RuntimeStateKeys) -> Result<Vec<u8>>;
}

fn dispatch(call: RuntimeCall) -> Result<()> {
	<<Environment as ink::env::Environment>::ChainExtension as ChainExtensionInstance>::instantiate(
	)
	.dispatch(call)
}

fn read_state(key: RuntimeStateKeys) -> Result<Vec<u8>> {
	<<Environment as ink::env::Environment>::ChainExtension as ChainExtensionInstance>::instantiate(
	)
	.read_state(key)
}

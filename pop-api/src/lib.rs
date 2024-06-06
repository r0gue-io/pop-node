#![cfg_attr(not(feature = "std"), no_std, no_main)]

use core::convert::TryInto;
use ink::{prelude::vec::Vec, ChainExtensionInstance};
use primitives::{cross_chain::*, storage_keys::*, AccountId as AccountId32};
pub use sp_runtime::{BoundedVec, MultiAddress, MultiSignature};
use v0::RuntimeCall;
pub use v0::{
	assets, balances, contracts, cross_chain, dispatch_error, nfts, relay_chain_block_number, state,
};

pub mod primitives;
pub mod v0;

// type AccountId = <Environment as ink::env::Environment>::AccountId;
type AccountId = AccountId32;
type Balance = <Environment as ink::env::Environment>::Balance;
type BlockNumber = <Environment as ink::env::Environment>::BlockNumber;
type StringLimit = u32;
type MaxTips = u32;

pub type Result<T> = core::result::Result<T, PopApiError>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PopApiError {
	Assets(assets::fungibles::AssetsError),
	Balances(balances::BalancesError),
	Contracts(contracts::Error),
	DecodingFailed,
	Nfts(nfts::Error),
	SystemCallFiltered,
	TokenError(dispatch_error::TokenError),
	UnknownModuleStatusCode(u32),
	UnknownDispatchStatusCode(u32),
	Xcm(cross_chain::Error),
}

impl ink::env::chain_extension::FromStatusCode for PopApiError {
	fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
		use crate::PopApiError::{
			Assets, Balances, Contracts, Nfts, TokenError, UnknownDispatchStatusCode,
			UnknownModuleStatusCode,
		};

		match status_code {
			0 => Ok(()),
			3_000_000..=3_999_999 => {
				let status_code = status_code - 3_000_000;
				match status_code {
					// CallFiltered originates from `frame_system` with pallet-index 0. The CallFiltered error is at index 5
					5 => Err(PopApiError::SystemCallFiltered),
					10_000..=10_999 => Err(Balances((status_code - 10_000).try_into()?)),
					40_000..=40_999 => Err(Contracts((status_code - 40_000).try_into()?)),
					50_000..=50_999 => Err(Nfts((status_code - 50_000).try_into()?)),
					52_000..=52_999 => Err(Assets((status_code - 52_000).try_into()?)),
					_ => Err(UnknownModuleStatusCode(status_code)),
				}
			},
			7_000_000..=7_999_999 => Err(TokenError((status_code - 7_000_000).try_into()?)),
			_ => Err(UnknownDispatchStatusCode(status_code)),
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

#[ink::chain_extension(extension = 909)]
pub trait PopApi {
	type ErrorCode = PopApiError;

	#[ink(function = 0)]
	#[allow(private_interfaces)]
	fn dispatch(call: RuntimeCall) -> Result<()>;

	#[ink(function = 1)]
	#[allow(private_interfaces)]
	fn read_state(key: RuntimeStateKeys) -> Result<Vec<u8>>;

	#[ink(function = 2)]
	#[allow(private_interfaces)]
	fn send_xcm(xcm: CrossChainMessage) -> Result<()>;
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

fn send_xcm(xcm: CrossChainMessage) -> Result<()> {
	<<Environment as ink::env::Environment>::ChainExtension as ChainExtensionInstance>::instantiate(
	)
	.send_xcm(xcm)
}

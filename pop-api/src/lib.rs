#![cfg_attr(not(feature = "std"), no_std, no_main)]

use crate::error::PopApiError;
use ink::{prelude::vec::Vec, ChainExtensionInstance};
use primitives::{cross_chain::*, storage_keys::*, AccountId as AccountId32};
use scale::Encode;
pub use sp_runtime::{BoundedVec, MultiAddress, MultiSignature};
use v0::RuntimeCall;
pub use v0::{assets, balances, cross_chain, nfts, relay_chain_block_number, state};

pub mod error;
pub mod primitives;
pub mod v0;

type AccountId = AccountId32;
// TODO: do the same as the AccountId above and check expanded macro code.
type Balance = <Environment as ink::env::Environment>::Balance;
type BlockNumber = <Environment as ink::env::Environment>::BlockNumber;
type StringLimit = u32;
type MaxTips = u32;

pub type Result<T> = core::result::Result<T, PopApiError>;

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

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use core::convert::TryInto;
use ink::{prelude::vec::Vec, ChainExtensionInstance};
use primitives::{cross_chain::*, storage_keys::*, AccountId as AccountId32};
use scale::{Decode, Encode};
pub use sp_runtime::{BoundedVec, MultiAddress, MultiSignature};
use v0::assets::use_cases::fungibles::{convert_to_fungibles_error, FungiblesError};
use v0::RuntimeCall;
pub use v0::{
	assets, balances, contracts, cross_chain, dispatch_error,
	dispatch_error::{ArithmeticError, TokenError, TransactionalError},
	nfts, relay_chain_block_number, state,
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

// TODO: Versioning?
#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[repr(u8)]
pub enum PopApiError {
	/// Some error occurred which is not handled by the pop api version.
	Other {
		// Index within the DispatchError
		dispatch_error_index: u8,
		// Index within the DispatchError variant.
		error_index: u8,
		// Index for further nesting, e.g. pallet error.
		error: u8,
	},
	/// Failed to lookup some data.
	CannotLookup,
	/// A bad origin.
	BadOrigin,
	/// A custom error in a module.
	Module {
		index: u8,
		error: u8,
	},
	/// At least one consumer is remaining so the account cannot be destroyed.
	ConsumerRemaining,
	/// There are no providers so the account cannot be created.
	NoProviders,
	/// There are too many consumers so the account cannot be created.
	TooManyConsumers,
	/// An error to do with tokens.
	Token(TokenError),
	/// An arithmetic error.
	Arithmetic(ArithmeticError),
	/// The number of transactional layers has been reached, or we are not in a transactional
	/// layer.
	Transactional(TransactionalError),
	/// Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
	Exhausted,
	/// The state is corrupt; this is generally not going to fix itself.
	Corruption,
	/// Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
	Unavailable,
	/// Root origin is not allowed.
	RootNotAllowed,
	UseCaseError(FungiblesError) = 254,
	DecodingFailed = 255,
}

impl ink::env::chain_extension::FromStatusCode for PopApiError {
	fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
		use PopApiError::*;
		match status_code {
			0 => Ok(()),
			_ => {
				// TODO: refactor
				let encoded = status_code.encode();
				let mut error =
					PopApiError::decode(&mut &encoded[..]).map_err(|_| DecodingFailed)?;
				ink::env::debug_println!("1st PopApiError: {:?}", error);
				error = if let Module { index, error } = error {
					convert_to_fungibles_error(index, error)
				} else {
					error
				};
				ink::env::debug_println!("2nd PopApiError: {:?}", error);
				Err(error)
			},
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

	// TODO: add versioning.
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

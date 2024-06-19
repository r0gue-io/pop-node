#![cfg_attr(not(feature = "std"), no_std, no_main)]

use crate::PopApiError::*;
use core::convert::TryInto;
use ink::{prelude::vec::Vec, ChainExtensionInstance};
use primitives::{cross_chain::*, storage_keys::*, AccountId as AccountId32};
use scale::{Decode, Encode};
pub use sp_runtime::{BoundedVec, MultiAddress, MultiSignature};
pub use v0::{
	assets, balances, contracts, cross_chain, dispatch_error,
	dispatch_error::{ArithmeticError, TokenError, TransactionalError},
	nfts, relay_chain_block_number, state,
};
use v0::{
	assets::use_cases::fungibles::{convert_to_fungibles_error, FungiblesError},
	RuntimeCall,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[repr(u8)]
pub enum PopApiError {
	/// Some unknown error occurred. Go to the Pop API docs section `Pop API error`.
	Other {
		// Index within the `DispatchError`
		dispatch_error_index: u8,
		// Index within the `DispatchError` variant.
		error_index: u8,
		// Index for further nesting, e.g. pallet error.
		error: u8,
	} = 0,
	/// Failed to lookup some data.
	CannotLookup = 1,
	/// A bad origin.
	BadOrigin = 2,
	/// A custom error in a module.
	Module {
		index: u8,
		error: u8,
	} = 3,
	/// At least one consumer is remaining so the account cannot be destroyed.
	ConsumerRemaining = 4,
	/// There are no providers so the account cannot be created.
	NoProviders = 5,
	/// There are too many consumers so the account cannot be created.
	TooManyConsumers = 6,
	/// An error to do with tokens.
	Token(TokenError) = 7,
	/// An arithmetic error.
	Arithmetic(ArithmeticError) = 8,
	/// The number of transactional layers has been reached, or we are not in a transactional
	/// layer.
	Transactional(TransactionalError) = 9,
	/// Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
	Exhausted = 10,
	/// The state is corrupt; this is generally not going to fix itself.
	Corruption = 11,
	/// Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
	Unavailable = 12,
	/// Root origin is not allowed.
	RootNotAllowed = 13,
	// TODO: make generic and add docs.
	UseCaseError(FungiblesError) = 254,
	DecodingFailed = 255,
}

impl ink::env::chain_extension::FromStatusCode for PopApiError {
	fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
		match status_code {
			0 => Ok(()),
			_ => Err(convert_to_pop_api_error(status_code)),
		}
	}
}

// `pub` because it is used in the test in the runtime.
pub fn convert_to_pop_api_error(status_code: u32) -> PopApiError {
	let mut encoded: [u8; 4] =
		status_code.encode().try_into().expect("qid u32 always encodes to 4 bytes");
	encoded = check_for_unknown_nested_pop_api_errors(encoded);
	let error = match PopApiError::decode(&mut &encoded[..]) {
		Err(_) => {
			// Failed decoding can be caused by a `PopApiError` variant that is not known
			// to this version. As a result, we convert it into the `Other` enum variant.
			encoded[3] = encoded[2];
			encoded[2] = encoded[1];
			encoded[1] = encoded[0];
			encoded[0] = 0;
			PopApiError::decode(&mut &encoded[..]).unwrap().into()
		},
		Ok(error) => {
			if let Module { index, error } = error {
				// TODO: make generic.
				convert_to_fungibles_error(index, error)
			} else {
				error
			}
		},
	};
	ink::env::debug_println!("PopApiError: {:?}", error);
	error
}

// If an non-nested variant of the `DispatchError` is changed to a nested variant. This function
// handles the conversion to the `Other` PopApiError variant.
fn check_for_unknown_nested_pop_api_errors(encoded_error: [u8; 4]) -> [u8; 4] {
	if non_nested_pop_api_errors().contains(&encoded_error[0])
		&& encoded_error[1..].iter().any(|x| *x != 0u8)
	{
		[0u8, encoded_error[0], encoded_error[1], encoded_error[2]]
	} else {
		encoded_error
	}
}

fn non_nested_pop_api_errors() -> [u8; 9] {
	const CANNOT_LOOKUP: u8 = 1;
	const BAD_ORIGIN: u8 = 2;
	const CONSUMER_REMAINING: u8 = 4;
	const NO_PROVIDERS: u8 = 5;
	const TOO_MANY_CONSUMERS: u8 = 6;
	const EXHAUSTED: u8 = 10;
	const CORRUPTION: u8 = 11;
	const UNAVAILABLE: u8 = 12;
	const ROOT_NOT_ALLOWED: u8 = 13;
	[
		CANNOT_LOOKUP,
		BAD_ORIGIN,
		CONSUMER_REMAINING,
		NO_PROVIDERS,
		TOO_MANY_CONSUMERS,
		EXHAUSTED,
		CORRUPTION,
		UNAVAILABLE,
		ROOT_NOT_ALLOWED,
	]
}

impl From<scale::Error> for PopApiError {
	fn from(_: scale::Error) -> Self {
		DecodingFailed
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

#[test]
fn u32_always_encodes_to_4_bytes() {
	assert_eq!(0u32.encode().len(), 4);
	assert_eq!(u32::MAX.encode().len(), 4);
}

// If decoding failed the encoded value is converted to the `PopApiError::Other`. This handles
// unknown errors coming from the runtime. This could happen if a contract is not upgraded to the
// latest Pop API version. This test checks for the correct conversion.
#[test]
fn test_non_existing_pop_api_errors() {
	let encoded_error = [7u8, 100u8, 0u8, 0u8];
	let status_code = u32::decode(&mut &encoded_error[..]).unwrap();
	let pop_api_error =
		<PopApiError as ink::env::chain_extension::FromStatusCode>::from_status_code(status_code);
	assert_eq!(
		Err(PopApiError::Other { dispatch_error_index: 7, error_index: 100, error: 0 }),
		pop_api_error
	);
}

// If an encoded value indicates for a nested PopApiError which in this Pop API version does not
// exist, the encoded value should be converted into `PopApiError::Other`. This test checks for the
// correct conversion.
#[test]
fn check_for_unknown_nested_pop_api_errors_works() {
	for &error_code in &non_nested_pop_api_errors() {
		let encoded_error = [error_code, 1, 2, 3];
		let result = check_for_unknown_nested_pop_api_errors(encoded_error);
		let decoded = PopApiError::decode(&mut &result[..]).unwrap();

		assert_eq!(
			decoded,
			PopApiError::Other { dispatch_error_index: error_code, error_index: 1, error: 2 },
			"Failed for error code: {}",
			error_code
		);
	}
}

// The `Module` error only has two nested values which requires max. 3 bytes. This test shows that
// a non-zero value for the 4th byte does not mess up the decoding of the PopApiError and results in
// a correct `Module` error.
#[test]
fn test_nested_pallet_erross() {
	let encoded_error = [3u8, 4u8, 5u8, 6u8];
	let status_code = u32::decode(&mut &encoded_error[..]).unwrap();
	let pop_api_error =
		<PopApiError as ink::env::chain_extension::FromStatusCode>::from_status_code(status_code);
	assert_eq!(Err(PopApiError::Module { index: 4, error: 5 }), pop_api_error);
}

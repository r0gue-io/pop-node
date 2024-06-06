use crate::assets::use_cases::fungibles::{convert_to_fungibles_error, FungiblesError};
use ink::env::chain_extension::FromStatusCode;
use scale::{Decode, Encode};
use PopApiError::*;

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

impl FromStatusCode for PopApiError {
	fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
		match status_code {
			0 => Ok(()),
			_ => Err(convert_to_pop_api_error(status_code)),
		}
	}
}

// `pub` because it is used in `runtime/devnet/src/extensions/tests/mod.rs`'s test:
// `dispatch_error_to_status_code_to_pop_api_error_works`
//
// This function converts a given `status_code` (u32) into a `PopApiError`. First it encodes the
// status code into a 4-byte array and checks for unknown nested errors. If decoding into
// `PopApiError` fails (e.g. a breaking change in the `DispatchError`), it handles the error by
// converting it to the `Other` variant by shifting each byte one position forward (the last byte is
// not used for anything)and setting the first byte to 0. If decoding succeeds, it checks if the
// error is of the `Module` variant and performs any necessary conversion based on the use case.
pub fn convert_to_pop_api_error(status_code: u32) -> PopApiError {
	let mut encoded: [u8; 4] =
		status_code.encode().try_into().expect("qid u32 always encodes to 4 bytes");
	encoded = check_for_unknown_nesting(encoded);
	let error = match PopApiError::decode(&mut &encoded[..]) {
		Err(_) => {
			encoded[3] = encoded[2];
			encoded[2] = encoded[1];
			encoded[1] = encoded[0];
			encoded[0] = 0;
			PopApiError::decode(&mut &encoded[..]).unwrap().into()
		},
		Ok(error) => {
			if let crate::PopApiError::Module { index, error } = error {
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

// If a unknown nested variant of the `DispatchError` is detected meaning any of the subsequent
// bytes are non-zero (e.g. breaking change in the DispatchError), the error needs to be converted
// into `PopApiError::Other`'s encoded value. This conversion is done by shifting the bytes one
// position forward (the last byte is discarded as it is not being used) and replacing the first
// byte with the `Other` encoded value (0u8). This ensures that the error is correctly categorized
// as an `Other` variant.
fn check_for_unknown_nesting(encoded_error: [u8; 4]) -> [u8; 4] {
	if non_nested_pop_api_errors().contains(&encoded_error[0])
		&& encoded_error[1..].iter().any(|x| *x != 0u8)
	{
		[0u8, encoded_error[0], encoded_error[1], encoded_error[2]]
	} else if singular_nested_pop_api_errors().contains(&encoded_error[0])
		&& encoded_error[2..].iter().any(|x| *x != 0u8)
	{
		[0u8, encoded_error[0], encoded_error[1], encoded_error[2]]
	} else {
		encoded_error
	}
}

impl From<scale::Error> for PopApiError {
	fn from(_: scale::Error) -> Self {
		DecodingFailed
	}
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum TokenError {
	/// Funds are unavailable.
	FundsUnavailable,
	/// Some part of the balance gives the only provider reference to the account and thus cannot
	/// be (re)moved.
	OnlyProvider,
	/// Account cannot exist with the funds that would be given.
	BelowMinimum,
	/// Account cannot be created.
	CannotCreate,
	/// The asset in question is unknown.
	UnknownAsset,
	/// Funds exist but are frozen.
	Frozen,
	/// Operation is not supported by the asset.
	Unsupported,
	/// Account cannot be created for a held balance.
	CannotCreateHold,
	/// Withdrawal would cause unwanted loss of account.
	NotExpendable,
	/// Account cannot receive the assets.
	Blocked,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ArithmeticError {
	/// Underflow.
	Underflow,
	/// Overflow.
	Overflow,
	/// Division by zero.
	DivisionByZero,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum TransactionalError {
	/// Too many transactional layers have been spawned.
	LimitReached,
	/// A transactional layer was expected, but does not exist.
	NoLayer,
}

fn singular_nested_pop_api_errors() -> [u8; 3] {
	const TOKEN_ERROR: u8 = 7;
	const ARITHMETIC_ERROR: u8 = 8;
	const TRANSACTION_ERROR: u8 = 9;
	[TOKEN_ERROR, ARITHMETIC_ERROR, TRANSACTION_ERROR]
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

#[test]
fn u32_always_encodes_to_4_bytes() {
	assert_eq!(0u32.encode().len(), 4);
	assert_eq!(u32::MAX.encode().len(), 4);
}

// If decoding failed the encoded value is converted to the `PopApiError::Other`. This handles
// unknown errors coming from the runtime. This could happen if a contract is not upgraded to the
// latest Pop API version.
#[test]
fn test_non_existing_pop_api_errors() {
	let encoded_error = [7u8, 100u8, 0u8, 0u8];
	let status_code = u32::decode(&mut &encoded_error[..]).unwrap();
	let pop_api_error = <PopApiError as FromStatusCode>::from_status_code(status_code);
	assert_eq!(Err(Other { dispatch_error_index: 7, error_index: 100, error: 0 }), pop_api_error);
}

// If the encoded value indicates a nested PopApiError which is not handled by the Pop API version,
// the encoded value is converted into `PopApiError::Other`.
#[test]
fn check_for_unknown_nested_pop_api_errors_works() {
	for &error_code in &non_nested_pop_api_errors() {
		let encoded_error = [error_code, 1, 2, 3];
		let result = check_for_unknown_nesting(encoded_error);
		let decoded = PopApiError::decode(&mut &result[..]).unwrap();

		assert_eq!(
			decoded,
			Other { dispatch_error_index: error_code, error_index: 1, error: 2 },
			"Failed for error code: {}",
			error_code
		);
	}
	for &error_code in &singular_nested_pop_api_errors() {
		let encoded_error = [error_code, 1, 2, 3];
		let result = check_for_unknown_nesting(encoded_error);
		let decoded = PopApiError::decode(&mut &result[..]).unwrap();

		assert_eq!(
			decoded,
			Other { dispatch_error_index: error_code, error_index: 1, error: 2 },
			"Failed for error code: {}",
			error_code
		);
	}
}

// This test ensures that a non-zero value for unused bytes does not interfere with the correct
// decoding of the error. It verifies that even with an additional byte, the errors are correctly
// decoded and represented in its correct variant.
#[test]
fn extra_byte_does_not_mess_up_decoding() {
	// Module error
	let encoded_error = [3u8, 4u8, 5u8, 6u8];
	let status_code = u32::decode(&mut &encoded_error[..]).unwrap();
	let pop_api_error = <PopApiError as FromStatusCode>::from_status_code(status_code);
	assert_eq!(Err(Module { index: 4, error: 5 }), pop_api_error);
}

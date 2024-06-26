use ink::env::chain_extension::FromStatusCode;
use scale::{Decode, Encode};

use Error::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct StatusCode(pub u32);

impl From<u32> for StatusCode {
	fn from(value: u32) -> Self {
		StatusCode(value)
	}
}
impl FromStatusCode for StatusCode {
	fn from_status_code(status_code: u32) -> Result<(), Self> {
		match status_code {
			0 => Ok(()),
			_ => {
				let mut encoded = status_code.to_le_bytes();
				convert_unknown_errors(&mut encoded);
				Err(StatusCode::from(u32::from_le_bytes(encoded)))
			},
		}
	}
}

impl From<scale::Error> for StatusCode {
	fn from(_: scale::Error) -> Self {
		DecodingFailed.into()
	}
}

// If an unknown variant of the `DispatchError` is detected the error needs to be converted
// into the encoded value of `Error::Other`. This conversion is performed by shifting the bytes one
// position forward (discarding the last byte as it is not used) and setting the first byte to the
// encoded value of `Other` (0u8). This ensures the error is correctly categorized as an `Other`
// variant which provides all the necessary information to debug which error occurred in the runtime.
//
// Byte layout explanation:
// - Byte 0: index of the variant within `Error`
// - Byte 1:
//   - Must be zero for `UNIT_ERRORS`.
//   - Represents the nested error in `SINGLE_NESTED_ERRORS`.
//   - Represents the first level of nesting in `DOUBLE_NESTED_ERRORS`.
// - Byte 2:
//   - Represents the second level of nesting in `DOUBLE_NESTED_ERRORS`.
// - Byte 3:
//   - Unused or represents further nested information.
//
// This mechanism ensures backward compatibility by correctly categorizing any unknown errors
// into the `Other` variant, thus preventing issues caused by breaking changes.
fn convert_unknown_errors(encoded_error: &mut [u8; 4]) {
	let all_errors = [
		UNIT_ERRORS.as_slice(),
		SINGLE_NESTED_ERRORS.as_slice(),
		DOUBLE_NESTED_ERRORS.as_slice(),
		// `DecodingFailed`.
		&[255u8],
	]
	.concat();
	// Unknown errors, i.e. an encoded value where the first byte is non-zero (indicating a variant
	// in `Error`) but unknown.
	if !all_errors.contains(&encoded_error[0]) {
		encoded_error[..].rotate_right(1);
		encoded_error[0] = 0u8;
	}
	convert_unknown_nested_errors(encoded_error);
}

// If an unknown nested variant of the `DispatchError` is detected (i.e. when any of the subsequent
// bytes are non-zero).
fn convert_unknown_nested_errors(encoded_error: &mut [u8; 4]) {
	// Converts single nested errors that are known to the Pop API as unit errors into `Other`.
	// match encoded_error {
	// 	[a, 0, 0, 0] => {},
	// 	[a, b, 0, 0] => {
	// 		if UNIT_ERRORS.contains(a) {
	// 			encoded_error[..].rotate_right(1);
	// 			encoded_error[0] = 0u8;
	// 		}
	// 	},
	// 	[a, b, c, 0] => {
	// 		if UNIT_ERRORS.contains(a) || SINGLE_NESTED_ERRORS.contains(a) {
	// 			encoded_error[..].rotate_right(1);
	// 			encoded_error[0] = 0u8;
	// 		}
	// 	},
	// 	[a, b, c, d] => {
	// 		if UNIT_ERRORS.contains(a)
	// 			|| SINGLE_NESTED_ERRORS.contains(a)
	// 			|| DOUBLE_NESTED_ERRORS.contains(a)
	// 		{
	// 			encoded_error[..].rotate_right(1);
	// 			encoded_error[0] = 0u8;
	// 		}
	// 	},
	// }
	if UNIT_ERRORS.contains(&encoded_error[0]) && encoded_error[1..].iter().any(|x| *x != 0u8) {
		encoded_error[..].rotate_right(1);
		encoded_error[0] = 0u8;
	// Converts double nested errors that are known to the Pop API as single nested errors into
	// `Other`.
	} else if SINGLE_NESTED_ERRORS.contains(&encoded_error[0])
		&& encoded_error[2..].iter().any(|x| *x != 0u8)
	{
		encoded_error[..].rotate_right(1);
		encoded_error[0] = 0u8;
	} else if DOUBLE_NESTED_ERRORS.contains(&encoded_error[0])
		&& encoded_error[3..].iter().any(|x| *x != 0u8)
	{
		encoded_error[..].rotate_right(1);
		encoded_error[0] = 0u8;
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[repr(u8)]
pub enum Error {
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
	DecodingFailed = 255,
}

// A const function is required for defining constants.
const fn error_to_u8(error: Error) -> u8 {
	match error {
		Error::Other { .. } => 0,
		Error::CannotLookup => 1,
		Error::BadOrigin => 2,
		Error::Module { .. } => 3,
		Error::ConsumerRemaining => 4,
		Error::NoProviders => 5,
		Error::TooManyConsumers => 6,
		Error::Token(_) => 7,
		Error::Arithmetic(_) => 8,
		Error::Transactional(_) => 9,
		Error::Exhausted => 10,
		Error::Corruption => 11,
		Error::Unavailable => 12,
		Error::RootNotAllowed => 13,
		Error::DecodingFailed => 255,
	}
}

macro_rules! unit_error_values {
    ($($variant:ident),*) => {
        [$(
            error_to_u8(Error::$variant)
        ),*]
    };
}

// Unit `Error` variants.
const UNIT_ERRORS: [u8; 10] = unit_error_values!(
	CannotLookup,
	BadOrigin,
	ConsumerRemaining,
	NoProviders,
	TooManyConsumers,
	Exhausted,
	Corruption,
	Unavailable,
	RootNotAllowed,
	DecodingFailed
);

// Macro for single nested errors
macro_rules! single_nested_error_values {
    ($($variant:ident($default:expr)),*) => {
        [$(
            error_to_u8(Error::$variant($default))
        ),*]
    };
}

// Single nested `Error` variants.
//
// Default values had to be given explicitly because non-const functions can be used for constan.
const SINGLE_NESTED_ERRORS: [u8; 3] = single_nested_error_values!(
	Token(TokenError::FundsUnavailable),
	Arithmetic(ArithmeticError::Underflow),
	Transactional(TransactionalError::LimitReached)
);
// const SINGLE_NESTED_ERRORS: [u8; 3] = single_error_values!(Token, Arithmetic, Transactional);

// Double nested `Error` variants
const DOUBLE_NESTED_ERRORS: [u8; 1] = [error_to_u8(Module { error: 0, index: 0 })];

impl From<Error> for StatusCode {
	fn from(value: Error) -> Self {
		let mut encoded_error = value.encode();
		// Resize the encoded value to 4 bytes in order to decode the value in a u32 (4 bytes).
		encoded_error.resize(4, 0);
		StatusCode::from(
			u32::decode(&mut &encoded_error[..]).expect("qid, resized to 4 bytes line above"),
		)
	}
}

impl From<StatusCode> for Error {
	fn from(value: StatusCode) -> Self {
		let encoded: [u8; 4] = value.0.to_le_bytes();
		Error::decode(&mut &encoded[..]).unwrap_or(DecodingFailed)
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::error::{ArithmeticError::*, TokenError::*, TransactionalError::*};

	#[test]
	fn u32_always_encodes_to_4_bytes() {
		assert_eq!(0u32.encode().len(), 4);
		assert_eq!(u32::MAX.encode().len(), 4);
	}

	// Decodes 4 bytes into a `u32` and converts it into `StatusCode`.
	fn into_status_code(encoded_error: [u8; 4]) -> StatusCode {
		let decoded_u32 = u32::decode(&mut &encoded_error[..]).unwrap();
		StatusCode::from_status_code(decoded_u32).unwrap_err()
	}

	// Decodes 4 bytes into a `u32` and converts it into `Error`.
	fn into_error(encoded_error: [u8; 4]) -> Error {
		let decoded_u32 = u32::decode(&mut &encoded_error[..]).unwrap();
		let status_code = StatusCode::from_status_code(decoded_u32).unwrap_err();
		status_code.into()
	}

	// Tests the `From<StatusCode>` implementation for `Error`.
	//
	// Unit variants:
	// If the encoded value indicates a nested `Error` which is known by the Pop API version as a
	// unit variant, the encoded value is converted into `Error::Other`.
	//
	// Example: the error `BadOrigin` (encoded: `[2, 0, 0, 0]`) with a non-zero value for one
	// of the bytes [1..4]: `[2, 0, 1, 0]` is converted into `[0, 2, 0, 1]`. This is decoded to
	// `Error::Other { dispatch_error: 2, index: 0, error: 1 }`.
	#[test]
	fn unit_error_variants() {
		let errors = vec![
			CannotLookup,
			BadOrigin,
			ConsumerRemaining,
			NoProviders,
			TooManyConsumers,
			Exhausted,
			Corruption,
			Unavailable,
			RootNotAllowed,
			DecodingFailed,
		];
		// Four scenarios, 2 tests each:
		// 1. Compare a `StatusCode`, which is converted from an encoded value, with a `StatusCode`
		// 	converted from an `Error`.
		// 2. Compare an `Error, which is converted from an encoded value, with the expected `Error`.
		for (i, &error_code) in UNIT_ERRORS.iter().enumerate() {
			// No nesting and unit variant correctly returned.
			assert_eq!(into_status_code([error_code, 0, 0, 0]), errors[i].into());
			assert_eq!(into_error([error_code, 0, 0, 0]), errors[i]);
			// Unexpected second byte nested.
			assert_eq!(
				into_status_code([error_code, 1, 0, 0]),
				(Other { dispatch_error_index: error_code, error_index: 1, error: 0 }).into(),
			);
			assert_eq!(
				into_error([error_code, 1, 0, 0]),
				Other { dispatch_error_index: error_code, error_index: 1, error: 0 },
			);
			// Unexpected third byte nested.
			assert_eq!(
				into_status_code([error_code, 1, 1, 0]),
				(Other { dispatch_error_index: error_code, error_index: 1, error: 1 }).into(),
			);
			assert_eq!(
				into_error([error_code, 1, 1, 0]),
				Other { dispatch_error_index: error_code, error_index: 1, error: 1 },
			);
			// Unexpected fourth byte nested.
			assert_eq!(
				into_status_code([error_code, 1, 1, 1]),
				(Other { dispatch_error_index: error_code, error_index: 1, error: 1 }).into(),
			);
			assert_eq!(
				into_error([error_code, 1, 1, 1]),
				Other { dispatch_error_index: error_code, error_index: 1, error: 1 },
			);
		}
	}

	// Single nested variants:
	// If the encoded value indicates a double nested `Error` which is known by the Pop API version
	// as a single nested variant, the encoded value is converted into `Error::Other`.
	//
	// Example: the error `Arithmetic(Overflow)` (encoded: `[8, 1, 0, 0]`) with a non-zero
	// value for one of the bytes [2..4]: `[8, 1, 1, 0]` is converted into `[0, 8, 1, 1]`. This is
	// decoded to `Error::Other { dispatch_error: 8, index: 1,  error: 1 }`.
	#[test]
	fn single_nested_error_variants() {
		let errors = vec![
			[Token(FundsUnavailable), Token(OnlyProvider)],
			[Arithmetic(Underflow), Arithmetic(Overflow)],
			[Transactional(LimitReached), Transactional(NoLayer)],
		];
		// Four scenarios, 2 tests each:
		// 1. Compare a `StatusCode`, which is converted from an encoded value, with a `StatusCode`
		// 	converted from an `Error`.
		// 2. Compare an `Error, which is converted from an encoded value, with the expected `Error`.
		for (i, &error_code) in SINGLE_NESTED_ERRORS.iter().enumerate() {
			// No nesting and unit variant correctly returned.
			assert_eq!(into_status_code([error_code, 0, 0, 0]), errors[i][0].into());
			assert_eq!(into_error([error_code, 0, 0, 0]), errors[i][0]);
			// Allowed single nesting variant correctly returned.
			assert_eq!(into_status_code([error_code, 1, 0, 0]), errors[i][1].into());
			assert_eq!(into_error([error_code, 1, 0, 0]), errors[i][1]);
			// Unexpected third byte nested.
			assert_eq!(
				into_status_code([error_code, 1, 1, 0]),
				(Other { dispatch_error_index: error_code, error_index: 1, error: 1 }).into(),
			);
			assert_eq!(
				into_error([error_code, 1, 1, 0]),
				Other { dispatch_error_index: error_code, error_index: 1, error: 1 },
			);
			// Unexpected fourth byte nested.
			assert_eq!(
				into_status_code([error_code, 1, 1, 1]),
				(Other { dispatch_error_index: error_code, error_index: 1, error: 1 }).into(),
			);
			assert_eq!(
				into_error([error_code, 1, 1, 1]),
				Other { dispatch_error_index: error_code, error_index: 1, error: 1 },
			);
		}
	}

	// Double nested variants:
	// If the encoded value indicates a triple nested `Error` which is known by the Pop API version
	// as a double nested variant, the encoded value is converted into `Error::Other`.
	//
	// Example: the error `Module { index: 10, error 5 }` (encoded: `[3, 10, 5, 0]`) with a non-zero
	// value for the last byte: `[3, 10, 5, 3]` is converted into `[0, 3, 10, 5]`. This is
	// decoded to `Error::Other { dispatch_error: 3, index: 10,  error: 5 }`.
	#[test]
	fn double_nested_error_variants() {
		// Four scenarios, 2 tests each:
		// 1. Compare a `StatusCode`, which is converted from an encoded value, with a `StatusCode`
		// 	converted from an `Error`.
		// 2. Compare an `Error, which is converted from an encoded value, with the expected `Error`.
		//
		// No nesting and unit variant correctly returned.
		assert_eq!(into_status_code([3, 0, 0, 0]), (Module { index: 0, error: 0 }).into());
		assert_eq!(into_error([3, 0, 0, 0]), Module { index: 0, error: 0 });
		// Allowed single nesting and variant correctly returned.
		assert_eq!(into_status_code([3, 1, 0, 0]), (Module { index: 1, error: 0 }).into());
		assert_eq!(into_error([3, 1, 0, 0]), Module { index: 1, error: 0 });
		// Allowed double nesting and variant correctly returned.
		assert_eq!(into_status_code([3, 1, 1, 0]), (Module { index: 1, error: 1 }).into());
		assert_eq!(into_error([3, 1, 1, 0]), Module { index: 1, error: 1 });
		// Unexpected fourth byte nested.
		assert_eq!(
			into_status_code([3, 1, 1, 1]),
			(Other { dispatch_error_index: 3, error_index: 1, error: 1 }).into(),
		);
		assert_eq!(
			into_error([3, 1, 1, 1]),
			Other { dispatch_error_index: 3, error_index: 1, error: 1 },
		);
	}

	#[test]
	fn test_random_encoded_values() {
		assert_eq!(
			into_error([100, 100, 100, 100]),
			Other { dispatch_error_index: 100, error_index: 100, error: 100 }
		);
		assert_eq!(
			into_error([200, 200, 200, 200]),
			Other { dispatch_error_index: 200, error_index: 200, error: 200 }
		);
	}
}

use ink::env::chain_extension::FromStatusCode;
use scale::{Decode, Encode};
use PopApiError::*;

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
	DecodingFailed = 255,
}

impl From<PopApiError> for StatusCode {
	fn from(value: PopApiError) -> Self {
		let mut encoded_error = value.encode();
		// Resize the encoded value to 4 bytes in order to decode the value in a u32 (4 bytes).
		encoded_error.resize(4, 0);
		StatusCode::from(
			u32::decode(&mut &encoded_error[..]).expect("qid, resized to 4 bytes line above"),
		)
	}
}

impl From<StatusCode> for PopApiError {
	// `pub` because it is used in `runtime/devnet/src/extensions/tests/mod.rs`'s test:
	// `dispatch_error_to_status_code_to_pop_api_error_works`
	//
	// This function converts a given `status_code` (u32) into a `PopApiError`.
	fn from(value: StatusCode) -> Self {
		let encoded: [u8; 4] = value.0.to_le_bytes();
		PopApiError::decode(&mut &encoded[..]).unwrap_or(DecodingFailed)
	}
}

// If an unknown nested variant of the `DispatchError` is detected (i.e., any of the subsequent
// bytes are non-zero, indicating a breaking change in the `DispatchError`), the error needs to be
// converted into the encoded value of `PopApiError::Other`. This conversion is performed by
// shifting the bytes one position forward (discarding the last byte as it is not used) and setting
// the first byte to the encoded value of `Other` (0u8). This ensures the error is correctly
// categorized as an `Other` variant.
//
// Byte layout explanation:
// - Byte 0: PopApiError
// - Byte 1:
//   - Must be zero for `UNIT_ERRORS`.
//   - Represents the nested error in `SINGLE_NESTED_ERRORS`.
//   - Represents the first level of nesting in `DOUBLE_NESTED_ERRORS`.
// - Byte 2:
//   - Represents the second level of nesting in `DOUBLE_NESTED_ERRORS`.
// - Byte 3:
//   - Unused or represents further nested information.
//
// This mechanism ensures backward compatibility by correctly categorizing any unknown nested errors
// into the `Other` variant, thus preventing issues caused by new or unexpected error formats.
pub(crate) fn convert_unknown_nested_errors(encoded_error: &mut [u8; 4]) {
	// Converts single nested errors that are known to the Pop API as unit errors into `Other`.
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

pub(crate) fn convert_unknown_errors(encoded_error: &mut [u8; 4]) {
	let all_errors = [
		UNIT_ERRORS.as_slice(),
		SINGLE_NESTED_ERRORS.as_slice(),
		DOUBLE_NESTED_ERRORS.as_slice(),
		// `DecodingFailed`.
		&[255u8],
	]
	.concat();
	if !all_errors.contains(&encoded_error[0]) {
		encoded_error[..].rotate_right(1);
		encoded_error[0] = 0u8;
	}
	convert_unknown_nested_errors(encoded_error);
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

// Unit `DispatchError` variants (variant: index):
// - CannotLookup: 1,
// - BadOrigin: 2,
// - ConsumerRemaining: 4,
// - NoProviders: 5,
// - TooManyConsumers: 6,
// - Exhausted: 10,
// - Corruption: 11,
// - Unavailable: 12,
// - RootNotAllowed: 13,
const UNIT_ERRORS: [u8; 9] = [1, 2, 4, 5, 6, 10, 11, 12, 13];

// Single nested `DispatchError` variants (variant: index):
// - Token: 3,
// - Arithmetic: 8,
// - Transaction: 9,
const SINGLE_NESTED_ERRORS: [u8; 3] = [7, 8, 9];

const DOUBLE_NESTED_ERRORS: [u8; 1] = [3];

#[cfg(test)]
mod tests {
	use super::*;
	use crate::error::{ArithmeticError::*, TokenError::*, TransactionalError::*};

	#[test]
	fn u32_always_encodes_to_4_bytes() {
		assert_eq!(0u32.encode().len(), 4);
		assert_eq!(u32::MAX.encode().len(), 4);
	}

	// Decodes into `StatusCode(u32)` and converts it into the `PopApiError`.
	fn into_pop_api_error(encoded_error: [u8; 4]) -> PopApiError {
		let status_code =
			StatusCode::from_status_code(u32::decode(&mut &encoded_error[..]).unwrap())
				.unwrap_err();
		status_code.into()
	}

	// Tests for the `From<StatusCode(u32)>` implementation for `PopApiError`.
	//
	// If the encoded value indicates a nested `PopApiError` which is not handled by the Pop API
	// version, the encoded value is converted into `PopApiError::Other`.
	#[test]
	fn test_unit_pop_api_error_variants() {
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
		];
		for (i, &error_code) in UNIT_ERRORS.iter().enumerate() {
			assert_eq!(into_pop_api_error([error_code, 0, 0, 0]), errors[i]);
			assert_eq!(
				into_pop_api_error([error_code, 1, 0, 0]),
				Other { dispatch_error_index: error_code, error_index: 1, error: 0 },
			);
			assert_eq!(
				into_pop_api_error([error_code, 1, 1, 0]),
				Other { dispatch_error_index: error_code, error_index: 1, error: 1 },
			);
			assert_eq!(
				into_pop_api_error([error_code, 1, 1, 1]),
				Other { dispatch_error_index: error_code, error_index: 1, error: 1 },
			);
		}
	}

	#[test]
	fn test_single_nested_pop_api_error_variants() {
		let errors = vec![
			[Token(FundsUnavailable), Token(OnlyProvider)],
			[Arithmetic(Underflow), Arithmetic(Overflow)],
			[Transactional(LimitReached), Transactional(NoLayer)],
		];
		for (i, &error_code) in SINGLE_NESTED_ERRORS.iter().enumerate() {
			assert_eq!(into_pop_api_error([error_code, 0, 0, 0]), errors[i][0]);
			assert_eq!(into_pop_api_error([error_code, 1, 0, 0]), errors[i][1]);
			assert_eq!(
				into_pop_api_error([error_code, 1, 1, 0]),
				Other { dispatch_error_index: error_code, error_index: 1, error: 1 },
			);
			assert_eq!(
				into_pop_api_error([error_code, 1, 1, 1]),
				Other { dispatch_error_index: error_code, error_index: 1, error: 1 },
			);
		}
	}

	#[test]
	fn test_double_nested_pop_api_error_variants() {
		assert_eq!(into_pop_api_error([3, 0, 0, 0]), Module { index: 0, error: 0 });
		assert_eq!(into_pop_api_error([3, 1, 0, 0]), Module { index: 1, error: 0 });
		assert_eq!(into_pop_api_error([3, 1, 1, 0]), Module { index: 1, error: 1 });
		// TODO: doesn't make sense.
		assert_eq!(
			into_pop_api_error([3, 1, 1, 1]),
			Other { dispatch_error_index: 3, error_index: 1, error: 1 },
		);
	}

	#[test]
	fn test_decoding_failed() {
		assert_eq!(into_pop_api_error([255, 0, 0, 0]), DecodingFailed);
		assert_eq!(into_pop_api_error([255, 255, 0, 0]), DecodingFailed);
		assert_eq!(into_pop_api_error([255, 255, 255, 0]), DecodingFailed);
		assert_eq!(into_pop_api_error([255, 255, 255, 255]), DecodingFailed);
	}

	#[test]
	fn test_random_encoded_values() {
		assert_eq!(
			into_pop_api_error([100, 100, 100, 100]),
			Other { dispatch_error_index: 100, error_index: 100, error: 100 }
		);
		assert_eq!(
			into_pop_api_error([200, 200, 200, 200]),
			Other { dispatch_error_index: 200, error_index: 200, error: 200 }
		);
	}
}

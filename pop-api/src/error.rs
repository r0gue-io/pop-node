use ink::env::chain_extension::FromStatusCode;
use scale::{Decode, Encode};

pub use pop_primitives::Error::{self, *};

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
				if unknown_errors(&encoded) {
					encoded[..].rotate_right(1);
					encoded[0] = 0u8;
				};
				Err(StatusCode::from(u32::from_le_bytes(encoded)))
			},
		}
	}
}

impl From<scale::Error> for StatusCode {
	fn from(_: scale::Error) -> Self {
		u32::from_le_bytes([255, 0, 0, 0]).into()
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
fn unknown_errors(encoded_error: &[u8; 4]) -> bool {
	match encoded_error[0] {
		code if UNIT_ERRORS.contains(&code) => nested_errors(&encoded_error[1..], None),
		// Single nested errors with a limit in their nesting.
		//
		// `TokenError`: has ten variants - translated to a limit of nine.
		7 => nested_errors(&encoded_error[1..], Some(9)),
		// `ArithmeticError`: has 3 variants - translated to a limit of two.
		8 => nested_errors(&encoded_error[1..], Some(2)),
		// `TransactionalError`: has 2 variants - translated to a limit of one.
		9 => nested_errors(&encoded_error[1..], Some(1)),
		code if DOUBLE_NESTED_ERRORS.contains(&code) => nested_errors(&encoded_error[3..], None),
		_ => true,
	}
}

// Checks for unknown nested errors within the `DispatchError`.
// - For single nested errors with a limit, it verifies if the nested value exceeds the limit.
// - For other nested errors, it checks if any subsequent bytes are non-zero.
//
// `nested_error` - The slice of bytes representing the nested error.
// `limit` - An optional limit for single nested errors.
fn nested_errors(nested_error: &[u8], limit: Option<u8>) -> bool {
	match limit {
		Some(l) => nested_error[0] > l || nested_error[1..].iter().any(|&x| x != 0u8),
		None => nested_error.iter().any(|&x| x != 0u8),
	}
}

// Unit `Error` variants.
// (variant: index):
// - CannotLookup: 1,
// - BadOrigin: 2,
// - ConsumerRemaining: 4,
// - NoProviders: 5,
// - TooManyConsumers: 6,
// - Exhausted: 10,
// - Corruption: 11,
// - Unavailable: 12,
// - RootNotAllowed: 13,
// - DecodingFailed: 255,
const UNIT_ERRORS: [u8; 10] = [1, 2, 4, 5, 6, 10, 11, 12, 13, 255];

#[cfg(test)]
const SINGLE_NESTED_ERRORS: [u8; 3] = [7, 8, 9];

// Double nested `Error` variants
// (variant: index):
// - Module: 3,
const DOUBLE_NESTED_ERRORS: [u8; 1] = [3];

#[cfg(test)]
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

#[cfg(test)]
mod tests {
	use super::*;
	use pop_primitives::{ArithmeticError::*, TokenError::*, TransactionalError::*};

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
	fn single_nested_unknown_variants() {
		// Unknown `TokenError` variant.
		assert_eq!(
			into_error([7, 10, 0, 0]),
			Other { dispatch_error_index: 7, error_index: 10, error: 0 }
		);
		assert_eq!(
			into_status_code([7, 10, 0, 0]),
			Other { dispatch_error_index: 7, error_index: 10, error: 0 }.into()
		);
		// Unknown `Arithmetic` variant.
		assert_eq!(
			into_error([8, 3, 0, 0]),
			Other { dispatch_error_index: 8, error_index: 3, error: 0 }
		);
		assert_eq!(
			into_status_code([8, 3, 0, 0]),
			Other { dispatch_error_index: 8, error_index: 3, error: 0 }.into()
		);
		// Unknown `Transactional` variant.
		assert_eq!(
			into_error([9, 2, 0, 0]),
			Other { dispatch_error_index: 9, error_index: 2, error: 0 }
		);
		assert_eq!(
			into_status_code([9, 2, 0, 0]),
			Other { dispatch_error_index: 9, error_index: 2, error: 0 }.into()
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

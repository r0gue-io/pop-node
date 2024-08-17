//! TODO

// Unit errors.
// - CannotLookup: 1,
// - BadOrigin: 2,
// - ConsumerRemaining: 4,
// - NoProviders: 5,
// - TooManyConsumers: 6,
// - Exhausted: 10,
// - Corruption: 11,
// - Unavailable: 12,
// - RootNotAllowed: 13,
// - UnknownCall: 254,
// - DecodingFailed: 255,
const UNIT_ERRORS: [u8; 11] = [1, 2, 4, 5, 6, 10, 11, 12, 13, 254, 255];

// Single nested errors.
const TOKEN_ERROR: u8 = 7;
const ARITHMETIC_ERROR: u8 = 8;
const TRANSACTIONAL_ERROR: u8 = 9;
// Starting indices at which specific single nested error variants are considered invalid. These
// constants define the boundary for valid error variant indices within certain `DispatchError` enums.
const INVALID_TOKEN_INDEX: u8 = 10;
const INVALID_ARITHMETIC_INDEX: u8 = 3;
const INVALID_TRANSACTIONAL_INDEX: u8 = 2;

// Double nested error.
const MODULE_ERROR: u8 = 3;

// If an unknown error of the `DispatchError` is detected, the error needs to be converted
// into the encoded value of `pop_api::Error::Other`. This conversion is performed by shifting the bytes one
// position forward (discarding the last byte as it is not used) and setting the first byte to the
// encoded value of `Other` (0u8). This ensures the error is correctly categorized as an `Other`
// variant which provides all the necessary information to debug which error occurred in the runtime.
//
// Byte layout explanation (prior to conversion by `handle_unknown_error`):
// - Byte 0: index of the error within `pop_api::Error`
// - Byte 1:
//   - Must be zero for `UNIT_ERRORS`.
//   - Represents the nested error in `SINGLE_NESTED_ERRORS`.
//   - Represents the first level of nesting in `DOUBLE_NESTED_ERRORS`.
// - Byte 2:
//   - Represents the second level of nesting in `DOUBLE_NESTED_ERRORS`.
// - Byte 3:
//   - Unused or represents further nested information.
pub(crate) fn handle_unknown_error(encoded_error: &mut [u8; 4]) {
	let is_unknown_error = match encoded_error[0] {
		code if UNIT_ERRORS.contains(&code) => nested_errors(&encoded_error[1..], None),
		TOKEN_ERROR => nested_errors(&encoded_error[1..], Some(INVALID_TOKEN_INDEX)),
		ARITHMETIC_ERROR => nested_errors(&encoded_error[1..], Some(INVALID_ARITHMETIC_INDEX)),
		TRANSACTIONAL_ERROR => {
			nested_errors(&encoded_error[1..], Some(INVALID_TRANSACTIONAL_INDEX))
		},
		MODULE_ERROR => nested_errors(&encoded_error[3..], None),
		_ => true,
	};
	if is_unknown_error {
		encoded_error[..].rotate_right(1);
		encoded_error[0] = 0u8;
	}
}

// Checks for unknown or invalid nested errors within the `DispatchError`.
fn nested_errors(nested_error: &[u8], invalid_index_start: Option<u8>) -> bool {
	match invalid_index_start {
		// Checks if the first byte is equal or exceeds `invalid_index_start` or if any subsequent
		// byte is non-zero.
		Some(i) => nested_error[0] >= i || nested_error[1..].iter().any(|&x| x != 0u8),
		// If no limit is provided, check if any byte is non-zero.
		None => nested_error.iter().any(|&x| x != 0u8),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::assert_encoding_matches;
	use sp_runtime::{ArithmeticError::*, DispatchError::*, TokenError::*, TransactionalError::*};

	// Assert encoding after `handle_unknown_error` with expected encoding.
	fn assert_error_cases(test_cases: Vec<([u8; 4], [u8; 4])>) {
		for (mut error, expected) in test_cases {
			handle_unknown_error(&mut error);
			assert_eq!(error, expected);
		}
	}

	// Tests for `handle_unknown_error`:
	// 1. Unit errors.
	// 2. Single nested errors.
	// 3. Double nested errors.

	// 1. Unit errors.
	//
	// If the encoded value indicates a nested enum, which is known by V0 as a
	// unit error, the encoded value is converted.
	//
	// Example: the error `BadOrigin` (encoded: `[2, 0, 0, 0]`) with a non-zero value for one
	// of the bytes [1..4] - `[2, 0, 1, 0]` - is converted into `[0, 2, 0, 1]` (shifting the bits
	// one forward). This is decoded to `Error::Other { dispatch_error: 2, index: 0, error: 1 }`.

	macro_rules! unit_error_test {
		($name:ident, $error_variant:expr, $encoded_value:expr) => {
			#[test]
			fn $name() {
				assert_encoding_matches($error_variant, $encoded_value);
				let index = $encoded_value[0];
				let test_cases = vec![
					($encoded_value, $encoded_value),
					([index, 1, 0, 0], [0, index, 1, 0]),
					([index, 1, 1, 0], [0, index, 1, 1]),
					([index, 1, 1, 1], [0, index, 1, 1]),
					([index, 1, 0, 1], [0, index, 1, 0]),
					([index, 0, 1, 1], [0, index, 0, 1]),
					([index, 0, 0, 1], [0, index, 0, 0]),
				];
				assert_error_cases(test_cases);
			}
		};
	}

	unit_error_test!(cannot_lookup_works, CannotLookup, [1u8, 0, 0, 0]);
	unit_error_test!(bad_origin_works, BadOrigin, [2u8, 0, 0, 0]);
	unit_error_test!(consumer_remaining_works, ConsumerRemaining, [4u8, 0, 0, 0]);
	unit_error_test!(no_providers_works, NoProviders, [5u8, 0, 0, 0]);
	unit_error_test!(too_many_consumers_works, TooManyConsumers, [6u8, 0, 0, 0]);
	unit_error_test!(exhausted_works, Exhausted, [10u8, 0, 0, 0]);
	unit_error_test!(corruption_works, Corruption, [11u8, 0, 0, 0]);
	unit_error_test!(unavailable_works, Unavailable, [12u8, 0, 0, 0]);
	unit_error_test!(root_not_allowed_works, RootNotAllowed, [13u8, 0, 0, 0]);

	// 2. Single nested errors.
	//
	// If the encoded value indicates a double nested error which is known by V0
	// as a single nested error, the encoded value is converted.
	//
	// Example: the error `Arithmetic(Overflow)` (encoded: `[8, 1, 0, 0]`) with a non-zero
	// value for one of the bytes [2..4]: `[8, 1, 1, 0]` is converted into `[0, 8, 1, 1]`. This is
	// decoded to `Error::Other { dispatch_error: 8, index: 1,  error: 1 }`.

	macro_rules! single_nested_error_test {
		($name:ident, $error_enum:path, $base_error_code:expr, $invalid_index:expr, $errors:expr) => {
			#[test]
			fn $name() {
				for (error, index) in $errors {
					let valid_variant_expected_encoding = [$base_error_code, index, 0, 0];
					assert_encoding_matches($error_enum(error), valid_variant_expected_encoding);
					// Test cases with valid single nested values.
					let test_cases = vec![
						([$base_error_code, index, 0, 0], valid_variant_expected_encoding),
						([$base_error_code, index, 1, 0], [0, $base_error_code, index, 1]),
						([$base_error_code, index, 0, 1], [0, $base_error_code, index, 0]),
						([$base_error_code, index, 1, 1], [0, $base_error_code, index, 1]),
					];
					assert_error_cases(test_cases);
				}

				// Test cases with invalid single nested values.
				for x in [$invalid_index, 100, u8::MAX] {
					let test_cases = vec![
						([$base_error_code, x, 0, 0], [0, $base_error_code, x, 0]),
						([$base_error_code, x, 1, 0], [0, $base_error_code, x, 1]),
						([$base_error_code, x, 0, 1], [0, $base_error_code, x, 0]),
						([$base_error_code, x, 1, 1], [0, $base_error_code, x, 1]),
					];
					assert_error_cases(test_cases);
				}
			}
		};
	}

	single_nested_error_test!(
		token_error_works,
		Token,
		TOKEN_ERROR,
		INVALID_TOKEN_INDEX,
		vec![
			(FundsUnavailable, 0),
			(OnlyProvider, 1),
			(BelowMinimum, 2),
			(CannotCreate, 3),
			(UnknownAsset, 4),
			(Frozen, 5),
			(Unsupported, 6),
			(CannotCreateHold, 7),
			(NotExpendable, 8),
			(Blocked, 9),
		]
	);

	single_nested_error_test!(
		arithmetic_error_works,
		Arithmetic,
		ARITHMETIC_ERROR,
		INVALID_ARITHMETIC_INDEX,
		vec![(Underflow, 0), (Overflow, 1), (DivisionByZero, 2)]
	);

	single_nested_error_test!(
		transactional_error_works,
		Transactional,
		TRANSACTIONAL_ERROR,
		INVALID_TRANSACTIONAL_INDEX,
		vec![(LimitReached, 0), (NoLayer, 1)]
	);

	// 3. Double nested error.
	//
	// If the encoded value indicates a triple nested error which is known by V0
	// as a double nested error, the encoded value is converted.
	//
	// Example: the error `Module { index: 10, error 5 }` (encoded: `[3, 10, 5, 0]`) with a non-zero
	// value for the last byte: `[3, 10, 5, 3]` is converted into `[0, 3, 10, 5]`. This is
	// decoded to `Error::Other { dispatch_error: 3, index: 10, error: 5 }`.

	#[test]
	fn double_nested_error_variants() {
		for x in [1, 100, u8::MAX] {
			let test_cases = vec![
				([MODULE_ERROR, x, 0, 0], [MODULE_ERROR, x, 0, 0]),
				([MODULE_ERROR, x, x, 0], [MODULE_ERROR, x, x, 0]),
				([MODULE_ERROR, x, x, x], [0, MODULE_ERROR, x, x]),
				([MODULE_ERROR, 0, x, x], [0, MODULE_ERROR, 0, x]),
				([MODULE_ERROR, 0, 0, x], [0, MODULE_ERROR, 0, 0]),
			];
			assert_error_cases(test_cases);
		}
	}

	#[test]
	fn test_random_encoded_values() {
		let test_cases = vec![
			([100, 100, 100, 100], [0, 100, 100, 100]),
			([u8::MAX, u8::MAX, u8::MAX, u8::MAX], [0, u8::MAX, u8::MAX, u8::MAX]),
		];
		assert_error_cases(test_cases);
	}
}

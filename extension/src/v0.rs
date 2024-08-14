#[cfg(test)]
use crate::convert_to_status_code;

pub(crate) fn handle_unknown_error(encoded_error: &mut [u8; 4]) {
	let unknown = match encoded_error[0] {
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
	};
	if unknown {
		encoded_error[..].rotate_right(1);
		encoded_error[0] = 0u8;
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
// - UnknownCall: 254,
// - DecodingFailed: 255,
const UNIT_ERRORS: [u8; 11] = [1, 2, 4, 5, 6, 10, 11, 12, 13, 254, 255];

#[cfg(test)]
const SINGLE_NESTED_ERRORS: [u8; 3] = [7, 8, 9];

// Double nested `Error` variants
// (variant: index):
// - Module: 3,
const DOUBLE_NESTED_ERRORS: [u8; 1] = [3];

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

#[cfg(test)]
mod tests {
	use super::*;
	use sp_runtime::{DispatchError, ModuleError};

	// Check if `DispatchError` is correctly converted into the expected status code (`u32`).
	// Each test case has an associated 4-byte array that represents the `u32` status code in
	// little-endian format.
	#[test]
	fn dispatch_error_to_status_code() {
		let test_cases = vec![
			(DispatchError::Other(""), [0u8, 0, 0, 0]),
			(DispatchError::CannotLookup, [1, 0, 0, 0]),
			(DispatchError::BadOrigin, [2, 0, 0, 0]),
			(
				DispatchError::Module(sp_runtime::ModuleError {
					index: 1,
					error: [2, 0, 0, 0],
					message: Some("hallo"),
				}),
				[3, 1, 2, 0],
				// Module { index: 1, error: 2 },
			),
			(DispatchError::ConsumerRemaining, [4, 0, 0, 0]),
			(DispatchError::NoProviders, [5, 0, 0, 0]),
			(DispatchError::TooManyConsumers, [6, 0, 0, 0]),
			(DispatchError::Token(sp_runtime::TokenError::BelowMinimum), [7, 2, 0, 0]),
			(DispatchError::Arithmetic(sp_runtime::ArithmeticError::Overflow), [8, 1, 0, 0]),
			(
				DispatchError::Transactional(sp_runtime::TransactionalError::LimitReached),
				[9, 0, 0, 0],
			),
			(DispatchError::Exhausted, [10, 0, 0, 0]),
			(DispatchError::Corruption, [11, 0, 0, 0]),
			(DispatchError::Unavailable, [12, 0, 0, 0]),
			(DispatchError::RootNotAllowed, [13, 0, 0, 0]),
			(DispatchError::Other("UnknownCall"), [254, 0, 0, 0]),
			(DispatchError::Other("DecodingFailed"), [255, 0, 0, 0]),
		];
		for (dispatch_error, expected) in test_cases {
			let status_code = convert_to_status_code(dispatch_error, 0);
			assert_eq!(status_code, u32::from_le_bytes(expected));
		}
	}

	// This test checks various nesting possibilities of `DispatchError::Module` and ensures
	// that they are correctly converted to a `u32` status code. The expected result for
	// each test case is the little-endian 4-byte array `[0u8, 3u8, 1u8, 2u8]`. I.e. the raw bytes
	// of the `pop_api::v0::Error::Other`.
	//
	// Also see the test `fn double_nested_error_variants()` regarding fourth byte nesting.
	#[test]
	fn test_module_error() {
		let test_cases = vec![
			DispatchError::Module(ModuleError {
				index: 1,
				error: [2, 2, 0, 0],
				message: Some("Random"),
			}),
			DispatchError::Module(ModuleError {
				index: 1,
				error: [2, 2, 2, 0],
				message: Some("Random"),
			}),
			DispatchError::Module(ModuleError {
				index: 1,
				error: [2, 2, 2, 2],
				message: Some("Random"),
			}),
		];
		for dispatch_error in test_cases {
			let status_code = convert_to_status_code(dispatch_error, 0);
			assert_eq!(status_code, u32::from_le_bytes([0u8, 3, 1, 2]));
		}
	}

	fn into_error(mut error_bytes: [u8; 4]) -> [u8; 4] {
		handle_unknown_error(&mut error_bytes);
		error_bytes
	}

	// Tests the `handle_unknown_error` for unit variants.
	//
	// If the encoded value indicates a nested enum error, which is known by V0 as a
	// unit error, the encoded value is converted into `pop_api::v0::Error::Other`.
	//
	// Example: the error `BadOrigin` (encoded: `[2, 0, 0, 0]`) with a non-zero value for one
	// of the bytes [1..4]: `[2, 0, 1, 0]` is converted into `[0, 2, 0, 1]` (shifting the bits
	// one forward). This is decoded to `pop_api::v0::Error::Other { dispatch_error: 2, index: 0, error: 1 }`.
	#[test]
	fn unit_error_variants() {
		let errors = vec![
			[1u8, 0, 0, 0],
			[2, 0, 0, 0],
			[4, 0, 0, 0],
			[5, 0, 0, 0],
			[6, 0, 0, 0],
			[10, 0, 0, 0],
			[11, 0, 0, 0],
			[12, 0, 0, 0],
			[13, 0, 0, 0],
			[254, 0, 0, 0],
			[255, 0, 0, 0],
		];
		// Compare an `Error`, which is converted from an encoded value, with the expected `Error`.
		for (i, &error_code) in UNIT_ERRORS.iter().enumerate() {
			// No nesting and unit variant correctly returned.
			assert_eq!(into_error([error_code, 0, 0, 0]), errors[i]);
			// Unexpected second byte nested.
			assert_eq!(into_error([error_code, 1, 0, 0]), [0, error_code, 1, 0],);
			// Unexpected third byte nested.
			assert_eq!(into_error([error_code, 1, 1, 0]), [0, error_code, 1, 1],);
			// Unexpected fourth byte nested.
			assert_eq!(into_error([error_code, 1, 1, 1]), [0, error_code, 1, 1],);
		}
	}

	// Tests the `handle_unknown_error` for single nested variants.
	//
	// If the encoded value indicates a double nested error which is known by V0
	// as a single nested error, the encoded value is converted into `pop_api::v0::Error::Other`.
	//
	// Example: the error `Arithmetic(Overflow)` (encoded: `[8, 1, 0, 0]`) with a non-zero
	// value for one of the bytes [2..4]: `[8, 1, 1, 0]` is converted into `[0, 8, 1, 1]`. This is
	// decoded to `Error::Other { dispatch_error: 8, index: 1,  error: 1 }`.
	#[test]
	fn single_nested_error_variants() {
		let errors = vec![
			[[7u8, 0, 0, 0], [7u8, 1, 0, 0]],
			[[8, 0, 0, 0], [8, 1, 0, 0]],
			[[9, 0, 0, 0], [9, 1, 0, 0]],
		];
		// Compare an `Error`, which is converted from an encoded value, with the expected `Error`.
		for (i, &error_code) in SINGLE_NESTED_ERRORS.iter().enumerate() {
			// No nested and single nested variant correctly returned.
			assert_eq!(into_error([error_code, 0, 0, 0]), errors[i][0]);
			assert_eq!(into_error([error_code, 1, 0, 0]), errors[i][1]);
			// Unexpected third byte nested.
			assert_eq!(into_error([error_code, 1, 1, 0]), [0, error_code, 1, 1],);
			// Unexpected fourth byte nested.
			assert_eq!(into_error([error_code, 1, 1, 1]), [0, error_code, 1, 1],);
		}
	}

	#[test]
	fn single_nested_unknown_variants() {
		// Unknown `TokenError` variant.
		assert_eq!(into_error([7, 10, 0, 0]), [0, 7, 10, 0],);
		// Unknown `Arithmetic` variant.
		assert_eq!(into_error([8, 3, 0, 0]), [0, 8, 3, 0],);
		// Unknown `Transactional` variant.
		assert_eq!(into_error([9, 2, 0, 0]), [0, 9, 2, 0],);
	}

	// Tests the `handle_unknown_error` for double nested variants.
	//
	// If the encoded value indicates a triple nested error which is known by V0
	// as a double nested error, the encoded value is converted into `pop_api::v0::Error::Other`.
	//
	// Example: the error `Module { index: 10, error 5 }` (encoded: `[3, 10, 5, 0]`) with a non-zero
	// value for the last byte: `[3, 10, 5, 3]` is converted into `[0, 3, 10, 5]`. This is
	// decoded to `Error::Other { dispatch_error: 3, index: 10,  error: 5 }`.
	#[test]
	fn double_nested_error_variants() {
		// Compare an `Error`, which is converted from an encoded value, with the expected `Error`.
		// No nesting and unit variant correctly returned.
		assert_eq!(into_error([3, 0, 0, 0]), [3, 0, 0, 0]);
		// Allowed single nesting and variant correctly returned.
		assert_eq!(into_error([3, 1, 0, 0]), [3, 1, 0, 0]);
		// Allowed double nesting and variant correctly returned.
		assert_eq!(into_error([3, 1, 1, 0]), [3, 1, 1, 0]);
		// Unexpected fourth byte nested.
		assert_eq!(into_error([3, 1, 1, 1]), [0, 3, 1, 1],);
	}

	#[test]
	fn test_random_encoded_values() {
		assert_eq!(into_error([100, 100, 100, 100]), [0, 100, 100, 100],);
		assert_eq!(into_error([200, 200, 200, 200]), [0, 200, 200, 200],);
	}
}

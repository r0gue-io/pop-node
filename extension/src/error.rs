use codec::Encode;
use sp_runtime::DispatchError;

pub(crate) const DECODING_FAILED_ERROR: DispatchError = DispatchError::Other("DecodingFailed");
// TODO: issue #93, we can also encode the `pop_primitives::Error::UnknownCall` which means we do use
//  `Error` in the runtime and it should stay in primitives. Perhaps issue #91 will also influence
//  here. Should be looked at together.
const DECODING_FAILED_ERROR_ENCODED: [u8; 4] = [255u8, 0, 0, 0];
pub(crate) const UNKNOWN_CALL_ERROR: DispatchError = DispatchError::Other("UnknownCall");
// TODO: see above.
const UNKNOWN_CALL_ERROR_ENCODED: [u8; 4] = [254u8, 0, 0, 0];

/// Converts a `DispatchError` to a `u32` status code based on the version of the API the contract uses.
/// The contract calling the chain extension can optionally convert the status code to the descriptive `Error`.
///
/// For `Error` see `pop_primitives::<version>::error::Error`.
///
/// The error encoding can vary per version, allowing for flexible and backward-compatible error handling.
/// As a result, contracts maintain compatibility across different versions of the runtime.
///
/// # Parameters
///
/// - `error`: The `DispatchError` encountered during contract execution.
/// - `version`: The version of the chain extension, used to determine the known errors.
pub(crate) fn convert_to_status_code(error: DispatchError, version: u8) -> u32 {
	let mut encoded_error: [u8; 4] = match error {
		// "UnknownCall" and "DecodingFailed" are mapped to specific errors in the API and will
		// never change.
		UNKNOWN_CALL_ERROR => UNKNOWN_CALL_ERROR_ENCODED,
		DECODING_FAILED_ERROR => DECODING_FAILED_ERROR_ENCODED,
		_ => {
			let mut encoded_error = error.encode();
			// Resize the encoded value to 4 bytes in order to decode the value in a u32 (4 bytes).
			encoded_error.resize(4, 0);
			encoded_error.try_into().expect("qed, resized to 4 bytes line above")
		},
	};
	match version {
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
		0 => crate::v0::handle_unknown_error(&mut encoded_error),
		_ => encoded_error = UNKNOWN_CALL_ERROR_ENCODED,
	}
	u32::from_le_bytes(encoded_error)
}

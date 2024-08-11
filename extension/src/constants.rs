use sp_runtime::DispatchError;

/// Logging target for categorizing messages from the Pop API extension module.
pub(crate) const LOG_TARGET: &str = "pop-api::extension";

pub const DECODING_FAILED_ERROR: DispatchError = DispatchError::Other("DecodingFailed");
// TODO: issue #93, we can also encode the `pop_primitives::Error::UnknownCall` which means we do use
//  `Error` in the runtime and it should stay in primitives. Perhaps issue #91 will also influence
//  here. Should be looked at together.
pub const DECODING_FAILED_ERROR_ENCODED: [u8; 4] = [255u8, 0, 0, 0];
pub const UNKNOWN_CALL_ERROR: DispatchError = DispatchError::Other("UnknownCall");
// TODO: see above.
pub const UNKNOWN_CALL_ERROR_ENCODED: [u8; 4] = [254u8, 0, 0, 0];

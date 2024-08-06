use crate::constants::UNKNOWN_CALL_ERROR;
use sp_runtime::DispatchError;

/// Function identifiers used in the Pop API.
///
/// The `FuncId` specifies the available functions that can be called through the Pop API. Each
/// variant corresponds to a specific functionality provided by the API, facilitating the
/// interaction between smart contracts and the runtime.
///
/// - `Dispatch`: Represents a function call to dispatch a runtime call.
/// - `ReadState`: Represents a function call to read the state from the runtime.
/// - `SendXcm`: Represents a function call to send an XCM message.
#[derive(Debug)]
pub enum FuncId {
	Dispatch,
	ReadState,
}

impl TryFrom<u8> for FuncId {
	type Error = DispatchError;

	/// Attempts to convert a `u8` value to its corresponding `FuncId` variant.
	///
	/// If the `u8` value does not match any known function identifier, it returns a
	/// `DispatchError::Other` indicating an unknown function ID.
	fn try_from(func_id: u8) -> Result<Self, Self::Error> {
		let id = match func_id {
			0 => Self::Dispatch,
			1 => Self::ReadState,
			_ => {
				return Err(UNKNOWN_CALL_ERROR);
			},
		};
		Ok(id)
	}
}

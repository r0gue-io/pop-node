#![cfg_attr(not(feature = "std"), no_std, no_main)]

use constants::DECODING_FAILED;
use ink::env::chain_extension::FromStatusCode;
pub use v0::assets;

pub mod primitives;
pub mod v0;

/// A result type used by the API, with the `StatusCode` as the error type.
pub type Result<T> = core::result::Result<T, StatusCode>;

mod constants {
	// Errors:
	pub(crate) const DECODING_FAILED: u32 = 255;
	pub(crate) const _MODULE_ERROR: u8 = 3;

	// Function IDs:
	pub(crate) const DISPATCH: u8 = 0;
	pub(crate) const READ_STATE: u8 = 1;

	// Modules:
	pub(crate) const ASSETS: u8 = 52;
	pub(crate) const BALANCES: u8 = 10;
	pub(crate) const FUNGIBLES: u8 = 150;
}

/// Represents a status code returned by the runtime.
///
/// `StatusCode` encapsulates a `u32` value that indicates the status of an operation performed
/// by the runtime. It helps to communicate the success or failure of a Pop API call to the contract,
/// providing a standardized way to handle errors.
///
/// This status code can be used to determine if an operation succeeded or if it encountered
/// an error. A `StatusCode` of `0` typically indicates success, while any other value represents
/// an error.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct StatusCode(pub u32);

impl From<u32> for StatusCode {
	/// Converts a `u32` into a `StatusCode`.
	fn from(value: u32) -> Self {
		StatusCode(value)
	}
}

impl FromStatusCode for StatusCode {
	/// Converts a `u32` status code to a `Result`.
	///
	/// `Ok(())` if the status code is `0` and `Err(StatusCode(status_code))` for any other status
	/// code.
	fn from_status_code(status_code: u32) -> Result<()> {
		match status_code {
			0 => Ok(()),
			_ => Err(StatusCode(status_code)),
		}
	}
}

impl From<ink::scale::Error> for StatusCode {
	/// Converts a scale decoding error into a `StatusCode` indicating a decoding failure.
	fn from(_: ink::scale::Error) -> Self {
		StatusCode(DECODING_FAILED)
	}
}

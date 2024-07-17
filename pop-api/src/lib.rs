//! The `pop-api` crate provides an API for smart contracts to interact with the Pop Network runtime.
//!
//! This crate abstracts away complexities to deliver a streamlined developer experience while supporting
//! multiple API versions to ensure backward compatibility. It is designed with a focus on stability,
//! future-proofing, and storage efficiency, allowing developers to easily integrate powerful runtime
//! features into their contracts without unnecessary overhead.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use constants::DECODING_FAILED;
use ink::env::chain_extension::{ChainExtensionMethod, FromStatusCode};
#[cfg(feature = "assets")]
pub use v0::assets;

/// Module providing primitives types.
pub mod primitives;
/// The first version of the API.
pub mod v0;

/// A result type used by the API, with the `StatusCode` as the error type.
pub type Result<T> = core::result::Result<T, StatusCode>;

mod constants {
	// Errors:
	pub(crate) const DECODING_FAILED: u32 = 255;
	// TODO: will be used in the future when the remaining fungibles features will be implemented.
	pub(crate) const _MODULE_ERROR: u8 = 3;

	// Function IDs:
	pub(crate) const DISPATCH: u8 = 0;
	pub(crate) const READ_STATE: u8 = 1;

	// Modules:
	pub(crate) const ASSETS: u8 = 52;
	pub(crate) const BALANCES: u8 = 10;
	pub(crate) const FUNGIBLES: u8 = 150;
}

// Helper method to build a dispatch call or a call to read state.
//
// Parameters:
// - 'version': The version of the chain extension.
// - 'function': The ID of the function.
// - 'module': The index of the runtime module.
// - 'dispatchable': The index of the module dispatchable functions.
fn build_extension_method(
	version: u8,
	function: u8,
	module: u8,
	dispatchable: u8,
) -> ChainExtensionMethod<(), (), (), false> {
	ChainExtensionMethod::build(u32::from_le_bytes([version, function, module, dispatchable]))
}

/// Represents a status code returned by the runtime.
///
/// `StatusCode` encapsulates a `u32` value that indicates the status of an operation performed by
/// the runtime. It helps to communicate the success or failure of a Pop API call to the contract,
/// providing a standardized way to handle errors.
///
/// This status code can be used to determine if an operation succeeded or if it encountered an
/// error. A `StatusCode` of `0` typically indicates success, while any other value represents an
/// error.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct StatusCode(pub u32);

pub(crate) const DECODING_FAILED: u32 = 255;

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

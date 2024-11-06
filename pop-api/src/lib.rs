//! The `pop-api` crate provides an API for smart contracts to interact with the Pop Network
//! runtime.
//!
//! This crate abstracts away complexities to deliver a streamlined developer experience while
//! supporting multiple API versions to ensure backward compatibility. It is designed with a focus
//! on stability, future-proofing, and storage efficiency, allowing developers to easily integrate
//! powerful runtime features into their contracts without unnecessary overhead.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use constants::DECODING_FAILED;
use ink::env::chain_extension::{ChainExtensionMethod, FromStatusCode};
pub use v0::*;

/// Module providing macros.
#[cfg(feature = "nonfungibles")]
pub mod macros;
/// Module providing primitives types.
pub mod primitives;
/// The first version of the API.
pub mod v0;

type ChainExtensionMethodApi = ChainExtensionMethod<(), (), (), false>;
/// The result type used by the API, with the `StatusCode` as the error type.
pub type Result<T> = core::result::Result<T, StatusCode>;

/// Represents a status code returned by the runtime.
///
/// `StatusCode` encapsulates a `u32` value that indicates the success or failure of a runtime call
/// via Pop API.
///
/// A `StatusCode` of `0` indicates success, while any other value represents an
/// error.
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

mod constants {
	// Error.
	pub(crate) const DECODING_FAILED: u32 = 255;

	// Function IDs.
	pub(crate) const DISPATCH: u8 = 0;
	pub(crate) const READ_STATE: u8 = 1;

	// Modules.
	#[cfg(feature = "fungibles")]
	pub(crate) const ASSETS: u8 = 52;
	#[cfg(feature = "fungibles")]
	pub(crate) const BALANCES: u8 = 10;
	#[cfg(feature = "fungibles")]
	pub(crate) const FUNGIBLES: u8 = 150;
	#[cfg(feature = "messaging")]
	pub(crate) const MESSAGING: u8 = 151;
	pub(crate) const INCENTIVES: u8 = 152;
	#[cfg(feature = "nonfungibles")]
	pub(crate) const NONFUNGIBLES: u8 = 154;
	#[cfg(feature = "sponsorships")]
	pub(crate) const SPONSORSHIPS: u8 = 153;
}

// Helper method to build a dispatch call or a call to read state.
//
// Parameters:
// - 'function': The ID of the function.
// - 'version': The version of the chain extension.
// - 'module': The index of the runtime module.
// - 'dispatchable': The index of the module dispatchable functions.
fn build_extension_method(
	function: u8,
	version: u8,
	module: u8,
	dispatchable: u8,
) -> ChainExtensionMethodApi {
	ChainExtensionMethod::build(u32::from_le_bytes([function, version, module, dispatchable]))
}

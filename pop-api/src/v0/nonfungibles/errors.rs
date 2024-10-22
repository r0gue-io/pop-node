//! A set of errors for use in smart contracts that interact with the nonfungibles api. This
//! includes errors compliant to standards.

use super::*;

/// The PSP34 error.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Psp34Error {
	/// Custom error type for cases if writer of traits added own restrictions
	Custom(String),
	/// Returned if owner approves self
	SelfApprove,
	/// Returned if the caller doesn't have allowance for transferring.
	NotApproved,
	/// Returned if the owner already own the token.
	TokenExists,
	/// Returned if the token doesn't exist
	TokenNotExists,
	/// Returned if safe transfer check fails
	SafeTransferCheckFailed(String),
}

impl From<StatusCode> for Psp34Error {
	/// Converts a `StatusCode` to a `PSP22Error`.
	fn from(value: StatusCode) -> Self {
		let encoded = value.0.to_le_bytes();
		match encoded {
			// TODO: Handle conversion.
			_ => Psp34Error::Custom(value.0.to_string()),
		}
	}
}

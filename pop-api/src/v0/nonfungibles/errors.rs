//! A set of errors for use in smart contracts that interact with the nonfungibles api. This
//! includes errors compliant to standards.

use ink::prelude::string::{String, ToString};

use super::*;

/// The PSP34 error.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Psp34Error {
	/// Custom error type for cases if writer of traits added own restrictions.
	Custom(String),
	/// Returned if owner approves self.
	SelfApprove,
	/// Returned if the caller doesn't have allowance for transferring.
	NotApproved,
	/// Returned if the owner already own the token.
	TokenExists,
	/// Returned if the token doesn't exist.
	TokenNotExists,
	/// Returned if safe transfer check fails.
	SafeTransferCheckFailed(String),
}

#[cfg(feature = "std")]
impl From<Psp34Error> for u32 {
	fn from(value: Psp34Error) -> u32 {
		match value {
			Psp34Error::NotApproved => u32::from_le_bytes([MODULE_ERROR, NFTS, 0, 0]),
			Psp34Error::TokenExists => u32::from_le_bytes([MODULE_ERROR, NFTS, 2, 0]),
			Psp34Error::TokenNotExists => u32::from_le_bytes([MODULE_ERROR, NFTS, 19, 0]),
			Psp34Error::Custom(value) => value.parse::<u32>().expect("Failed to parse"),
			_ => unimplemented!("Variant is not supported"),
		}
	}
}

impl From<StatusCode> for Psp34Error {
	/// Converts a `StatusCode` to a `PSP34Error`.
	fn from(value: StatusCode) -> Self {
		let encoded = value.0.to_le_bytes();
		match encoded {
			// NoPermission.
			[MODULE_ERROR, NFTS, 0, _] => Psp34Error::NotApproved,
			// AlreadyExists.
			[MODULE_ERROR, NFTS, 2, _] => Psp34Error::TokenExists,
			// ApprovalExpired.
			[MODULE_ERROR, NFTS, 3, _] => Psp34Error::NotApproved,
			// UnknownItem.
			[MODULE_ERROR, NFTS, 19, _] => Psp34Error::TokenNotExists,
			_ => Psp34Error::Custom(value.0.to_string()),
		}
	}
}

//! A set of errors for use in smart contracts that interact with the fungibles api. This includes
//! errors compliant to standards.

use ink::{
	prelude::string::{String, ToString},
	scale::{Decode, Encode},
};

use super::*;

/// Represents various errors related to fungible tokens.
///
/// The `FungiblesError` provides a detailed and specific set of error types that can occur when
/// interacting with fungible tokens. Each variant signifies a particular error
/// condition, facilitating precise error handling and debugging.
///
/// It is designed to be lightweight, including only the essential errors relevant to fungible token
/// operations. The `Other` variant serves as a catch-all for any unexpected errors. For more
/// detailed debugging, the `Other` variant can be converted into the richer `Error` type defined in
/// the primitives crate.
/// NOTE: The `FungiblesError` is WIP
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum FungiblesError {
	/// An unspecified or unknown error occurred.
	Other(StatusCode),
	/// The token is not live; either frozen or being destroyed.
	NotLive,
	/// Not enough allowance to fulfill a request is available.
	InsufficientAllowance,
	/// Not enough balance to fulfill a request is available.
	InsufficientBalance,
	/// The token ID is already taken.
	InUse,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
	/// The account to alter does not exist.
	NoAccount,
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given token ID is unknown.
	Unknown,
	/// No balance for creation of tokens or fees.
	// TODO: Originally `pallet_balances::Error::InsufficientBalance` but collides with the
	//  `InsufficientBalance` error that is used for `pallet_assets::Error::BalanceLow` to adhere
	//  to the standard. This deserves a second look.
	NoBalance,
}

impl From<StatusCode> for FungiblesError {
	/// Converts a `StatusCode` to a `FungiblesError`.
	///
	/// This conversion maps a `StatusCode`, returned by the runtime, to a more descriptive
	/// `FungiblesError`. This provides better context and understanding of the error, allowing
	/// developers to handle the most important errors effectively.
	fn from(value: StatusCode) -> Self {
		let encoded = value.0.to_le_bytes();
		match encoded {
			// Balances.
			[_, BALANCES, 2, _] => FungiblesError::NoBalance,
			// Assets.
			[_, ASSETS, 0, _] => FungiblesError::NoAccount,
			[_, ASSETS, 1, _] => FungiblesError::NoPermission,
			[_, ASSETS, 2, _] => FungiblesError::Unknown,
			[_, ASSETS, 3, _] => FungiblesError::InUse,
			[_, ASSETS, 5, _] => FungiblesError::MinBalanceZero,
			[_, ASSETS, 7, _] => FungiblesError::InsufficientAllowance,
			[_, ASSETS, 10, _] => FungiblesError::NotLive,
			_ => FungiblesError::Other(value),
		}
	}
}

/// The PSP22 error.
// TODO: Issue https://github.com/r0gue-io/pop-node/issues/298
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum PSP22Error {
	/// Custom error type for implementation-based errors.
	Custom(String),
	/// Returned when an account does not have enough tokens to complete the operation.
	InsufficientBalance,
	/// Returned if there is not enough allowance to complete the operation.
	InsufficientAllowance,
	/// Returned if recipient's address is zero.
	ZeroRecipientAddress,
	/// Returned if sender's address is zero.
	ZeroSenderAddress,
	/// Returned if a safe transfer check failed.
	SafeTransferCheckFailed(String),
}

#[cfg(feature = "std")]
impl From<PSP22Error> for u32 {
	fn from(value: PSP22Error) -> u32 {
		match value {
			PSP22Error::InsufficientBalance => u32::from_le_bytes([3, ASSETS, 0, 0]),
			PSP22Error::InsufficientAllowance => u32::from_le_bytes([3, ASSETS, 10, 0]),
			PSP22Error::Custom(value) => value.parse::<u32>().expect("Failed to parse"),
			_ => unimplemented!("Variant is not supported"),
		}
	}
}

impl From<StatusCode> for PSP22Error {
	/// Converts a `StatusCode` to a `PSP22Error`.
	fn from(value: StatusCode) -> Self {
		let encoded = value.0.to_le_bytes();
		match encoded {
			// BalanceLow.
			[3, ASSETS, 0, _] => PSP22Error::InsufficientBalance,
			// Unapproved.
			[3, ASSETS, 10, _] => PSP22Error::InsufficientAllowance,
			// Custom error with status code.
			_ => PSP22Error::Custom(value.0.to_string()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		constants::{ASSETS, BALANCES},
		primitives::{
			ArithmeticError::*,
			Error::{self, *},
			TokenError::*,
			TransactionalError::*,
		},
		StatusCode,
	};

	fn error_into_status_code(error: Error) -> StatusCode {
		let mut encoded_error = error.encode();
		encoded_error.resize(4, 0);
		let value = u32::from_le_bytes(
			encoded_error.try_into().expect("qed, resized to 4 bytes line above"),
		);
		value.into()
	}

	fn into_error<T: From<StatusCode>>(error: Error) -> T {
		error_into_status_code(error).into()
	}

	// If we ever want to change the conversion from bytes to `u32`.
	#[test]
	fn status_code_vs_encoded() {
		assert_eq!(u32::decode(&mut &[3u8, 10, 2, 0][..]).unwrap(), 133635u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 0, 0][..]).unwrap(), 13315u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 1, 0][..]).unwrap(), 78851u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 2, 0][..]).unwrap(), 144387u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 3, 0][..]).unwrap(), 209923u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 5, 0][..]).unwrap(), 340995u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 7, 0][..]).unwrap(), 472067u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 10, 0][..]).unwrap(), 668675u32);
	}

	#[test]
	fn converting_status_code_into_fungibles_error_works() {
		let other_errors = vec![
			Other,
			CannotLookup,
			BadOrigin,
			// `ModuleError` other than assets module.
			Module { index: 2, error: [5, 0] },
			ConsumerRemaining,
			NoProviders,
			TooManyConsumers,
			Token(OnlyProvider),
			Arithmetic(Overflow),
			Transactional(NoLayer),
			Exhausted,
			Corruption,
			Unavailable,
			RootNotAllowed,
			Unknown { dispatch_error_index: 5, error_index: 5, error: 1 },
			DecodingFailed,
		];
		for error in other_errors {
			let status_code: StatusCode = error_into_status_code(error);
			let fungibles_error: FungiblesError = status_code.into();
			assert_eq!(fungibles_error, FungiblesError::Other(status_code))
		}

		assert_eq!(
			into_error::<FungiblesError>(Module { index: BALANCES, error: [2, 0] }),
			FungiblesError::NoBalance
		);
		assert_eq!(
			into_error::<FungiblesError>(Module { index: ASSETS, error: [0, 0] }),
			FungiblesError::NoAccount
		);
		assert_eq!(
			into_error::<FungiblesError>(Module { index: ASSETS, error: [1, 0] }),
			FungiblesError::NoPermission
		);
		assert_eq!(
			into_error::<FungiblesError>(Module { index: ASSETS, error: [2, 0] }),
			FungiblesError::Unknown
		);
		assert_eq!(
			into_error::<FungiblesError>(Module { index: ASSETS, error: [3, 0] }),
			FungiblesError::InUse
		);
		assert_eq!(
			into_error::<FungiblesError>(Module { index: ASSETS, error: [5, 0] }),
			FungiblesError::MinBalanceZero
		);
		assert_eq!(
			into_error::<FungiblesError>(Module { index: ASSETS, error: [7, 0] }),
			FungiblesError::InsufficientAllowance
		);
		assert_eq!(
			into_error::<FungiblesError>(Module { index: ASSETS, error: [10, 0] }),
			FungiblesError::NotLive
		);
	}

	#[test]
	fn converting_status_code_into_psp22_error_works() {
		let other_errors = vec![
			Other,
			CannotLookup,
			BadOrigin,
			// `ModuleError` other than assets module.
			Module { index: 2, error: [5, 0] },
			ConsumerRemaining,
			NoProviders,
			TooManyConsumers,
			Token(OnlyProvider),
			Arithmetic(Overflow),
			Transactional(NoLayer),
			Exhausted,
			Corruption,
			Unavailable,
			RootNotAllowed,
			Unknown { dispatch_error_index: 5, error_index: 5, error: 1 },
			DecodingFailed,
		];
		for error in other_errors {
			let status_code: StatusCode = error_into_status_code(error);
			let fungibles_error: PSP22Error = status_code.into();
			assert_eq!(fungibles_error, PSP22Error::Custom(status_code.0.to_string()))
		}

		assert_eq!(
			into_error::<PSP22Error>(Module { index: ASSETS, error: [0, 0] }),
			PSP22Error::InsufficientBalance
		);
		assert_eq!(
			into_error::<PSP22Error>(Module { index: ASSETS, error: [10, 0] }),
			PSP22Error::InsufficientAllowance
		);
		assert_eq!(
			into_error::<PSP22Error>(Module { index: ASSETS, error: [3, 0] }),
			PSP22Error::Custom(String::from("Unknown"))
		);
	}
}

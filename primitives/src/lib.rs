//! The `pop-primitives` crate provides types used by other crates.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use codec::{Decode, Encode};
#[cfg(test)]
use enum_iterator::Sequence;
#[cfg(feature = "std")]
use scale_info::TypeInfo;
#[cfg(feature = "runtime")]
use sp_runtime::{DispatchError, ModuleError};
pub use v0::*;

/// The identifier of a token.
pub type TokenId = u32;

/// The first version of primitives' types.
pub mod v0 {
	pub use error::*;

	use super::*;

	mod error {
		use super::*;

		/// Reason why a call failed.
		#[derive(Encode, Decode, Debug)]
		#[cfg_attr(feature = "std", derive(TypeInfo, Eq, PartialEq, Clone))]
		#[cfg_attr(test, derive(Sequence))]
		#[repr(u8)]
		#[allow(clippy::unnecessary_cast)]
		pub enum Error {
			/// Some error occurred.
			Other = 0,
			/// Failed to look up some data.
			CannotLookup = 1,
			/// A bad origin.
			BadOrigin = 2,
			/// A custom error in a module.
			Module {
				/// The pallet index.
				index: u8,
				/// The error within the pallet.
				// Supports a single level of nested error only, due to status code type size
				// constraints.
				error: [u8; 2],
			} = 3,
			/// At least one consumer is remaining so the account cannot be destroyed.
			ConsumerRemaining = 4,
			/// There are no providers so the account cannot be created.
			NoProviders = 5,
			/// There are too many consumers so the account cannot be created.
			TooManyConsumers = 6,
			/// An error to do with tokens.
			Token(TokenError) = 7,
			/// An arithmetic error.
			Arithmetic(ArithmeticError) = 8,
			/// The number of transactional layers has been reached, or we are not in a
			/// transactional layer.
			Transactional(TransactionalError) = 9,
			/// Resources exhausted, e.g. attempt to read/write data which is too large to
			/// manipulate.
			Exhausted = 10,
			/// The state is corrupt; this is generally not going to fix itself.
			Corruption = 11,
			/// Some resource (e.g. a preimage) is unavailable right now. This might fix itself
			/// later.
			Unavailable = 12,
			/// Root origin is not allowed.
			RootNotAllowed = 13,
			/// Decoding failed.
			DecodingFailed = 254,
			/// An unknown error occurred. This variant captures any unexpected errors that the
			/// contract cannot specifically handle. It is useful for cases where there are
			/// breaking changes in the runtime or when an error falls outside the predefined
			/// categories.
			Unknown {
				/// The index within the `DispatchError`.
				dispatch_error_index: u8,
				/// The index within the `DispatchError` variant (e.g. a `TokenError`).
				error_index: u8,
				/// The specific error code or sub-index, providing additional context (e.g.
				/// `error` in `ModuleError`).
				error: u8,
			} = 255,
		}

		impl From<u32> for Error {
			/// Converts a `u32` status code into an `Error`.
			///
			/// This conversion maps a raw status code returned by the runtime into the more
			/// descriptive `Error` enum variant, providing better context and understanding of the
			/// error.
			fn from(value: u32) -> Self {
				let encoded = value.to_le_bytes();
				Error::decode(&mut &encoded[..]).unwrap_or(Error::DecodingFailed)
			}
		}

		impl From<Error> for u32 {
			fn from(value: Error) -> Self {
				let mut encoded_error = value.encode();
				// Resize the encoded value to 4 bytes in order to decode the value into a u32 (4
				// bytes).
				encoded_error.resize(4, 0);
				u32::from_le_bytes(
					encoded_error.try_into().expect("qed, resized to 4 bytes line above"),
				)
			}
		}

		#[cfg(feature = "runtime")]
		impl From<DispatchError> for Error {
			fn from(error: DispatchError) -> Self {
				use sp_runtime::{
					ArithmeticError::*, DispatchError::*, TokenError::*, TransactionalError::*,
				};
				// Mappings exist here to avoid taking a dependency of sp_runtime on pop-primitives
				match error {
					Other(_message) => {
						// Note: lossy conversion: message not used due to returned contract status
						// code size limitation
						Error::Other
					},
					CannotLookup => Error::CannotLookup,
					BadOrigin => Error::BadOrigin,
					Module(error) => {
						// Note: message not used
						let ModuleError { index, error, message: _message } = error;
						Error::Module { index, error: [error[0], error[1]] }
					},
					ConsumerRemaining => Error::ConsumerRemaining,
					NoProviders => Error::NoProviders,
					TooManyConsumers => Error::TooManyConsumers,
					Token(error) => Error::Token(match error {
						FundsUnavailable => TokenError::FundsUnavailable,
						OnlyProvider => TokenError::OnlyProvider,
						BelowMinimum => TokenError::BelowMinimum,
						CannotCreate => TokenError::CannotCreate,
						UnknownAsset => TokenError::UnknownAsset,
						Frozen => TokenError::Frozen,
						Unsupported => TokenError::Unsupported,
						CannotCreateHold => TokenError::CannotCreateHold,
						NotExpendable => TokenError::NotExpendable,
						Blocked => TokenError::Blocked,
					}),
					Arithmetic(error) => Error::Arithmetic(match error {
						Underflow => ArithmeticError::Underflow,
						Overflow => ArithmeticError::Overflow,
						DivisionByZero => ArithmeticError::DivisionByZero,
					}),
					Transactional(error) => Error::Transactional(match error {
						LimitReached => TransactionalError::LimitReached,
						NoLayer => TransactionalError::NoLayer,
					}),
					Exhausted => Error::Exhausted,
					Corruption => Error::Corruption,
					Unavailable => Error::Unavailable,
					RootNotAllowed => Error::RootNotAllowed,
				}
			}
		}

		/// Description of what went wrong when trying to complete an operation on a token.
		#[derive(Encode, Decode, Debug)]
		#[cfg_attr(test, derive(Sequence))]
		#[cfg_attr(feature = "std", derive(TypeInfo, Eq, PartialEq, Clone))]
		pub enum TokenError {
			/// Funds are unavailable.
			FundsUnavailable,
			/// Some part of the balance gives the only provider reference to the account and thus
			/// cannot be (re)moved.
			OnlyProvider,
			/// Account cannot exist with the funds that would be given.
			BelowMinimum,
			/// Account cannot be created.
			CannotCreate,
			/// The asset in question is unknown.
			UnknownAsset,
			/// Funds exist but are frozen.
			Frozen,
			/// Operation is not supported by the asset.
			Unsupported,
			/// Account cannot be created for a held balance.
			CannotCreateHold,
			/// Withdrawal would cause unwanted loss of account.
			NotExpendable,
			/// Account cannot receive the assets.
			Blocked,
		}

		/// Arithmetic errors.
		#[derive(Encode, Decode, Debug)]
		#[cfg_attr(test, derive(Sequence))]
		#[cfg_attr(feature = "std", derive(TypeInfo, Eq, PartialEq, Clone))]
		pub enum ArithmeticError {
			/// Underflow.
			Underflow,
			/// Overflow.
			Overflow,
			/// Division by zero.
			DivisionByZero,
		}

		/// Errors related to transactional storage layers.
		#[derive(Encode, Decode, Debug)]
		#[cfg_attr(test, derive(Sequence))]
		#[cfg_attr(feature = "std", derive(TypeInfo, Eq, PartialEq, Clone))]
		pub enum TransactionalError {
			/// Too many transactional layers have been spawned.
			LimitReached,
			/// A transactional layer was expected, but does not exist.
			NoLayer,
		}
	}

	#[cfg(test)]
	mod tests {
		use enum_iterator::all;

		use super::{Error::*, *};

		// Conversion method for `Error` to `u32`.
		fn convert_error_into_u32(error: &Error) -> u32 {
			let mut encoded_error = error.encode();
			encoded_error.resize(4, 0);
			u32::from_le_bytes(
				encoded_error.try_into().expect("qed, resized to 4 bytes line above"),
			)
		}

		#[test]
		fn test_error_u32_conversion_with_all_variants() {
			// Test conversion for all Error variants
			all::<Error>().collect::<Vec<_>>().into_iter().for_each(|error| {
				let status_code = u32::from(error.clone());
				let expected = convert_error_into_u32(&error);
				assert_eq!(status_code, expected);
				let decoded_error = Error::from(status_code);
				assert_eq!(decoded_error, error);
			});
		}

		#[test]
		fn test_invalid_u32_values_result_in_decoding_failed() {
			// U32 values that don't map to a valid Error.
			vec![111u32, 999u32, 1234u32].into_iter().for_each(|invalid_value| {
				let error: Error = invalid_value.into();
				assert_eq!(error, DecodingFailed,);
			});
		}

		// Compare all the different `DispatchError` variants with the expected `Error`.
		#[test]
		#[cfg(feature = "runtime")]
		fn from_dispatch_error_to_error_works() {
			use sp_runtime::DispatchError::*;
			let test_cases = vec![
				(Other(""), (Error::Other)),
				(Other("UnknownCall"), Error::Other),
				(Other("DecodingFailed"), Error::Other),
				(Other("Random"), (Error::Other)),
				(CannotLookup, Error::CannotLookup),
				(BadOrigin, Error::BadOrigin),
				(
					Module(ModuleError { index: 1, error: [2, 0, 0, 0], message: Some("hallo") }),
					Error::Module { index: 1, error: [2, 0] },
				),
				(
					Module(ModuleError { index: 1, error: [2, 2, 0, 0], message: Some("hallo") }),
					Error::Module { index: 1, error: [2, 2] },
				),
				(
					Module(ModuleError { index: 1, error: [2, 2, 2, 0], message: Some("hallo") }),
					Error::Module { index: 1, error: [2, 2] },
				),
				(
					Module(ModuleError { index: 1, error: [2, 2, 2, 4], message: Some("hallo") }),
					Error::Module { index: 1, error: [2, 2] },
				),
				(ConsumerRemaining, Error::ConsumerRemaining),
				(NoProviders, Error::NoProviders),
				(TooManyConsumers, Error::TooManyConsumers),
				(
					Token(sp_runtime::TokenError::BelowMinimum),
					Error::Token(TokenError::BelowMinimum),
				),
				(
					Arithmetic(sp_runtime::ArithmeticError::Overflow),
					Error::Arithmetic(ArithmeticError::Overflow),
				),
				(
					Transactional(sp_runtime::TransactionalError::LimitReached),
					Error::Transactional(TransactionalError::LimitReached),
				),
				(Exhausted, Error::Exhausted),
				(Corruption, Error::Corruption),
				(Unavailable, Error::Unavailable),
				(RootNotAllowed, Error::RootNotAllowed),
			];
			for (dispatch_error, expected) in test_cases {
				let error = Error::from(dispatch_error);
				assert_eq!(error, expected);
			}
		}
	}
}

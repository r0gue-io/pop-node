#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use bounded_collections::{BoundedBTreeMap, BoundedBTreeSet, BoundedVec};
use scale::{Decode, Encode, MaxEncodedLen};
#[cfg(feature = "std")]
use scale_info::TypeInfo;
pub use v0::error;

#[cfg(feature = "cross-chain")]
pub mod cross_chain;
pub mod storage_keys;

/// An opaque 32-byte cryptographic identifier.
#[derive(Encode, Decode, Debug, MaxEncodedLen, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
pub struct AccountId(pub [u8; 32]);

/// Identifier for the class of asset.
pub type AssetId = u32;

#[cfg(feature = "nfts")]
pub mod nfts {
	use bounded_collections::ConstU32;

	/// Id used for identifying non-fungible collections.
	pub type CollectionId = u32;
	/// Id used for identifying non-fungible items.
	pub type ItemId = u32;
	/// The maximum length of an attribute key.
	pub type KeyLimit = ConstU32<64>;
	/// The maximum approvals an item could have.
	pub type ApprovalsLimit = ConstU32<20>;
}

pub mod v0 {
	use super::*;
	pub mod error {
		use super::*;

		/// Reason why a Pop API function call failed.
		#[derive(Encode, Decode, Debug, Eq, PartialEq)]
		#[cfg_attr(feature = "std", derive(TypeInfo))]
		#[repr(u8)]
		pub enum Error {
			/// An unknown error occurred. This variant captures any unexpected errors that the
			/// contract cannot specifically handle. It is useful for cases where there are breaking
			/// changes in the runtime or when an error falls outside the predefined categories. The
			/// variant includes:
			///
			/// - `dispatch_error_index`: The index within the `DispatchError`.
			/// - `error_index`: The index within the `DispatchError` variant (e.g. a `TokenError`).
			/// - `error`: The specific error code or sub-index, providing additional context (e.g.
			///   `error` in `ModuleError`).
			Other { dispatch_error_index: u8, error_index: u8, error: u8 },
			/// Failed to lookup some data.
			CannotLookup,
			/// A bad origin.
			BadOrigin,
			/// A custom error in a module.
			///
			/// - `index`: The pallet index.
			/// - `error`: The error within the pallet.
			Module { index: u8, error: u8 },
			/// At least one consumer is remaining so the account cannot be destroyed.
			ConsumerRemaining,
			/// There are no providers so the account cannot be created.
			NoProviders,
			/// There are too many consumers so the account cannot be created.
			TooManyConsumers,
			/// An error to do with tokens.
			Token(TokenError),
			/// An arithmetic error.
			Arithmetic(ArithmeticError),
			/// The number of transactional layers has been reached, or we are not in a transactional
			/// layer.
			Transactional(TransactionalError),
			/// Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
			Exhausted,
			/// The state is corrupt; this is generally not going to fix itself.
			Corruption,
			/// Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
			Unavailable,
			/// Root origin is not allowed.
			RootNotAllowed,
			/// Unknown function called.
			UnknownFunctionCall = 254,
			/// Decoding failed.
			DecodingFailed = 255,
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

		/// Description of what went wrong when trying to complete an operation on a token.
		#[derive(Encode, Decode, Clone, Debug, MaxEncodedLen, Eq, PartialEq, Ord, PartialOrd)]
		#[cfg_attr(feature = "std", derive(TypeInfo))]
		pub enum TokenError {
			/// Funds are unavailable.
			FundsUnavailable,
			/// Some part of the balance gives the only provider reference to the account and thus cannot
			/// be (re)moved.
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
		#[derive(Encode, Decode, Debug, Eq, PartialEq)]
		#[cfg_attr(feature = "std", derive(TypeInfo))]
		pub enum ArithmeticError {
			/// Underflow.
			Underflow,
			/// Overflow.
			Overflow,
			/// Division by zero.
			DivisionByZero,
		}

		/// Errors related to transactional storage layers.
		#[derive(Encode, Decode, Debug, Eq, PartialEq)]
		#[cfg_attr(feature = "std", derive(TypeInfo))]
		pub enum TransactionalError {
			/// Too many transactional layers have been spawned.
			LimitReached,
			/// A transactional layer was expected, but does not exist.
			NoLayer,
		}
	}
}

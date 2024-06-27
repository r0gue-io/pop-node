#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use bounded_collections::{BoundedBTreeMap, BoundedBTreeSet, BoundedVec};
use scale::{Decode, Encode, MaxEncodedLen};
use sp_std::vec::Vec;
#[cfg(feature = "std")]
use {scale_decode::DecodeAsType, scale_encode::EncodeAsType, scale_info::TypeInfo};

#[cfg(feature = "cross-chain")]
pub mod cross_chain;
pub mod storage_keys;

pub mod v0 {
	use super::*;
	#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	#[repr(u8)]
	pub enum Error {
		/// Some unknown error occurred. Go to the Pop API docs section `Pop API error`.
		Other {
			// Index within the `DispatchError`
			dispatch_error_index: u8,
			// Index within the `DispatchError` variant.
			error_index: u8,
			// Index for further nesting, e.g. pallet error.
			error: u8,
		} = 0,
		/// Failed to lookup some data.
		CannotLookup = 1,
		/// A bad origin.
		BadOrigin = 2,
		/// A custom error in a module.
		Module {
			index: u8,
			error: u8,
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
		/// The number of transactional layers has been reached, or we are not in a transactional
		/// layer.
		Transactional(TransactionalError) = 9,
		/// Resources exhausted, e.g. attempt to read/write data which is too large to manipulate.
		Exhausted = 10,
		/// The state is corrupt; this is generally not going to fix itself.
		Corruption = 11,
		/// Some resource (e.g. a preimage) is unavailable right now. This might fix itself later.
		Unavailable = 12,
		/// Root origin is not allowed.
		RootNotAllowed = 13,
		NewError,
		DecodingFailed = 255,
	}

	#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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

	#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub enum ArithmeticError {
		/// Underflow.
		Underflow,
		/// Overflow.
		Overflow,
		/// Division by zero.
		DivisionByZero,
	}

	#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
	#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
	pub enum TransactionalError {
		/// Too many transactional layers have been spawned.
		LimitReached,
		/// A transactional layer was expected, but does not exist.
		NoLayer,
	}

	// If an unknown variant of the `DispatchError` is detected the error needs to be converted
	// into the encoded value of `Error::Other`. This conversion is performed by shifting the bytes one
	// position forward (discarding the last byte as it is not used) and setting the first byte to the
	// encoded value of `Other` (0u8). This ensures the error is correctly categorized as an `Other`
	// variant which provides all the necessary information to debug which error occurred in the runtime.
	//
	// Byte layout explanation:
	// - Byte 0: index of the variant within `Error`
	// - Byte 1:
	//   - Must be zero for `UNIT_ERRORS`.
	//   - Represents the nested error in `SINGLE_NESTED_ERRORS`.
	//   - Represents the first level of nesting in `DOUBLE_NESTED_ERRORS`.
	// - Byte 2:
	//   - Represents the second level of nesting in `DOUBLE_NESTED_ERRORS`.
	// - Byte 3:
	//   - Unused or represents further nested information.
	//
	// This mechanism ensures backward compatibility by correctly categorizing any unknown errors
	// into the `Other` variant, thus preventing issues caused by breaking changes.
	pub fn unknown_errors(encoded_error: &[u8; 4]) -> bool {
		match encoded_error[0] {
			code if UNIT_ERRORS.contains(&code) => nested_errors(&encoded_error[1..], None),
			// Single nested errors with a limit in their nesting.
			//
			// `TokenError`: has ten variants - translated to a limit of nine.
			7 => nested_errors(&encoded_error[1..], Some(9)),
			// `ArithmeticError`: has 3 variants - translated to a limit of two.
			8 => nested_errors(&encoded_error[1..], Some(2)),
			// `TransactionalError`: has 2 variants - translated to a limit of one.
			9 => nested_errors(&encoded_error[1..], Some(1)),
			code if DOUBLE_NESTED_ERRORS.contains(&code) => {
				nested_errors(&encoded_error[3..], None)
			},
			_ => true,
		}
	}

	// Checks for unknown nested errors within the `DispatchError`.
	// - For single nested errors with a limit, it verifies if the nested value exceeds the limit.
	// - For other nested errors, it checks if any subsequent bytes are non-zero.
	//
	// `nested_error` - The slice of bytes representing the nested error.
	// `limit` - An optional limit for single nested errors.
	fn nested_errors(nested_error: &[u8], limit: Option<u8>) -> bool {
		match limit {
			Some(l) => nested_error[0] > l || nested_error[1..].iter().any(|&x| x != 0u8),
			None => nested_error.iter().any(|&x| x != 0u8),
		}
	}

	// Unit `Error` variants.
	// (variant: index):
	// - CannotLookup: 1,
	// - BadOrigin: 2,
	// - ConsumerRemaining: 4,
	// - NoProviders: 5,
	// - TooManyConsumers: 6,
	// - Exhausted: 10,
	// - Corruption: 11,
	// - Unavailable: 12,
	// - RootNotAllowed: 13,
	// - DecodingFailed: 255,
	const UNIT_ERRORS: [u8; 10] = [1, 2, 4, 5, 6, 10, 11, 12, 13, 255];

	#[cfg(test)]
	const SINGLE_NESTED_ERRORS: [u8; 3] = [7, 8, 9];

	// Double nested `Error` variants
	// (variant: index):
	// - Module: 3,
	const DOUBLE_NESTED_ERRORS: [u8; 1] = [3];
}

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(TypeInfo, DecodeAsType, EncodeAsType))]
pub struct AccountId(pub [u8; 32]);

#[derive(Encode, Decode, Debug)]
// #[cfg_attr(feature = "std", derive(Hash))]
pub enum MultiAddress<AccountIndex> {
	/// It's an account ID (pubkey).
	Id(AccountId),
	/// It's an account index.
	Index(#[codec(compact)] AccountIndex),
	/// It's some arbitrary raw bytes.
	Raw(Vec<u8>),
	/// It's a 32 byte representation.
	Address32([u8; 32]),
	/// It's a 20 byte representation.
	Address20([u8; 20]),
}

impl<AccountIndex> From<AccountId> for MultiAddress<AccountIndex> {
	fn from(a: AccountId) -> Self {
		Self::Address32(a.0)
	}
}

// Identifier for the class of asset.
pub type AssetId = u32;

#[cfg(feature = "nfts")]
pub mod nfts {
	use bounded_collections::ConstU32;

	// Id used for identifying non-fungible collections.
	pub type CollectionId = u32;
	// Id used for identifying non-fungible items.
	pub type ItemId = u32;
	/// The maximum length of an attribute key.
	pub type KeyLimit = ConstU32<64>;
	/// The maximum approvals an item could have.
	pub type ApprovalsLimit = ConstU32<20>;
}

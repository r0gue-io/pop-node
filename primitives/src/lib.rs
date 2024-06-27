#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use bounded_collections::{BoundedBTreeMap, BoundedBTreeSet, BoundedVec};
use scale::{Decode, Encode, MaxEncodedLen};
use sp_std::vec::Vec;
#[cfg(feature = "std")]
use {scale_decode::DecodeAsType, scale_encode::EncodeAsType, scale_info::TypeInfo};

#[cfg(feature = "cross-chain")]
pub mod cross_chain;
pub mod storage_keys;

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

#[derive(Encode, Decode, Debug, MaxEncodedLen, Eq, PartialEq, Ord, PartialOrd, Clone)]
#[cfg_attr(feature = "std", derive(TypeInfo, DecodeAsType, EncodeAsType, Hash))]
pub struct AccountId(pub [u8; 32]);

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "std", derive(Hash))]
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
		Self::Id(a)
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

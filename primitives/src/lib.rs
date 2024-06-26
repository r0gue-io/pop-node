#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use bounded_collections::{BoundedBTreeMap, BoundedBTreeSet, BoundedVec};
use scale::{Decode, Encode, MaxEncodedLen};
use sp_std::vec::Vec;
#[cfg(feature = "std")]
use {scale_decode::DecodeAsType, scale_encode::EncodeAsType, scale_info::TypeInfo};

#[cfg(feature = "cross-chain")]
pub mod cross_chain;
pub mod storage_keys;

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

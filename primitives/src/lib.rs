#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use bounded_collections::{BoundedBTreeMap, BoundedBTreeSet, BoundedVec, ConstU32};
use scale::{Decode, Encode, MaxEncodedLen};
#[cfg(feature = "std")]
use {scale_decode::DecodeAsType, scale_encode::EncodeAsType, scale_info::TypeInfo};

pub mod cross_chain;
pub mod storage_keys;

#[derive(Encode, Decode, Debug, MaxEncodedLen, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "std", derive(TypeInfo, DecodeAsType, EncodeAsType))]
pub struct AccountId(pub [u8; 32]);

// Identifier for the class of asset.
pub type AssetId = u32;
// Id used for identifying non-fungible collections.
pub type CollectionId = u32;
// Id used for identifying non-fungible items.
pub type ItemId = u32;
/// The maximum length of an attribute key.
pub type KeyLimit = ConstU32<64>;
/// The maximum approvals an item could have.
pub type ApprovalsLimit = ConstU32<20>;

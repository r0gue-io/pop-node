#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use bounded_collections::{BoundedBTreeMap, BoundedBTreeSet, BoundedVec, ConstU32};

pub mod cross_chain;
pub mod storage_keys;

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

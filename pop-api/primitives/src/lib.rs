#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use bounded_collections::{BoundedBTreeMap, BoundedBTreeSet, BoundedVec, ConstU32};
//use scale::{Decode, Encode, MaxEncodedLen};

pub mod cross_chain;
pub mod storage_keys;

// /// Some way of identifying an account on the chain.
// #[derive(Encode, Decode, Debug, MaxEncodedLen)]
// pub struct AccountId([u8; 32]);
// Id used for identifying non-fungible collections.
pub type CollectionId = u32;
// Id used for identifying non-fungible items.
pub type ItemId = u32;
/// The maximum length of an attribute key.
pub type KeyLimit = ConstU32<64>;
/// The maximum approvals an item could have.
pub type ApprovalsLimit = ConstU32<20>;

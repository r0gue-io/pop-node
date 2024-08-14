//! The `pop-primitives` crate provides types used by both the Pop Network runtime and the `pop-api`.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Identifier for the class of asset.
pub type AssetId = u32;

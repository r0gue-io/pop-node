use alloc::string::String;

pub(super) use pallet_revive::precompiles::alloy::primitives::U256;
use pallet_revive::precompiles::RuntimeCosts;

// use weights::WeightInfo;
use super::*;

/// APIs for nonfungible tokens conforming to the ERC721 standard.
pub mod erc721;

/// The first version of the Nonfungibles API.
pub mod v0;

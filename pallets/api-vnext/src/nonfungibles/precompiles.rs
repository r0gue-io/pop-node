use alloc::string::String;

pub(super) use pallet_revive::precompiles::alloy::{
	primitives::{
		ruint::{UintTryFrom, UintTryTo},
		U256,
	},
	sol_types::SolCall,
};

// use weights::WeightInfo;
use super::*;

/// APIs for fungible tokens conforming to the ERC721 standard.
pub mod erc721;

/// The first version of the Nonfungibles API.
pub mod v0;

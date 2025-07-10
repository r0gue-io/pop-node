use alloc::string::String;

pub(super) use pallet_revive::precompiles::alloy::{
	primitives::{Address, U256},
	sol_types::SolCall,
};
use pallet_revive::precompiles::{Error, RuntimeCosts};
use weights::WeightInfo;

use super::*;

/// APIs for fungible tokens conforming to the ERC20 standard.
pub mod erc20;

/// The first version of the Fungibles API.
pub mod v0;

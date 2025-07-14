use ink::{
	contract_ref,
	prelude::{string::String, vec::Vec},
	U256,
};
use sol::Sol;
pub use v0::*;

use super::*;

/// APIs for fungible tokens conforming to the ERC20 standard.
pub mod erc20;

/// The first version of the Fungibles API.
pub mod v0;

pub type TokenId = u32;

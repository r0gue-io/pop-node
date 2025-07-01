pub use v0::*;

use super::{
	contract_ref, prefixed_address, Address, Pop, Sol, SolAddress, SolEncode, SolError, SolType,
	SolTypeEncode, String, TokenId, Uint, Vec, U256,
};
use crate::ensure;

/// The first version of the Erc20 API.
pub mod v0;

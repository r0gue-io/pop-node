use ink::scale::Encode;
pub use v0::*;

use super::{
	contract_ref, fixed_address, BlockNumber, DynBytes, MessageId, MessageStatus, Pop, Sol, Vec,
	Weight,
};

/// The first version of the XCM API.
pub mod v0;

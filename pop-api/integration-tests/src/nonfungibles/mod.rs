use pop_api::nonfungibles::{
	events::{Approval, AttributeSet, Transfer},
	types::*,
};
use pop_primitives::{ArithmeticError::*, Error, Error::*, TokenError::*};
use utils::*;

use super::*;

mod utils;

const CONTRACT: &str = "contracts/fungibles/target/ink/fungibles.wasm";

pub use ink::{abi::Sol, SolEncode};

/// Reverts the current contract execution, rolling back any changes and returning the specified
/// `error`.
// Helper until Solidity support added for Rust errors for automatic reversion based on returning an
// error.
pub fn revert(error: &impl for<'a> SolEncode<'a>) -> ! {
	use ink::env::{return_value_solidity, ReturnFlags};
	return_value_solidity(ReturnFlags::REVERT, error)
}

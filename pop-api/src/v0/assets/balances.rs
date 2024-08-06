use crate::{
	constants::BALANCES,
	primitives::{AccountId, Balance},
	Result, StatusCode,
};
use constants::*;
use ink::env::chain_extension::ChainExtensionMethod;

/// Helper method to build a dispatch call `ChainExtensionMethod` for native balances.
///
/// Parameters:
/// - 'dispatchable': The index of the module dispatchable functions
fn build_dispatch(dispatchable: u8) -> ChainExtensionMethod<(), (), (), false> {
	crate::v0::build_dispatch(BALANCES, dispatchable)
}

/// Helper method to build a dispatch call `ChainExtensionMethod` for native balances.
///
/// Parameters:
/// - 'state_query': The index of the runtime state query
fn build_read_state(state_query: u8) -> ChainExtensionMethod<(), (), (), false> {
	crate::v0::build_read_state(BALANCES, state_query)
}

mod constants {
	/// - total_issuance
	pub const TOTAL_ISSUANCE: u8 = 0;
	/// - transfer_keep_alive
	pub(super) const TRANSFER_KEEP_ALIVE: u8 = 3;
}

/// Returns the total supply for a native token
///
/// # Returns
/// The total supply of the token, or an error if the operation fails.
#[inline]
pub fn total_issuance() -> Result<Balance> {
	build_read_state(TOTAL_ISSUANCE)
		.output::<Result<Balance>, true>()
		.handle_error_code::<StatusCode>()
		.call(&())
}

/// Transfers `value` amount of tokens from the caller's account to account `to`, with additional
/// `data` in unspecified format.
///
/// # Arguments
/// * `to` - The recipient account.
/// * `value` - The number of native tokens to transfer.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the transfer fails.
#[inline]
pub fn transfer_keep_alive(target: AccountId, amount: Balance) -> Result<()> {
	build_dispatch(TRANSFER_KEEP_ALIVE)
		.input::<(AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(target, amount))
}

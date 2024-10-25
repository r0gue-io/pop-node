use ink::env::DefaultEnvironment;

use crate::{
	constants::INCENTIVES,
	primitives::{AccountId, Balance, Era},
	ChainExtensionMethodApi, Result, StatusCode,
};

// Dispatchables
pub(super) const REGISTER: u8 = 0;
pub(super) const CLAIM: u8 = 1;
pub(super) const DEPOSIT: u8 = 2;

fn build_dispatch(dispatchable: u8) -> ChainExtensionMethodApi {
	crate::v0::build_dispatch(INCENTIVES, dispatchable)
}

/// Register to receive rewards.
///
/// # Parameters
/// - `beneficiary`: The account that will be the beneficiary of the rewards.
#[inline]
pub fn register(beneficiary: AccountId) -> Result<()> {
	build_dispatch(REGISTER)
		.input::<AccountId>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&beneficiary)
}

/// Claims rewards for a given era.
///
/// Parameters:
/// - `era`: The era for which rewards are being claimed.
#[inline]
pub fn claim(era: Era) -> Result<()> {
	build_dispatch(CLAIM)
		.input::<(AccountId, Era)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(ink::env::account_id::<DefaultEnvironment>(), era))
}

/// Deposit funds into the reward pool.
///
/// Parameters:
/// - 'amount': Amount to be deposited.
#[inline]
pub fn deposit_funds(amount: Balance) -> Result<()> {
	build_dispatch(DEPOSIT)
		.input::<Balance>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&amount)
}

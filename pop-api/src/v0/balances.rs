use crate::{dispatch, primitives::MultiAddress, v0::RuntimeCall, AccountId, PopApiError};

type Result<T> = core::result::Result<T, PopApiError>;

pub fn transfer_keep_alive(
	dest: impl Into<MultiAddress<AccountId, ()>>,
	value: u128,
) -> Result<()> {
	Ok(dispatch(RuntimeCall::Balances(BalancesCall::TransferKeepAlive {
		dest: dest.into(),
		value,
	}))?)
}

#[derive(scale::Encode)]
#[allow(dead_code)]
pub enum BalancesCall {
	#[codec(index = 3)]
	TransferKeepAlive {
		dest: MultiAddress<AccountId, ()>,
		#[codec(compact)]
		value: u128,
	},
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum BalancesError {
	/// Vesting balance too high to send value.
	VestingBalance,
	/// Account liquidity restrictions prevent withdrawal.
	LiquidityRestrictions,
	/// Balance too low to send value.
	InsufficientBalance,
	/// Value too low to create account due to existential deposit.
	ExistentialDeposit,
	/// Transfer/payment would kill account.
	Expendability,
	/// A vesting schedule already exists for this account.
	ExistingVestingSchedule,
	/// Beneficiary account must pre-exist.
	DeadAccount,
	/// Number of named reserves exceed `MaxReserves`.
	TooManyReserves,
	/// Number of holds exceed `VariantCountOf<T::RuntimeHoldReason>`.
	TooManyHolds,
	/// Number of freezes exceed `MaxFreezes`.
	TooManyFreezes,
	/// The issuance cannot be modified since it is already deactivated.
	IssuanceDeactivated,
	/// The delta cannot be zero.
	DeltaZero,
	UnknownError(u8),
}

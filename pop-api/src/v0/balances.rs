use crate::{
	dispatch, primitives::MultiAddress, v0::RuntimeCall, AccountId, PopApiError,
	PopApiError::UnknownModuleStatusCode,
};

type Result<T> = core::result::Result<T, BalancesError>;

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
pub(crate) enum BalancesCall {
	#[codec(index = 3)]
	TransferKeepAlive {
		dest: MultiAddress<AccountId, ()>,
		#[codec(compact)]
		value: u128,
	},
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub(crate) enum BalancesError {
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
}

impl TryFrom<u32> for BalancesError {
	type Error = PopApiError;

	fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
		use BalancesError::*;
		match status_code {
			0 => Ok(VestingBalance),
			1 => Ok(LiquidityRestrictions),
			2 => Ok(InsufficientBalance),
			3 => Ok(ExistentialDeposit),
			4 => Ok(Expendability),
			5 => Ok(ExistingVestingSchedule),
			6 => Ok(DeadAccount),
			7 => Ok(TooManyReserves),
			8 => Ok(TooManyHolds),
			9 => Ok(TooManyFreezes),
			10 => Ok(IssuanceDeactivated),
			11 => Ok(DeltaZero),
			_ => Err(UnknownModuleStatusCode(status_code)),
		}
	}
}

impl From<PopApiError> for BalancesError {
	fn from(error: PopApiError) -> Self {
		match error {
			PopApiError::Balances(e) => e,
			_ => todo!(),
		}
	}
}

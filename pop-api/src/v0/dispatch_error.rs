use super::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub(crate) enum TokenError {
	/// Funds are unavailable.
	FundsUnavailable,
	/// Some part of the balance gives the only provider reference to the account and thus cannot
	/// be (re)moved.
	OnlyProvider,
	/// Account cannot exist with the funds that would be given.
	BelowMinimum,
	/// Account cannot be created.
	CannotCreate,
	/// The asset in question is unknown.
	UnknownAsset,
	/// Funds exist but are frozen.
	Frozen,
	/// Operation is not supported by the asset.
	Unsupported,
	/// Account cannot be created for a held balance.
	CannotCreateHold,
	/// Withdrawal would cause unwanted loss of account.
	NotExpendable,
	/// Account cannot receive the assets.
	Blocked,
}

impl TryFrom<u32> for TokenError {
	type Error = crate::PopApiError;

	fn try_from(status_code: u32) -> core::result::Result<Self, Self::Error> {
		use TokenError::*;
		match status_code {
			0 => Ok(FundsUnavailable),
			1 => Ok(OnlyProvider),
			2 => Ok(BelowMinimum),
			3 => Ok(CannotCreate),
			4 => Ok(UnknownAsset),
			5 => Ok(Frozen),
			6 => Ok(Unsupported),
			7 => Ok(CannotCreateHold),
			8 => Ok(NotExpendable),
			9 => Ok(Blocked),
			_ => todo!(),
		}
	}
}

impl From<PopApiError> for TokenError {
	fn from(error: PopApiError) -> Self {
		match error {
			PopApiError::TokenError(e) => e,
			_ => todo!(),
		}
	}
}

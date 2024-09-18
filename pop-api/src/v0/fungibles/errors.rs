use super::*;

/// Represents various errors related to fungible tokens.
///
/// The `FungiblesError` provides a detailed and specific set of error types that can occur when
/// interacting with fungible tokens. Each variant signifies a particular error
/// condition, facilitating precise error handling and debugging.
///
/// It is designed to be lightweight, including only the essential errors relevant to fungible token
/// operations. The `Other` variant serves as a catch-all for any unexpected errors. For more
/// detailed debugging, the `Other` variant can be converted into the richer `Error` type defined in
/// the primitives crate.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum FungiblesError {
	/// An unspecified or unknown error occurred.
	Other(StatusCode),
	/// The token is not live; either frozen or being destroyed.
	NotLive,
	/// Not enough allowance to fulfill a request is available.
	InsufficientAllowance,
	/// Not enough balance to fulfill a request is available.
	InsufficientBalance,
	/// The token ID is already taken.
	InUse,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
	/// The account to alter does not exist.
	NoAccount,
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given token ID is unknown.
	Unknown,
	/// No balance for creation of tokens or fees.
	// TODO: Originally `pallet_balances::Error::InsufficientBalance` but collides with the
	//  `InsufficientBalance` error that is used for `pallet_assets::Error::BalanceLow` to adhere
	//  to the standard. This deserves a second look.
	NoBalance,
}

impl From<StatusCode> for FungiblesError {
	/// Converts a `StatusCode` to a `FungiblesError`.
	///
	/// This conversion maps a `StatusCode`, returned by the runtime, to a more descriptive
	/// `FungiblesError`. This provides better context and understanding of the error, allowing
	/// developers to handle the most important errors effectively.
	fn from(value: StatusCode) -> Self {
		let encoded = value.0.to_le_bytes();
		match encoded {
			// Balances.
			[_, BALANCES, 2, _] => FungiblesError::NoBalance,
			// Assets.
			[_, ASSETS, 0, _] => FungiblesError::NoAccount,
			[_, ASSETS, 1, _] => FungiblesError::NoPermission,
			[_, ASSETS, 2, _] => FungiblesError::Unknown,
			[_, ASSETS, 3, _] => FungiblesError::InUse,
			[_, ASSETS, 5, _] => FungiblesError::MinBalanceZero,
			[_, ASSETS, 7, _] => FungiblesError::InsufficientAllowance,
			[_, ASSETS, 10, _] => FungiblesError::NotLive,
			_ => FungiblesError::Other(value),
		}
	}
}

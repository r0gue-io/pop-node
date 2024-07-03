use ink::prelude::vec::Vec;

use crate::{
	assets,
	primitives::{AssetId, MultiAddress},
	AccountId, Balance, StatusCode,
};

type Result<T> = core::result::Result<T, StatusCode>;

/// Local Fungibles:
/// 1. PSP-22 Interface
/// 2. PSP-22 Metadata Interface
/// 3. Asset Management

/// 1. PSP-22 Interface:
/// - total_supply
/// - balance_of
/// - allowance
/// - transfer
/// - transfer_from
/// - approve
/// - increase_allowance
/// - decrease_allowance

/// Returns the total token supply for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// The total supply of the token, or an error if the operation fails.
#[inline]
pub fn total_supply(id: AssetId) -> Result<Balance> {
	assets::total_supply(id)
}

/// Returns the account balance for the specified `owner` for a given asset ID. Returns `0` if
/// the account is non-existent.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `owner` - The account whose balance is being queried.
///
/// # Returns
/// The balance of the specified account, or an error if the operation fails.
#[inline]
pub fn balance_of(id: AssetId, owner: AccountId) -> Result<Balance> {
	assets::balance_of(id, owner)
}

/// Returns the amount which `spender` is still allowed to withdraw from `owner` for a given
/// asset ID. Returns `0` if no allowance has been set.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `owner` - The account that owns the tokens.
/// * `spender` - The account that is allowed to spend the tokens.
///
/// # Returns
/// The remaining allowance, or an error if the operation fails.
#[inline]
pub fn allowance(id: AssetId, owner: AccountId, spender: AccountId) -> Result<Balance> {
	assets::allowance(id, owner, spender)
}

/// Transfers `value` amount of tokens from the caller's account to account `to`, with additional
/// `data` in unspecified format.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `to` - The recipient account.
/// * `value` - The number of tokens to transfer.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the transfer fails.
#[inline]
pub fn transfer(
	id: AssetId,
	to: impl Into<MultiAddress<AccountId, ()>>,
	value: Balance,
) -> Result<()> {
	assets::transfer(id, to, value)
}

/// Transfers `value` tokens on behalf of `from` to account `to` with additional `data`
/// in unspecified format. If `from` is equal to `None`, tokens will be minted to account `to`. If
/// `to` is equal to `None`, tokens will be burned from account `from`.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `from` - The account from which the tokens are transferred.
/// * `to` - The recipient account.
/// * `value` - The number of tokens to transfer.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the transfer fails.
#[inline]
pub fn transfer_from(
	id: AssetId,
	from: impl Into<MultiAddress<AccountId, ()>>,
	to: impl Into<MultiAddress<AccountId, ()>>,
	value: Balance,
) -> Result<()> {
	assets::transfer_approved(id, from, to, value)
}

/// Approves an account to spend a specified number of tokens on behalf of the caller.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `spender` - The account that is allowed to spend the tokens.
/// * `value` - The number of tokens to approve.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the approval fails.
#[inline]
pub fn approve(id: AssetId, spender: AccountId, value: Balance) -> Result<()> {
	assets::approve_transfer(id, spender, value)
}

/// Increases the allowance of a spender.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `spender` - The account that is allowed to spend the tokens.
/// * `value` - The number of tokens to increase the allowance by.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
#[inline]
pub fn increase_allowance(id: AssetId, spender: AccountId, value: Balance) -> Result<()> {
	assets::approve_transfer(id, spender, value)
}

/// Decreases the allowance of a spender.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `spender` - The account that is allowed to spend the tokens.
/// * `value` - The number of tokens to decrease the allowance by.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
#[inline]
pub fn decrease_allowance(id: AssetId, spender: AccountId, value: Balance) -> Result<()> {
	assets::cancel_approval(id, spender.clone())?;
	assets::approve_transfer(id, spender, value)
}

/// 2. PSP-22 Metadata Interface:
/// - token_name
/// - token_symbol
/// - token_decimals

/// Returns the token name for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// The name of the token as a byte vector, or an error if the operation fails.
#[inline]
pub fn token_name(id: AssetId) -> Result<Vec<u8>> {
	assets::token_name(id)
}

/// Returns the token symbol for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
///  The symbol of the token as a byte vector, or an error if the operation fails.
#[inline]
pub fn token_symbol(id: AssetId) -> Result<Vec<u8>> {
	assets::token_symbol(id)
}

/// Returns the token decimals for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
///  The number of decimals of the token as a byte vector, or an error if the operation fails.
#[inline]
pub fn token_decimals(id: AssetId) -> Result<u8> {
	assets::token_decimals(id)
}

// /// 3. Asset Management:
// /// - create
// /// - start_destroy
// /// - destroy_accounts
// /// - destroy_approvals
// /// - finish_destroy
// /// - set_metadata
// /// - clear_metadata
//
// /// Create a new token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// /// * `admin` - The account that will administer the asset.
// /// * `min_balance` - The minimum balance required for accounts holding this asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the creation fails.
// // pub fn create(id: AssetId, admin: impl Into<MultiAddress<AccountId, ()>>, min_balance: Balance) -> Result<()> {
// // 	assets::create(id, admin, min_balance)
// // }
//
// /// Start the process of destroying a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// // fn start_destroy(id: AssetId) -> Result<()> {
// // 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::StartDestroy {
// // 		id: id.into(),
// // 	}))?)
// // }
//
// /// Destroy all accounts associated with a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// // fn destroy_accounts(id: AssetId) -> Result<()> {
// // 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::DestroyAccounts {
// // 		id: id.into(),
// // 	}))?)
// // }
//
// /// Destroy all approvals associated with a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// // fn destroy_approvals(id: AssetId) -> Result<()> {
// // 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::DestroyApprovals {
// // 		id: id.into(),
// // 	}))?)
// // }
//
// /// Complete the process of destroying a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// // fn finish_destroy(id: AssetId) -> Result<()> {
// // 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::FinishDestroy {
// // 		id: id.into(),
// // 	}))?)
// // }
//
// /// Set the metadata for a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// // pub fn set_metadata(id: AssetId, name: Vec<u8>, symbol: Vec<u8>, decimals: u8) -> Result<()> {
// // 	assets::set_metadata(id, name, symbol, decimals)
// // }
//
// /// Clear the metadata for a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// // fn clear_metadata(id: AssetId) -> Result<()> {
// // 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::ClearMetadata {
// // 		id: id.into(),
// // 	}))?)
// // }
//
// pub fn asset_exists(id: AssetId) -> Result<bool> {
// 	assets::asset_exists(id)
// }

#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum FungiblesError {
	Other(StatusCode),
	/// The asset is not live; either frozen or being destroyed.
	AssetNotLive,
	/// Not enough allowance to fulfill a request is available.
	InsufficientAllowance,
	/// Not enough balance to fulfill a request is available.
	InsufficientBalance,
	/// The asset ID is already taken.
	InUse,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
	/// The account to alter does not exist.
	NoAccount,
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given asset ID is unknown.
	Unknown,
	// // TODO:
	// // - Originally `InsufficientBalance` for the deposit but this would result in the same error
	// // as the error when there is insufficient balance for transferring an asset.
	/// No balance for creation of assets or fees.
	NoBalance,
}

impl From<StatusCode> for FungiblesError {
	fn from(value: StatusCode) -> Self {
		let encoded = value.0.to_le_bytes();
		match encoded {
			// Balances.
			[3, 10, 2, _] => FungiblesError::NoBalance,
			// Assets.
			[3, 52, 0, _] => FungiblesError::NoAccount,
			[3, 52, 1, _] => FungiblesError::NoPermission,
			[3, 52, 2, _] => FungiblesError::Unknown,
			[3, 52, 3, _] => FungiblesError::InUse,
			[3, 52, 5, _] => FungiblesError::MinBalanceZero,
			[3, 52, 7, _] => FungiblesError::InsufficientAllowance,
			[3, 52, 10, _] => FungiblesError::AssetNotLive,
			_ => FungiblesError::Other(value),
		}
	}
}

#[cfg(test)]
mod tests {
	use scale::Decode;

	use super::FungiblesError;
	use crate::error::{
		ArithmeticError::*,
		Error::{self, *},
		TokenError::*,
		TransactionalError::*,
	};
	use crate::StatusCode;

	fn into_fungibles_error(error: Error) -> FungiblesError {
		let status_code: StatusCode = error.into();
		status_code.into()
	}

	#[test]
	fn status_code_vs_encoded() {
		assert_eq!(u32::decode(&mut &[3u8, 10, 2, 0][..]).unwrap(), 133635u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 0, 0][..]).unwrap(), 13315u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 1, 0][..]).unwrap(), 78851u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 2, 0][..]).unwrap(), 144387u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 3, 0][..]).unwrap(), 209923u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 5, 0][..]).unwrap(), 340995u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 7, 0][..]).unwrap(), 472067u32);
		assert_eq!(u32::decode(&mut &[3u8, 52, 10, 0][..]).unwrap(), 668675u32);
	}

	#[test]
	fn conversion_status_code_into_fungibles_error_works() {
		let errors = vec![
			Other { dispatch_error_index: 5, error_index: 5, error: 1 },
			CannotLookup,
			BadOrigin,
			Module { index: 2, error: 5 },
			ConsumerRemaining,
			NoProviders,
			TooManyConsumers,
			Token(OnlyProvider),
			Arithmetic(Overflow),
			Transactional(NoLayer),
			Exhausted,
			Corruption,
			Unavailable,
			RootNotAllowed,
			DecodingFailed,
		];
		for error in errors {
			let status_code: StatusCode = error.into();
			let fungibles_error: FungiblesError = status_code.into();
			assert_eq!(fungibles_error, FungiblesError::Other(status_code))
		}

		assert_eq!(into_fungibles_error(Module { index: 10, error: 2 }), FungiblesError::NoBalance);
		assert_eq!(into_fungibles_error(Module { index: 52, error: 0 }), FungiblesError::NoAccount);
		assert_eq!(
			into_fungibles_error(Module { index: 52, error: 1 }),
			FungiblesError::NoPermission
		);
		assert_eq!(into_fungibles_error(Module { index: 52, error: 2 }), FungiblesError::Unknown);
		assert_eq!(into_fungibles_error(Module { index: 52, error: 3 }), FungiblesError::InUse);
		assert_eq!(
			into_fungibles_error(Module { index: 52, error: 5 }),
			FungiblesError::MinBalanceZero
		);
		assert_eq!(
			into_fungibles_error(Module { index: 52, error: 7 }),
			FungiblesError::InsufficientAllowance
		);
		assert_eq!(
			into_fungibles_error(Module { index: 52, error: 10 }),
			FungiblesError::AssetNotLive
		);
	}
}

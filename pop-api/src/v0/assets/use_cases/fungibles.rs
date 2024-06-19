#![allow(dead_code)]

use crate::{assets::pallets, AccountId, Balance, *};
use ink::prelude::vec::Vec;
use primitives::AssetId;

type Result<T> = core::result::Result<T, PopApiError>;

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
pub fn total_supply(id: AssetId) -> Result<Balance> {
	Ok(pallets::assets::total_supply(id)?)
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
pub fn balance_of(id: AssetId, owner: AccountId) -> Result<Balance> {
	Ok(pallets::assets::balance_of(id, owner)?)
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
pub fn allowance(id: AssetId, owner: AccountId, spender: AccountId) -> Result<Balance> {
	Ok(pallets::assets::allowance(id, owner, spender)?)
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
pub fn transfer(
	id: AssetId,
	to: impl Into<MultiAddress<AccountId, ()>>,
	value: Balance,
) -> Result<()> {
	Ok(pallets::assets::transfer(id, to, value)?)
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
pub fn transfer_from(
	id: AssetId,
	from: Option<impl Into<MultiAddress<AccountId, ()>>>,
	to: Option<impl Into<MultiAddress<AccountId, ()>>>,
	value: Balance,
	_data: &[u8],
) -> Result<()> {
	match (from, to) {
		(None, Some(to)) => Ok(pallets::assets::mint(id, to, value)?),
		(Some(from), None) => Ok(pallets::assets::burn(id, from, value)?),
		(Some(from), Some(to)) => Ok(pallets::assets::transfer_approved(id, from, to, value)?),
		_ => Ok(()),
	}
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
// #[allow(unused_variables)]
// fn approve(id: AssetId, spender: AccountId, value: Balance) -> Result<()> {
// 	todo!()
// 	// TODO: read allowance and increase or decrease.
// 	// Ok(dispatch(RuntimeCall::Assets(AssetsCall::ApproveTransfer {
// 	// 	id: id.into(),
// 	// 	delegate: spender.into(),
// 	// 	amount: Compact(value),
// 	// }))?)
// }

/// Increases the allowance of a spender.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `spender` - The account that is allowed to spend the tokens.
/// * `value` - The number of tokens to increase the allowance by.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn increase_allowance(id: AssetId, spender: AccountId, value: Balance) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::ApproveTransfer {
// 		id: id.into(),
// 		delegate: spender.into(),
// 		amount: Compact(value),
// 	}))?)
// }

/// Decreases the allowance of a spender.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `spender` - The account that is allowed to spend the tokens.
/// * `value` - The number of tokens to decrease the allowance by.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// #[allow(unused_variables)]
// fn decrease_allowance(id: AssetId, spender: AccountId, value: Balance) -> Result<()> {
// 	todo!()
// 	// TODO: cancel_approval + approve_transfer
// 	// Ok(dispatch(RuntimeCall::Assets(AssetsCall::CancelApproval {
// 	// 	id: id.into(),
// 	// 	delegate: delegate.into(),
// 	// }))?)
// 	// Ok(dispatch(RuntimeCall::Assets(AssetsCall::ApproveTransfer {
// 	// 	id: id.into(),
// 	// 	delegate: spender.into(),
// 	// 	amount: Compact(value),
// 	// }))?)
// }

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
// #[allow(unused_variables)]
// pub fn token_name(id: AssetId) -> Result<Option<Vec<u8>>> {
// 	todo!()
// 	// Ok(state::read(RuntimeStateKeys::Assets(AssetsKeys::TokenName(id)))?)
// }

/// Returns the token symbol for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
///  The symbol of the token as a byte vector, or an error if the operation fails.
// #[allow(unused_variables)]
// fn token_symbol(id: AssetId) -> Result<Option<Vec<u8>>> {
// 	todo!()
// }

/// Returns the token decimals for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
///  The number of decimals of the token as a byte vector, or an error if the operation fails.
// #[allow(unused_variables)]
// fn token_decimals(id: AssetId) -> Result<Option<Vec<u8>>> {
// 	todo!()
// }

/// 3. Asset Management:
/// - create
/// - start_destroy
/// - destroy_accounts
/// - destroy_approvals
/// - finish_destroy
/// - set_metadata
/// - clear_metadata

/// Create a new token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
/// * `admin` - The account that will administer the asset.
/// * `min_balance` - The minimum balance required for accounts holding this asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the creation fails.
pub fn create(
	id: AssetId,
	admin: impl Into<MultiAddress<AccountId, ()>>,
	min_balance: Balance,
) -> Result<()> {
	Ok(pallets::assets::create(id, admin, min_balance)?)
}

/// Start the process of destroying a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn start_destroy(id: AssetId) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::StartDestroy {
// 		id: id.into(),
// 	}))?)
// }

/// Destroy all accounts associated with a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn destroy_accounts(id: AssetId) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::DestroyAccounts {
// 		id: id.into(),
// 	}))?)
// }

/// Destroy all approvals associated with a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn destroy_approvals(id: AssetId) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::DestroyApprovals {
// 		id: id.into(),
// 	}))?)
// }

/// Complete the process of destroying a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn finish_destroy(id: AssetId) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::FinishDestroy {
// 		id: id.into(),
// 	}))?)
// }

/// Set the metadata for a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
pub fn set_metadata(id: AssetId, name: Vec<u8>, symbol: Vec<u8>, decimals: u8) -> Result<()> {
	Ok(pallets::assets::set_metadata(id, name, symbol, decimals)?)
}

/// Clear the metadata for a token with a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// Returns `Ok(())` if successful, or an error if the operation fails.
// fn clear_metadata(id: AssetId) -> Result<()> {
// 	Ok(dispatch(RuntimeCall::Assets(AssetsCall::ClearMetadata {
// 		id: id.into(),
// 	}))?)
// }

pub fn asset_exists(id: AssetId) -> Result<bool> {
	Ok(pallets::assets::asset_exists(id)?)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FungiblesError {
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
	// // - verify fees
	/// No balance for creation of assets or fees.
	NoBalance,
}

pub(crate) fn convert_to_fungibles_error(index: u8, error: u8) -> PopApiError {
	match index {
		10 => balance_into(error),
		52 => assets_into(error),
		_ => Module { index, error },
	}
}

fn balance_into(error: u8) -> PopApiError {
	match error {
		2 => UseCaseError(FungiblesError::NoBalance),
		_ => Module { index: 10, error },
	}
}

fn assets_into(error: u8) -> PopApiError {
	match error {
		0 => UseCaseError(FungiblesError::InsufficientBalance),
		1 => UseCaseError(FungiblesError::NoAccount),
		2 => UseCaseError(FungiblesError::NoPermission),
		3 => UseCaseError(FungiblesError::Unknown),
		5 => UseCaseError(FungiblesError::InUse),
		7 => UseCaseError(FungiblesError::MinBalanceZero),
		10 => UseCaseError(FungiblesError::InsufficientAllowance),
		16 => UseCaseError(FungiblesError::AssetNotLive),
		_ => Module { index: 52, error },
	}
}

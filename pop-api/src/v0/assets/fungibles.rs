use crate::{
	constants::{ASSETS, BALANCES, FUNGIBLES},
	primitives::{AccountId, AssetId, Balance},
	Result, StatusCode,
};
use constants::*;
use ink::{env::chain_extension::ChainExtensionMethod, prelude::vec::Vec};
pub use metadata::*;

/// Helper method to build a dispatch call `ChainExtensionMethod` for fungibles `v0`
///
/// Parameters:
/// - 'dispatchable': The index of the module dispatchable functions
fn build_dispatch(dispatchable: u8) -> ChainExtensionMethod<(), (), (), false> {
	crate::v0::build_dispatch(FUNGIBLES, dispatchable)
}

/// Helper method to build a dispatch call `ChainExtensionMethod` for fungibles `v0``
///
/// Parameters:
/// - 'state_query': The index of the runtime state query
fn build_read_state(state_query: u8) -> ChainExtensionMethod<(), (), (), false> {
	crate::v0::build_read_state(FUNGIBLES, state_query)
}

/// Local Fungibles:
/// 1. PSP-22 Interface
/// 2. PSP-22 Metadata Interface
/// 3. Asset Management

mod constants {
	/// 1. PSP-22 Interface:
	/// - total_supply
	pub const TOTAL_SUPPLY: u8 = 0;
	/// - balance_of
	pub const BALANCE_OF: u8 = 1;
	/// - allowance
	pub const ALLOWANCE: u8 = 2;
	/// - transfer
	pub(super) const TRANSFER: u8 = 3;
	/// - transfer_from
	pub(super) const TRANSFER_FROM: u8 = 4;
	/// - approve
	pub(super) const APPROVE: u8 = 5;
	/// - increase_allowance
	pub(super) const INCREASE_ALLOWANCE: u8 = 6;
	/// - decrease_allowance
	pub(super) const DECREASE_ALLOWANCE: u8 = 7;

	/// 2. PSP-22 Metadata Interface:
	/// - token_name
	pub const TOKEN_NAME: u8 = 8;
	/// - token_symbol
	pub const TOKEN_SYMBOL: u8 = 9;
	/// - token_decimals
	pub const TOKEN_DECIMALS: u8 = 10;

	// 3. Asset Management:
	// - create
	// - start_destroy
	// - destroy_accounts
	// - destroy_approvals
	// - finish_destroy
	// - set_metadata
	// - clear_metadata
}

/// Returns the total token supply for a given asset ID.
///
/// # Arguments
/// * `id` - The ID of the asset.
///
/// # Returns
/// The total supply of the token, or an error if the operation fails.
#[inline]
pub fn total_supply(id: AssetId) -> Result<Balance> {
	build_read_state(TOTAL_SUPPLY)
		.input::<AssetId>()
		.output::<Result<Balance>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id))
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
	build_read_state(BALANCE_OF)
		.input::<(AssetId, AccountId)>()
		.output::<Result<Balance>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, owner))
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
	build_read_state(ALLOWANCE)
		.input::<(AssetId, AccountId, AccountId)>()
		.output::<Result<Balance>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, owner, spender))
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
pub fn transfer(id: AssetId, target: AccountId, amount: Balance) -> Result<()> {
	build_dispatch(TRANSFER)
		.input::<(AssetId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, target, amount))
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
pub fn transfer_from(id: AssetId, from: AccountId, to: AccountId, amount: Balance) -> Result<()> {
	build_dispatch(TRANSFER_FROM)
		.input::<(AssetId, AccountId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, from, to, amount))
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
pub fn approve(id: AssetId, spender: AccountId, amount: Balance) -> Result<()> {
	build_dispatch(APPROVE)
		.input::<(AssetId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, spender, amount))
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
	build_dispatch(INCREASE_ALLOWANCE)
		.input::<(AssetId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, spender, value))
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
	build_dispatch(DECREASE_ALLOWANCE)
		.input::<(AssetId, AccountId, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, spender, value))
}

pub mod metadata {
	use super::*;
	/// Returns the token name for a given asset ID.
	///
	/// # Arguments
	/// * `id` - The ID of the asset.
	///
	/// # Returns
	/// The name of the token as a byte vector, or an error if the operation fails.
	#[inline]
	pub fn token_name(id: AssetId) -> Result<Vec<u8>> {
		build_read_state(TOKEN_NAME)
			.input::<AssetId>()
			.output::<Result<Vec<u8>>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(id))
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
		build_read_state(TOKEN_SYMBOL)
			.input::<AssetId>()
			.output::<Result<Vec<u8>>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(id))
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
		build_read_state(TOKEN_DECIMALS)
			.input::<AssetId>()
			.output::<Result<u8>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(id))
	}
}

// pub asset_management {
// /// Create a new token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// /// * `admin` - The account that will administer the asset.
// /// * `min_balance` - The minimum balance required for accounts holding this asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the creation fails.
// pub fn create(id: AssetId, admin: impl Into<MultiAddress<AccountId, ()>>, min_balance: Balance) -> Result<()> {
// 	Ok(())
// }
//
// /// Start the process of destroying a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// fn start_destroy(id: AssetId) -> Result<()> {
// 	Ok(())
// }
//
// /// Destroy all accounts associated with a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// fn destroy_accounts(id: AssetId) -> Result<()> {
// 	Ok(())
// }
//
// /// Destroy all approvals associated with a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// fn destroy_approvals(id: AssetId) -> Result<()> {
// 	Ok(())
// }
//
// /// Complete the process of destroying a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// fn finish_destroy(id: AssetId) -> Result<()> {
// 	Ok(())
// }
//
// /// Set the metadata for a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// pub fn set_metadata(id: AssetId, name: Vec<u8>, symbol: Vec<u8>, decimals: u8) -> Result<()> {
// 	Ok(())
// }
//
// /// Clear the metadata for a token with a given asset ID.
// ///
// /// # Arguments
// /// * `id` - The ID of the asset.
// ///
// /// # Returns
// /// Returns `Ok(())` if successful, or an error if the operation fails.
// fn clear_metadata(id: AssetId) -> Result<()> {
//  Ok(())
// }
// }
//
// pub fn asset_exists(id: AssetId) -> Result<bool> {
// 	assets::asset_exists(id)
// }

/// Represents various errors related to local fungible assets in the Pop API.
///
/// The `FungiblesError` provides a detailed and specific set of error types that can occur when
/// interacting with fungible assets through the Pop API. Each variant signifies a particular error
/// condition, facilitating precise error handling and debugging.
///
/// It is designed to be lightweight, including only the essential errors relevant to fungible asset
/// operations. The `Other` variant serves as a catch-all for any unexpected errors. For more
/// detailed debugging, the `Other` variant can be converted into the richer `Error` type defined in
/// the primitives crate.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum FungiblesError {
	/// An unspecified or unknown error occurred.
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
	/// No balance for creation of assets or fees.
	// TODO: Originally `pallet_balances::Error::InsufficientBalance` but collides with the
	//  `InsufficientBalance` error that is used for `pallet_assets::Error::BalanceLow` to adhere to
	//   standard. This deserves a second look.
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
			[_, ASSETS, 10, _] => FungiblesError::AssetNotLive,
			_ => FungiblesError::Other(value),
		}
	}
}

#[cfg(test)]
mod tests {
	use ink::scale::{Decode, Encode};

	use super::FungiblesError;
	use crate::{
		constants::{ASSETS, BALANCES},
		primitives::error::{
			ArithmeticError::*,
			Error::{self, *},
			TokenError::*,
			TransactionalError::*,
		},
		StatusCode,
	};

	fn error_into_status_code(error: Error) -> StatusCode {
		let mut encoded_error = error.encode();
		encoded_error.resize(4, 0);
		let value = u32::from_le_bytes(
			encoded_error.try_into().expect("qed, resized to 4 bytes line above"),
		);
		value.into()
	}

	fn into_fungibles_error(error: Error) -> FungiblesError {
		let status_code: StatusCode = error_into_status_code(error);
		status_code.into()
	}

	// If we ever want to change the conversion from bytes to `u32`.
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
		let other_errors = vec![
			Other { dispatch_error_index: 5, error_index: 5, error: 1 },
			CannotLookup,
			BadOrigin,
			// `ModuleError` other than assets module.
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
			UnknownCall,
			DecodingFailed,
		];
		for error in other_errors {
			let status_code: StatusCode = error_into_status_code(error);
			let fungibles_error: FungiblesError = status_code.into();
			assert_eq!(fungibles_error, FungiblesError::Other(status_code))
		}

		assert_eq!(
			into_fungibles_error(Module { index: BALANCES, error: 2 }),
			FungiblesError::NoBalance
		);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 0 }),
			FungiblesError::NoAccount
		);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 1 }),
			FungiblesError::NoPermission
		);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 2 }),
			FungiblesError::Unknown
		);
		assert_eq!(into_fungibles_error(Module { index: ASSETS, error: 3 }), FungiblesError::InUse);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 5 }),
			FungiblesError::MinBalanceZero
		);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 7 }),
			FungiblesError::InsufficientAllowance
		);
		assert_eq!(
			into_fungibles_error(Module { index: ASSETS, error: 10 }),
			FungiblesError::AssetNotLive
		);
	}
}

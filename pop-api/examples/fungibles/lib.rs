#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Local Fungibles:
/// 1. PSP-22 Interface
/// 2. PSP-22 Metadata Interface
/// 3. Asset Management
///
use ink::prelude::vec::Vec;
use pop_api::{
	assets::fungibles::*,
	primitives::{AccountId as AccountId32, AssetId},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
	/// The asset is not live; either frozen or being destroyed.
	AssetNotLive,
	/// The amount to mint is less than the existential deposit.
	BelowMinimum,
	/// Unspecified dispatch error, providing the index and its error index (if none `0`).
	DispatchError { index: u8, error: u8 },
	/// Not enough allowance to fulfill a request is available.
	InsufficientAllowance,
	/// Not enough balance to fulfill a request is available.
	InsufficientBalance,
	/// The asset ID is already taken.
	InUse,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
	/// Unspecified pallet error, providing pallet index and error index.
	ModuleError { pallet: u8, error: u16 },
	/// The account to alter does not exist.
	NoAccount,
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given asset ID is unknown.
	Unknown,
}

impl From<FungiblesError> for ContractError {
	fn from(error: FungiblesError) -> Self {
		match error {
			FungiblesError::AssetNotLive => ContractError::AssetNotLive,
			FungiblesError::BelowMinimum => ContractError::BelowMinimum,
			FungiblesError::DispatchError { index, error } => {
				ContractError::DispatchError { index, error }
			},
			FungiblesError::InsufficientAllowance => ContractError::InsufficientAllowance,
			FungiblesError::InsufficientBalance => ContractError::InsufficientBalance,
			FungiblesError::InUse => ContractError::InUse,
			FungiblesError::MinBalanceZero => ContractError::MinBalanceZero,
			FungiblesError::ModuleError { pallet, error } => {
				ContractError::ModuleError { pallet, error }
			},
			FungiblesError::NoAccount => ContractError::NoAccount,
			FungiblesError::NoPermission => ContractError::NoPermission,
			FungiblesError::Unknown => ContractError::Unknown,
		}
	}
}

/// The fungibles result type.
pub type Result<T> = core::result::Result<T, ContractError>;

#[ink::contract(env = pop_api::Environment)]
mod fungibles {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Fungibles;

	impl Fungibles {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("PopApiAssetsExample::new");
			Default::default()
		}

		/// 1. PSP-22 Interface:
		/// - total_supply
		/// - balance_of
		/// - allowance
		/// - transfer
		/// - transfer_from
		/// - approve
		/// - increase_allowance
		/// - decrease_allowance

		#[ink(message)]
		pub fn total_supply(&self, id: AssetId) -> Result<Balance> {
			total_supply(id).map_err(From::from)
		}

		#[ink(message)]
		pub fn balance_of(&self, id: AssetId, owner: AccountId32) -> Result<Balance> {
			balance_of(id, owner).map_err(From::from)
		}

		#[ink(message)]
		pub fn allowance(
			&self,
			id: AssetId,
			owner: AccountId32,
			spender: AccountId32,
		) -> Result<Balance> {
			allowance(id, owner, spender).map_err(From::from)
		}

		#[ink(message)]
		pub fn transfer(&self, id: AssetId, to: AccountId32, value: Balance) -> Result<()> {
			ink::env::debug_println!(
				"PopApiAssetsExample::transfer: id: {:?}, to: {:?} value: {:?}",
				id,
				to,
				value,
			);

			let result = transfer(id, to, value);
			ink::env::debug_println!("Result: {:?}", result);
			result.map_err(From::from)
		}

		#[ink(message)]
		pub fn transfer_from(
			&self,
			id: AssetId,
			from: Option<AccountId32>,
			to: Option<AccountId32>,
			value: Balance,
			// Size needs to be known at compile time or ink's `Vec`
			data: Vec<u8>,
		) -> Result<()> {
			ink::env::debug_println!(
				"PopApiAssetsExample::transfer_from: id: {:?}, from: {:?}, to: {:?} value: {:?}",
				id,
				from,
				to,
				value,
			);

			let result = transfer_from(id, from, to, value, &data);
			ink::env::debug_println!("Result: {:?}", result);
			result.map_err(From::from)
		}

		/// 2. PSP-22 Metadata Interface:
		/// - token_name
		/// - token_symbol
		/// - token_decimals

		/// 3. Asset Management:
		/// - create
		/// - start_destroy
		/// - destroy_accounts
		/// - destroy_approvals
		/// - finish_destroy
		/// - set_metadata
		/// - clear_metadata

		#[ink(message)]
		pub fn create(&self, id: AssetId, admin: AccountId32, min_balance: Balance) -> Result<()> {
			ink::env::debug_println!(
				"PopApiAssetsExample::create: id: {:?} admin: {:?} min_balance: {:?}",
				id,
				admin,
				min_balance,
			);
			let result = create(id, admin, min_balance);
			ink::env::debug_println!("Result: {:?}", result);
			result.map_err(From::from)
		}

		#[ink(message)]
		pub fn set_metadata(
			&self,
			id: AssetId,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> Result<()> {
			ink::env::debug_println!(
				"PopApiAssetsExample::set_metadata: id: {:?} name: {:?} symbol: {:?}, decimals: {:?}",
				id,
				name,
				symbol,
				decimals,
			);
			let result = set_metadata(id, name, symbol, decimals);
			ink::env::debug_println!("Result: {:?}", result);
			result.map_err(From::from)
		}

		#[ink(message)]
		pub fn asset_exists(&self, id: AssetId) -> Result<bool> {
			asset_exists(id).map_err(From::from)
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			PopApiAssetsExample::new();
		}
	}
}

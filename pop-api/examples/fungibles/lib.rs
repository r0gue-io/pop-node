#![cfg_attr(not(feature = "std"), no_std, no_main)]

// Fungibles wrapper contract to allow contracts to interact with local fungibles without the pop api.
use ink::prelude::vec::Vec;
use pop_api::{
	assets::fungibles::*,
	primitives::{AccountId as AccountId32, AssetId},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
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
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given asset ID is unknown.
	Unknown,
}

impl From<FungiblesError> for ContractError {
	fn from(error: FungiblesError) -> Self {
		match error {
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
		pub fn asset_exists(&self, id: AssetId) -> Result<bool> {
			asset_exists(id).map_err(From::from)
		}

		#[ink(message)]
		pub fn create(&self, id: AssetId, admin: AccountId32, min_balance: Balance) -> Result<()> {
			// create(id, admin, min_balance).map_err(From::from)
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
			// set_metadata(id, name, symbol, decimals).map_err(From::from)
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
		pub fn mint(&self, id: AssetId, beneficiary: AccountId32, amount: Balance) -> Result<()> {
			ink::env::debug_println!(
				"PopApiAssetsExample::mint: id: {:?}, beneficiary: {:?} amount: {:?}",
				id,
				beneficiary,
				amount,
			);

			let result = mint(id, beneficiary, amount);
			ink::env::debug_println!("Result: {:?}", result);
			result.map_err(From::from)
		}

		// #[ink(message)]
		// pub fn transfer_from(
		// 	id: AssetId,
		// 	from: Option<AccountId32>,
		// 	to: Option<AccountId32>,
		// 	value: Balance,
		// 	data: [u8],
		// ) -> Result<()> {
		// 	ink::env::debug_println!(
		// 		"PopApiAssetsExample::transfer_from: id: {:?}, from: {:?}, to: {:?} value: {:?}",
		// 		id,
		// 		from,
		// 		to,
		// 		value,
		// 	);
		//
		// 	let result = transfer_from(id, from, to, value)?;
		// 	ink::env::debug_println!("Result: {:?}", result);
		// 	result
		// }
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

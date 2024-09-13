//! The fungibles pallet offers a streamlined interface for interacting with fungible tokens. The
//! goal is to provide a simplified, consistent API that adheres to standards in the smart contract
//! space.

use frame_support::traits::fungibles::{metadata::Inspect as MetadataInspect, Inspect};
pub use pallet::*;
use pallet_assets::WeightInfo as AssetsWeightInfoTrait;
use weights::WeightInfo;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod tests;
pub mod weights;
use pop_primitives::{ArithmeticError::*, Error::*, TokenError::*, TokenId, *};
use utils::*;

use super::*;

mod utils;

const TOKEN_ID: TokenId = 1;
const CONTRACT: &str = "contracts/fungibles/target/ink/fungibles.wasm";

/// 1. PSP-22 Interface:
/// - total_supply
/// - balance_of
/// - allowance
/// - transfer
/// - transfer_from
/// - approve
/// - increase_allowance
/// - decrease_allowance

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(total_supply(addr.clone(), TOKEN_ID), Ok(Assets::total_supply(TOKEN_ID)));
		assert_eq!(total_supply(addr.clone(), TOKEN_ID), Ok(0));

		// Tokens in circulation.
		create_asset_and_mint_to(addr.clone(), TOKEN_ID, BOB, 100);
		assert_eq!(total_supply(addr.clone(), TOKEN_ID), Ok(Assets::total_supply(TOKEN_ID)));
		assert_eq!(total_supply(addr, TOKEN_ID), Ok(100));
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(balance_of(addr.clone(), TOKEN_ID, BOB), Ok(Assets::balance(TOKEN_ID, BOB)));
		assert_eq!(balance_of(addr.clone(), TOKEN_ID, BOB), Ok(0));

		// Tokens in circulation.
		create_asset_and_mint_to(addr.clone(), TOKEN_ID, BOB, 100);
		assert_eq!(balance_of(addr.clone(), TOKEN_ID, BOB), Ok(Assets::balance(TOKEN_ID, BOB)));
		assert_eq!(balance_of(addr, TOKEN_ID, BOB), Ok(100));
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(
			allowance(addr.clone(), TOKEN_ID, BOB, ALICE),
			Ok(Assets::allowance(TOKEN_ID, &BOB, &ALICE))
		);
		assert_eq!(allowance(addr.clone(), TOKEN_ID, BOB, ALICE), Ok(0));

		// Tokens in circulation.
		create_asset_mint_and_approve(addr.clone(), TOKEN_ID, BOB, 100, ALICE, 50);
		assert_eq!(
			allowance(addr.clone(), TOKEN_ID, BOB, ALICE),
			Ok(Assets::allowance(TOKEN_ID, &BOB, &ALICE))
		);
		assert_eq!(allowance(addr, TOKEN_ID, BOB, ALICE), Ok(50));
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Asset does not exist.
		assert_eq!(
			transfer(addr.clone(), 1, BOB, amount),
			Err(Module { index: 52, error: [3, 0] })
		);
		// Create asset with Alice as owner and mint `amount` to contract address.
		let asset = create_asset_and_mint_to(ALICE, 1, addr.clone(), amount);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(ALICE, asset);
		assert_eq!(
			transfer(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [16, 0] })
		);
		thaw_asset(ALICE, asset);
		// Not enough balance.
		assert_eq!(
			transfer(addr.clone(), asset, BOB, amount + 1 * UNIT),
			Err(Module { index: 52, error: [0, 0] })
		);
		// Not enough balance due to ED.
		assert_eq!(
			transfer(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [0, 0] })
		);
		// Successful transfer.
		let balance_before_transfer = Assets::balance(asset, &BOB);
		assert_ok!(transfer(addr.clone(), asset, BOB, amount / 2));
		let balance_after_transfer = Assets::balance(asset, &BOB);
		assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
		// Transfer asset to account that does not exist.
		assert_eq!(transfer(addr.clone(), asset, FERDIE, amount / 4), Err(Token(CannotCreate)));
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(ALICE, asset);
		assert_eq!(
			transfer(addr.clone(), asset, BOB, amount / 4),
			Err(Module { index: 52, error: [16, 0] })
		);
	});
}

#[test]
fn transfer_from_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Asset does not exist.
		assert_eq!(
			transfer_from(addr.clone(), 1, ALICE, BOB, amount / 2),
			Err(Module { index: 52, error: [3, 0] }),
		);
		// Create asset with Alice as owner and mint `amount` to contract address.
		let asset = create_asset_and_mint_to(ALICE, 1, ALICE, amount);
		// Unapproved transfer.
		assert_eq!(
			transfer_from(addr.clone(), asset, ALICE, BOB, amount / 2),
			Err(Module { index: 52, error: [10, 0] })
		);
		assert_ok!(Assets::approve_transfer(
			RuntimeOrigin::signed(ALICE.into()),
			asset.into(),
			addr.clone().into(),
			amount + 1 * UNIT,
		));
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(ALICE, asset);
		assert_eq!(
			transfer_from(addr.clone(), asset, ALICE, BOB, amount),
			Err(Module { index: 52, error: [16, 0] }),
		);
		thaw_asset(ALICE, asset);
		// Not enough balance.
		assert_eq!(
			transfer_from(addr.clone(), asset, ALICE, BOB, amount + 1 * UNIT),
			Err(Module { index: 52, error: [0, 0] }),
		);
		// Successful transfer.
		let balance_before_transfer = Assets::balance(asset, &BOB);
		assert_ok!(transfer_from(addr.clone(), asset, ALICE, BOB, amount / 2));
		let balance_after_transfer = Assets::balance(asset, &BOB);
		assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
	});
}

#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, 0, vec![]);
		let amount: Balance = 100 * UNIT;

		// Asset does not exist.
		assert_eq!(approve(addr.clone(), 0, BOB, amount), Err(Module { index: 52, error: [3, 0] }));
		let asset = create_asset_and_mint_to(ALICE, 0, addr.clone(), amount);
		assert_eq!(approve(addr.clone(), asset, BOB, amount), Err(ConsumerRemaining));
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![1]);
		// Create asset with Alice as owner and mint `amount` to contract address.
		let asset = create_asset_and_mint_to(ALICE, 1, addr.clone(), amount);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(ALICE, asset);
		assert_eq!(
			approve(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [16, 0] })
		);
		thaw_asset(ALICE, asset);
		// Successful approvals:
		assert_eq!(0, Assets::allowance(asset, &addr, &BOB));
		assert_ok!(approve(addr.clone(), asset, BOB, amount));
		assert_eq!(Assets::allowance(asset, &addr, &BOB), amount);
		// Non-additive, sets new value.
		assert_ok!(approve(addr.clone(), asset, BOB, amount / 2));
		assert_eq!(Assets::allowance(asset, &addr, &BOB), amount / 2);
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(ALICE, asset);
		assert_eq!(
			approve(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [16, 0] })
		);
	});
}

#[test]
fn increase_allowance_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let amount: Balance = 100 * UNIT;
		// Instantiate a contract without balance - test `ConsumerRemaining.
		let addr = instantiate(CONTRACT, 0, vec![]);
		// Asset does not exist.
		assert_eq!(
			increase_allowance(addr.clone(), 0, BOB, amount),
			Err(Module { index: 52, error: [3, 0] })
		);
		let asset = create_asset_and_mint_to(ALICE, 0, addr.clone(), amount);
		assert_eq!(increase_allowance(addr.clone(), asset, BOB, amount), Err(ConsumerRemaining));

		// Instantiate a contract with balance.
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![1]);
		// Create asset with Alice as owner and mint `amount` to contract address.
		let asset = create_asset_and_mint_to(ALICE, 1, addr.clone(), amount);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(ALICE, asset);
		assert_eq!(
			increase_allowance(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [16, 0] })
		);
		thaw_asset(ALICE, asset);
		// Successful approvals:
		assert_eq!(0, Assets::allowance(asset, &addr, &BOB));
		assert_ok!(increase_allowance(addr.clone(), asset, BOB, amount));
		assert_eq!(Assets::allowance(asset, &addr, &BOB), amount);
		// Additive.
		assert_ok!(increase_allowance(addr.clone(), asset, BOB, amount));
		assert_eq!(Assets::allowance(asset, &addr, &BOB), amount * 2);
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(ALICE, asset);
		assert_eq!(
			increase_allowance(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [16, 0] })
		);
	});
}

#[test]
fn decrease_allowance_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Asset does not exist.
		assert_eq!(
			decrease_allowance(addr.clone(), 0, BOB, amount),
			Err(Module { index: 52, error: [3, 0] }),
		);
		// Create asset and mint `amount` to contract address, then approve Bob to spend `amount`.
		let asset =
			create_asset_mint_and_approve(addr.clone(), 0, addr.clone(), amount, BOB, amount);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(addr.clone(), asset);
		assert_eq!(
			decrease_allowance(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [16, 0] }),
		);
		thaw_asset(addr.clone(), asset);
		// Successfully decrease allowance.
		let allowance_before = Assets::allowance(asset, &addr, &BOB);
		assert_ok!(decrease_allowance(addr.clone(), 0, BOB, amount / 2 - 1 * UNIT));
		let allowance_after = Assets::allowance(asset, &addr, &BOB);
		assert_eq!(allowance_before - allowance_after, amount / 2 - 1 * UNIT);
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(addr.clone(), asset);
		assert_eq!(
			decrease_allowance(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [16, 0] }),
		);
	});
}

/// 2. PSP-22 Metadata Interface:
/// - token_name
/// - token_symbol
/// - token_decimals

#[test]
fn token_metadata_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let name: Vec<u8> = vec![11, 12, 13];
		let symbol: Vec<u8> = vec![21, 22, 23];
		let decimals: u8 = 69;

		// Token does not exist.
		assert_eq!(token_name(addr.clone(), TOKEN_ID), Ok(token_name_asset(TOKEN_ID)));
		assert_eq!(token_name(addr.clone(), TOKEN_ID), Ok(Vec::<u8>::new()));
		assert_eq!(token_symbol(addr.clone(), TOKEN_ID), Ok(token_symbol_asset(TOKEN_ID)));
		assert_eq!(token_symbol(addr.clone(), TOKEN_ID), Ok(Vec::<u8>::new()));
		assert_eq!(token_decimals(addr.clone(), TOKEN_ID), Ok(token_decimals_asset(TOKEN_ID)));
		assert_eq!(token_decimals(addr.clone(), TOKEN_ID), Ok(0));
		// Create Token.
		create_asset_and_set_metadata(
			addr.clone(),
			TOKEN_ID,
			name.clone(),
			symbol.clone(),
			decimals,
		);
		assert_eq!(token_name(addr.clone(), TOKEN_ID), Ok(token_name_asset(TOKEN_ID)));
		assert_eq!(token_name(addr.clone(), TOKEN_ID), Ok(name));
		assert_eq!(token_symbol(addr.clone(), TOKEN_ID), Ok(token_symbol_asset(TOKEN_ID)));
		assert_eq!(token_symbol(addr.clone(), TOKEN_ID), Ok(symbol));
		assert_eq!(token_decimals(addr.clone(), TOKEN_ID), Ok(token_decimals_asset(TOKEN_ID)));
		assert_eq!(token_decimals(addr.clone(), TOKEN_ID), Ok(decimals));
	});
}

/// 3. Asset Management:
/// - create
/// - start_destroy
/// - set_metadata
/// - clear_metadata
/// - token_exists

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		// Instantiate a contract without balance for fees.
		let addr = instantiate(CONTRACT, 0, vec![0]);
		// No balance to pay for fees.
		assert_eq!(
			create(addr.clone(), TOKEN_ID, addr.clone(), 1),
			Err(Module { index: 10, error: [2, 0] }),
		);

		// Instantiate a contract without balance for deposit.
		let addr = instantiate(CONTRACT, 100, vec![1]);
		// No balance to pay the deposit.
		assert_eq!(
			create(addr.clone(), TOKEN_ID, addr.clone(), 1),
			Err(Module { index: 10, error: [2, 0] }),
		);

		// Instantiate a contract with enough balance.
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![2]);
		assert_eq!(
			create(addr.clone(), TOKEN_ID, BOB, 0),
			Err(Module { index: 52, error: [7, 0] }),
		);
		// The minimal balance for an asset must be non zero.
		assert_eq!(
			create(addr.clone(), TOKEN_ID, BOB, 0),
			Err(Module { index: 52, error: [7, 0] }),
		);
		// Create asset successfully.
		assert_ok!(create(addr.clone(), TOKEN_ID, BOB, 1));
		// Asset ID is already taken.
		assert_eq!(
			create(addr.clone(), TOKEN_ID, BOB, 1),
			Err(Module { index: 52, error: [5, 0] }),
		);
	});
}

// Testing a contract that creates an asset in the constructor.
#[test]
fn instantiate_and_create_fungible_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let contract =
			"contracts/create_token_in_constructor/target/ink/create_token_in_constructor.wasm";
		// Asset already exists.
		create_asset(ALICE, 0, 1);
		assert_eq!(
			instantiate_and_create_fungible(contract, 0, 1),
			Err(Module { index: 52, error: [5, 0] })
		);
		// Successfully create an asset when instantiating the contract.
		assert_ok!(instantiate_and_create_fungible(contract, TOKEN_ID, 1));
		assert!(Assets::asset_exists(TOKEN_ID));
	});
}

#[test]
fn start_destroy_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![2]);

		// Asset does not exist.
		assert_eq!(start_destroy(addr.clone(), TOKEN_ID), Err(Module { index: 52, error: [3, 0] }),);
		// Create assets where contract is not the owner.
		let asset = create_asset(ALICE, 0, 1);
		// No Permission.
		assert_eq!(start_destroy(addr.clone(), asset), Err(Module { index: 52, error: [2, 0] }),);
		let asset = create_asset(addr.clone(), TOKEN_ID, 1);
		assert_ok!(start_destroy(addr.clone(), asset));
	});
}

#[test]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let name = vec![42];
		let symbol = vec![42];
		let decimals = 42u8;
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Asset does not exist.
		assert_eq!(
			set_metadata(addr.clone(), TOKEN_ID, vec![0], vec![0], 0u8),
			Err(Module { index: 52, error: [3, 0] }),
		);
		// Create assets where contract is not the owner.
		let asset = create_asset(ALICE, 0, 1);
		// No Permission.
		assert_eq!(
			set_metadata(addr.clone(), asset, vec![0], vec![0], 0u8),
			Err(Module { index: 52, error: [2, 0] }),
		);
		let asset = create_asset(addr.clone(), TOKEN_ID, 1);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(addr.clone(), asset);
		assert_eq!(
			set_metadata(addr.clone(), TOKEN_ID, vec![0], vec![0], 0u8),
			Err(Module { index: 52, error: [16, 0] }),
		);
		thaw_asset(addr.clone(), asset);
		// TODO: calling the below with a vector of length `100_000` errors in pallet contracts
		//  `OutputBufferTooSmall. Added to security analysis issue #131 to revisit.
		// Set bad metadata - too large values.
		assert_eq!(
			set_metadata(addr.clone(), TOKEN_ID, vec![0; 1000], vec![0; 1000], 0u8),
			Err(Module { index: 52, error: [9, 0] }),
		);
		// Set metadata successfully.
		assert_ok!(set_metadata(addr.clone(), TOKEN_ID, name, symbol, decimals));
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(addr.clone(), asset);
		assert_eq!(
			set_metadata(addr.clone(), TOKEN_ID, vec![0], vec![0], 0),
			Err(Module { index: 52, error: [16, 0] }),
		);
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let name = vec![42];
		let symbol = vec![42];
		let decimals = 42u8;
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Asset does not exist.
		assert_eq!(clear_metadata(addr.clone(), 0), Err(Module { index: 52, error: [3, 0] }),);
		// Create assets where contract is not the owner.
		let asset = create_asset_and_set_metadata(ALICE, 0, vec![0], vec![0], 0);
		// No Permission.
		assert_eq!(clear_metadata(addr.clone(), asset), Err(Module { index: 52, error: [2, 0] }),);
		let asset = create_asset(addr.clone(), TOKEN_ID, 1);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(addr.clone(), asset);
		assert_eq!(clear_metadata(addr.clone(), asset), Err(Module { index: 52, error: [16, 0] }),);
		thaw_asset(addr.clone(), asset);
		// No metadata set.
		assert_eq!(clear_metadata(addr.clone(), asset), Err(Module { index: 52, error: [3, 0] }),);
		set_metadata_asset(addr.clone(), asset, name, symbol, decimals);
		// Clear metadata successfully.
		assert_ok!(clear_metadata(addr.clone(), TOKEN_ID));
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(addr.clone(), asset);
		assert_eq!(
			set_metadata(addr.clone(), TOKEN_ID, vec![0], vec![0], decimals),
			Err(Module { index: 52, error: [16, 0] }),
		);
	});
}

#[test]
fn token_exists_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(token_exists(addr.clone(), TOKEN_ID), Ok(Assets::asset_exists(TOKEN_ID)));

		// Tokens in circulation.
		create_asset(addr.clone(), TOKEN_ID, 1);
		assert_eq!(token_exists(addr.clone(), TOKEN_ID), Ok(Assets::asset_exists(TOKEN_ID)));
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Asset does not exist.
		assert_eq!(mint(addr.clone(), 1, BOB, amount), Err(Token(UnknownAsset)));
		let asset = create_asset(ALICE, 1, 1);
		// Minting can only be done by the owner.
		assert_eq!(mint(addr.clone(), asset, BOB, 1), Err(Module { index: 52, error: [2, 0] }));
		let asset = create_asset(addr.clone(), 2, 2);
		// Minimum balance of an asset can not be zero.
		assert_eq!(mint(addr.clone(), asset, BOB, 1), Err(Token(BelowMinimum)));
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(addr.clone(), asset);
		assert_eq!(
			mint(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [16, 0] })
		);
		thaw_asset(addr.clone(), asset);
		// Successful mint.
		let balance_before_mint = Assets::balance(asset, &BOB);
		assert_ok!(mint(addr.clone(), asset, BOB, amount));
		let balance_after_mint = Assets::balance(asset, &BOB);
		assert_eq!(balance_after_mint, balance_before_mint + amount);
		// Account can not hold more tokens than Balance::MAX.
		assert_eq!(mint(addr.clone(), asset, BOB, Balance::MAX,), Err(Arithmetic(Overflow)));
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(addr.clone(), asset);
		assert_eq!(
			mint(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [16, 0] })
		);
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Asset does not exist.
		assert_eq!(burn(addr.clone(), 1, BOB, amount), Err(Module { index: 52, error: [3, 0] }));
		let asset = create_asset(ALICE, 1, 1);
		// Bob has no tokens and thus pallet assets doesn't know the account.
		assert_eq!(burn(addr.clone(), asset, BOB, 1), Err(Module { index: 52, error: [1, 0] }));
		// Burning can only be done by the manager.
		mint_asset(ALICE, asset, BOB, amount);
		assert_eq!(burn(addr.clone(), asset, BOB, 1), Err(Module { index: 52, error: [2, 0] }));
		let asset = create_asset_and_mint_to(addr.clone(), 2, BOB, amount);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(addr.clone(), asset);
		assert_eq!(
			burn(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [16, 0] })
		);
		thaw_asset(addr.clone(), asset);
		// Successful mint.
		let balance_before_burn = Assets::balance(asset, &BOB);
		assert_ok!(burn(addr.clone(), asset, BOB, amount));
		let balance_after_burn = Assets::balance(asset, &BOB);
		assert_eq!(balance_after_burn, balance_before_burn - amount);
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(addr.clone(), asset);
		assert_eq!(
			burn(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: [17, 0] })
		);
	});
}

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type TokenIdOf<T> = <pallet_assets::Pallet<T, AssetsInstanceOf<T>> as Inspect<
	<T as frame_system::Config>::AccountId,
>>::AssetId;
type TokenIdParameterOf<T> = <T as pallet_assets::Config<AssetsInstanceOf<T>>>::AssetIdParameter;
type AssetsOf<T> = pallet_assets::Pallet<T, AssetsInstanceOf<T>>;
type AssetsInstanceOf<T> = <T as Config>::AssetsInstance;
type AssetsWeightInfoOf<T> = <T as pallet_assets::Config<AssetsInstanceOf<T>>>::WeightInfo;
type BalanceOf<T> = <pallet_assets::Pallet<T, AssetsInstanceOf<T>> as Inspect<
	<T as frame_system::Config>::AccountId,
>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use core::cmp::Ordering::*;

	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo, WithPostDispatchInfo},
		pallet_prelude::*,
		traits::fungibles::approvals::Inspect as ApprovalInspect,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{StaticLookup, Zero},
		Saturating,
	};
	use sp_std::vec::Vec;

	use super::*;

	/// State reads for the fungibles API with required input.
	#[derive(Encode, Decode, Debug, MaxEncodedLen)]
	#[repr(u8)]
	#[allow(clippy::unnecessary_cast)]
	pub enum Read<T: Config> {
		/// Total token supply for a specified token.
		#[codec(index = 0)]
		TotalSupply(TokenIdOf<T>),
		/// Account balance for a specified `token` and `owner`.
		#[codec(index = 1)]
		BalanceOf {
			/// The token.
			token: TokenIdOf<T>,
			/// The owner of the token.
			owner: AccountIdOf<T>,
		},
		/// Allowance for a `spender` approved by an `owner`, for a specified `token`.
		#[codec(index = 2)]
		Allowance {
			/// The token.
			token: TokenIdOf<T>,
			/// The owner of the token.
			owner: AccountIdOf<T>,
			/// The spender with an allowance.
			spender: AccountIdOf<T>,
		},
		/// Name of the specified token.
		#[codec(index = 8)]
		TokenName(TokenIdOf<T>),
		/// Symbol for the specified token.
		#[codec(index = 9)]
		TokenSymbol(TokenIdOf<T>),
		/// Decimals for the specified token.
		#[codec(index = 10)]
		TokenDecimals(TokenIdOf<T>),
		/// Check if a specified token exists.
		#[codec(index = 18)]
		TokenExists(TokenIdOf<T>),
	}

	/// Results of state reads for the fungibles API.
	#[derive(Debug)]
	pub enum ReadResult<T: Config> {
		/// Total token supply for a specified token.
		TotalSupply(BalanceOf<T>),
		/// Account balance for a specified token and owner.
		BalanceOf(BalanceOf<T>),
		/// Allowance for a spender approved by an owner, for a specified token.
		Allowance(BalanceOf<T>),
		/// Name of the specified token.
		TokenName(Vec<u8>),
		/// Symbol for the specified token.
		TokenSymbol(Vec<u8>),
		/// Decimals for the specified token.
		TokenDecimals(u8),
		/// Whether the specified token exists.
		TokenExists(bool),
	}

	impl<T: Config> ReadResult<T> {
		/// Encodes the result.
		pub fn encode(&self) -> Vec<u8> {
			use ReadResult::*;
			match self {
				TotalSupply(result) => result.encode(),
				BalanceOf(result) => result.encode(),
				Allowance(result) => result.encode(),
				TokenName(result) => result.encode(),
				TokenSymbol(result) => result.encode(),
				TokenDecimals(result) => result.encode(),
				TokenExists(result) => result.encode(),
			}
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config<Self::AssetsInstance> {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The instance of pallet assets it is tightly coupled to.
		type AssetsInstance;
		/// Weight information for dispatchables in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event emitted when allowance by `owner` to `spender` changes.
		Approval {
			/// The token.
			token: TokenIdOf<T>,
			/// The owner providing the allowance.
			owner: AccountIdOf<T>,
			/// The beneficiary of the allowance.
			spender: AccountIdOf<T>,
			/// The new allowance amount.
			value: BalanceOf<T>,
		},
		/// Event emitted when a token transfer occurs.
		Transfer {
			/// The token.
			token: TokenIdOf<T>,
			/// The source of the transfer. `None` when minting.
			from: Option<AccountIdOf<T>>,
			/// The recipient of the transfer. `None` when burning.
			to: Option<AccountIdOf<T>>,
			/// The amount transferred (or minted/burned).
			value: BalanceOf<T>,
		},
		/// Event emitted when an token is created.
		Create {
			/// The token identifier.
			id: TokenIdOf<T>,
			/// The creator of the token.
			creator: AccountIdOf<T>,
			/// The administrator of the token.
			admin: AccountIdOf<T>,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfers `value` amount of tokens from the caller's account to account `to`.
		///
		/// # Parameters
		/// - `token` - The token to transfer.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[pallet::call_index(3)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::transfer_keep_alive())]
		pub fn transfer(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			to: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			let from = ensure_signed(origin.clone())?;
			AssetsOf::<T>::transfer_keep_alive(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(to.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { token, from: Some(from), to: Some(to), value });
			Ok(())
		}

		/// Transfers `value` amount tokens on behalf of `from` to account `to` with additional
		/// `data` in unspecified format.
		///
		/// # Parameters
		/// - `token` - The token to transfer.
		/// - `from` - The account from which the token balance will be withdrawn.
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[pallet::call_index(4)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::transfer_approved())]
		pub fn transfer_from(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			from: AccountIdOf<T>,
			to: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::transfer_approved(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(from.clone()),
				T::Lookup::unlookup(to.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { token, from: Some(from), to: Some(to), value });
			Ok(())
		}

		/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
		///
		/// # Parameters
		/// - `token` - The token to approve.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to approve.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 1))]
		pub fn approve(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 0)))?;
			let current_allowance = AssetsOf::<T>::allowance(token.clone(), &owner, &spender);

			let weight = match value.cmp(&current_allowance) {
				// If the new value is equal to the current allowance, do nothing.
				Equal => Self::weight_approve(0, 0),
				// If the new value is greater than the current allowance, approve the difference
				// because `approve_transfer` works additively (see `pallet-assets`).
				Greater => {
					AssetsOf::<T>::approve_transfer(
						origin,
						token.clone().into(),
						T::Lookup::unlookup(spender.clone()),
						value.saturating_sub(current_allowance),
					)
					.map_err(|e| e.with_weight(Self::weight_approve(1, 0)))?;
					Self::weight_approve(1, 0)
				},
				// If the new value is less than the current allowance, cancel the approval and
				// set the new value.
				Less => {
					let token_param: TokenIdParameterOf<T> = token.clone().into();
					let spender_source = T::Lookup::unlookup(spender.clone());
					AssetsOf::<T>::cancel_approval(
						origin.clone(),
						token_param.clone(),
						spender_source.clone(),
					)
					.map_err(|e| e.with_weight(Self::weight_approve(0, 1)))?;
					if value.is_zero() {
						Self::weight_approve(0, 1)
					} else {
						AssetsOf::<T>::approve_transfer(
							origin,
							token_param,
							spender_source,
							value,
						)?;
						Self::weight_approve(1, 1)
					}
				},
			};
			Self::deposit_event(Event::Approval { token, owner, spender, value });
			Ok(Some(weight).into())
		}

		/// Increases the allowance of `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `token` - The token to have an allowance increased.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to increase the allowance by.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 0))]
		pub fn increase_allowance(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 0)))?;
			AssetsOf::<T>::approve_transfer(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(spender.clone()),
				value,
			)
			.map_err(|e| e.with_weight(AssetsWeightInfoOf::<T>::approve_transfer()))?;
			let value = AssetsOf::<T>::allowance(token.clone(), &owner, &spender);
			Self::deposit_event(Event::Approval { token, owner, spender, value });
			Ok(().into())
		}

		/// Decreases the allowance of `spender` by `value` amount of tokens.
		///
		/// # Parameters
		/// - `token` - The token to have an allowance decreased.
		/// - `spender` - The account that is allowed to spend the tokens.
		/// - `value` - The number of tokens to decrease the allowance by.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as Config>::WeightInfo::approve(1, 1))]
		pub fn decrease_allowance(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			spender: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let owner = ensure_signed(origin.clone())
				.map_err(|e| e.with_weight(Self::weight_approve(0, 0)))?;
			if value.is_zero() {
				return Ok(Some(Self::weight_approve(0, 0)).into());
			}
			let current_allowance = AssetsOf::<T>::allowance(token.clone(), &owner, &spender);
			let spender_source = T::Lookup::unlookup(spender.clone());
			let token_param: TokenIdParameterOf<T> = token.clone().into();

			// Cancel the approval and set the new value if `new_allowance` is more than zero.
			AssetsOf::<T>::cancel_approval(
				origin.clone(),
				token_param.clone(),
				spender_source.clone(),
			)
			.map_err(|e| e.with_weight(Self::weight_approve(0, 1)))?;
			let new_allowance = current_allowance.saturating_sub(value);
			let weight = if new_allowance.is_zero() {
				Self::weight_approve(0, 1)
			} else {
				AssetsOf::<T>::approve_transfer(
					origin,
					token_param,
					spender_source,
					new_allowance,
				)?;
				Self::weight_approve(1, 1)
			};
			Self::deposit_event(Event::Approval { token, owner, spender, value: new_allowance });
			Ok(Some(weight).into())
		}

		/// Create a new token with a given identifier.
		///
		/// # Parameters
		/// - `id` - The identifier of the token.
		/// - `admin` - The account that will administer the token.
		/// - `min_balance` - The minimum balance required for accounts holding this token.
		#[pallet::call_index(11)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::create())]
		pub fn create(
			origin: OriginFor<T>,
			id: TokenIdOf<T>,
			admin: AccountIdOf<T>,
			min_balance: BalanceOf<T>,
		) -> DispatchResult {
			let creator = ensure_signed(origin.clone())?;
			AssetsOf::<T>::create(
				origin,
				id.clone().into(),
				T::Lookup::unlookup(admin.clone()),
				min_balance,
			)?;
			Self::deposit_event(Event::Create { id, creator, admin });
			Ok(())
		}

		/// Start the process of destroying a token.
		///
		/// # Parameters
		/// - `token` - The token to be destroyed.
		#[pallet::call_index(12)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::start_destroy())]
		pub fn start_destroy(origin: OriginFor<T>, token: TokenIdOf<T>) -> DispatchResult {
			AssetsOf::<T>::start_destroy(origin, token.into())
		}

		/// Set the metadata for a token.
		///
		/// # Parameters
		/// - `token`: The token to update.
		/// - `name`: The user friendly name of this token.
		/// - `symbol`: The exchange symbol for this token.
		/// - `decimals`: The number of decimals this token uses to represent one unit.
		#[pallet::call_index(16)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::set_metadata(name.len() as u32, symbol.len() as u32))]
		pub fn set_metadata(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			name: Vec<u8>,
			symbol: Vec<u8>,
			decimals: u8,
		) -> DispatchResult {
			AssetsOf::<T>::set_metadata(origin, token.into(), name, symbol, decimals)
		}

		/// Clear the metadata for a token.
		///
		/// # Parameters
		/// - `token` - The token to update.
		#[pallet::call_index(17)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::clear_metadata())]
		pub fn clear_metadata(origin: OriginFor<T>, token: TokenIdOf<T>) -> DispatchResult {
			AssetsOf::<T>::clear_metadata(origin, token.into())
		}

		/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
		/// supply.
		///
		/// # Parameters
		/// - `token` - The token to mint.
		/// - `account` - The account to be credited with the created tokens.
		/// - `value` - The number of tokens to mint.
		#[pallet::call_index(19)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::mint())]
		pub fn mint(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			account: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::mint(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(account.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { token, from: None, to: Some(account), value });
			Ok(())
		}

		/// Destroys `value` amount of tokens from `account`, reducing the total supply.
		///
		/// # Parameters
		/// - `token` - the token to burn.
		/// - `account` - The account from which the tokens will be destroyed.
		/// - `value` - The number of tokens to destroy.
		#[pallet::call_index(20)]
		#[pallet::weight(AssetsWeightInfoOf::<T>::burn())]
		pub fn burn(
			origin: OriginFor<T>,
			token: TokenIdOf<T>,
			account: AccountIdOf<T>,
			value: BalanceOf<T>,
		) -> DispatchResult {
			AssetsOf::<T>::burn(
				origin,
				token.clone().into(),
				T::Lookup::unlookup(account.clone()),
				value,
			)?;
			Self::deposit_event(Event::Transfer { token, from: Some(account), to: None, value });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn weight_approve(approve: u32, cancel: u32) -> Weight {
			<T as Config>::WeightInfo::approve(cancel, approve)
		}
	}

	impl<T: Config> crate::Read for Pallet<T> {
		/// The type of read requested.
		type Read = Read<T>;
		/// The type or result returned.
		type Result = ReadResult<T>;

		/// Determines the weight of the requested read, used to charge the appropriate weight
		/// before the read is performed.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn weight(_request: &Self::Read) -> Weight {
			// TODO: match on request and return benchmarked weight
			T::DbWeight::get().reads(1_u64)
		}

		/// Performs the requested read and returns the result.
		///
		/// # Parameters
		/// - `request` - The read request.
		fn read(request: Self::Read) -> Self::Result {
			use Read::*;
			match request {
				TotalSupply(token) => ReadResult::TotalSupply(AssetsOf::<T>::total_supply(token)),
				BalanceOf { token, owner } => {
					ReadResult::BalanceOf(AssetsOf::<T>::balance(token, owner))
				},
				Allowance { token, owner, spender } => {
					ReadResult::Allowance(AssetsOf::<T>::allowance(token, &owner, &spender))
				},
				TokenName(token) => ReadResult::TokenName(<AssetsOf<T> as MetadataInspect<
					AccountIdOf<T>,
				>>::name(token)),
				TokenSymbol(token) => ReadResult::TokenSymbol(<AssetsOf<T> as MetadataInspect<
					AccountIdOf<T>,
				>>::symbol(token)),
				TokenDecimals(token) => ReadResult::TokenDecimals(
					<AssetsOf<T> as MetadataInspect<AccountIdOf<T>>>::decimals(token),
				),
				TokenExists(token) => ReadResult::TokenExists(AssetsOf::<T>::asset_exists(token)),
			}
		}
	}
}

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

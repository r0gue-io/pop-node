use pop_api::fungibles::events::{
	Approval, Created, DestroyStarted, MetadataCleared, MetadataSet, Transfer,
};
use pop_primitives::{ArithmeticError::*, Error, Error::*, TokenError::*, TokenId};
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
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(total_supply(&addr, TOKEN_ID), Ok(Assets::total_supply(TOKEN_ID)));
		assert_eq!(total_supply(&addr, TOKEN_ID), Ok(0));

		// Tokens in circulation.
		assets::create_and_mint_to(&addr, TOKEN_ID, &BOB, 100);
		assert_eq!(total_supply(&addr, TOKEN_ID), Ok(Assets::total_supply(TOKEN_ID)));
		assert_eq!(total_supply(&addr, TOKEN_ID), Ok(100));
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(balance_of(&addr, TOKEN_ID, BOB), Ok(Assets::balance(TOKEN_ID, BOB)));
		assert_eq!(balance_of(&addr, TOKEN_ID, BOB), Ok(0));

		// Tokens in circulation.
		assets::create_and_mint_to(&addr, TOKEN_ID, &BOB, 100);
		assert_eq!(balance_of(&addr, TOKEN_ID, BOB), Ok(Assets::balance(TOKEN_ID, BOB)));
		assert_eq!(balance_of(&addr, TOKEN_ID, BOB), Ok(100));
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(
			allowance(&addr, TOKEN_ID, BOB, ALICE),
			Ok(Assets::allowance(TOKEN_ID, &BOB, &ALICE))
		);
		assert_eq!(allowance(&addr, TOKEN_ID, BOB, ALICE), Ok(0));

		// Tokens in circulation.
		assets::create_mint_and_approve(&addr, TOKEN_ID, &BOB, 100, &ALICE, 50);
		assert_eq!(
			allowance(&addr, TOKEN_ID, BOB, ALICE),
			Ok(Assets::allowance(TOKEN_ID, &BOB, &ALICE))
		);
		assert_eq!(allowance(&addr, TOKEN_ID, BOB, ALICE), Ok(50));
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Token does not exist.
		assert_eq!(transfer(&addr, 1, BOB, amount), Err(Module { index: 52, error: [3, 0] }));
		// Create token with Alice as owner and mint `amount` to contract address.
		let token = assets::create_and_mint_to(&ALICE, 1, &addr, amount);
		// Token is not live, i.e. frozen or being destroyed.
		assets::freeze(&ALICE, token);
		assert_eq!(transfer(&addr, token, BOB, amount), Err(Module { index: 52, error: [16, 0] }));
		assets::thaw(&ALICE, token);
		// Not enough balance.
		assert_eq!(
			transfer(&addr, token, BOB, amount + 1 * UNIT),
			Err(Module { index: 52, error: [0, 0] })
		);
		// Not enough balance due to ED.
		assert_eq!(transfer(&addr, token, BOB, amount), Err(Module { index: 52, error: [0, 0] }));
		// Successful transfer.
		let balance_before_transfer = Assets::balance(token, &BOB);
		assert_ok!(transfer(&addr, token, BOB, amount / 2));
		let balance_after_transfer = Assets::balance(token, &BOB);
		assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
		// Successfully emit event.
		let from = account_id_from_slice(addr.as_ref());
		let to = account_id_from_slice(BOB.as_ref());
		let expected = Transfer { from: Some(from), to: Some(to), value: amount / 2 }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// Transfer token to account that does not exist.
		assert_eq!(transfer(&addr, token, FERDIE, amount / 4), Err(Token(CannotCreate)));
		// Token is not live, i.e. frozen or being destroyed.
		assets::start_destroy(&ALICE, token);
		assert_eq!(
			transfer(&addr, token, BOB, amount / 4),
			Err(Module { index: 52, error: [16, 0] })
		);
	});
}

#[test]
fn transfer_from_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Token does not exist.
		assert_eq!(
			transfer_from(&addr, 1, ALICE, BOB, amount / 2),
			Err(Module { index: 52, error: [3, 0] }),
		);
		// Create token with Alice as owner and mint `amount` to contract address.
		let token = assets::create_and_mint_to(&ALICE, 1, &ALICE, amount);
		// Unapproved transfer.
		assert_eq!(
			transfer_from(&addr, token, ALICE, BOB, amount / 2),
			Err(Module { index: 52, error: [10, 0] })
		);
		assert_ok!(Assets::approve_transfer(
			RuntimeOrigin::signed(ALICE.into()),
			token.into(),
			addr.clone().into(),
			amount + 1 * UNIT,
		));
		// Token is not live, i.e. frozen or being destroyed.
		assets::freeze(&ALICE, token);
		assert_eq!(
			transfer_from(&addr, token, ALICE, BOB, amount),
			Err(Module { index: 52, error: [16, 0] }),
		);
		assets::thaw(&ALICE, token);
		// Not enough balance.
		assert_eq!(
			transfer_from(&addr, token, ALICE, BOB, amount + 1 * UNIT),
			Err(Module { index: 52, error: [0, 0] }),
		);
		// Successful transfer.
		let balance_before_transfer = Assets::balance(token, &BOB);
		assert_ok!(transfer_from(&addr, token, ALICE, BOB, amount / 2));
		let balance_after_transfer = Assets::balance(token, &BOB);
		assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
		// Successfully emit event.
		let from = account_id_from_slice(ALICE.as_ref());
		let to = account_id_from_slice(BOB.as_ref());
		let expected = Transfer { from: Some(from), to: Some(to), value: amount / 2 }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
	});
}

#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, 0, vec![]);
		let amount: Balance = 100 * UNIT;

		// Token does not exist.
		assert_eq!(approve(&addr, 0, &BOB, amount), Err(Module { index: 52, error: [3, 0] }));
		let token = assets::create_and_mint_to(&ALICE, 0, &addr, amount);
		assert_eq!(approve(&addr, token, &BOB, amount), Err(ConsumerRemaining));
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![1]);
		// Create token with Alice as owner and mint `amount` to contract address.
		let token = assets::create_and_mint_to(&ALICE, 1, &addr, amount);
		// Token is not live, i.e. frozen or being destroyed.
		assets::freeze(&ALICE, token);
		assert_eq!(approve(&addr, token, &BOB, amount), Err(Module { index: 52, error: [16, 0] }));
		assets::thaw(&ALICE, token);
		// Successful approvals.
		assert_eq!(0, Assets::allowance(token, &addr, &BOB));
		assert_ok!(approve(&addr, token, &BOB, amount));
		assert_eq!(Assets::allowance(token, &addr, &BOB), amount);
		// Successfully emit event.
		let owner = account_id_from_slice(addr.as_ref());
		let spender = account_id_from_slice(BOB.as_ref());
		let expected = Approval { owner, spender, value: amount }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// Non-additive, sets new value.
		assert_ok!(approve(&addr, token, &BOB, amount / 2));
		assert_eq!(Assets::allowance(token, &addr, &BOB), amount / 2);
		// Successfully emit event.
		let owner = account_id_from_slice(addr.as_ref());
		let spender = account_id_from_slice(BOB.as_ref());
		let expected = Approval { owner, spender, value: amount / 2 }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// Token is not live, i.e. frozen or being destroyed.
		assets::start_destroy(&ALICE, token);
		assert_eq!(approve(&addr, token, &BOB, amount), Err(Module { index: 52, error: [16, 0] }));
	});
}

#[test]
fn increase_allowance_works() {
	new_test_ext().execute_with(|| {
		let amount: Balance = 100 * UNIT;
		// Instantiate a contract without balance - test `ConsumerRemaining.
		let addr = instantiate(CONTRACT, 0, vec![]);
		// Token does not exist.
		assert_eq!(
			increase_allowance(&addr, 0, &BOB, amount),
			Err(Module { index: 52, error: [3, 0] })
		);
		let token = assets::create_and_mint_to(&ALICE, 0, &addr, amount);
		assert_eq!(increase_allowance(&addr, token, &BOB, amount), Err(ConsumerRemaining));

		// Instantiate a contract with balance.
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![1]);
		// Create token with Alice as owner and mint `amount` to contract address.
		let token = assets::create_and_mint_to(&ALICE, 1, &addr, amount);
		// Token is not live, i.e. frozen or being destroyed.
		assets::freeze(&ALICE, token);
		assert_eq!(
			increase_allowance(&addr, token, &BOB, amount),
			Err(Module { index: 52, error: [16, 0] })
		);
		assets::thaw(&ALICE, token);
		// Successful approvals:
		assert_eq!(0, Assets::allowance(token, &addr, &BOB));
		assert_ok!(increase_allowance(&addr, token, &BOB, amount));
		assert_eq!(Assets::allowance(token, &addr, &BOB), amount);
		// Additive.
		assert_ok!(increase_allowance(&addr, token, &BOB, amount));
		assert_eq!(Assets::allowance(token, &addr, &BOB), amount * 2);
		// Token is not live, i.e. frozen or being destroyed.
		assets::start_destroy(&ALICE, token);
		assert_eq!(
			increase_allowance(&addr, token, &BOB, amount),
			Err(Module { index: 52, error: [16, 0] })
		);
	});
}

#[test]
fn decrease_allowance_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Create token and mint `amount` to contract address, then approve Bob to spend `amount`.
		let token = assets::create_mint_and_approve(&addr, 0, &addr, amount, &BOB, amount);
		// Token is not live, i.e. frozen or being destroyed.
		assets::freeze(&addr, token);
		assert_eq!(
			decrease_allowance(&addr, token, &BOB, amount),
			Err(Module { index: 52, error: [16, 0] }),
		);
		assets::thaw(&addr, token);
		// Successfully decrease allowance.
		let allowance_before = Assets::allowance(token, &addr, &BOB);
		assert_ok!(decrease_allowance(&addr, 0, &BOB, amount / 2 - 1 * UNIT));
		let allowance_after = Assets::allowance(token, &addr, &BOB);
		assert_eq!(allowance_before - allowance_after, amount / 2 - 1 * UNIT);
		// Token is not live, i.e. frozen or being destroyed.
		assets::start_destroy(&addr, token);
		assert_eq!(
			decrease_allowance(&addr, token, &BOB, amount),
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
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let name: Vec<u8> = vec![11, 12, 13];
		let symbol: Vec<u8> = vec![21, 22, 23];
		let decimals: u8 = 69;

		// Token does not exist.
		assert_eq!(token_name(&addr, TOKEN_ID), Ok(Assets::name(TOKEN_ID)));
		assert_eq!(token_name(&addr, TOKEN_ID), Ok(Vec::<u8>::new()));
		assert_eq!(token_symbol(&addr, TOKEN_ID), Ok(Assets::symbol(TOKEN_ID)));
		assert_eq!(token_symbol(&addr, TOKEN_ID), Ok(Vec::<u8>::new()));
		assert_eq!(token_decimals(&addr, TOKEN_ID), Ok(Assets::decimals(TOKEN_ID)));
		assert_eq!(token_decimals(&addr, TOKEN_ID), Ok(0));
		// Create Token.
		assets::create_and_set_metadata(&addr, TOKEN_ID, name.clone(), symbol.clone(), decimals);
		assert_eq!(token_name(&addr, TOKEN_ID), Ok(Assets::name(TOKEN_ID)));
		assert_eq!(token_name(&addr, TOKEN_ID), Ok(name));
		assert_eq!(token_symbol(&addr, TOKEN_ID), Ok(Assets::symbol(TOKEN_ID)));
		assert_eq!(token_symbol(&addr, TOKEN_ID), Ok(symbol));
		assert_eq!(token_decimals(&addr, TOKEN_ID), Ok(Assets::decimals(TOKEN_ID)));
		assert_eq!(token_decimals(&addr, TOKEN_ID), Ok(decimals));
	});
}
/// 3. Management:
/// - create
/// - start_destroy
/// - set_metadata
/// - clear_metadata
/// - token_exists

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a contract without balance for fees.
		let addr = instantiate(CONTRACT, 0, vec![0]);
		// No balance to pay for fees.
		assert_eq!(create(&addr, TOKEN_ID, &addr, 1), Err(Module { index: 10, error: [2, 0] }),);

		// Instantiate a contract without balance for deposit.
		let addr = instantiate(CONTRACT, 100, vec![1]);
		// No balance to pay the deposit.
		assert_eq!(create(&addr, TOKEN_ID, &addr, 1), Err(Module { index: 10, error: [2, 0] }),);

		// Instantiate a contract with enough balance.
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![2]);
		assert_eq!(create(&addr, TOKEN_ID, &BOB, 0), Err(Module { index: 52, error: [7, 0] }),);
		// The minimal balance for a token must be non zero.
		assert_eq!(create(&addr, TOKEN_ID, &BOB, 0), Err(Module { index: 52, error: [7, 0] }),);
		// Create token successfully.
		assert_ok!(create(&addr, TOKEN_ID, &BOB, 1));
		assert_eq!(Assets::owner(TOKEN_ID), Some(addr.clone()));
		// Successfully emit event.
		let admin = account_id_from_slice(BOB.as_ref());
		let expected = Created { id: TOKEN_ID, creator: admin, admin }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// Token ID is already taken.
		assert_eq!(create(&addr, TOKEN_ID, &BOB, 1), Err(Module { index: 52, error: [5, 0] }),);
	});
}

// Testing a contract that creates a token in the constructor.
#[test]
fn instantiate_and_create_fungible_works() {
	new_test_ext().execute_with(|| {
		let contract =
			"contracts/create_token_in_constructor/target/ink/create_token_in_constructor.wasm";
		// Token already exists.
		assets::create(&ALICE, 0, 1);
		assert_eq!(
			instantiate_and_create_fungible(contract, 0, 1),
			Err(Module { index: 52, error: [5, 0] })
		);
		// Successfully create a token when instantiating the contract.
		let result_with_address = instantiate_and_create_fungible(contract, TOKEN_ID, 1);
		let instantiator = result_with_address.clone().ok();
		assert_ok!(result_with_address);
		assert_eq!(&Assets::owner(TOKEN_ID), &instantiator);
		assert!(Assets::asset_exists(TOKEN_ID));
		// Successfully emit event.
		let instantiator = account_id_from_slice(instantiator.unwrap().as_ref());
		let expected =
			Created { id: TOKEN_ID, creator: instantiator.clone(), admin: instantiator }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
	});
}

#[test]
fn start_destroy_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![2]);

		// Token does not exist.
		assert_eq!(start_destroy(&addr, TOKEN_ID), Err(Module { index: 52, error: [3, 0] }),);
		// Create tokens where contract is not the owner.
		let token = assets::create(&ALICE, 0, 1);
		// No Permission.
		assert_eq!(start_destroy(&addr, token), Err(Module { index: 52, error: [2, 0] }),);
		let token = assets::create(&addr, TOKEN_ID, 1);
		assert_ok!(start_destroy(&addr, token));
		// Successfully emit event.
		let expected = DestroyStarted { token: TOKEN_ID }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
	});
}

#[test]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let name = vec![42];
		let symbol = vec![42];
		let decimals = 42u8;
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Token does not exist.
		assert_eq!(
			set_metadata(&addr, TOKEN_ID, vec![0], vec![0], 0u8),
			Err(Module { index: 52, error: [3, 0] }),
		);
		// Create token where contract is not the owner.
		let token = assets::create(&ALICE, 0, 1);
		// No Permission.
		assert_eq!(
			set_metadata(&addr, token, vec![0], vec![0], 0u8),
			Err(Module { index: 52, error: [2, 0] }),
		);
		let token = assets::create(&addr, TOKEN_ID, 1);
		// Token is not live, i.e. frozen or being destroyed.
		assets::freeze(&addr, token);
		assert_eq!(
			set_metadata(&addr, TOKEN_ID, vec![0], vec![0], 0u8),
			Err(Module { index: 52, error: [16, 0] }),
		);
		assets::thaw(&addr, token);
		// TODO: calling the below with a vector of length `100_000` errors in pallet contracts
		//  `OutputBufferTooSmall. Added to security analysis issue #131 to revisit.
		// Set bad metadata - too large values.
		assert_eq!(
			set_metadata(&addr, TOKEN_ID, vec![0; 1000], vec![0; 1000], 0u8),
			Err(Module { index: 52, error: [9, 0] }),
		);
		// Set metadata successfully.
		assert_ok!(set_metadata(&addr, TOKEN_ID, name.clone(), symbol.clone(), decimals));
		// Successfully emit event.
		let expected = MetadataSet { token: TOKEN_ID, name, symbol, decimals }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// Token is not live, i.e. frozen or being destroyed.
		assets::start_destroy(&addr, token);
		assert_eq!(
			set_metadata(&addr, TOKEN_ID, vec![0], vec![0], 0),
			Err(Module { index: 52, error: [16, 0] }),
		);
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let name = vec![42];
		let symbol = vec![42];
		let decimals = 42u8;
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Token does not exist.
		assert_eq!(clear_metadata(&addr, 0), Err(Module { index: 52, error: [3, 0] }),);
		// Create token where contract is not the owner.
		let token = assets::create_and_set_metadata(&ALICE, 0, vec![0], vec![0], 0);
		// No Permission.
		assert_eq!(clear_metadata(&addr, token), Err(Module { index: 52, error: [2, 0] }),);
		let token = assets::create(&addr, TOKEN_ID, 1);
		// Token is not live, i.e. frozen or being destroyed.
		assets::freeze(&addr, token);
		assert_eq!(clear_metadata(&addr, token), Err(Module { index: 52, error: [16, 0] }),);
		assets::thaw(&addr, token);
		// No metadata set.
		assert_eq!(clear_metadata(&addr, token), Err(Module { index: 52, error: [3, 0] }),);
		assets::set_metadata(&addr, token, name, symbol, decimals);
		// Clear metadata successfully.
		assert_ok!(clear_metadata(&addr, TOKEN_ID));
		// Successfully emit event.
		let expected = MetadataCleared { token: TOKEN_ID }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// Token is not live, i.e. frozen or being destroyed.
		assets::start_destroy(&addr, token);
		assert_eq!(
			set_metadata(&addr, TOKEN_ID, vec![0], vec![0], decimals),
			Err(Module { index: 52, error: [16, 0] }),
		);
	});
}

#[test]
fn token_exists_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(token_exists(&addr, TOKEN_ID), Ok(Assets::asset_exists(TOKEN_ID)));

		// Tokens in circulation.
		assets::create(&addr, TOKEN_ID, 1);
		assert_eq!(token_exists(&addr, TOKEN_ID), Ok(Assets::asset_exists(TOKEN_ID)));
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Token does not exist.
		assert_eq!(mint(&addr, 1, &BOB, amount), Err(Token(UnknownAsset)));
		let token = assets::create(&ALICE, 1, 1);
		// Minting can only be done by the owner.
		assert_eq!(mint(&addr, token, &BOB, 1), Err(Module { index: 52, error: [2, 0] }));
		let token = assets::create(&addr, 2, 2);
		// Minimum balance of a token can not be zero.
		assert_eq!(mint(&addr, token, &BOB, 1), Err(Token(BelowMinimum)));
		// Token is not live, i.e. frozen or being destroyed.
		assets::freeze(&addr, token);
		assert_eq!(mint(&addr, token, &BOB, amount), Err(Module { index: 52, error: [16, 0] }));
		assets::thaw(&addr, token);
		// Successful mint.
		let balance_before_mint = Assets::balance(token, &BOB);
		assert_ok!(mint(&addr, token, &BOB, amount));
		let balance_after_mint = Assets::balance(token, &BOB);
		assert_eq!(balance_after_mint, balance_before_mint + amount);
		// Account can not hold more tokens than Balance::MAX.
		assert_eq!(mint(&addr, token, &BOB, Balance::MAX,), Err(Arithmetic(Overflow)));
		// Token is not live, i.e. frozen or being destroyed.
		assets::start_destroy(&addr, token);
		assert_eq!(mint(&addr, token, &BOB, amount), Err(Module { index: 52, error: [16, 0] }));
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Token does not exist.
		assert_eq!(burn(&addr, 1, &BOB, amount), Err(Module { index: 52, error: [3, 0] }));
		let token = assets::create(&ALICE, 1, 1);
		// Bob has no tokens and therefore doesn't exist.
		assert_eq!(burn(&addr, token, &BOB, 1), Err(Module { index: 52, error: [1, 0] }));
		// Burning can only be done by the manager.
		assets::mint(&ALICE, token, &BOB, amount);
		assert_eq!(burn(&addr, token, &BOB, 1), Err(Module { index: 52, error: [2, 0] }));
		let token = assets::create_and_mint_to(&addr, 2, &BOB, amount);
		// Token is not live, i.e. frozen or being destroyed.
		assets::freeze(&addr, token);
		assert_eq!(burn(&addr, token, &BOB, amount), Err(Module { index: 52, error: [16, 0] }));
		assets::thaw(&addr, token);
		// Successful mint.
		let balance_before_burn = Assets::balance(token, &BOB);
		assert_ok!(burn(&addr, token, &BOB, amount));
		let balance_after_burn = Assets::balance(token, &BOB);
		assert_eq!(balance_after_burn, balance_before_burn - amount);
		// Token is not live, i.e. frozen or being destroyed.
		assets::start_destroy(&addr, token);
		assert_eq!(burn(&addr, token, &BOB, amount), Err(Module { index: 52, error: [17, 0] }));
	});
}

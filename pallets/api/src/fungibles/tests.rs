use crate::mock::*;
use frame_support::{
	assert_ok,
	traits::fungibles::{approvals::Inspect, metadata::Inspect as MetadataInspect},
};

const ASSET: u32 = 42;

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let amount: Balance = 100 * UNIT;
		create_asset_and_mint_to(ALICE, ASSET, ALICE, amount);
		let balance_before_transfer = Assets::balance(ASSET, &BOB);
		assert_ok!(Fungibles::transfer(signed(ALICE), ASSET, BOB, amount / 2));
		let balance_after_transfer = Assets::balance(ASSET, &BOB);
		assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
	});
}

// Non-additive, sets new value.
#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let amount: Balance = 100 * UNIT;
		create_asset_and_mint_to(ALICE, ASSET, ALICE, amount);
		assert_eq!(0, Assets::allowance(ASSET, &ALICE, &BOB));
		assert_ok!(Fungibles::approve(signed(ALICE), ASSET, BOB, amount));
		assert_eq!(Assets::allowance(ASSET, &ALICE, &BOB), amount);
		// Approves an amount to spend that is lower than the current allowance.
		assert_ok!(Fungibles::approve(signed(ALICE), ASSET, BOB, amount / 2));
		assert_eq!(Assets::allowance(ASSET, &ALICE, &BOB), amount / 2);
		// Approves an amount to spend that is higher than the current allowance.
		assert_ok!(Fungibles::approve(signed(ALICE), ASSET, BOB, amount * 2));
		assert_eq!(Assets::allowance(ASSET, &ALICE, &BOB), amount * 2);
		// Sets allowance to zero.
		assert_ok!(Fungibles::approve(signed(ALICE), ASSET, BOB, 0));
		assert_eq!(Assets::allowance(ASSET, &ALICE, &BOB), 0);
	});
}

#[test]
fn increase_allowance_works() {
	new_test_ext().execute_with(|| {
		let amount: Balance = 100 * UNIT;
		create_asset_and_mint_to(ALICE, ASSET, ALICE, amount);
		assert_eq!(0, Assets::allowance(ASSET, &ALICE, &BOB));
		assert_ok!(Fungibles::increase_allowance(signed(ALICE), ASSET, BOB, amount));
		assert_eq!(Assets::allowance(ASSET, &ALICE, &BOB), amount);
		// Additive.
		assert_ok!(Fungibles::increase_allowance(signed(ALICE), ASSET, BOB, amount));
		assert_eq!(Assets::allowance(ASSET, &ALICE, &BOB), amount * 2);
	});
}

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		create_asset_and_mint_to(ALICE, ASSET, ALICE, 100);
		assert_eq!(Assets::total_supply(ASSET), Fungibles::total_supply(ASSET));
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		create_asset_and_mint_to(ALICE, ASSET, ALICE, 100);
		assert_eq!(Assets::balance(ASSET, ALICE), Fungibles::balance_of(ASSET, &ALICE));
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		create_asset_mint_and_approve(ALICE, ASSET, BOB, 100, ALICE, 50);
		assert_eq!(
			Assets::allowance(ASSET, &ALICE, &BOB),
			Fungibles::allowance(ASSET, &ALICE, &BOB)
		);
	});
}

#[test]
fn token_metadata_works() {
	new_test_ext().execute_with(|| {
		let name: Vec<u8> = vec![11, 12, 13];
		let symbol: Vec<u8> = vec![21, 22, 23];
		let decimals: u8 = 69;
		create_asset_and_set_metadata(ALICE, ASSET, name.clone(), symbol.clone(), decimals);
		assert_eq!(Assets::name(ASSET), Fungibles::token_name(ASSET));
		assert_eq!(Assets::symbol(ASSET), Fungibles::token_symbol(ASSET));
		assert_eq!(Assets::decimals(ASSET), Fungibles::token_decimals(ASSET));
	});
}

fn signed(account: AccountId) -> RuntimeOrigin {
	RuntimeOrigin::signed(account)
}

fn create_asset(owner: AccountId, asset_id: AssetId, min_balance: Balance) {
	assert_ok!(Assets::create(signed(owner), asset_id, owner, min_balance));
}

fn mint_asset(owner: AccountId, asset_id: AssetId, to: AccountId, value: Balance) {
	assert_ok!(Assets::mint(signed(owner), asset_id, to, value));
}

fn create_asset_and_mint_to(owner: AccountId, asset_id: AssetId, to: AccountId, value: Balance) {
	create_asset(owner, asset_id, 1);
	mint_asset(owner, asset_id, to, value)
}

fn create_asset_mint_and_approve(
	owner: AccountId,
	asset_id: AssetId,
	to: AccountId,
	mint: Balance,
	spender: AccountId,
	approve: Balance,
) {
	create_asset_and_mint_to(owner, asset_id, to, mint);
	assert_ok!(Assets::approve_transfer(signed(to), asset_id, spender, approve,));
}

fn create_asset_and_set_metadata(
	owner: AccountId,
	asset_id: AssetId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) {
	assert_ok!(Assets::create(signed(owner), asset_id, owner, 100));
	set_metadata_asset(owner, asset_id, name, symbol, decimals);
}

fn set_metadata_asset(
	owner: AccountId,
	asset_id: AssetId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) {
	assert_ok!(Assets::set_metadata(signed(owner), asset_id, name, symbol, decimals));
}

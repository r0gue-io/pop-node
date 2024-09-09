use codec::Encode;
use frame_support::{
	assert_ok,
	sp_runtime::traits::Zero,
	traits::fungibles::{
		approvals::Inspect as ApprovalInspect, metadata::Inspect as MetadataInspect, Inspect,
	},
};

use crate::{fungibles::Read::*, mock::*, Read};

const TOKEN: u32 = 42;

type Event = crate::fungibles::Event<Test>;

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let from = Some(ALICE);
		let to = Some(BOB);

		create_token_and_mint_to(ALICE, token, ALICE, value * 2);
		let balance_before_transfer = Assets::balance(token, &BOB);
		assert_ok!(Fungibles::transfer(signed(ALICE), token, BOB, value));
		let balance_after_transfer = Assets::balance(token, &BOB);
		assert_eq!(balance_after_transfer, balance_before_transfer + value);
		System::assert_last_event(Event::Transfer { token, from, to, value }.into());
	});
}

#[test]
fn transfer_from_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let from = Some(ALICE);
		let to = Some(BOB);

		// Approve CHARLIE to transfer up to `value` to BOB.
		create_token_mint_and_approve(ALICE, token, ALICE, value * 2, CHARLIE, value);
		// Successfully call transfer from.
		let alice_balance_before_transfer = Assets::balance(token, &ALICE);
		let bob_balance_before_transfer = Assets::balance(token, &BOB);
		assert_ok!(Fungibles::transfer_from(signed(CHARLIE), token, ALICE, BOB, value));
		let alice_balance_after_transfer = Assets::balance(token, &ALICE);
		let bob_balance_after_transfer = Assets::balance(token, &BOB);
		// Check that BOB receives the `value` and ALICE `amount` is spent successfully by CHARLIE.
		assert_eq!(bob_balance_after_transfer, bob_balance_before_transfer + value);
		assert_eq!(alice_balance_after_transfer, alice_balance_before_transfer - value);
		System::assert_last_event(Event::Transfer { token, from, to, value }.into());
	});
}

// Non-additive, sets new value.
#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let owner = ALICE;
		let spender = BOB;

		create_token_and_mint_to(ALICE, token, ALICE, value);
		assert_eq!(0, Assets::allowance(token, &ALICE, &BOB));
		assert_ok!(Fungibles::approve(signed(ALICE), token, BOB, value));
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value);
		System::assert_last_event(Event::Approval { token, owner, spender, value }.into());
		// Approves an value to spend that is lower than the current allowance.
		assert_ok!(Fungibles::approve(signed(ALICE), token, BOB, value / 2));
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value / 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value / 2 }.into(),
		);
		// Approves an value to spend that is higher than the current allowance.
		assert_ok!(Fungibles::approve(signed(ALICE), token, BOB, value * 2));
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value * 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value * 2 }.into(),
		);
		// Approves an value to spend that is equal to the current allowance.
		assert_ok!(Fungibles::approve(signed(ALICE), token, BOB, value * 2));
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value * 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value * 2 }.into(),
		);
		// Sets allowance to zero.
		assert_ok!(Fungibles::approve(signed(ALICE), token, BOB, 0));
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), 0);
		System::assert_last_event(Event::Approval { token, owner, spender, value: 0 }.into());
	});
}

#[test]
fn increase_allowance_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let owner = ALICE;
		let spender = BOB;

		create_token_and_mint_to(ALICE, token, ALICE, value);
		assert_eq!(0, Assets::allowance(token, &ALICE, &BOB));
		assert_ok!(Fungibles::increase_allowance(signed(ALICE), token, BOB, value));
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value);
		System::assert_last_event(Event::Approval { token, owner, spender, value }.into());
		// Additive.
		assert_ok!(Fungibles::increase_allowance(signed(ALICE), token, BOB, value));
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value * 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value * 2 }.into(),
		);
	});
}

#[test]
fn decrease_allowance_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let owner = ALICE;
		let spender = BOB;

		create_token_mint_and_approve(ALICE, token, ALICE, value, BOB, value);
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value);
		// Owner balance is not changed if decreased by zero.
		assert_ok!(Fungibles::decrease_allowance(signed(ALICE), token, BOB, 0));
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value);
		// Decrease allowance successfully.
		assert_ok!(Fungibles::decrease_allowance(signed(ALICE), token, BOB, value / 2));
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value / 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value / 2 }.into(),
		);
		// Saturating if current allowance is decreased more than the owner balance.
		assert_ok!(Fungibles::decrease_allowance(signed(ALICE), token, BOB, value));
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), 0);
		System::assert_last_event(Event::Approval { token, owner, spender, value: 0 }.into());
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let id = TOKEN;
		let creator = ALICE;
		let admin = ALICE;

		assert!(!Assets::asset_exists(id));
		assert_ok!(Fungibles::create(signed(creator), id, admin, 100));
		assert!(Assets::asset_exists(id));
		System::assert_last_event(Event::Create { id, creator, admin }.into());
	});
}

#[test]
fn start_destroy_works() {
	new_test_ext().execute_with(|| {
		let token = TOKEN;

		create_token(ALICE, token);
		assert_ok!(Fungibles::start_destroy(signed(ALICE), token));
	});
}

#[test]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let token = TOKEN;
		let name = vec![42];
		let symbol = vec![42];
		let decimals = 42;

		create_token(ALICE, token);
		assert_ok!(Fungibles::set_metadata(
			signed(ALICE),
			token,
			name.clone(),
			symbol.clone(),
			decimals
		));
		assert_eq!(Assets::name(token), name);
		assert_eq!(Assets::symbol(token), symbol);
		assert_eq!(Assets::decimals(token), decimals);
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let token = TOKEN;

		create_token_and_set_metadata(ALICE, token, vec![42], vec![42], 42);
		assert_ok!(Fungibles::clear_metadata(signed(ALICE), token));
		assert!(Assets::name(token).is_empty());
		assert!(Assets::symbol(token).is_empty());
		assert!(Assets::decimals(token).is_zero());
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let from = None;
		let to = Some(BOB);

		create_token(ALICE, token);
		let balance_before_mint = Assets::balance(token, &BOB);
		assert_ok!(Fungibles::mint(signed(ALICE), token, BOB, value));
		let balance_after_mint = Assets::balance(token, &BOB);
		assert_eq!(balance_after_mint, balance_before_mint + value);
		System::assert_last_event(Event::Transfer { token, from, to, value }.into());
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let from = Some(BOB);
		let to = None;

		create_token_and_mint_to(ALICE, token, BOB, value);
		let balance_before_burn = Assets::balance(token, &BOB);
		assert_ok!(Fungibles::burn(signed(ALICE), token, BOB, value));
		let balance_after_burn = Assets::balance(token, &BOB);
		assert_eq!(balance_after_burn, balance_before_burn - value);
		System::assert_last_event(Event::Transfer { token, from, to, value }.into());
	});
}

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		create_token_and_mint_to(ALICE, TOKEN, ALICE, 100);
		assert_eq!(
			Assets::total_supply(TOKEN).encode(),
			Fungibles::read(TotalSupply(TOKEN)).encode()
		);
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		create_token_and_mint_to(ALICE, TOKEN, ALICE, 100);
		assert_eq!(
			Assets::balance(TOKEN, ALICE).encode(),
			Fungibles::read(BalanceOf { token: TOKEN, owner: ALICE }).encode()
		);
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		create_token_mint_and_approve(ALICE, TOKEN, BOB, 100, ALICE, 50);
		assert_eq!(
			Assets::allowance(TOKEN, &ALICE, &BOB).encode(),
			Fungibles::read(Allowance { token: TOKEN, owner: ALICE, spender: BOB }).encode()
		);
	});
}

#[test]
fn token_metadata_works() {
	new_test_ext().execute_with(|| {
		let name: Vec<u8> = vec![11, 12, 13];
		let symbol: Vec<u8> = vec![21, 22, 23];
		let decimals: u8 = 69;
		create_token_and_set_metadata(ALICE, TOKEN, name.clone(), symbol.clone(), decimals);
		assert_eq!(Assets::name(TOKEN).encode(), Fungibles::read(TokenName(TOKEN)).encode());
		assert_eq!(Assets::symbol(TOKEN).encode(), Fungibles::read(TokenSymbol(TOKEN)).encode());
		assert_eq!(
			Assets::decimals(TOKEN).encode(),
			Fungibles::read(TokenDecimals(TOKEN)).encode()
		);
	});
}

#[test]
fn token_exists_works() {
	new_test_ext().execute_with(|| {
		create_token(ALICE, TOKEN);
		assert_eq!(
			Assets::asset_exists(TOKEN).encode(),
			Fungibles::read(TokenExists(TOKEN)).encode()
		);
	});
}

fn signed(account: AccountId) -> RuntimeOrigin {
	RuntimeOrigin::signed(account)
}

fn create_token(owner: AccountId, token: TokenId) {
	assert_ok!(Assets::create(signed(owner), token, owner, 1));
}

fn mint_token(owner: AccountId, token: TokenId, to: AccountId, value: Balance) {
	assert_ok!(Assets::mint(signed(owner), token, to, value));
}

fn create_token_and_mint_to(owner: AccountId, token: TokenId, to: AccountId, value: Balance) {
	create_token(owner, token);
	mint_token(owner, token, to, value)
}

fn create_token_mint_and_approve(
	owner: AccountId,
	token: TokenId,
	to: AccountId,
	mint: Balance,
	spender: AccountId,
	approve: Balance,
) {
	create_token_and_mint_to(owner, token, to, mint);
	assert_ok!(Assets::approve_transfer(signed(to), token, spender, approve,));
}

fn create_token_and_set_metadata(
	owner: AccountId,
	token: TokenId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) {
	assert_ok!(Assets::create(signed(owner), token, owner, 100));
	set_metadata_token(owner, token, name, symbol, decimals);
}

fn set_metadata_token(
	owner: AccountId,
	token: TokenId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) {
	assert_ok!(Assets::set_metadata(signed(owner), token, name, symbol, decimals));
}

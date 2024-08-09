use crate::{fungibles::Read::*, mock::*};
use codec::Encode;
use frame_support::{
	assert_ok,
	traits::fungibles::{
		approvals::Inspect as ApprovalInspect, metadata::Inspect as MetadataInspect, Inspect,
	},
};

const ASSET: u32 = 42;

type Event = crate::fungibles::Event<Test>;

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let id = ASSET;
		let from = Some(ALICE);
		let to = Some(BOB);

		create_asset_and_mint_to(ALICE, id, ALICE, value * 2);
		let balance_before_transfer = Assets::balance(id, &BOB);
		assert_ok!(Fungibles::transfer(signed(ALICE), id, BOB, value));
		let balance_after_transfer = Assets::balance(id, &BOB);
		assert_eq!(balance_after_transfer, balance_before_transfer + value);
		System::assert_last_event(Event::Transfer { id, from, to, value }.into());
	});
}

#[test]
fn transfer_from_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let id = ASSET;
		let from = Some(ALICE);
		let to = Some(BOB);

		// Approve CHARLIE to transfer up to `value` to BOB.
		create_asset_mint_and_approve(ALICE, id, ALICE, value * 2, CHARLIE, value);
		// Successfully call transfer from.
		let alice_balance_before_transfer = Assets::balance(id, &ALICE);
		let bob_balance_before_transfer = Assets::balance(id, &BOB);
		assert_ok!(Fungibles::transfer_from(signed(CHARLIE), id, ALICE, BOB, value));
		let alice_balance_after_transfer = Assets::balance(id, &ALICE);
		let bob_balance_after_transfer = Assets::balance(id, &BOB);
		// Check that BOB receives the `value` and ALICE `amount` is spent successfully by CHARLIE.
		assert_eq!(bob_balance_after_transfer, bob_balance_before_transfer + value);
		assert_eq!(alice_balance_after_transfer, alice_balance_before_transfer - value);
		System::assert_last_event(Event::Transfer { id, from, to, value }.into());
	});
}

// Non-additive, sets new value.
#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let id = ASSET;
		let owner = ALICE;
		let spender = BOB;

		create_asset_and_mint_to(ALICE, id, ALICE, value);
		assert_eq!(0, Assets::allowance(id, &ALICE, &BOB));
		assert_ok!(Fungibles::approve(signed(ALICE), id, BOB, value));
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), value);
		System::assert_last_event(Event::Approval { id, owner, spender, value }.into());
		// Approves an value to spend that is lower than the current allowance.
		assert_ok!(Fungibles::approve(signed(ALICE), id, BOB, value / 2));
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), value / 2);
		System::assert_last_event(Event::Approval { id, owner, spender, value: value / 2 }.into());
		// Approves an value to spend that is higher than the current allowance.
		assert_ok!(Fungibles::approve(signed(ALICE), id, BOB, value * 2));
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), value * 2);
		System::assert_last_event(Event::Approval { id, owner, spender, value: value * 2 }.into());
		// Approves an value to spend that is equal to the current allowance.
		assert_ok!(Fungibles::approve(signed(ALICE), id, BOB, value * 2));
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), value * 2);
		System::assert_last_event(Event::Approval { id, owner, spender, value: value * 2 }.into());
		// Sets allowance to zero.
		assert_ok!(Fungibles::approve(signed(ALICE), id, BOB, 0));
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), 0);
		System::assert_last_event(Event::Approval { id, owner, spender, value: 0 }.into());
	});
}

#[test]
fn increase_allowance_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let id = ASSET;
		let owner = ALICE;
		let spender = BOB;

		create_asset_and_mint_to(ALICE, id, ALICE, value);
		assert_eq!(0, Assets::allowance(id, &ALICE, &BOB));
		assert_ok!(Fungibles::increase_allowance(signed(ALICE), id, BOB, value));
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), value);
		System::assert_last_event(Event::Approval { id, owner, spender, value }.into());
		// Additive.
		assert_ok!(Fungibles::increase_allowance(signed(ALICE), id, BOB, value));
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), value * 2);
		System::assert_last_event(Event::Approval { id, owner, spender, value: value * 2 }.into());
	});
}

#[test]
fn decrease_allowance_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let id = ASSET;
		let owner = ALICE;
		let spender = BOB;

		create_asset_mint_and_approve(ALICE, id, ALICE, value, BOB, value);
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), value);
		// Owner balance is not changed if decreased by zero.
		assert_ok!(Fungibles::decrease_allowance(signed(ALICE), id, BOB, 0));
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), value);
		// Decrease allowance successfully.
		assert_ok!(Fungibles::decrease_allowance(signed(ALICE), id, BOB, value / 2));
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), value / 2);
		System::assert_last_event(Event::Approval { id, owner, spender, value: value / 2 }.into());
		// Saturating if current allowance is decreased more than the owner balance.
		assert_ok!(Fungibles::decrease_allowance(signed(ALICE), id, BOB, value));
		assert_eq!(Assets::allowance(id, &ALICE, &BOB), 0);
		System::assert_last_event(Event::Approval { id, owner, spender, value: 0 }.into());
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let id = ASSET;
		let owner = ALICE;
		let admin = ALICE;

		assert!(!Assets::asset_exists(id));
		assert_ok!(Fungibles::create(signed(ALICE), id, ALICE, 100));
		assert!(Assets::asset_exists(id));
		System::assert_last_event(Event::Create { id, owner, admin }.into());
	});
}

#[test]
fn start_destroy_works() {
	new_test_ext().execute_with(|| {
		let id = ASSET;

		create_asset(ALICE, id);
		assert_ok!(Fungibles::start_destroy(signed(ALICE), id));
		System::assert_last_event(Event::StartDestroy { id }.into());
	});
}

#[test]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let id = ASSET;
		let name = vec![42];
		let symbol = vec![42];
		let decimals = 42;

		create_asset(ALICE, id);
		assert_ok!(Fungibles::set_metadata(
			signed(ALICE),
			id,
			name.clone(),
			symbol.clone(),
			decimals
		));
		assert_eq!(Assets::name(id), name);
		assert_eq!(Assets::symbol(id), symbol);
		assert_eq!(Assets::decimals(id), decimals);
		System::assert_last_event(Event::SetMetadata { id, name, symbol, decimals }.into());
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let id = ASSET;
		let name = vec![42];
		let symbol = vec![42];
		let decimals = 42;

		create_asset_and_set_metadata(ALICE, id, name, symbol, decimals);
		assert_ok!(Fungibles::clear_metadata(signed(ALICE), id));
		assert_eq!(Assets::name(id), Vec::<u8>::new());
		assert_eq!(Assets::symbol(id), Vec::<u8>::new());
		assert_eq!(Assets::decimals(id), 0u8);
		System::assert_last_event(Event::ClearMetadata { id }.into());
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let id = ASSET;
		let from = None;
		let to = Some(BOB);

		create_asset(ALICE, id);
		let balance_before_mint = Assets::balance(id, &BOB);
		assert_ok!(Fungibles::mint(signed(ALICE), id, BOB, value));
		let balance_after_mint = Assets::balance(id, &BOB);
		assert_eq!(balance_after_mint, balance_before_mint + value);
		System::assert_last_event(Event::Transfer { id, from, to, value }.into());
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let id = ASSET;
		let from = Some(BOB);
		let to = None;

		create_asset_and_mint_to(ALICE, id, BOB, value);
		let balance_before_burn = Assets::balance(id, &BOB);
		assert_ok!(Fungibles::burn(signed(ALICE), id, BOB, value));
		let balance_after_burn = Assets::balance(id, &BOB);
		assert_eq!(balance_after_burn, balance_before_burn - value);
		System::assert_last_event(Event::Transfer { id, from, to, value }.into());
	});
}

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		create_asset_and_mint_to(ALICE, ASSET, ALICE, 100);
		assert_eq!(Assets::total_supply(ASSET).encode(), Fungibles::read_state(TotalSupply(ASSET)));
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		create_asset_and_mint_to(ALICE, ASSET, ALICE, 100);
		assert_eq!(
			Assets::balance(ASSET, ALICE).encode(),
			Fungibles::read_state(BalanceOf { id: ASSET, owner: ALICE })
		);
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		create_asset_mint_and_approve(ALICE, ASSET, BOB, 100, ALICE, 50);
		assert_eq!(
			Assets::allowance(ASSET, &ALICE, &BOB).encode(),
			Fungibles::read_state(Allowance { id: ASSET, owner: ALICE, spender: BOB })
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
		assert_eq!(Assets::name(ASSET).encode(), Fungibles::read_state(TokenName(ASSET)));
		assert_eq!(Assets::symbol(ASSET).encode(), Fungibles::read_state(TokenSymbol(ASSET)));
		assert_eq!(Assets::decimals(ASSET).encode(), Fungibles::read_state(TokenDecimals(ASSET)));
	});
}

#[test]
fn asset_exists_works() {
	new_test_ext().execute_with(|| {
		create_asset(ALICE, ASSET);
		assert_eq!(Assets::asset_exists(ASSET).encode(), Fungibles::read_state(AssetExists(ASSET)));
	});
}

fn signed(account: AccountId) -> RuntimeOrigin {
	RuntimeOrigin::signed(account)
}

fn create_asset(owner: AccountId, asset_id: AssetId) {
	assert_ok!(Assets::create(signed(owner), asset_id, owner, 1));
}

fn mint_asset(owner: AccountId, asset_id: AssetId, to: AccountId, value: Balance) {
	assert_ok!(Assets::mint(signed(owner), asset_id, to, value));
}

fn create_asset_and_mint_to(owner: AccountId, asset_id: AssetId, to: AccountId, value: Balance) {
	create_asset(owner, asset_id);
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

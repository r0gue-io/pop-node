use crate::{fungibles::Read::*, mock::*};
use codec::Encode;
use frame_support::{
	assert_ok,
	sp_runtime::traits::Zero,
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
		let asset = ASSET;
		let from = Some(account(ALICE));
		let to = Some(account(BOB));

		create_asset_and_mint_to(account(ALICE), asset, account(ALICE), value * 2);
		let balance_before_transfer = Assets::balance(asset, &account(BOB));
		assert_ok!(Fungibles::transfer(signed(account(ALICE)), asset, account(BOB), value));
		let balance_after_transfer = Assets::balance(asset, &account(BOB));
		assert_eq!(balance_after_transfer, balance_before_transfer + value);
		System::assert_last_event(Event::Transfer { asset, from, to, value }.into());
	});
}

#[test]
fn transfer_from_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let asset = ASSET;
		let from = Some(account(ALICE));
		let to = Some(account(BOB));

		// Approve CHARLIE to transfer up to `value` to BOB.
		create_asset_mint_and_approve(
			account(ALICE),
			asset,
			account(ALICE),
			value * 2,
			account(CHARLIE),
			value,
		);
		// Successfully call transfer from.
		let alice_balance_before_transfer = Assets::balance(asset, &account(ALICE));
		let bob_balance_before_transfer = Assets::balance(asset, &account(BOB));
		assert_ok!(Fungibles::transfer_from(
			signed(account(CHARLIE)),
			asset,
			account(ALICE),
			account(BOB),
			value
		));
		let alice_balance_after_transfer = Assets::balance(asset, &account(ALICE));
		let bob_balance_after_transfer = Assets::balance(asset, &account(BOB));
		// Check that BOB receives the `value` and ALICE `amount` is spent successfully by CHARLIE.
		assert_eq!(bob_balance_after_transfer, bob_balance_before_transfer + value);
		assert_eq!(alice_balance_after_transfer, alice_balance_before_transfer - value);
		System::assert_last_event(Event::Transfer { asset, from, to, value }.into());
	});
}

// Non-additive, sets new value.
#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let asset = ASSET;
		let owner = account(ALICE);
		let spender = account(BOB);

		create_asset_and_mint_to(account(ALICE), asset, account(ALICE), value);
		assert_eq!(0, Assets::allowance(asset, &account(ALICE), &account(BOB)));
		assert_ok!(Fungibles::approve(signed(account(ALICE)), asset, account(BOB), value));
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), value);
		System::assert_last_event(
			Event::Approval { asset, owner: owner.clone(), spender: spender.clone(), value }.into(),
		);
		// Approves an value to spend that is lower than the current allowance.
		assert_ok!(Fungibles::approve(signed(account(ALICE)), asset, account(BOB), value / 2));
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), value / 2);
		System::assert_last_event(
			Event::Approval {
				asset,
				owner: owner.clone(),
				spender: spender.clone(),
				value: value / 2,
			}
			.into(),
		);
		// Approves an value to spend that is higher than the current allowance.
		assert_ok!(Fungibles::approve(signed(account(ALICE)), asset, account(BOB), value * 2));
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), value * 2);
		System::assert_last_event(
			Event::Approval {
				asset,
				owner: owner.clone(),
				spender: spender.clone(),
				value: value * 2,
			}
			.into(),
		);
		// Approves an value to spend that is equal to the current allowance.
		assert_ok!(Fungibles::approve(signed(account(ALICE)), asset, account(BOB), value * 2));
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), value * 2);
		System::assert_last_event(
			Event::Approval {
				asset,
				owner: owner.clone(),
				spender: spender.clone(),
				value: value * 2,
			}
			.into(),
		);
		// Sets allowance to zero.
		assert_ok!(Fungibles::approve(signed(account(ALICE)), asset, account(BOB), 0));
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), 0);
		System::assert_last_event(Event::Approval { asset, owner, spender, value: 0 }.into());
	});
}

#[test]
fn increase_allowance_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let asset = ASSET;
		let owner = account(ALICE);
		let spender = account(BOB);

		create_asset_and_mint_to(account(ALICE), asset, account(ALICE), value);
		assert_eq!(0, Assets::allowance(asset, &account(ALICE), &account(BOB)));
		assert_ok!(Fungibles::increase_allowance(
			signed(account(ALICE)),
			asset,
			account(BOB),
			value
		));
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), value);
		System::assert_last_event(
			Event::Approval { asset, owner: owner.clone(), spender: spender.clone(), value }.into(),
		);
		// Additive.
		assert_ok!(Fungibles::increase_allowance(
			signed(account(ALICE)),
			asset,
			account(BOB),
			value
		));
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), value * 2);
		System::assert_last_event(
			Event::Approval { asset, owner, spender, value: value * 2 }.into(),
		);
	});
}

#[test]
fn decrease_allowance_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let asset = ASSET;
		let owner = account(ALICE);
		let spender = account(BOB);

		create_asset_mint_and_approve(
			account(ALICE),
			asset,
			account(ALICE),
			value,
			account(BOB),
			value,
		);
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), value);
		// Owner balance is not changed if decreased by zero.
		assert_ok!(Fungibles::decrease_allowance(signed(account(ALICE)), asset, account(BOB), 0));
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), value);
		// Decrease allowance successfully.
		assert_ok!(Fungibles::decrease_allowance(
			signed(account(ALICE)),
			asset,
			account(BOB),
			value / 2
		));
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), value / 2);
		System::assert_last_event(
			Event::Approval {
				asset,
				owner: owner.clone(),
				spender: spender.clone(),
				value: value / 2,
			}
			.into(),
		);
		// Saturating if current allowance is decreased more than the owner balance.
		assert_ok!(Fungibles::decrease_allowance(
			signed(account(ALICE)),
			asset,
			account(BOB),
			value
		));
		assert_eq!(Assets::allowance(asset, &account(ALICE), &account(BOB)), 0);
		System::assert_last_event(Event::Approval { asset, owner, spender, value: 0 }.into());
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let id = ASSET;
		let creator = account(ALICE);
		let admin = account(ALICE);

		assert!(!Assets::asset_exists(id));
		assert_ok!(Fungibles::create(signed(creator.clone()), id, admin.clone(), 100));
		assert!(Assets::asset_exists(id));
		System::assert_last_event(Event::Create { id, creator, admin }.into());
	});
}

#[test]
fn start_destroy_works() {
	new_test_ext().execute_with(|| {
		let asset = ASSET;

		create_asset(account(ALICE), asset);
		assert_ok!(Fungibles::start_destroy(signed(account(ALICE)), asset));
	});
}

#[test]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let asset = ASSET;
		let name = vec![42];
		let symbol = vec![42];
		let decimals = 42;

		create_asset(account(ALICE), asset);
		assert_ok!(Fungibles::set_metadata(
			signed(account(ALICE)),
			asset,
			name.clone(),
			symbol.clone(),
			decimals
		));
		assert_eq!(Assets::name(asset), name);
		assert_eq!(Assets::symbol(asset), symbol);
		assert_eq!(Assets::decimals(asset), decimals);
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let asset = ASSET;

		create_asset_and_set_metadata(account(ALICE), asset, vec![42], vec![42], 42);
		assert_ok!(Fungibles::clear_metadata(signed(account(ALICE)), asset));
		assert!(Assets::name(asset).is_empty());
		assert!(Assets::symbol(asset).is_empty());
		assert!(Assets::decimals(asset).is_zero());
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let asset = ASSET;
		let from = None;
		let to = Some(account(BOB));

		create_asset(account(ALICE), asset);
		let balance_before_mint = Assets::balance(asset, &account(BOB));
		assert_ok!(Fungibles::mint(signed(account(ALICE)), asset, account(BOB), value));
		let balance_after_mint = Assets::balance(asset, &account(BOB));
		assert_eq!(balance_after_mint, balance_before_mint + value);
		System::assert_last_event(Event::Transfer { asset, from, to, value }.into());
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let asset = ASSET;
		let from = Some(account(BOB));
		let to = None;

		create_asset_and_mint_to(account(ALICE), asset, account(BOB), value);
		let balance_before_burn = Assets::balance(asset, &account(BOB));
		assert_ok!(Fungibles::burn(signed(account(ALICE)), asset, account(BOB), value));
		let balance_after_burn = Assets::balance(asset, &account(BOB));
		assert_eq!(balance_after_burn, balance_before_burn - value);
		System::assert_last_event(Event::Transfer { asset, from, to, value }.into());
	});
}

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		create_asset_and_mint_to(account(ALICE), ASSET, account(ALICE), 100);
		assert_eq!(Assets::total_supply(ASSET).encode(), Fungibles::read_state(TotalSupply(ASSET)));
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		create_asset_and_mint_to(account(ALICE), ASSET, account(ALICE), 100);
		assert_eq!(
			Assets::balance(ASSET, account(ALICE)).encode(),
			Fungibles::read_state(BalanceOf { asset: ASSET, owner: account(ALICE) })
		);
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		create_asset_mint_and_approve(account(ALICE), ASSET, account(BOB), 100, account(ALICE), 50);
		assert_eq!(
			Assets::allowance(ASSET, &account(ALICE), &account(BOB)).encode(),
			Fungibles::read_state(Allowance {
				asset: ASSET,
				owner: account(ALICE),
				spender: account(BOB)
			})
		);
	});
}

#[test]
fn token_metadata_works() {
	new_test_ext().execute_with(|| {
		let name: Vec<u8> = vec![11, 12, 13];
		let symbol: Vec<u8> = vec![21, 22, 23];
		let decimals: u8 = 69;
		create_asset_and_set_metadata(
			account(ALICE),
			ASSET,
			name.clone(),
			symbol.clone(),
			decimals,
		);
		assert_eq!(Assets::name(ASSET).encode(), Fungibles::read_state(TokenName(ASSET)));
		assert_eq!(Assets::symbol(ASSET).encode(), Fungibles::read_state(TokenSymbol(ASSET)));
		assert_eq!(Assets::decimals(ASSET).encode(), Fungibles::read_state(TokenDecimals(ASSET)));
	});
}

#[test]
fn asset_exists_works() {
	new_test_ext().execute_with(|| {
		create_asset(account(ALICE), ASSET);
		assert_eq!(Assets::asset_exists(ASSET).encode(), Fungibles::read_state(AssetExists(ASSET)));
	});
}

fn signed(account: AccountId) -> RuntimeOrigin {
	RuntimeOrigin::signed(account)
}

fn create_asset(owner: AccountId, asset: AssetId) {
	assert_ok!(Assets::create(signed(owner.clone()), asset, owner, 1));
}

fn mint_asset(owner: AccountId, asset: AssetId, to: AccountId, value: Balance) {
	assert_ok!(Assets::mint(signed(owner), asset, to, value));
}

fn create_asset_and_mint_to(owner: AccountId, asset: AssetId, to: AccountId, value: Balance) {
	create_asset(owner.clone(), asset);
	mint_asset(owner, asset, to, value)
}

fn create_asset_mint_and_approve(
	owner: AccountId,
	asset: AssetId,
	to: AccountId,
	mint: Balance,
	spender: AccountId,
	approve: Balance,
) {
	create_asset_and_mint_to(owner, asset, to.clone(), mint);
	assert_ok!(Assets::approve_transfer(signed(to), asset, spender, approve,));
}

fn create_asset_and_set_metadata(
	owner: AccountId,
	asset: AssetId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) {
	assert_ok!(Assets::create(signed(owner.clone()), asset, owner.clone(), 100));
	set_metadata_asset(owner, asset, name, symbol, decimals);
}

fn set_metadata_asset(
	owner: AccountId,
	asset: AssetId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) {
	assert_ok!(Assets::set_metadata(signed(owner), asset, name, symbol, decimals));
}

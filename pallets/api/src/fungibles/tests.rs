use codec::Encode;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::WithPostDispatchInfo,
	sp_runtime::{traits::Zero, DispatchError::BadOrigin},
	traits::fungibles::{
		approvals::Inspect as ApprovalInspect, metadata::Inspect as MetadataInspect, Inspect,
	},
};

use crate::{
	fungibles::{
		weights::WeightInfo as WeightInfoTrait, AssetsInstanceOf, AssetsWeightInfoOf,
		AssetsWeightInfoTrait, Config, Read::*,
	},
	mock::*,
	Read,
};

const TOKEN: u32 = 42;

type AssetsError = pallet_assets::Error<Test, AssetsInstanceOf<Test>>;
type AssetsWeightInfo = AssetsWeightInfoOf<Test>;
type Event = crate::fungibles::Event<Test>;
type WeightInfo = <Test as Config>::WeightInfo;

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let from = Some(ALICE);
		let to = Some(BOB);

		// Check error works for `Assets::transfer_keep_alive()`.
		assert_noop!(Fungibles::transfer(signed(ALICE), token, BOB, value), AssetsError::Unknown);
		pallet_assets_create_and_mint_to(ALICE, token, ALICE, value * 2);
		for origin in vec![root(), none()] {
			assert_noop!(Fungibles::transfer(origin, token, BOB, value), BadOrigin);
		}
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

		// Check error works for `Assets::transfer_approved()`.
		assert_noop!(
			Fungibles::transfer_from(signed(CHARLIE), token, ALICE, BOB, value),
			AssetsError::Unknown
		);
		// Approve CHARLIE to transfer up to `value`.
		pallet_assets_create_mint_and_approve(ALICE, token, ALICE, value * 2, CHARLIE, value);
		for origin in vec![root(), none()] {
			assert_noop!(Fungibles::transfer_from(origin, token, ALICE, BOB, value), BadOrigin);
		}
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

		// Approves a value to spend.
		//
		// Check error works for `Assets::approve_transfer()` in `Greater` match arm.
		assert_noop!(
			Fungibles::approve(signed(ALICE), token, BOB, value),
			AssetsError::Unknown.with_weight(WeightInfo::approve(1, 0))
		);
		pallet_assets_create_and_mint_to(ALICE, token, ALICE, value);
		for origin in vec![root(), none()] {
			assert_noop!(
				Fungibles::approve(origin, token, BOB, value),
				BadOrigin.with_weight(WeightInfo::approve(0, 0))
			);
		}
		assert_eq!(0, Assets::allowance(token, &ALICE, &BOB));
		assert_eq!(
			Fungibles::approve(signed(ALICE), token, BOB, value),
			Ok(Some(WeightInfo::approve(1, 0)).into())
		);
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value);
		System::assert_last_event(Event::Approval { token, owner, spender, value }.into());
		// Approves a value to spend that is lower than the current allowance.
		//
		// Check error works for `Assets::cancel_approval()` in `Less` match arm. No error test for
		// `approve_transfer` in `Less` arm because it is not possible.
		pallet_assets_freeze_asset(ALICE, token);
		assert_noop!(
			Fungibles::approve(signed(ALICE), token, BOB, value / 2),
			AssetsError::AssetNotLive.with_weight(WeightInfo::approve(0, 1))
		);
		pallet_assets_thaw_asset(ALICE, token);
		assert_eq!(
			Fungibles::approve(signed(ALICE), token, BOB, value / 2),
			Ok(Some(WeightInfo::approve(1, 1)).into())
		);
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value / 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value / 2 }.into(),
		);
		// Approves a value to spend that is higher than the current allowance.
		assert_eq!(
			Fungibles::approve(signed(ALICE), token, BOB, value * 2),
			Ok(Some(WeightInfo::approve(1, 0)).into())
		);
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value * 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value * 2 }.into(),
		);
		// Approves a value to spend that is equal to the current allowance.
		assert_eq!(
			Fungibles::approve(signed(ALICE), token, BOB, value * 2),
			Ok(Some(WeightInfo::approve(0, 0)).into())
		);
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value * 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value * 2 }.into(),
		);
		// Sets allowance to zero.
		assert_eq!(
			Fungibles::approve(signed(ALICE), token, BOB, 0),
			Ok(Some(WeightInfo::approve(0, 1)).into())
		);
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

		// Check error works for `Assets::approve_transfer()`.
		assert_noop!(
			Fungibles::increase_allowance(signed(ALICE), token, BOB, value),
			AssetsError::Unknown.with_weight(AssetsWeightInfo::approve_transfer())
		);
		pallet_assets_create_and_mint_to(ALICE, token, ALICE, value);
		for origin in vec![root(), none()] {
			assert_noop!(
				Fungibles::increase_allowance(origin, token, BOB, value),
				BadOrigin.with_weight(WeightInfo::approve(0, 0))
			);
		}
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

		// Check error works for `Assets::cancel_approval()`. No error test for `approve_transfer`
		// because it is not possible.
		assert_noop!(
			Fungibles::decrease_allowance(signed(ALICE), token, BOB, value / 2),
			AssetsError::Unknown.with_weight(WeightInfo::approve(0, 1))
		);
		pallet_assets_create_mint_and_approve(ALICE, token, ALICE, value, BOB, value);
		for origin in vec![root(), none()] {
			assert_noop!(
				Fungibles::decrease_allowance(origin, token, BOB, 0),
				BadOrigin.with_weight(WeightInfo::approve(0, 0))
			);
		}
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value);
		// Owner balance is not changed if decreased by zero.
		assert_eq!(
			Fungibles::decrease_allowance(signed(ALICE), token, BOB, 0),
			Ok(Some(WeightInfo::approve(0, 0)).into())
		);
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value);
		// Decrease allowance successfully.
		assert_eq!(
			Fungibles::decrease_allowance(signed(ALICE), token, BOB, value / 2),
			Ok(Some(WeightInfo::approve(1, 1)).into())
		);
		assert_eq!(Assets::allowance(token, &ALICE, &BOB), value / 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value / 2 }.into(),
		);
		// Saturating if current allowance is decreased more than the owner balance.
		assert_eq!(
			Fungibles::decrease_allowance(signed(ALICE), token, BOB, value),
			Ok(Some(WeightInfo::approve(0, 1)).into())
		);
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

		for origin in vec![root(), none()] {
			assert_noop!(Fungibles::create(origin, id, admin, 100), BadOrigin);
		}
		assert!(!Assets::asset_exists(id));
		assert_ok!(Fungibles::create(signed(creator), id, admin, 100));
		assert!(Assets::asset_exists(id));
		System::assert_last_event(Event::Create { id, creator, admin }.into());
		// Check error works for `Assets::create()`.
		assert_noop!(Fungibles::create(signed(creator), id, admin, 100), AssetsError::InUse);
	});
}

#[test]
fn start_destroy_works() {
	new_test_ext().execute_with(|| {
		let token = TOKEN;

		// Check error works for `Assets::start_destroy()`.
		assert_noop!(Fungibles::start_destroy(signed(ALICE), token), AssetsError::Unknown);
		pallet_assets_create(ALICE, token);
		assert_ok!(Fungibles::start_destroy(signed(ALICE), token));
		assert!(Fungibles::start_destroy(signed(BOB), token).is_err());
	});
}

#[test]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let token = TOKEN;
		let name = vec![42];
		let symbol = vec![42];
		let decimals = 42;

		// Check error works for `Assets::set_metadata()`.
		assert_noop!(
			Fungibles::set_metadata(signed(ALICE), token, name.clone(), symbol.clone(), decimals),
			AssetsError::Unknown
		);
		pallet_assets_create(ALICE, token);
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

		// Check error works for `Assets::clear_metadata()`.
		assert_noop!(Fungibles::clear_metadata(signed(ALICE), token), AssetsError::Unknown);
		pallet_assets_create_and_set_metadata(ALICE, token, vec![42], vec![42], 42);
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

		// Check error works for `Assets::mint()`.
		assert_noop!(
			Fungibles::mint(signed(ALICE), token, BOB, value),
			sp_runtime::TokenError::UnknownAsset
		);
		pallet_assets_create(ALICE, token);
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
		let total_supply = value * 2;

		// Check error works for `Assets::burn()`.
		assert_noop!(Fungibles::burn(signed(ALICE), token, BOB, value), AssetsError::Unknown);
		pallet_assets_create_and_mint_to(ALICE, token, BOB, total_supply);
		assert_eq!(Assets::total_supply(TOKEN), total_supply);
		let balance_before_burn = Assets::balance(token, &BOB);
		assert_ok!(Fungibles::burn(signed(ALICE), token, BOB, value));
		assert_eq!(Assets::total_supply(TOKEN), total_supply - value);
		let balance_after_burn = Assets::balance(token, &BOB);
		assert_eq!(balance_after_burn, balance_before_burn - value);
		System::assert_last_event(Event::Transfer { token, from, to, value }.into());
	});
}

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let total_supply = INIT_AMOUNT;
		pallet_assets_create_and_mint_to(ALICE, TOKEN, ALICE, total_supply);
		assert_eq!(
			Fungibles::read(TotalSupply(TOKEN)).encode(),
			Assets::total_supply(TOKEN).encode(),
		);
		assert_eq!(Fungibles::read(TotalSupply(TOKEN)).encode(), total_supply.encode(),);
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let value = 1_000 * UNIT;
		pallet_assets_create_and_mint_to(ALICE, TOKEN, ALICE, value);
		assert_eq!(
			Fungibles::read(BalanceOf { token: TOKEN, owner: ALICE }).encode(),
			Assets::balance(TOKEN, ALICE).encode(),
		);
		assert_eq!(
			Fungibles::read(BalanceOf { token: TOKEN, owner: ALICE }).encode(),
			value.encode()
		);
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let value = 1_000 * UNIT;
		pallet_assets_create_mint_and_approve(ALICE, TOKEN, ALICE, value * 2, BOB, value);
		assert_eq!(
			Fungibles::read(Allowance { token: TOKEN, owner: ALICE, spender: BOB }).encode(),
			Assets::allowance(TOKEN, &ALICE, &BOB).encode(),
		);
		assert_eq!(
			Fungibles::read(Allowance { token: TOKEN, owner: ALICE, spender: BOB }).encode(),
			value.encode()
		);
	});
}

#[test]
fn token_metadata_works() {
	new_test_ext().execute_with(|| {
		let name: Vec<u8> = vec![11, 12, 13];
		let symbol: Vec<u8> = vec![21, 22, 23];
		let decimals: u8 = 69;
		pallet_assets_create_and_set_metadata(ALICE, TOKEN, name.clone(), symbol.clone(), decimals);
		assert_eq!(Fungibles::read(TokenName(TOKEN)).encode(), Assets::name(TOKEN).encode());
		assert_eq!(Fungibles::read(TokenSymbol(TOKEN)).encode(), Assets::symbol(TOKEN).encode());
		assert_eq!(
			Fungibles::read(TokenDecimals(TOKEN)).encode(),
			Assets::decimals(TOKEN).encode(),
		);
		assert_eq!(Fungibles::read(TokenName(TOKEN)).encode(), name.encode());
		assert_eq!(Fungibles::read(TokenSymbol(TOKEN)).encode(), symbol.encode());
		assert_eq!(Fungibles::read(TokenDecimals(TOKEN)).encode(), decimals.encode());
	});
}

#[test]
fn token_exists_works() {
	new_test_ext().execute_with(|| {
		pallet_assets_create(ALICE, TOKEN);
		assert_eq!(
			Fungibles::read(TokenExists(TOKEN)).encode(),
			Assets::asset_exists(TOKEN).encode(),
		);
		assert_eq!(Fungibles::read(TokenExists(TOKEN)).encode(), true.encode());
	});
}

fn signed(account: AccountId) -> RuntimeOrigin {
	RuntimeOrigin::signed(account)
}

fn root() -> RuntimeOrigin {
	RuntimeOrigin::root()
}

fn none() -> RuntimeOrigin {
	RuntimeOrigin::none()
}

fn pallet_assets_create(owner: AccountId, token: TokenId) {
	assert_ok!(Assets::create(signed(owner), token, owner, 1));
}

fn pallet_assets_mint(owner: AccountId, token: TokenId, to: AccountId, value: Balance) {
	assert_ok!(Assets::mint(signed(owner), token, to, value));
}

fn pallet_assets_create_and_mint_to(
	owner: AccountId,
	token: TokenId,
	to: AccountId,
	value: Balance,
) {
	pallet_assets_create(owner, token);
	pallet_assets_mint(owner, token, to, value)
}

fn pallet_assets_create_mint_and_approve(
	owner: AccountId,
	token: TokenId,
	to: AccountId,
	mint: Balance,
	spender: AccountId,
	approve: Balance,
) {
	pallet_assets_create_and_mint_to(owner, token, to, mint);
	assert_ok!(Assets::approve_transfer(signed(to), token, spender, approve,));
}

fn pallet_assets_create_and_set_metadata(
	owner: AccountId,
	token: TokenId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) {
	assert_ok!(Assets::create(signed(owner), token, owner, 100));
	pallet_assets_set_metadata(owner, token, name, symbol, decimals);
}

fn pallet_assets_set_metadata(
	owner: AccountId,
	token: TokenId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) {
	assert_ok!(Assets::set_metadata(signed(owner), token, name, symbol, decimals));
}

fn pallet_assets_freeze_asset(owner: AccountId, token: TokenId) {
	assert_ok!(Assets::freeze_asset(signed(owner), token));
}

fn pallet_assets_thaw_asset(owner: AccountId, token: TokenId) {
	assert_ok!(Assets::thaw_asset(signed(owner), token));
}

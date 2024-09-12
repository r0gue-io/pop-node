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
		AssetsWeightInfoTrait, Config, Read::*, ReadResult,
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
fn encoding_read_result_works() {}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let from = ALICE;
		let to = BOB;

		// Check error works for `Assets::transfer_keep_alive()`.
		assert_noop!(Fungibles::transfer(signed(from), token, to, value), AssetsError::Unknown);
		pallet_assets_create_and_mint_to(from, token, from, value * 2);
		for origin in vec![root(), none()] {
			assert_noop!(Fungibles::transfer(origin, token, to, value), BadOrigin);
		}
		let balance_before_transfer = Assets::balance(token, &to);
		assert_ok!(Fungibles::transfer(signed(from), token, to, value));
		let balance_after_transfer = Assets::balance(token, &to);
		assert_eq!(balance_after_transfer, balance_before_transfer + value);
		System::assert_last_event(
			Event::Transfer { token, from: Some(from), to: Some(to), value }.into(),
		);
	});
}

#[test]
fn transfer_from_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let from = ALICE;
		let to = BOB;
		let spender = CHARLIE;

		// Check error works for `Assets::transfer_approved()`.
		assert_noop!(
			Fungibles::transfer_from(signed(spender), token, from, to, value),
			AssetsError::Unknown
		);
		// Approve `spender` to transfer up to `value`.
		pallet_assets_create_mint_and_approve(spender, token, from, value * 2, spender, value);
		for origin in vec![root(), none()] {
			assert_noop!(Fungibles::transfer_from(origin, token, from, to, value), BadOrigin);
		}
		// Successfully call transfer from.
		let alice_balance_before_transfer = Assets::balance(token, &from);
		let bob_balance_before_transfer = Assets::balance(token, &to);
		assert_ok!(Fungibles::transfer_from(signed(spender), token, from, to, value));
		let alice_balance_after_transfer = Assets::balance(token, &from);
		let bob_balance_after_transfer = Assets::balance(token, &to);
		// Check that `to` has received the `value` tokens from `from`.
		assert_eq!(bob_balance_after_transfer, bob_balance_before_transfer + value);
		assert_eq!(alice_balance_after_transfer, alice_balance_before_transfer - value);
		System::assert_last_event(
			Event::Transfer { token, from: Some(from), to: Some(to), value }.into(),
		);
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
			Fungibles::approve(signed(owner), token, spender, value),
			AssetsError::Unknown.with_weight(WeightInfo::approve(1, 0))
		);
		pallet_assets_create_and_mint_to(owner, token, owner, value);
		for origin in vec![root(), none()] {
			assert_noop!(
				Fungibles::approve(origin, token, spender, value),
				BadOrigin.with_weight(WeightInfo::approve(0, 0))
			);
		}
		assert_eq!(0, Assets::allowance(token, &owner, &spender));
		assert_eq!(
			Fungibles::approve(signed(owner), token, spender, value),
			Ok(Some(WeightInfo::approve(1, 0)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), value);
		System::assert_last_event(Event::Approval { token, owner, spender, value }.into());
		// Approves a value to spend that is lower than the current allowance.
		//
		// Check error works for `Assets::cancel_approval()` in `Less` match arm. No error test for
		// `approve_transfer` in `Less` arm because it is not possible.
		pallet_assets_freeze_asset(owner, token);
		assert_noop!(
			Fungibles::approve(signed(owner), token, spender, value / 2),
			AssetsError::AssetNotLive.with_weight(WeightInfo::approve(0, 1))
		);
		pallet_assets_thaw_asset(owner, token);
		assert_eq!(
			Fungibles::approve(signed(owner), token, spender, value / 2),
			Ok(Some(WeightInfo::approve(1, 1)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), value / 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value / 2 }.into(),
		);
		// Approves a value to spend that is higher than the current allowance.
		assert_eq!(
			Fungibles::approve(signed(owner), token, spender, value * 2),
			Ok(Some(WeightInfo::approve(1, 0)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), value * 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value * 2 }.into(),
		);
		// Approves a value to spend that is equal to the current allowance.
		assert_eq!(
			Fungibles::approve(signed(owner), token, spender, value * 2),
			Ok(Some(WeightInfo::approve(0, 0)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), value * 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value * 2 }.into(),
		);
		// Sets allowance to zero.
		assert_eq!(
			Fungibles::approve(signed(owner), token, spender, 0),
			Ok(Some(WeightInfo::approve(0, 1)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), 0);
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
			Fungibles::increase_allowance(signed(owner), token, spender, value),
			AssetsError::Unknown.with_weight(AssetsWeightInfo::approve_transfer())
		);
		pallet_assets_create_and_mint_to(owner, token, owner, value);
		for origin in vec![root(), none()] {
			assert_noop!(
				Fungibles::increase_allowance(origin, token, spender, value),
				BadOrigin.with_weight(WeightInfo::approve(0, 0))
			);
		}
		assert_eq!(0, Assets::allowance(token, &owner, &spender));
		assert_ok!(Fungibles::increase_allowance(signed(owner), token, spender, value));
		assert_eq!(Assets::allowance(token, &owner, &spender), value);
		System::assert_last_event(Event::Approval { token, owner, spender, value }.into());
		// Additive.
		assert_ok!(Fungibles::increase_allowance(signed(owner), token, spender, value));
		assert_eq!(Assets::allowance(token, &owner, &spender), value * 2);
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
			Fungibles::decrease_allowance(signed(owner), token, spender, value / 2),
			AssetsError::Unknown.with_weight(WeightInfo::approve(0, 1))
		);
		pallet_assets_create_mint_and_approve(owner, token, owner, value, spender, value);
		for origin in vec![root(), none()] {
			assert_noop!(
				Fungibles::decrease_allowance(origin, token, spender, 0),
				BadOrigin.with_weight(WeightInfo::approve(0, 0))
			);
		}
		assert_eq!(Assets::allowance(token, &owner, &spender), value);
		// Owner balance is not changed if decreased by zero.
		assert_eq!(
			Fungibles::decrease_allowance(signed(owner), token, spender, 0),
			Ok(Some(WeightInfo::approve(0, 0)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), value);
		// Decrease allowance successfully.
		assert_eq!(
			Fungibles::decrease_allowance(signed(owner), token, spender, value / 2),
			Ok(Some(WeightInfo::approve(1, 1)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), value / 2);
		System::assert_last_event(
			Event::Approval { token, owner, spender, value: value / 2 }.into(),
		);
		// Saturating if current allowance is decreased more than the owner balance.
		assert_eq!(
			Fungibles::decrease_allowance(signed(owner), token, spender, value),
			Ok(Some(WeightInfo::approve(0, 1)).into())
		);
		assert_eq!(Assets::allowance(token, &owner, &spender), 0);
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
		// Check that the token is not live after starting the destroy process.
		assert_noop!(
			Assets::mint(signed(ALICE), token, ALICE, 10 * UNIT),
			AssetsError::AssetNotLive
		);
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
		let from = ALICE;
		let to = BOB;

		// Check error works for `Assets::mint()`.
		assert_noop!(
			Fungibles::mint(signed(from), token, to, value),
			sp_runtime::TokenError::UnknownAsset
		);
		pallet_assets_create(from, token);
		let balance_before_mint = Assets::balance(token, &to);
		assert_ok!(Fungibles::mint(signed(from), token, to, value));
		let balance_after_mint = Assets::balance(token, &to);
		assert_eq!(balance_after_mint, balance_before_mint + value);
		System::assert_last_event(
			Event::Transfer { token, from: None, to: Some(to), value }.into(),
		);
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let value: Balance = 100 * UNIT;
		let token = TOKEN;
		let owner = ALICE;
		let from = BOB;
		let total_supply = value * 2;

		// Check error works for `Assets::burn()`.
		assert_noop!(Fungibles::burn(signed(owner), token, from, value), AssetsError::Unknown);
		pallet_assets_create_and_mint_to(owner, token, from, total_supply);
		assert_eq!(Assets::total_supply(TOKEN), total_supply);
		let balance_before_burn = Assets::balance(token, &from);
		assert_ok!(Fungibles::burn(signed(owner), token, from, value));
		assert_eq!(Assets::total_supply(TOKEN), total_supply - value);
		let balance_after_burn = Assets::balance(token, &from);
		assert_eq!(balance_after_burn, balance_before_burn - value);
		System::assert_last_event(
			Event::Transfer { token, from: Some(from), to: None, value }.into(),
		);
	});
}

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let total_supply = INIT_AMOUNT;
		assert_eq!(
			Fungibles::read(TotalSupply(TOKEN)),
			ReadResult::TotalSupply(Default::default())
		);
		pallet_assets_create_and_mint_to(ALICE, TOKEN, ALICE, total_supply);
		assert_eq!(Fungibles::read(TotalSupply(TOKEN)), ReadResult::TotalSupply(total_supply));
		assert_eq!(
			Fungibles::read(TotalSupply(TOKEN)).encode(),
			Assets::total_supply(TOKEN).encode(),
		);
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let value = 1_000 * UNIT;
		assert_eq!(
			Fungibles::read(BalanceOf { token: TOKEN, owner: ALICE }),
			ReadResult::BalanceOf(Default::default())
		);
		pallet_assets_create_and_mint_to(ALICE, TOKEN, ALICE, value);
		assert_eq!(
			Fungibles::read(BalanceOf { token: TOKEN, owner: ALICE }),
			ReadResult::BalanceOf(value)
		);
		assert_eq!(
			Fungibles::read(BalanceOf { token: TOKEN, owner: ALICE }).encode(),
			Assets::balance(TOKEN, ALICE).encode(),
		);
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let value = 1_000 * UNIT;
		assert_eq!(
			Fungibles::read(Allowance { token: TOKEN, owner: ALICE, spender: BOB }),
			ReadResult::Allowance(Default::default())
		);
		pallet_assets_create_mint_and_approve(ALICE, TOKEN, ALICE, value * 2, BOB, value);
		assert_eq!(
			Fungibles::read(Allowance { token: TOKEN, owner: ALICE, spender: BOB }),
			ReadResult::Allowance(value)
		);
		assert_eq!(
			Fungibles::read(Allowance { token: TOKEN, owner: ALICE, spender: BOB }).encode(),
			Assets::allowance(TOKEN, &ALICE, &BOB).encode(),
		);
	});
}

#[test]
fn token_metadata_works() {
	new_test_ext().execute_with(|| {
		let name: Vec<u8> = vec![11, 12, 13];
		let symbol: Vec<u8> = vec![21, 22, 23];
		let decimals: u8 = 69;
		assert_eq!(Fungibles::read(TokenName(TOKEN)), ReadResult::TokenName(Default::default()));
		assert_eq!(
			Fungibles::read(TokenSymbol(TOKEN)),
			ReadResult::TokenSymbol(Default::default())
		);
		assert_eq!(
			Fungibles::read(TokenDecimals(TOKEN)),
			ReadResult::TokenDecimals(Default::default())
		);
		pallet_assets_create_and_set_metadata(ALICE, TOKEN, name.clone(), symbol.clone(), decimals);
		assert_eq!(Fungibles::read(TokenName(TOKEN)), ReadResult::TokenName(name));
		assert_eq!(Fungibles::read(TokenSymbol(TOKEN)), ReadResult::TokenSymbol(symbol));
		assert_eq!(Fungibles::read(TokenDecimals(TOKEN)), ReadResult::TokenDecimals(decimals));
		assert_eq!(Fungibles::read(TokenName(TOKEN)).encode(), Assets::name(TOKEN).encode());
		assert_eq!(Fungibles::read(TokenSymbol(TOKEN)).encode(), Assets::symbol(TOKEN).encode());
		assert_eq!(
			Fungibles::read(TokenDecimals(TOKEN)).encode(),
			Assets::decimals(TOKEN).encode(),
		);
	});
}

#[test]
fn token_exists_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Fungibles::read(TokenExists(TOKEN)), ReadResult::TokenExists(false));
		pallet_assets_create(ALICE, TOKEN);
		assert_eq!(Fungibles::read(TokenExists(TOKEN)), ReadResult::TokenExists(true));
		assert_eq!(
			Fungibles::read(TokenExists(TOKEN)).encode(),
			Assets::asset_exists(TOKEN).encode(),
		);
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

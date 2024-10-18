use codec::Encode;
use frame_support::{assert_ok, traits::nonfungibles_v2::InspectEnumerable};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_nfts::{AccountBalance, CollectionConfig, CollectionSettings, MintSettings};

use super::types::{AccountIdOf, CollectionIdOf, ItemIdOf};
use crate::{
	mock::*,
	nonfungibles::{Event, Read::*},
	Read,
};

const ITEM: u32 = 1;

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let owner = account(ALICE);
		let collection = create_collection(owner.clone());
		// Successfully mint a new collection item.
		assert_ok!(NonFungibles::mint(
			signed(owner.clone()),
			owner.clone(),
			collection,
			ITEM,
			None
		));
		System::assert_last_event(
			Event::Transfer { collection, item: ITEM, from: None, to: Some(owner), value: None }
				.into(),
		);
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let owner = account(ALICE);
		let (collection, item) = create_collection_mint(owner.clone(), ITEM);
		// Successfully burn an existing new collection item.
		assert_ok!(NonFungibles::burn(signed(owner.clone()), collection, ITEM));
		System::assert_last_event(
			Event::Transfer { collection, item, from: Some(owner), to: None, value: None }.into(),
		);
	});
}

#[test]
fn transfer() {
	new_test_ext().execute_with(|| {
		let owner = account(ALICE);
		let dest = account(BOB);
		let (collection, item) = create_collection_mint(owner.clone(), ITEM);
		// Successfully burn an existing new collection item.
		assert_ok!(NonFungibles::transfer(signed(owner.clone()), collection, ITEM, dest.clone()));
		System::assert_last_event(
			Event::Transfer { collection, item, from: Some(owner), to: Some(dest), value: None }
				.into(),
		);
	});
}

#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let owner = account(ALICE);
		let operator = account(BOB);
		let (collection, item) = create_collection_mint(owner.clone(), ITEM);
		// Successfully approve `spender` to transfer the collection item.
		assert_ok!(NonFungibles::approve(
			signed(owner.clone()),
			collection,
			Some(item),
			operator.clone(),
			true
		));
		System::assert_last_event(
			Event::Approval {
				collection,
				item: Some(item),
				owner,
				operator: operator.clone(),
				approved: true,
			}
			.into(),
		);
		// Successfully transfer the item by the delegated account `spender`.
		assert_ok!(Nfts::transfer(signed(operator.clone()), collection, item, operator));
	});
}

#[test]
fn cancel_approval_works() {
	new_test_ext().execute_with(|| {
		let owner = account(ALICE);
		let spender = account(BOB);
		let (collection, item) =
			create_collection_mint_and_approve(owner.clone(), ITEM, spender.clone());
		// Successfully cancel the transfer approval of `spender` by `owner`.
		assert_ok!(NonFungibles::approve(
			signed(owner),
			collection,
			Some(item),
			spender.clone(),
			false
		));
		// Failed to transfer the item by `spender` without permission.
		assert!(Nfts::transfer(signed(spender.clone()), collection, item, spender).is_err());
	});
}

#[test]
fn owner_of_works() {}

#[test]
fn collection_owner_works() {
	new_test_ext().execute_with(|| {
		let collection = create_collection(account(ALICE));
		assert_eq!(
			NonFungibles::read(CollectionOwner(collection)).encode(),
			Nfts::collection_owner(collection).encode()
		);
	});
}

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let (collection, _) = create_collection_mint(account(ALICE), ITEM);
		assert_eq!(
			NonFungibles::read(TotalSupply(collection)).encode(),
			Nfts::collection_items(collection).unwrap_or_default().encode()
		);
	});
}

#[test]
fn collection_works() {
	new_test_ext().execute_with(|| {
		let (collection, _) = create_collection_mint(account(ALICE), ITEM);
		assert_eq!(
			NonFungibles::read(Collection(collection)).encode(),
			pallet_nfts::Collection::<Test>::get(&collection).encode(),
		);
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let owner = account(ALICE);
		let (collection, _) = create_collection_mint(owner.clone(), ITEM);
		assert_eq!(
			NonFungibles::read(BalanceOf { collection, owner: owner.clone() }).encode(),
			AccountBalance::<Test>::get(collection, owner).encode()
		);
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let owner = account(ALICE);
		let operator = account(BOB);
		let (collection, item) =
			create_collection_mint_and_approve(owner.clone(), ITEM, operator.clone());
		assert_eq!(
			NonFungibles::read(Allowance {
				collection,
				item: Some(item),
				owner: owner.clone(),
				operator: operator.clone(),
			})
			.encode(),
			Nfts::check_allowance(&collection, &Some(item), &owner, &operator)
				.is_ok()
				.encode()
		);
	});
}

fn signed(account: AccountId) -> RuntimeOrigin {
	RuntimeOrigin::signed(account)
}

fn create_collection_mint_and_approve(
	owner: AccountIdOf<Test>,
	item: ItemIdOf<Test>,
	spender: AccountIdOf<Test>,
) -> (u32, u32) {
	let (collection, item) = create_collection_mint(owner.clone(), item);
	assert_ok!(Nfts::approve_transfer(signed(owner), collection, Some(item), spender, None));
	(collection, item)
}

fn create_collection_mint(owner: AccountIdOf<Test>, item: ItemIdOf<Test>) -> (u32, u32) {
	let collection = create_collection(owner.clone());
	assert_ok!(Nfts::mint(signed(owner.clone()), collection, item, owner, None));
	(collection, item)
}

fn create_collection(owner: AccountIdOf<Test>) -> u32 {
	let next_id = next_collection_id();
	assert_ok!(Nfts::create(
		signed(owner.clone()),
		owner.clone(),
		collection_config_with_all_settings_enabled()
	));
	next_id
}

fn next_collection_id() -> u32 {
	pallet_nfts::NextCollectionId::<Test>::get().unwrap_or_default()
}

fn collection_config_with_all_settings_enabled(
) -> CollectionConfig<Balance, BlockNumberFor<Test>, CollectionIdOf<Test>> {
	CollectionConfig {
		settings: CollectionSettings::all_enabled(),
		max_supply: None,
		mint_settings: MintSettings::default(),
	}
}

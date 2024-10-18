use codec::Encode;
use frame_support::assert_ok;
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_nfts::{AccountBalance, CollectionConfig, CollectionSettings, MintSettings};

use super::types::{AccountIdOf, CollectionIdOf, ItemIdOf};
use crate::{
	mock::*,
	nonfungibles::{Event, Read::*, ReadResult},
	Read,
};

const ITEM: u32 = 1;

mod encoding_read_result {
	use pallet_nfts::{CollectionDetails, ItemDeposit, ItemDetails};
	use sp_runtime::{BoundedBTreeMap, BoundedVec};

	use super::*;

	#[test]
	fn total_supply() {
		let total_supply: u32 = 1_000_000;
		assert_eq!(ReadResult::TotalSupply::<Test>(total_supply).encode(), total_supply.encode());
	}

	#[test]
	fn balance_of() {
		let balance: u32 = 100;
		assert_eq!(ReadResult::BalanceOf::<Test>(balance).encode(), balance.encode());
	}

	#[test]
	fn allowance() {
		let allowance = false;
		assert_eq!(ReadResult::Allowance::<Test>(allowance).encode(), allowance.encode());
	}

	#[test]
	fn owner_of() {
		let mut owner = Some(account(ALICE));
		assert_eq!(ReadResult::OwnerOf::<Test>(owner.clone()).encode(), owner.encode());
		owner = None;
		assert_eq!(ReadResult::OwnerOf::<Test>(owner.clone()).encode(), owner.encode());
	}

	#[test]
	fn get_attribute() {
		let mut attribute = Some(BoundedVec::truncate_from("some attribute".as_bytes().to_vec()));
		assert_eq!(
			ReadResult::GetAttribute::<Test>(attribute.clone()).encode(),
			attribute.encode()
		);
		attribute = None;
		assert_eq!(
			ReadResult::GetAttribute::<Test>(attribute.clone()).encode(),
			attribute.encode()
		);
	}

	#[test]
	fn collection_owner() {
		let mut collection_owner = Some(account(ALICE));
		assert_eq!(
			ReadResult::CollectionOwner::<Test>(collection_owner.clone()).encode(),
			collection_owner.encode()
		);
		collection_owner = None;
		assert_eq!(
			ReadResult::CollectionOwner::<Test>(collection_owner.clone()).encode(),
			collection_owner.encode()
		);
	}

	#[test]
	fn collection() {
		let mut collection_details = Some(CollectionDetails {
			owner: account(ALICE),
			owner_deposit: 0,
			items: 0,
			item_metadatas: 0,
			item_configs: 0,
			attributes: 0,
		});
		assert_eq!(
			ReadResult::Collection::<Test>(collection_details.clone()).encode(),
			collection_details.encode()
		);
		collection_details = None;
		assert_eq!(
			ReadResult::Collection::<Test>(collection_details.clone()).encode(),
			collection_details.encode()
		);
	}

	#[test]
	fn item() {
		let mut item_details = Some(ItemDetails {
			owner: account(ALICE),
			approvals: BoundedBTreeMap::default(),
			deposit: ItemDeposit { amount: 0, account: account(BOB) },
		});
		assert_eq!(ReadResult::Item::<Test>(item_details.clone()).encode(), item_details.encode());
		item_details = None;
		assert_eq!(ReadResult::Item::<Test>(item_details.clone()).encode(), item_details.encode());
	}
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let owner = account(ALICE);
		let collection = create_collection(owner.clone());
		// Successfully mint a new collection item.
		let balance_before_mint = AccountBalance::<Test>::get(collection.clone(), owner.clone());
		//
		assert_ok!(NonFungibles::mint(
			signed(owner.clone()),
			owner.clone(),
			collection,
			ITEM,
			None
		));
		let balance_after_mint = AccountBalance::<Test>::get(collection.clone(), owner.clone());
		assert_eq!(balance_after_mint, 1);
		assert_eq!(balance_after_mint - balance_before_mint, 1);
		System::assert_last_event(
			Event::Transfer { collection, item: ITEM, from: None, to: Some(owner), price: None }
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
			Event::Transfer { collection, item, from: Some(owner), to: None, price: None }.into(),
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
			Event::Transfer { collection, item, from: Some(owner), to: Some(dest), price: None }
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

use frame_support::BoundedVec;
use pallet_nfts::CollectionConfig;
use pop_api::{
	nonfungibles::{
		events::{Approval, AttributeSet, Transfer},
		types::*,
	},
	primitives::BlockNumber,
};
use pop_primitives::{ArithmeticError::*, Error, Error::*, TokenError::*};
use utils::*;

use super::*;

mod utils;

const COLLECTION_ID: CollectionId = 0;
const ITEM_ID: ItemId = 1;
const CONTRACT: &str = "contracts/nonfungibles/target/ink/nonfungibles.wasm";

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(
			total_supply(&addr, COLLECTION_ID),
			Ok(Nfts::collection_items(COLLECTION_ID).unwrap_or_default() as u128)
		);
		assert_eq!(total_supply(&addr, COLLECTION_ID), Ok(0));

		// Tokens in circulation.
		nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM_ID);
		assert_eq!(
			total_supply(&addr, COLLECTION_ID),
			Ok(Nfts::collection_items(COLLECTION_ID).unwrap_or_default() as u128)
		);
		assert_eq!(total_supply(&addr, COLLECTION_ID), Ok(1));
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(
			balance_of(&addr, COLLECTION_ID, ALICE),
			Ok(nfts::balance_of(COLLECTION_ID, ALICE)),
		);
		assert_eq!(total_supply(&addr, COLLECTION_ID), Ok(0));

		// Tokens in circulation.
		nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM_ID);
		assert_eq!(
			balance_of(&addr, COLLECTION_ID, ALICE),
			Ok(nfts::balance_of(COLLECTION_ID, ALICE)),
		);
		assert_eq!(total_supply(&addr, COLLECTION_ID), Ok(1));
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		// No tokens in circulation.
		assert_eq!(
			allowance(&addr.clone(), COLLECTION_ID, addr.clone(), ALICE, None),
			Ok(!Nfts::check_allowance(&COLLECTION_ID, &None, &addr, &ALICE).is_err()),
		);
		assert_eq!(allowance(&addr.clone(), COLLECTION_ID, addr.clone(), ALICE, None), Ok(false));

		let (collection, item) =
			nfts::create_collection_mint_and_approve(&addr, &addr, ITEM_ID, &addr, &ALICE);
		assert_eq!(
			allowance(&addr.clone(), COLLECTION_ID, addr.clone(), ALICE, Some(item)),
			Ok(Nfts::check_allowance(&COLLECTION_ID, &Some(item), &addr.clone(), &ALICE).is_ok()),
		);
		assert_eq!(
			allowance(&addr.clone(), COLLECTION_ID, addr.clone(), ALICE, Some(item)),
			Ok(true)
		);
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM_ID);
		let before_transfer_balance = nfts::balance_of(COLLECTION_ID, ALICE);
		assert_ok!(transfer(&addr, collection, item, ALICE));
		let after_transfer_balance = nfts::balance_of(COLLECTION_ID, ALICE);
		assert_eq!(after_transfer_balance - before_transfer_balance, 1);
	});
}

#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM_ID);
		assert_ok!(approve(&addr, collection, Some(item), ALICE, true));
		assert!(Nfts::check_allowance(&collection, &Some(item), &addr.clone(), &ALICE).is_ok(),);

		assert_ok!(Nfts::transfer(RuntimeOrigin::signed(ALICE), collection, item, BOB.into()));
	});
}

#[test]
fn owner_of_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM_ID);
		assert_eq!(owner_of(&addr, collection, item), Ok(ALICE));
	});
}

// TODO
#[test]
fn get_attribute_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM_ID);

		assert_ok!(Nfts::set_attribute(
			RuntimeOrigin::signed(addr.clone()),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::CollectionOwner,
			BoundedVec::truncate_from("some attribute".as_bytes().to_vec()),
			BoundedVec::truncate_from("some value".as_bytes().to_vec()),
		));
		assert_eq!(
			get_attribute(
				&addr.clone(),
				collection,
				item,
				AttributeNamespace::CollectionOwner,
				"some attribute".as_bytes().to_vec(),
			),
			Ok("some value".as_bytes().to_vec())
		);
	});
}

// TODO
#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM_ID);
		assert_eq!(owner_of(&addr, collection, item), Ok(ALICE));
	});
}

// TODO
#[test]
fn destroy_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM_ID);
		assert_eq!(owner_of(&addr, collection, item), Ok(ALICE));
	});
}

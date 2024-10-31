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
const CONTRACT: &str = "contracts/nonfungibles/target/ink/nonfungibles.riscv";

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		// No tokens in circulation.
		assert_eq!(
			total_supply(&addr, COLLECTION_ID),
			Ok(Nfts::collection_items(COLLECTION_ID).unwrap_or_default() as u128)
		);
		assert_eq!(total_supply(&addr, COLLECTION_ID), Ok(0));

		// Tokens in circulation.
		nfts::create_collection_and_mint_to(&account_id, &account_id, &ALICE, ITEM_ID);
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
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		// No tokens in circulation.
		assert_eq!(
			balance_of(&addr, COLLECTION_ID, ALICE),
			Ok(nfts::balance_of(COLLECTION_ID, ALICE)),
		);
		assert_eq!(total_supply(&addr, COLLECTION_ID), Ok(0));

		// Tokens in circulation.
		nfts::create_collection_and_mint_to(&account_id, &account_id, &ALICE, ITEM_ID);
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
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);
		// No tokens in circulation.
		assert_eq!(
			allowance(&addr.clone(), COLLECTION_ID, account_id.clone(), ALICE, None),
			Ok(!Nfts::check_allowance(&COLLECTION_ID, &None, &account_id, &ALICE).is_err()),
		);
		assert_eq!(
			allowance(&addr.clone(), COLLECTION_ID, account_id.clone(), ALICE, None),
			Ok(false)
		);

		let (_, item) = nfts::create_collection_mint_and_approve(
			&account_id,
			&account_id,
			ITEM_ID,
			&account_id,
			&ALICE,
		);
		assert_eq!(
			allowance(&addr.clone(), COLLECTION_ID, account_id.clone(), ALICE, Some(item)),
			Ok(Nfts::check_allowance(&COLLECTION_ID, &Some(item), &account_id.clone(), &ALICE)
				.is_ok()),
		);
		assert_eq!(
			allowance(&addr.clone(), COLLECTION_ID, account_id.clone(), ALICE, Some(item)),
			Ok(true)
		);
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &account_id, ITEM_ID);
		let before_transfer_balance = nfts::balance_of(COLLECTION_ID, ALICE);
		assert_ok!(transfer(&addr, collection, item, ALICE));
		let after_transfer_balance = nfts::balance_of(COLLECTION_ID, ALICE);
		assert_eq!(after_transfer_balance - before_transfer_balance, 1);
	});
}

#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &account_id, ITEM_ID);
		assert_ok!(approve(&addr, collection, Some(item), ALICE, true));
		assert!(
			Nfts::check_allowance(&collection, &Some(item), &account_id.clone(), &ALICE).is_ok(),
		);

		assert_ok!(Nfts::transfer(RuntimeOrigin::signed(ALICE), collection, item, BOB.into()));
	});
}

#[test]
fn owner_of_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);
		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &ALICE, ITEM_ID);
		assert_eq!(owner_of(&addr, collection, item), Ok(ALICE));
	});
}

#[test]
fn get_attribute_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &account_id, ITEM_ID);

		assert_ok!(Nfts::set_attribute(
			RuntimeOrigin::signed(account_id.clone()),
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
			Ok(Some("some value".as_bytes().to_vec()))
		);
	});
}

#[test]
fn set_attribute_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &account_id, ITEM_ID);

		assert_ok!(set_attribute(
			&addr.clone(),
			collection,
			item,
			AttributeNamespace::CollectionOwner,
			"some attribute".as_bytes().to_vec(),
			"some value".as_bytes().to_vec(),
		));

		assert_eq!(
			pallet_nfts::Attribute::<Runtime>::get((
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::CollectionOwner,
				AttributeKey::truncate_from("some attribute".as_bytes().to_vec()),
			))
			.map(|attribute| attribute.0),
			Some(AttributeValue::truncate_from("some value".as_bytes().to_vec()))
		);
	});
}

#[test]
fn clear_attribute_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &account_id, ITEM_ID);
		assert_ok!(Nfts::set_attribute(
			RuntimeOrigin::signed(account_id.clone()),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::CollectionOwner,
			BoundedVec::truncate_from("some attribute".as_bytes().to_vec()),
			BoundedVec::truncate_from("some value".as_bytes().to_vec()),
		));
		assert_ok!(clear_attribute(
			&addr.clone(),
			collection,
			item,
			AttributeNamespace::CollectionOwner,
			"some attribute".as_bytes().to_vec()
		));
		assert_eq!(
			pallet_nfts::Attribute::<Runtime>::get((
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::CollectionOwner,
				AttributeKey::truncate_from("some attribute".as_bytes().to_vec()),
			))
			.map(|attribute| attribute.0),
			None
		);
	});
}

#[test]
fn approve_item_attributes_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &account_id, ITEM_ID);
		assert_ok!(approve_item_attributes(&addr.clone(), collection, item, ALICE));
		assert_ok!(Nfts::set_attribute(
			RuntimeOrigin::signed(ALICE),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::Account(ALICE),
			BoundedVec::truncate_from("some attribute".as_bytes().to_vec()),
			BoundedVec::truncate_from("some value".as_bytes().to_vec()),
		));
		assert_eq!(
			pallet_nfts::Attribute::<Runtime>::get((
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::Account(ALICE),
				AttributeKey::truncate_from("some attribute".as_bytes().to_vec()),
			))
			.map(|attribute| attribute.0),
			Some(AttributeValue::truncate_from("some value".as_bytes().to_vec()))
		);
	});
}

#[test]
fn cancel_item_attributes_approval_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &account_id, ITEM_ID);
		assert_ok!(Nfts::approve_item_attributes(
			RuntimeOrigin::signed(account_id.clone()),
			collection,
			item,
			ALICE.into()
		));
		assert_ok!(Nfts::set_attribute(
			RuntimeOrigin::signed(ALICE),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::Account(ALICE),
			BoundedVec::truncate_from("some attribute".as_bytes().to_vec()),
			BoundedVec::truncate_from("some value".as_bytes().to_vec()),
		));
		assert_ok!(cancel_item_attributes_approval(
			&addr.clone(),
			collection,
			item,
			ALICE,
			CancelAttributesApprovalWitness { account_attributes: 1 }
		));
		assert!(Nfts::set_attribute(
			RuntimeOrigin::signed(ALICE),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::Account(ALICE),
			BoundedVec::truncate_from("some attribute".as_bytes().to_vec()),
			BoundedVec::truncate_from("some value".as_bytes().to_vec()),
		)
		.is_err());
	});
}

#[test]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &account_id, ITEM_ID);
		assert_ok!(set_metadata(&addr.clone(), collection, item, vec![]));
		assert_eq!(Nfts::item_metadata(collection, item), Some(MetadataData::default()));
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &account_id, ITEM_ID);
		assert_ok!(Nfts::set_metadata(
			RuntimeOrigin::signed(account_id.clone()),
			collection,
			item,
			MetadataData::default()
		));
		assert_ok!(clear_metadata(&addr.clone(), collection, item));
		assert_eq!(Nfts::item_metadata(collection, item), None);
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let collection = nfts::next_collection_id();
		assert_ok!(create(
			&addr.clone(),
			COLLECTION_ID,
			account_id.clone(),
			CreateCollectionConfig {
				max_supply: Some(100),
				mint_type: MintType::Public,
				price: None,
				start_block: None,
				end_block: None,
			}
		));
		assert_eq!(
			pallet_nfts::Collection::<Runtime>::get(collection),
			Some(pallet_nfts::CollectionDetails {
				owner: account_id.clone(),
				owner_deposit: 100000000000,
				items: 0,
				item_metadatas: 0,
				item_configs: 0,
				attributes: 0,
			})
		);
	});
}

#[test]
fn destroy_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let collection = nfts::create_collection(&account_id, &account_id);
		assert_ok!(destroy(
			&addr.clone(),
			collection,
			DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
		));
		assert_eq!(pallet_nfts::Collection::<Runtime>::get(collection), None);
	});
}

#[test]
fn set_max_supply_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);
		let value = 10;

		let collection = nfts::create_collection(&account_id, &account_id);
		assert_ok!(set_max_supply(&addr.clone(), collection, value));

		(0..value).into_iter().for_each(|i| {
			assert_ok!(Nfts::mint(
				RuntimeOrigin::signed(account_id.clone()),
				collection,
				i,
				ALICE.into(),
				None
			));
		});
		assert!(Nfts::mint(
			RuntimeOrigin::signed(account_id.clone()),
			collection,
			value + 1,
			ALICE.into(),
			None
		)
		.is_err());
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);
		let value = 10;

		let collection = nfts::create_collection(&account_id, &account_id);
		assert_ok!(mint(
			&addr.clone(),
			ALICE,
			collection,
			ITEM_ID,
			MintWitness { mint_price: None, owned_item: None }
		));
		assert_eq!(nfts::balance_of(COLLECTION_ID, ALICE), 1);
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let (addr, account_id) =
			instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]);

		let (collection, item) =
			nfts::create_collection_and_mint_to(&account_id, &account_id, &account_id, ITEM_ID);
		assert_ok!(burn(&addr.clone(), collection, ITEM_ID,));
		assert_eq!(nfts::balance_of(COLLECTION_ID, account_id), 0);
	});
}

use frame_support::BoundedVec;
use pallet_api::nonfungibles::types::*;
use pop_api::{
	nonfungibles::{
		events::{Approval, AttributeSet, Transfer},
		AttributeNamespace, CancelAttributesApprovalWitness, CollectionConfig, CollectionId,
		CollectionSettings, DestroyWitness, ItemId, MintSettings, MintWitness,
	},
	primitives::BlockNumber,
};
use pop_primitives::{ArithmeticError::*, Error, Error::*, TokenError::*};
use utils::*;

use super::*;

mod utils;

const ITEM: ItemId = 0;
const COLLECTION: CollectionId = 0;
const CONTRACT: &str = "contracts/nonfungibles/target/ink/nonfungibles.wasm";

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No collection item is created.
		assert_eq!(
			total_supply(&addr, COLLECTION),
			Ok(Nfts::collection_items(COLLECTION).unwrap_or_default() as u128)
		);
		assert_eq!(total_supply(&addr, COLLECTION), Ok(0));

		// Collection item is created.
		nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM);
		assert_eq!(
			total_supply(&addr, COLLECTION),
			Ok(Nfts::collection_items(COLLECTION).unwrap_or_default() as u128)
		);
		assert_eq!(total_supply(&addr, COLLECTION), Ok(1));
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No collection item is created.
		assert_eq!(balance_of(&addr, COLLECTION, ALICE), Ok(nfts::balance_of(COLLECTION, ALICE)),);
		assert_eq!(total_supply(&addr, COLLECTION), Ok(0));

		// Collection item is created.
		nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM);
		assert_eq!(balance_of(&addr, COLLECTION, ALICE), Ok(nfts::balance_of(COLLECTION, ALICE)),);
		assert_eq!(total_supply(&addr, COLLECTION), Ok(1));
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No collection item is created.
		assert_eq!(
			allowance(&addr.clone(), COLLECTION, None, addr.clone(), ALICE),
			Ok(!Nfts::check_approval_permission(&COLLECTION, &None, &addr, &ALICE).is_err()),
		);
		assert_eq!(allowance(&addr.clone(), COLLECTION, None, addr.clone(), ALICE), Ok(false));

		// Collection item is created.
		let (_, item) = nfts::create_collection_mint_and_approve(&addr, &addr, ITEM, &addr, &ALICE);
		assert_eq!(
			allowance(&addr.clone(), COLLECTION, Some(item), addr.clone(), ALICE),
			Ok(Nfts::check_approval_permission(&COLLECTION, &Some(item), &addr.clone(), &ALICE)
				.is_ok()),
		);
		assert_eq!(allowance(&addr.clone(), COLLECTION, Some(item), addr.clone(), ALICE), Ok(true));
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection item does not exist, throws module error `UnknownItem`.
		assert_eq!(
			transfer(&addr, COLLECTION, ITEM, ALICE),
			Err(Module { index: 50, error: [20, 0] })
		);
		// Create a collection and mint to a contract address.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		// Privilege to transfer a collection item is locked.
		nfts::lock_item_transfer(&addr, COLLECTION, ITEM);
		assert_eq!(
			transfer(&addr, COLLECTION, ITEM, ALICE),
			Err(Module { index: 50, error: [12, 0] })
		);
		nfts::unlock_item_transfer(&addr, COLLECTION, ITEM);
		// Successful transfer.
		let before_transfer_balance = nfts::balance_of(COLLECTION, ALICE);
		assert_ok!(transfer(&addr, collection, item, ALICE));
		let after_transfer_balance = nfts::balance_of(COLLECTION, ALICE);
		assert_eq!(after_transfer_balance - before_transfer_balance, 1);
		// Successfully emit event.
		let from = account_id_from_slice(addr.as_ref());
		let to = account_id_from_slice(ALICE.as_ref());
		let expected = Transfer { from: Some(from), to: Some(to), item: ITEM }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// Collection item does not exist, i.e. burnt.
		nfts::burn(COLLECTION, ITEM, &ALICE);
		assert_eq!(
			transfer(&addr, COLLECTION, ITEM, ALICE),
			Err(Module { index: 50, error: [20, 0] })
		);
	});
}

#[test]
fn approve_item_transfer_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist, throws module error `UnknownItem`.
		assert_eq!(
			approve(&addr, COLLECTION, Some(ITEM), ALICE, true),
			Err(Module { index: 50, error: [20, 0] })
		);
		// Successful approvals.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(approve(&addr, collection, Some(item), ALICE, true));
		assert!(Nfts::check_approval_permission(&collection, &Some(item), &addr.clone(), &ALICE)
			.is_ok());
		// Successfully emit event.
		let owner = account_id_from_slice(addr.as_ref());
		let operator = account_id_from_slice(ALICE.as_ref());
		let expected = Approval { owner, operator, item: Some(item), approved: true }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// New value overrides old value.
		assert_ok!(approve(&addr, collection, Some(item), ALICE, false));
		assert!(Nfts::check_approval_permission(&collection, &Some(item), &addr.clone(), &ALICE)
			.is_err());
	});
}

#[test]
fn approve_collection_transfer_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist, throws module error `NoItemOwned`.
		assert_eq!(
			approve(&addr, COLLECTION, None, ALICE, true),
			Err(Module { index: 50, error: [45, 0] })
		);
		// Successful approvals.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(approve(&addr, collection, None, ALICE, true));
		assert!(Nfts::check_approval_permission(&collection, &None, &addr.clone(), &ALICE).is_ok());
		// Successfully emit event.
		let owner = account_id_from_slice(addr.as_ref());
		let operator = account_id_from_slice(ALICE.as_ref());
		let expected = Approval { owner, operator, item: None, approved: true }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// New value overrides old value.
		assert_ok!(approve(&addr, collection, None, ALICE, false));
		assert!(Nfts::check_approval_permission(&collection, &None, &addr.clone(), &ALICE).is_err());
	});
}

#[test]
fn owner_of_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM);
		assert_eq!(owner_of(&addr, collection, item), Ok(ALICE));
	});
}

#[test]
fn get_attribute_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);

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
			Ok(Some("some value".as_bytes().to_vec()))
		);
	});
}

#[test]
fn set_attribute_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);

		assert_ok!(set_attribute(
			&addr.clone(),
			collection,
			item,
			AttributeNamespace::CollectionOwner,
			"some attribute".as_bytes().to_vec(),
			"some value".as_bytes().to_vec(),
		));

		assert_eq!(
			AttributeOf::<Runtime>::get((
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::CollectionOwner,
				AttributeKey::<Runtime>::truncate_from("some attribute".as_bytes().to_vec()),
			))
			.map(|attribute| attribute.0),
			Some(AttributeValue::<Runtime>::truncate_from("some value".as_bytes().to_vec()))
		);
	});
}

#[test]
fn clear_attribute_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(Nfts::set_attribute(
			RuntimeOrigin::signed(addr.clone()),
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
			AttributeOf::<Runtime>::get((
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::CollectionOwner,
				AttributeKey::<Runtime>::truncate_from("some attribute".as_bytes().to_vec()),
			))
			.map(|attribute| attribute.0),
			None
		);
	});
}

#[test]
fn approve_item_attributes_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
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
			AttributeOf::<Runtime>::get((
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::Account(ALICE),
				AttributeKey::<Runtime>::truncate_from("some attribute".as_bytes().to_vec()),
			))
			.map(|attribute| attribute.0),
			Some(AttributeValue::<Runtime>::truncate_from("some value".as_bytes().to_vec()))
		);
	});
}

#[test]
fn cancel_item_attributes_approval_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(Nfts::approve_item_attributes(
			RuntimeOrigin::signed(addr.clone()),
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
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist.
		assert_eq!(
			set_metadata(&addr.clone(), COLLECTION, ITEM, vec![]),
			Err(Module { index: 50, error: [0, 0] })
		);
		// No Permission.
		let (collection, item) = nfts::create_collection_and_mint_to(&ALICE, &ALICE, &ALICE, ITEM);
		assert_eq!(
			set_metadata(&addr.clone(), collection, item, vec![]),
			Err(Module { index: 50, error: [0, 0] }),
		);
		// Successful set metadata.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(set_metadata(&addr.clone(), collection, item, vec![]));
		assert_eq!(Nfts::item_metadata(collection, item), Some(MetadataData::<Runtime>::default()));
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist.
		assert_eq!(
			clear_metadata(&addr.clone(), COLLECTION, ITEM),
			Err(Module { index: 50, error: [0, 0] })
		);
		// Successful clear metadata.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(Nfts::set_metadata(
			RuntimeOrigin::signed(addr.clone()),
			collection,
			item,
			MetadataData::<Runtime>::default()
		));
		assert_ok!(clear_metadata(&addr.clone(), collection, item));
		assert_eq!(Nfts::item_metadata(collection, item), None);
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		assert_ok!(create(
			&addr.clone(),
			addr.clone(),
			CollectionConfig {
				max_supply: Some(100),
				mint_settings: MintSettings::default(),
				settings: CollectionSettings::all_enabled(),
			}
		));
		assert_eq!(Nfts::collection_owner(COLLECTION), Some(addr.clone()));
	});
}

#[test]
fn destroy_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist.
		assert_eq!(
			destroy(
				&addr.clone(),
				COLLECTION,
				DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
			),
			Err(Module { index: 50, error: [1, 0] })
		);
		// Destroying can only be done by the collection owner.
		let collection = nfts::create_collection(&ALICE, &ALICE);
		assert_eq!(
			destroy(
				&addr.clone(),
				collection,
				DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
			),
			Err(Module { index: 50, error: [0, 0] })
		);
		// Successful destroy.
		let collection = nfts::create_collection(&addr, &addr);
		assert_ok!(destroy(
			&addr.clone(),
			collection,
			DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
		));
		assert_eq!(CollectionOf::<Runtime>::get(collection), None);
	});
}

#[test]
fn set_max_supply_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let value = 10;

		// Collection does not exist.
		assert_eq!(
			set_max_supply(&addr.clone(), COLLECTION, value),
			Err(Module { index: 50, error: [32, 0] })
		);
		// Sucessfully set max supply.
		let collection = nfts::create_collection(&addr, &addr);
		assert_ok!(set_max_supply(&addr.clone(), collection, value));
		assert_eq!(nfts::max_supply(collection), Some(value));
		// Non-additive, sets new value.
		assert_ok!(set_max_supply(&addr.clone(), collection, value + 1));
		assert_eq!(nfts::max_supply(collection), Some(value + 1));
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let value = 10;

		// Collection does not exist.
		assert_eq!(
			mint(
				&addr.clone(),
				ALICE,
				COLLECTION,
				ITEM,
				MintWitness { mint_price: None, owned_item: None }
			),
			Err(Module { index: 50, error: [32, 0] })
		);
		// Mitning can only be done by the collection owner.
		let collection = nfts::create_collection(&ALICE, &ALICE);
		assert_eq!(
			mint(
				&addr.clone(),
				ALICE,
				collection,
				ITEM,
				MintWitness { mint_price: None, owned_item: None }
			),
			Err(Module { index: 50, error: [0, 0] })
		);
		// Successful mint.
		let collection = nfts::create_collection(&addr, &addr);
		assert_ok!(mint(
			&addr.clone(),
			ALICE,
			collection,
			ITEM,
			MintWitness { owned_item: None, mint_price: None }
		));
		assert_eq!(nfts::balance_of(collection, ALICE), 1);
		// Minting an existing item ID.
		assert_eq!(
			mint(
				&addr.clone(),
				ALICE,
				collection,
				ITEM,
				MintWitness { owned_item: None, mint_price: None }
			),
			Err(Module { index: 50, error: [2, 0] })
		);
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection item does not exist.
		assert_eq!(
			burn(&addr.clone(), COLLECTION, ITEM),
			Err(Module { index: 50, error: [20, 0] })
		);
		// Burning can only be done by the collection item owner.
		let (collection, item) = nfts::create_collection_and_mint_to(&ALICE, &ALICE, &BOB, ITEM);
		assert_eq!(burn(&addr.clone(), collection, item), Err(Module { index: 50, error: [0, 0] }));
		// Successful burn.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(burn(&addr.clone(), collection, item));
		assert_eq!(nfts::balance_of(COLLECTION, addr.clone()), 0);
	});
}

use pallet_api::nonfungibles::{
	AccountBalanceOf, AttributeKeyOf, AttributeOf, AttributeValueOf, CollectionApprovalsOf,
	CollectionConfigOf, CollectionDetailsOf, CollectionOf, MetadataOf, NextCollectionIdOf,
};
use pop_api::{
	nonfungibles::{
		events::{Approval, AttributeSet, Transfer},
		AttributeNamespace, CancelAttributesApprovalWitness, CollectionConfig, CollectionId,
		CollectionSettings, DestroyWitness, ItemId, MintSettings, MintWitness,
	},
	primitives::BlockNumber,
};
use pop_primitives::{Error, Error::*};
use sp_runtime::traits::Zero;
use utils::{collection, *};

use super::*;

mod utils;

type Attribute = AttributeOf<Runtime>;
type AttributeValue = AttributeValueOf<Runtime>;
type AttributeKey = AttributeKeyOf<Runtime>;
type CollectionDetails = CollectionDetailsOf<Runtime>;
type Metadata = MetadataOf<Runtime>;

const ITEM: ItemId = 0;
const COLLECTION: CollectionId = 0;
const CONTRACT: &str = "contracts/nonfungibles/target/ink/nonfungibles.wasm";

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
fn owner_of_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM);
		assert_eq!(owner_of(&addr, collection, item), Ok(ALICE));
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No collection item is created.
		assert_eq!(
			allowance(&addr, COLLECTION, None, addr.clone(), ALICE),
			Ok(!Nfts::check_approval_permission(&COLLECTION, &None, &addr, &ALICE).is_err()),
		);
		assert_eq!(allowance(&addr, COLLECTION, None, addr.clone(), ALICE), Ok(false));

		// Collection item is created.
		let (_, item) = nfts::create_collection_mint_and_approve(&addr, &addr, ITEM, &addr, &ALICE);
		assert_eq!(
			allowance(&addr, COLLECTION, Some(item), addr.clone(), ALICE),
			Ok(Nfts::check_approval_permission(&COLLECTION, &Some(item), &addr, &ALICE).is_ok()),
		);
		assert_eq!(allowance(&addr, COLLECTION, Some(item), addr.clone(), ALICE), Ok(true));
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection item does not exist, throws module error `UnknownItem`.
		assert_eq!(
			transfer(&addr, COLLECTION, ITEM, ALICE),
			Err(Module { index: 50, error: [19, 0] })
		);
		// Create a collection and mint to a contract address.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		// Privilege to transfer a collection item is locked.
		nfts::lock_item_transfer(&addr, COLLECTION, ITEM);
		assert_eq!(
			transfer(&addr, COLLECTION, ITEM, ALICE),
			Err(Module { index: 50, error: [11, 0] })
		);
		nfts::unlock_item_transfer(&addr, COLLECTION, ITEM);
		// Successfully transfer.
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
			Err(Module { index: 50, error: [19, 0] })
		);
	});
}

mod approve {
	use super::*;

	#[test]
	fn approve_item_transfer_works() {
		new_test_ext().execute_with(|| {
			let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

			// Collection does not exist, throws module error `UnknownItem`.
			assert_eq!(
				approve(&addr, COLLECTION, Some(ITEM), ALICE, true),
				Err(Module { index: 50, error: [19, 0] })
			);
			// Successfully approvals.
			let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
			assert_ok!(approve(&addr, collection, Some(item), ALICE, true));
			assert!(
				Nfts::check_approval_permission(&collection, &Some(item), &addr, &ALICE).is_ok()
			);
			// Successfully emit event.
			let owner = account_id_from_slice(addr.as_ref());
			let operator = account_id_from_slice(ALICE.as_ref());
			let expected = Approval { owner, operator, item: Some(item), approved: true }.encode();
			assert_eq!(last_contract_event(), expected.as_slice());
			// New value overrides old value.
			assert_ok!(approve(&addr, collection, Some(item), ALICE, false));
			assert!(
				Nfts::check_approval_permission(&collection, &Some(item), &addr, &ALICE).is_err()
			);
		});
	}

	#[test]
	fn approve_collection_transfer_works() {
		new_test_ext().execute_with(|| {
			let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

			// Collection does not exist, throws module error `NoItemOwned`.
			assert_eq!(
				approve(&addr, COLLECTION, None, ALICE, true),
				Err(Module { index: 50, error: [44, 0] })
			);
			// Successfully approvals.
			let (collection, _) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
			assert_ok!(approve(&addr, collection, None, ALICE, true));
			assert!(Nfts::check_approval_permission(&collection, &None, &addr, &ALICE).is_ok());
			// Successfully emit event.
			let owner = account_id_from_slice(addr.as_ref());
			let operator = account_id_from_slice(ALICE.as_ref());
			let expected = Approval { owner, operator, item: None, approved: true }.encode();
			assert_eq!(last_contract_event(), expected.as_slice());
			// New value overrides old value.
			assert_ok!(approve(&addr, collection, None, ALICE, false));
			assert!(Nfts::check_approval_permission(&collection, &None, &addr, &ALICE).is_err());
		});
	}
}

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
fn get_attribute_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);

		assert_ok!(Nfts::set_attribute(
			RuntimeOrigin::signed(addr.clone()),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::CollectionOwner,
			AttributeKey::truncate_from("some attribute".as_bytes().to_vec()),
			AttributeValue::truncate_from("some value".as_bytes().to_vec()),
		));
		assert_eq!(
			get_attribute(
				&addr,
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				"some attribute".as_bytes().to_vec(),
			),
			Ok(Some("some value".as_bytes().to_vec()))
		);
	});
}

#[test]
fn collection_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist.
		assert_eq!(collection(&addr, COLLECTION), Ok(None));

		// Collection is created.
		nfts::create_collection(&addr, &ALICE);
		assert_eq!(
			collection(&addr, COLLECTION),
			Ok(Some(CollectionDetails {
				owner: addr,
				owner_deposit: 0,
				items: 0,
				item_metadatas: 0,
				item_configs: 0,
				attributes: 0,
			}))
		);
	});
}

#[test]
fn next_collection_id_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		assert_eq!(next_collection_id(&addr), Ok(Some(COLLECTION)));
	});
}

#[test]
fn item_metadata_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let metadata = Metadata::truncate_from("some metadata".as_bytes().to_vec());

		// Collection metadata is not set.
		assert_eq!(Nfts::item_metadata(COLLECTION, ITEM), None);
		// Successfully set metadata.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(Nfts::set_metadata(
			RuntimeOrigin::signed(addr.clone()),
			collection,
			item,
			metadata.clone()
		));
		assert_eq!(Nfts::item_metadata(collection, item), Some(metadata));
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		assert_ok!(create(
			&addr,
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
				&addr,
				COLLECTION,
				DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
			),
			Err(Module { index: 50, error: [1, 0] })
		);
		// Destroying can only be done by the collection owner.
		let collection = nfts::create_collection(&ALICE, &ALICE);
		assert_eq!(
			destroy(
				&addr,
				collection,
				DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
			),
			Err(Module { index: 50, error: [0, 0] })
		);
		// Successfully destroy.
		let collection = nfts::create_collection(&addr, &addr);
		assert_ok!(destroy(
			&addr,
			collection,
			DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
		));
		assert_eq!(CollectionOf::<Runtime>::get(collection), None);
	});
}

#[test]
fn set_attribute_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let attribute_key = "some attribute".as_bytes().to_vec();
		let attribute_value = "some value".as_bytes().to_vec();

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);

		assert_ok!(set_attribute(
			&addr,
			collection,
			Some(item),
			AttributeNamespace::CollectionOwner,
			attribute_key.clone(),
			attribute_value.clone()
		));

		assert_eq!(
			Attribute::get((
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::CollectionOwner,
				AttributeKey::truncate_from(attribute_key),
			))
			.map(|attribute| attribute.0),
			Some(AttributeValue::truncate_from(attribute_value))
		);
	});
}

#[test]
fn clear_attribute_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let attribute_key = "some attribute".as_bytes().to_vec();
		let attribute_value = "some value".as_bytes().to_vec();

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(Nfts::set_attribute(
			RuntimeOrigin::signed(addr.clone()),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::CollectionOwner,
			AttributeKey::truncate_from(attribute_key.clone()),
			AttributeValue::truncate_from(attribute_value),
		));
		assert_ok!(clear_attribute(
			&addr,
			collection,
			Some(item),
			AttributeNamespace::CollectionOwner,
			attribute_key.clone()
		));
		assert_eq!(
			Attribute::get((
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::CollectionOwner,
				AttributeKey::truncate_from(attribute_key),
			))
			.map(|attribute| attribute.0),
			None
		);
	});
}

#[test]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist, throws module error `NoPermission`.
		assert_eq!(
			set_metadata(&addr, COLLECTION, ITEM, vec![]),
			Err(Module { index: 50, error: [0, 0] })
		);
		// Collection exists but no permission, throws module error `NoPermission`.
		let (collection, item) = nfts::create_collection_and_mint_to(&ALICE, &ALICE, &ALICE, ITEM);
		assert_eq!(
			set_metadata(&addr, collection, item, vec![]),
			Err(Module { index: 50, error: [0, 0] }),
		);
		// Successfully set metadata.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(set_metadata(&addr, collection, item, vec![]));
		assert_eq!(Nfts::item_metadata(collection, item), Some(Metadata::default()));
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist.
		assert_eq!(
			clear_metadata(&addr, COLLECTION, ITEM),
			Err(Module { index: 50, error: [0, 0] })
		);
		// Successfully clear metadata.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(Nfts::set_metadata(
			RuntimeOrigin::signed(addr.clone()),
			collection,
			item,
			Metadata::default()
		));
		assert_ok!(clear_metadata(&addr, collection, item));
		assert_eq!(Nfts::item_metadata(collection, item), None);
	});
}

#[test]
fn set_max_supply_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let value = 10;

		// Collection does not exist.
		assert_eq!(
			set_max_supply(&addr, COLLECTION, value),
			Err(Module { index: 50, error: [31, 0] })
		);
		// Sucessfully set max supply.
		let collection = nfts::create_collection(&addr, &addr);
		assert_ok!(set_max_supply(&addr, collection, value));
		assert_eq!(nfts::max_supply(collection), Some(value));
		// Non-additive, sets new value.
		assert_ok!(set_max_supply(&addr, collection, value + 1));
		assert_eq!(nfts::max_supply(collection), Some(value + 1));
	});
}

#[test]
fn approve_item_attributes_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let attribute_key = AttributeKey::truncate_from("some attribute".as_bytes().to_vec());
		let attribute_value = AttributeValue::truncate_from("some value".as_bytes().to_vec());

		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(approve_item_attributes(&addr, collection, item, ALICE));
		assert_ok!(Nfts::set_attribute(
			RuntimeOrigin::signed(ALICE),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::Account(ALICE),
			attribute_key.clone(),
			attribute_value.clone()
		));
		assert_eq!(
			Attribute::get((
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::Account(ALICE),
				attribute_key
			))
			.map(|attribute| attribute.0),
			Some(attribute_value)
		);
	});
}

#[test]
fn cancel_item_attributes_approval_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let attribute_key = AttributeKey::truncate_from("some attribute".as_bytes().to_vec());
		let attribute_value = AttributeValue::truncate_from("some value".as_bytes().to_vec());

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
			attribute_key.clone(),
			attribute_value.clone()
		));
		assert_ok!(cancel_item_attributes_approval(
			&addr,
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
			attribute_key,
			attribute_value
		)
		.is_err());
	});
}

#[test]
fn clear_all_transfer_approvals_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let operator = ALICE;

		// Collection does not exist, throws module error `UnknownCollection`.
		assert_eq!(
			clear_all_transfer_approvals(&addr, COLLECTION, ITEM),
			Err(Module { index: 50, error: [1, 0] })
		);

		let (collection, item) =
			nfts::create_collection_mint_and_approve(&addr, &addr, ITEM, &addr, &operator);
		// Successfully clear all transfer approvals.
		assert_ok!(clear_all_transfer_approvals(&addr, collection, item));
		assert!(
			Nfts::check_approval_permission(&collection, &Some(item), &addr, &operator).is_err()
		);
	});
}

#[test]
fn clear_collection_approvals_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let mut approvals = 0;
		let operators = 0..10;

		let (collection, _) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		for operator in operators.clone() {
			assert_ok!(Nfts::approve_collection_transfer(
				RuntimeOrigin::signed(addr.clone()),
				collection,
				account(operator).into(),
				None,
			));
		}
		// Partially clear collection approvals.
		approvals += 1;
		assert_ok!(clear_collection_approvals(&addr, collection, approvals));
		assert_eq!(
			CollectionApprovalsOf::<Runtime>::iter_prefix((collection, &addr)).count(),
			operators.len() - approvals as usize
		);

		// Successfully clear all collection approvals.
		assert_ok!(clear_collection_approvals(
			&addr,
			collection,
			operators.len() as u32 - approvals
		));
		assert!(CollectionApprovalsOf::<Runtime>::iter_prefix((collection, &addr))
			.count()
			.is_zero());
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist, throws module error `NoConfig`.
		assert_eq!(
			mint(&addr, COLLECTION, ALICE, ITEM, None),
			Err(Module { index: 50, error: [31, 0] })
		);
		// Mitning can only be done by the collection owner.
		let collection = nfts::create_collection(&ALICE, &ALICE);
		assert_eq!(
			mint(&addr, collection, ALICE, ITEM, None),
			Err(Module { index: 50, error: [0, 0] })
		);
		// Successfully mint.
		let collection = nfts::create_collection(&addr, &addr);
		assert_ok!(mint(&addr, collection, ALICE, ITEM, None));
		assert_eq!(nfts::balance_of(collection, ALICE), 1);
		// Minting an existing item ID.
		assert_eq!(
			mint(&addr, collection, ALICE, ITEM, None),
			Err(Module { index: 50, error: [2, 0] })
		);
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection item does not exist.
		assert_eq!(burn(&addr, COLLECTION, ITEM), Err(Module { index: 50, error: [19, 0] }));
		// Burning can only be done by the collection item owner.
		let (collection, item) = nfts::create_collection_and_mint_to(&ALICE, &ALICE, &BOB, ITEM);
		assert_eq!(burn(&addr, collection, item), Err(Module { index: 50, error: [0, 0] }));
		// Successfully burn.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(burn(&addr, collection, item));
		assert_eq!(nfts::balance_of(COLLECTION, addr.clone()), 0);
	});
}

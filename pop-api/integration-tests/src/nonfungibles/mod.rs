use frame_support::BoundedVec;
use pallet_api::nonfungibles::Config;
use pop_api::{
	nonfungibles::{
		events::{Approval, AttributeSet, Transfer},
		AttributeNamespace, CancelAttributesApprovalWitness, CollectionConfig, CollectionId,
		CollectionSettings, DestroyWitness, ItemId, ItemSettings, MintSettings, MintType,
		MintWitness,
	},
	primitives::BlockNumber,
};
use pop_primitives::{Error, Error::*};
use sp_runtime::traits::Zero;
use utils::*;

use super::*;

mod utils;

type NftsInstance = <Runtime as Config>::NftsInstance;
type AccountBalance = pallet_nfts::AccountBalance<Runtime, NftsInstance>;
type Attribute = pallet_nfts::Attribute<Runtime, NftsInstance>;
type AttributeKey = BoundedVec<u8, <Runtime as pallet_nfts::Config<NftsInstance>>::KeyLimit>;
type AttributeValue = BoundedVec<u8, <Runtime as pallet_nfts::Config<NftsInstance>>::ValueLimit>;
type CollectionApprovals = pallet_nfts::CollectionApprovals<Runtime, NftsInstance>;
type Collection = pallet_nfts::Collection<Runtime, NftsInstance>;
type ItemAttributesApprovals = pallet_nfts::ItemAttributesApprovalsOf<Runtime, NftsInstance>;
type Metadata = BoundedVec<u8, <Runtime as pallet_nfts::Config<NftsInstance>>::StringLimit>;
type NextCollectionId = pallet_nfts::NextCollectionId<Runtime, NftsInstance>;

const COLLECTION: CollectionId = 0;
const CONTRACT: &str = "contracts/nonfungibles/target/ink/nonfungibles.wasm";
const ITEM: ItemId = 0;

/// 1. PSP-34 Interface:
/// - balance_of
/// - owner_of
/// - allowance
/// - approve
/// - transfer
/// - total_supply

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No collection item is created.
		assert_eq!(balance_of(&addr, COLLECTION, ALICE), Ok(nfts::balance_of(COLLECTION, ALICE)));
		assert_eq!(balance_of(&addr, COLLECTION, ALICE), Ok(0));

		// Collection item is created.
		nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM);
		assert_eq!(balance_of(&addr, COLLECTION, ALICE), Ok(nfts::balance_of(COLLECTION, ALICE)),);
		assert_eq!(balance_of(&addr, COLLECTION, ALICE), Ok(1));
	});
}

#[test]
fn owner_of_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No owner found.
		assert_eq!(owner_of(&addr, COLLECTION, ITEM), Ok(None));

		// Owner found for the collection item.
		nfts::create_collection_and_mint_to(&addr, &addr, &ALICE, ITEM);
		assert_eq!(owner_of(&addr, COLLECTION, ITEM), Ok(Some(ALICE)));
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No collection item is created.
		assert_eq!(
			allowance(&addr, COLLECTION, addr.clone(), ALICE, None),
			Ok(!Nfts::check_approval_permission(&COLLECTION, &None, &addr, &ALICE).is_err()),
		);
		assert_eq!(allowance(&addr, COLLECTION, addr.clone(), ALICE, None), Ok(false));

		// Collection item is created.
		nfts::create_collection_mint_and_approve(&addr, &addr, ITEM, &addr, &ALICE);
		assert_eq!(
			allowance(&addr, COLLECTION, addr.clone(), ALICE, Some(ITEM)),
			Ok(Nfts::check_approval_permission(&COLLECTION, &Some(ITEM), &addr, &ALICE).is_ok()),
		);
		assert_eq!(allowance(&addr, COLLECTION, addr.clone(), ALICE, Some(ITEM)), Ok(true));
	});
}

mod approve {
	use super::*;

	#[test]
	fn approve_item_works() {
		new_test_ext().execute_with(|| {
			let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

			// Collection does not exist, throws module error `UnknownItem`.
			assert_eq!(
				approve(&addr, COLLECTION, ALICE, Some(ITEM), true),
				Err(Module { index: 50, error: [19, 0] })
			);
			// Successfully approve item.
			let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
			assert_ok!(approve(&addr, collection, ALICE, Some(item), true));
			assert!(
				Nfts::check_approval_permission(&collection, &Some(item), &addr, &ALICE).is_ok()
			);
			// Successfully emit event.
			let owner = account_id_from_slice(addr.as_ref());
			let operator = account_id_from_slice(ALICE.as_ref());
			let expected = Approval { owner, operator, item: Some(item), approved: true }.encode();
			assert_eq!(last_contract_event(), expected.as_slice());
		});
	}

	#[test]
	fn approve_collection_works() {
		new_test_ext().execute_with(|| {
			let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

			// Collection does not exist, throws module error `NoItemOwned`.
			assert_eq!(
				approve(&addr, COLLECTION, ALICE, None, true),
				Err(Module { index: 50, error: [44, 0] })
			);
			// Successfully approve collection.
			let (collection, _) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
			assert_ok!(approve(&addr, collection, ALICE, None, true));
			assert!(Nfts::check_approval_permission(&collection, &None, &addr, &ALICE).is_ok());
			// Successfully emit event.
			let owner = account_id_from_slice(addr.as_ref());
			let operator = account_id_from_slice(ALICE.as_ref());
			let expected = Approval { owner, operator, item: None, approved: true }.encode();
			assert_eq!(last_contract_event(), expected.as_slice());
		});
	}

	#[test]
	fn cancel_item_approval_works() {
		new_test_ext().execute_with(|| {
			let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

			// Collection does not exist, throws module error `UnknownItem`.
			assert_eq!(
				approve(&addr, COLLECTION, ALICE, Some(ITEM), false),
				Err(Module { index: 50, error: [19, 0] })
			);
			// Successfully cancel item approval.
			let (collection, item) =
				nfts::create_collection_mint_and_approve(&addr, &addr, ITEM, &addr, &ALICE);
			assert_ok!(approve(&addr, collection, ALICE, Some(item), false));
			assert!(
				Nfts::check_approval_permission(&collection, &Some(item), &addr, &ALICE).is_err()
			);
		});
	}

	#[test]
	fn cancel_collection_approval_works() {
		new_test_ext().execute_with(|| {
			let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

			// Collection does not exist, throws module error `UnknownItem`.
			assert_eq!(
				approve(&addr, COLLECTION, ALICE, Some(ITEM), true),
				Err(Module { index: 50, error: [19, 0] })
			);
			// Successfully cancel collection approval.
			let (collection, _) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
			assert_ok!(Nfts::approve_collection_transfer(
				RuntimeOrigin::signed(addr.clone()),
				collection,
				ALICE.into(),
				None
			));
			assert_ok!(approve(&addr, collection, ALICE, None, false));
			assert!(Nfts::check_approval_permission(&collection, &None, &addr, &ALICE).is_err());
		});
	}
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection item does not exist, throws module error `UnknownItem`.
		assert_eq!(
			transfer(&addr, COLLECTION, ALICE, ITEM),
			Err(Module { index: 50, error: [19, 0] })
		);
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		// Successfully transfer.
		let before_transfer_balance = nfts::balance_of(collection, ALICE);
		assert_ok!(transfer(&addr, collection, ALICE, item));
		let after_transfer_balance = nfts::balance_of(collection, ALICE);
		assert_eq!(after_transfer_balance - before_transfer_balance, 1);
		// Successfully emit event.
		let expected = Transfer {
			from: Some(account_id_from_slice(addr.as_ref())),
			to: Some(account_id_from_slice(ALICE.as_ref())),
			item,
		}
		.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// Collection item does not exist, i.e. burnt.
		nfts::burn(collection, item, &ALICE);
		assert_eq!(
			transfer(&addr, collection, ALICE, item),
			Err(Module { index: 50, error: [19, 0] })
		);
	});
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

/// 2. PSP-34 Metadata Interface:
/// - get_attribute

#[test]
fn get_attribute_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let attribute_key = "some attribute".as_bytes().to_vec();
		let attribute_value = "some value".as_bytes().to_vec();
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);

		// No attribute value found for key.
		assert_eq!(
			get_attribute(
				&addr,
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				attribute_key.clone(),
			),
			Ok(None)
		);

		// Attribute is set.
		assert_ok!(Nfts::set_attribute(
			RuntimeOrigin::signed(addr.clone()),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::CollectionOwner,
			AttributeKey::truncate_from(attribute_key.clone()),
			AttributeValue::truncate_from(attribute_value.clone()),
		));
		assert_eq!(
			get_attribute(
				&addr,
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				attribute_key.clone(),
			),
			Ok(Some(attribute_value))
		);
		assert_eq!(
			get_attribute(
				&addr,
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				attribute_key.clone(),
			),
			Ok(Attribute::get((
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::CollectionOwner,
				AttributeKey::truncate_from(attribute_key),
			))
			.map(|(attribute, _)| attribute.to_vec()))
		);
	});
}

/// 3. Asset Management:
/// - next_collection_id
/// - item_metadata
/// - create
/// - destroy
/// - set_attribute
/// - clear_attribute
/// - set_metadata
/// - clear_metadata
/// - set_max_supply
/// - approve_item_attributes
/// - cancel_item_attributes_approval
/// - clear_all_transfer_approvals
/// - clear_collection_approvals

#[test]
fn next_collection_id_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		assert_eq!(next_collection_id(&addr), Ok(Some(COLLECTION)));
		assert_eq!(
			next_collection_id(&addr),
			Ok(Some(NextCollectionId::get().unwrap_or_default()))
		);

		// Create a new collection and increment the collection ID.
		nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_eq!(next_collection_id(&addr), Ok(Some(COLLECTION + 1)));
		assert_eq!(
			next_collection_id(&addr),
			Ok(Some(NextCollectionId::get().unwrap_or_default()))
		);
	});
}

#[test]
fn item_metadata_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let metadata = "some metadata".as_bytes().to_vec();

		// No item metadata found.
		assert_eq!(item_metadata(&addr, COLLECTION, ITEM), Ok(None));

		// Item metadata is set.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(Nfts::set_metadata(
			RuntimeOrigin::signed(addr.clone()),
			collection,
			item,
			Metadata::truncate_from(metadata.clone())
		));
		assert_eq!(item_metadata(&addr, collection, item), Ok(Some(metadata)));
		assert_eq!(
			item_metadata(&addr, collection, item),
			Ok(Nfts::item_metadata(collection, item).map(|metadata| metadata.to_vec()))
		);
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a contract without balance for fees.
		let addr = instantiate(CONTRACT, 0, vec![0]);

		// No balance to pay for fees.
		assert_eq!(
			create(&addr, addr.clone(), nfts::default_collection_config()),
			Err(Module { index: 10, error: [2, 0] })
		);

		let addr = instantiate(CONTRACT, INIT_VALUE, vec![1]);
		// Successfully create a collection.
		assert_ok!(create(&addr, addr.clone(), nfts::default_collection_config()));
		assert_eq!(Nfts::collection_owner(COLLECTION), Some(addr));
	});
}

#[test]
fn destroy_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist, throws module error `UnknownCollection`.
		assert_eq!(
			destroy(
				&addr,
				COLLECTION,
				DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
			),
			Err(Module { index: 50, error: [1, 0] })
		);
		// Successfully destroy.
		let collection = nfts::create_collection(&addr, &addr);
		assert_ok!(destroy(
			&addr,
			collection,
			DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
		));
		assert_eq!(Collection::get(collection), None);
	});
}

#[test]
fn set_attribute_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let attribute_key = "some attribute".as_bytes().to_vec();
		let attribute_value = "some value".as_bytes().to_vec();

		// Collection does not exist, throws module error `UnknownCollection`.
		assert_eq!(
			set_attribute(
				&addr,
				COLLECTION,
				Some(ITEM),
				AttributeNamespace::CollectionOwner,
				attribute_key.clone(),
				attribute_value.clone()
			),
			Err(Module { index: 50, error: [1, 0] })
		);
		// Successfully set attribute.
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
			last_contract_event(),
			AttributeSet {
				item: Some(item),
				key: attribute_key.clone(),
				data: attribute_value.clone(),
			}
			.encode()
			.as_slice()
		);
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
		// Attribute does not exist, throws module error `MetadataNotFound`.
		assert_eq!(
			clear_attribute(
				&addr,
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				attribute_key.clone()
			),
			Err(Module { index: 50, error: [22, 0] })
		);
		// Successfully clear attribute.
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
		let metadata = "some metadata".as_bytes().to_vec();

		// Collection does not exist, throws module error `NoPermission`.
		assert_eq!(
			set_metadata(&addr, COLLECTION, ITEM, vec![]),
			Err(Module { index: 50, error: [0, 0] })
		);
		// Successfully set metadata.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(set_metadata(&addr, collection, item, metadata.clone()));
		assert_eq!(Nfts::item_metadata(collection, item), Some(Metadata::truncate_from(metadata)));
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let metadata = "some metadata".as_bytes().to_vec();

		// Collection does not exist, throws module error `NoPermission`.
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
			Metadata::truncate_from(metadata)
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

		// Collection does not exist, throws module error `NoConfig`.
		assert_eq!(
			set_max_supply(&addr, COLLECTION, value),
			Err(Module { index: 50, error: [31, 0] })
		);
		// Sucessfully set max supply.
		let collection = nfts::create_collection(&addr, &addr);
		assert_ok!(set_max_supply(&addr, collection, value));
		assert_eq!(nfts::max_supply(collection), Some(value));
	});
}

#[test]
fn approve_item_attributes_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist, throws module error `UnknownItem`.
		assert_eq!(
			approve_item_attributes(&addr, COLLECTION, ITEM, ALICE),
			Err(Module { index: 50, error: [19, 0] })
		);
		// Successfully approve attribute.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(approve_item_attributes(&addr, collection, item, ALICE));
		assert!(ItemAttributesApprovals::get(collection, item).contains(&ALICE));
	});
}

#[test]
fn cancel_item_attributes_approval_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist, throws module error `UnknownItem`.
		assert_eq!(
			cancel_item_attributes_approval(
				&addr,
				COLLECTION,
				ITEM,
				ALICE,
				CancelAttributesApprovalWitness { account_attributes: 1 }
			),
			Err(Module { index: 50, error: [19, 0] })
		);
		// Successfully cancel item attributes approval.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(Nfts::approve_item_attributes(
			RuntimeOrigin::signed(addr.clone()),
			collection,
			item,
			ALICE.into()
		));
		assert_ok!(cancel_item_attributes_approval(
			&addr,
			collection,
			item,
			ALICE,
			CancelAttributesApprovalWitness { account_attributes: 1 }
		));
		assert!(!ItemAttributesApprovals::get(collection, item).contains(&ALICE));
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
		// Successfully clear all transfer approvals.
		let (collection, item) =
			nfts::create_collection_mint_and_approve(&addr, &addr, ITEM, &addr, &operator);
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
		let (collection, _) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);

		assert_ok!(Nfts::approve_collection_transfer(
			RuntimeOrigin::signed(addr.clone()),
			collection,
			ALICE.into(),
			None,
		));
		// Successfully clear all collection approvals.
		assert_ok!(clear_collection_approvals(&addr, collection, 1));
		assert!(CollectionApprovals::iter_prefix((collection, &addr)).count().is_zero());
	});
}

/// 4. PSP-34 Mintable & Burnable Interface:
/// - mint
/// - burn

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// Collection does not exist, throws module error `NoConfig`.
		assert_eq!(
			mint(&addr, COLLECTION, ALICE, ITEM, None),
			Err(Module { index: 50, error: [31, 0] })
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

		// Collection item does not exist, throws module error `UnknownItem`.
		assert_eq!(burn(&addr, COLLECTION, ITEM), Err(Module { index: 50, error: [19, 0] }));
		// Successfully burn.
		let (collection, item) = nfts::create_collection_and_mint_to(&addr, &addr, &addr, ITEM);
		assert_ok!(burn(&addr, collection, item));
		assert_eq!(nfts::balance_of(COLLECTION, addr), 0);
	});
}

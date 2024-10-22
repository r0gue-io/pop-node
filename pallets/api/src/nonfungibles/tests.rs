use codec::Encode;
use frame_support::{assert_noop, assert_ok, traits::Incrementable};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_nfts::{
	AccountBalance, CollectionConfig, CollectionDetails, CollectionSettings, ItemDeposit,
	ItemDetails, MintSettings,
};
use sp_runtime::{BoundedBTreeMap, BoundedVec, DispatchError::BadOrigin};

use super::types::{CollectionIdOf, ItemIdOf};
use crate::{
	mock::*,
	nonfungibles::{Event, Read::*, ReadResult},
	Read,
};

const ITEM: u32 = 1;

type NftsError = pallet_nfts::Error<Test>;

mod encoding_read_result {
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

	#[test]
	fn next_collection_id_works() {
		let mut next_collection_id = Some(0);
		assert_eq!(
			ReadResult::NextCollectionId::<Test>(next_collection_id).encode(),
			next_collection_id.encode()
		);
		next_collection_id = None;
		assert_eq!(
			ReadResult::NextCollectionId::<Test>(next_collection_id).encode(),
			next_collection_id.encode()
		);
	}
}

#[test]
fn transfer() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let dest = BOB;

		let (collection, item) = nfts::create_collection_mint(owner, ITEM);
		for origin in vec![root(), none()] {
			assert_noop!(
				NonFungibles::transfer(origin, collection, item, account(dest)),
				BadOrigin
			);
		}
		// Successfully burn an existing new collection item.
		let balance_before_transfer = AccountBalance::<Test>::get(collection, &account(dest));
		assert_ok!(NonFungibles::transfer(signed(owner), collection, ITEM, account(dest)));
		let balance_after_transfer = AccountBalance::<Test>::get(collection, &account(dest));
		assert_eq!(AccountBalance::<Test>::get(collection, &account(owner)), 0);
		assert_eq!(balance_after_transfer - balance_before_transfer, 1);
		System::assert_last_event(
			Event::Transfer {
				collection,
				item,
				from: Some(account(owner)),
				to: Some(account(dest)),
				price: None,
			}
			.into(),
		);
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let collection = nfts::create_collection(owner);

		// Successfully mint a new collection item.
		let balance_before_mint = AccountBalance::<Test>::get(collection, account(owner));
		assert_ok!(NonFungibles::mint(signed(owner), account(owner), collection, ITEM, None));
		let balance_after_mint = AccountBalance::<Test>::get(collection, account(owner));
		assert_eq!(balance_after_mint, 1);
		assert_eq!(balance_after_mint - balance_before_mint, 1);
		System::assert_last_event(
			Event::Transfer {
				collection,
				item: ITEM,
				from: None,
				to: Some(account(owner)),
				price: None,
			}
			.into(),
		);
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;

		// Successfully burn an existing new collection item.
		let (collection, item) = nfts::create_collection_mint(owner, ITEM);
		assert_ok!(NonFungibles::burn(signed(owner), collection, ITEM));
		System::assert_last_event(
			Event::Transfer { collection, item, from: Some(account(owner)), to: None, price: None }
				.into(),
		);
	});
}

#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let operator = BOB;
		let (collection, item) = nfts::create_collection_mint(owner, ITEM);
		// Successfully approve `oeprator` to transfer the collection item.
		assert_ok!(NonFungibles::approve(
			signed(owner),
			collection,
			Some(item),
			account(operator),
			true
		));
		System::assert_last_event(
			Event::Approval {
				collection,
				item: Some(item),
				owner: account(owner),
				operator: account(operator),
				approved: true,
			}
			.into(),
		);
		// Successfully transfer the item by the delegated account `operator`.
		assert_ok!(Nfts::transfer(signed(operator), collection, item, account(operator)));
	});
}

#[test]
fn cancel_approval_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let operator = BOB;
		let (collection, item) = nfts::create_collection_mint_and_approve(owner, ITEM, operator);
		// Successfully cancel the transfer approval of `operator` by `owner`.
		assert_ok!(NonFungibles::approve(
			signed(owner),
			collection,
			Some(item),
			account(operator),
			false
		));
		// Failed to transfer the item by `operator` without permission.
		assert_noop!(
			Nfts::transfer(signed(operator), collection, item, account(operator)),
			NftsError::NoPermission
		);
	});
}

#[test]
fn set_max_supply_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let collection = nfts::create_collection(owner);
		assert_ok!(NonFungibles::set_max_supply(signed(owner), collection, 10));
		(0..10).into_iter().for_each(|i| {
			assert_ok!(Nfts::mint(signed(owner), collection, i, account(owner), None));
		});
		assert_noop!(
			Nfts::mint(signed(owner), collection, 42, account(owner), None),
			NftsError::MaxSupplyReached
		);
	});
}

#[test]
fn owner_of_works() {
	new_test_ext().execute_with(|| {
		let (collection, item) = nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(
			NonFungibles::read(OwnerOf { collection, item }).encode(),
			Nfts::owner(collection, item).encode()
		);
	});
}

#[test]
fn get_attribute_works() {
	new_test_ext().execute_with(|| {
		let (collection, item) = nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(NonFungibles::read(NextCollectionId).encode(), Some(1).encode());
		let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
		let value = BoundedVec::truncate_from("some value".as_bytes().to_vec());
		let mut result: Option<BoundedVec<u8, <Test as pallet_nfts::Config>::ValueLimit>> = None;
		// No attribute set.
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item: Some(item),
				namespace: pallet_nfts::AttributeNamespace::CollectionOwner,
				key: attribute.clone()
			})
			.encode(),
			result.encode()
		);
		// Successfully get an existing attribute.
		result = Some(value.clone());
		assert_ok!(Nfts::set_attribute(
			signed(ALICE),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::CollectionOwner,
			attribute.clone(),
			value,
		));
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item: Some(item),
				namespace: pallet_nfts::AttributeNamespace::CollectionOwner,
				key: attribute
			})
			.encode(),
			result.encode()
		);
	});
}

#[test]
fn set_metadata_works() {
	unimplemented!()
}

#[test]
fn clear_attribute_works() {
	new_test_ext().execute_with(|| {
		let (collection, item) = nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(NonFungibles::read(NextCollectionId).encode(), Some(1).encode());
		let mut attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
		let result: Option<BoundedVec<u8, <Test as pallet_nfts::Config>::ValueLimit>> = None;
		assert_ok!(Nfts::set_attribute(
			signed(ALICE),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::CollectionOwner,
			attribute.clone(),
			BoundedVec::truncate_from("some value".as_bytes().to_vec())
		));
		// Successfully clear an attribute.
		assert_ok!(Nfts::clear_attribute(
			signed(ALICE),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::CollectionOwner,
			attribute.clone(),
		));
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item: Some(item),
				namespace: pallet_nfts::AttributeNamespace::CollectionOwner,
				key: attribute
			})
			.encode(),
			result.encode()
		);
	});
}

#[test]
fn approve_item_attribute_works() {
	new_test_ext().execute_with(|| {
		let (collection, item) = nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(NonFungibles::read(NextCollectionId).encode(), Some(1).encode());
		let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
		let value = BoundedVec::truncate_from("some value".as_bytes().to_vec());
		let mut result: Option<BoundedVec<u8, <Test as pallet_nfts::Config>::ValueLimit>> = None;
		// Successfully approve delegate to set attributes.
		assert_ok!(Nfts::approve_item_attributes(signed(ALICE), collection, item, account(BOB)));
		assert_ok!(Nfts::set_attribute(
			signed(BOB),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::Account(account(BOB)),
			attribute.clone(),
			value.clone()
		));
		result = Some(value);
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item: Some(item),
				namespace: pallet_nfts::AttributeNamespace::Account(account(BOB)),
				key: attribute
			})
			.encode(),
			result.encode()
		);
	});
}

#[test]
fn cancel_item_attribute_approval_works() {
	new_test_ext().execute_with(|| {
		let (collection, item) = nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(NonFungibles::read(NextCollectionId).encode(), Some(1).encode());
		let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
		let value = BoundedVec::truncate_from("some value".as_bytes().to_vec());
		let result: Option<BoundedVec<u8, <Test as pallet_nfts::Config>::ValueLimit>> = None;
		// Successfully approve delegate to set attributes.
		assert_ok!(Nfts::approve_item_attributes(signed(ALICE), collection, item, account(BOB)));
		assert_ok!(Nfts::set_attribute(
			signed(BOB),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::Account(account(BOB)),
			attribute.clone(),
			value.clone()
		));
		assert_ok!(Nfts::cancel_item_attributes_approval(
			signed(ALICE),
			collection,
			item,
			account(BOB),
			pallet_nfts::CancelAttributesApprovalWitness { account_attributes: 1 }
		));
		assert_noop!(
			Nfts::set_attribute(
				signed(BOB),
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::Account(account(BOB)),
				attribute.clone(),
				value.clone()
			),
			NftsError::NoPermission
		);
	});
}

#[test]
fn next_collection_id_works() {
	new_test_ext().execute_with(|| {
		let _ = nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(NonFungibles::read(NextCollectionId).encode(), Some(1).encode());
		assert_eq!(
			NonFungibles::read(NextCollectionId).encode(),
			pallet_nfts::NextCollectionId::<Test>::get()
				.or(CollectionIdOf::<Test>::initial_value())
				.encode(),
		);
	});
}

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let collection = nfts::create_collection(owner);
		(0..10).into_iter().for_each(|i| {
			assert_ok!(Nfts::mint(signed(owner), collection, i, account(owner), None));
			assert_eq!(NonFungibles::read(TotalSupply(collection)).encode(), (i + 1).encode());
			assert_eq!(
				NonFungibles::read(TotalSupply(collection)).encode(),
				Nfts::collection_items(collection).unwrap_or_default().encode()
			);
		});
	});
}

#[test]
fn collection_works() {
	new_test_ext().execute_with(|| {
		let (collection, _) = nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(
			NonFungibles::read(Collection(collection)).encode(),
			pallet_nfts::Collection::<Test>::get(&collection).encode(),
		);
	});
}

#[test]
fn item_works() {
	new_test_ext().execute_with(|| {
		let (collection, item) = nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(
			NonFungibles::read(Item { collection, item }).encode(),
			pallet_nfts::Item::<Test>::get(&collection, &item).encode(),
		);
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let collection = nfts::create_collection(owner);
		(0..10).into_iter().for_each(|i| {
			assert_ok!(Nfts::mint(signed(owner), collection, i, account(owner), None));
			assert_eq!(
				NonFungibles::read(BalanceOf { collection, owner: account(owner) }).encode(),
				(i + 1).encode()
			);
			assert_eq!(
				NonFungibles::read(BalanceOf { collection, owner: account(owner) }).encode(),
				AccountBalance::<Test>::get(collection, account(owner)).encode()
			);
		});
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let operator = BOB;
		let (collection, item) = nfts::create_collection_mint_and_approve(owner, ITEM, operator);
		assert_eq!(
			NonFungibles::read(Allowance {
				collection,
				item: Some(item),
				owner: account(owner),
				operator: account(operator),
			})
			.encode(),
			true.encode()
		);
		assert_eq!(
			NonFungibles::read(Allowance {
				collection,
				item: Some(item),
				owner: account(owner),
				operator: account(operator),
			})
			.encode(),
			Nfts::check_allowance(&collection, &Some(item), &account(owner), &account(operator))
				.is_ok()
				.encode()
		);
	});
}

fn signed(account_id: u8) -> RuntimeOrigin {
	RuntimeOrigin::signed(account(account_id))
}

fn root() -> RuntimeOrigin {
	RuntimeOrigin::root()
}

fn none() -> RuntimeOrigin {
	RuntimeOrigin::none()
}

mod nfts {
	use super::*;

	pub(super) fn create_collection_mint_and_approve(
		owner: u8,
		item: ItemIdOf<Test>,
		operator: u8,
	) -> (u32, u32) {
		let (collection, item) = create_collection_mint(owner, item);
		assert_ok!(Nfts::approve_transfer(
			signed(owner),
			collection,
			Some(item),
			account(operator),
			None
		));
		(collection, item)
	}

	pub(super) fn create_collection_mint(owner: u8, item: ItemIdOf<Test>) -> (u32, u32) {
		let collection = create_collection(owner);
		assert_ok!(Nfts::mint(signed(owner), collection, item, account(owner), None));
		(collection, item)
	}

	pub(super) fn create_collection(owner: u8) -> u32 {
		let next_id = next_collection_id();
		assert_ok!(Nfts::create(
			signed(owner),
			account(owner),
			collection_config_with_all_settings_enabled()
		));
		next_id
	}

	pub(super) fn next_collection_id() -> u32 {
		pallet_nfts::NextCollectionId::<Test>::get().unwrap_or_default()
	}

	pub(super) fn collection_config_with_all_settings_enabled(
	) -> CollectionConfig<Balance, BlockNumberFor<Test>, CollectionIdOf<Test>> {
		CollectionConfig {
			settings: CollectionSettings::all_enabled(),
			max_supply: None,
			mint_settings: MintSettings::default(),
		}
	}
}

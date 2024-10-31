use codec::Encode;
use frame_support::{assert_noop, assert_ok, traits::Incrementable};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_nfts::{
	AccountBalance, CollectionConfig, CollectionDetails, CollectionSettings, DestroyWitness,
	MintSettings, MintWitness,
};
use sp_runtime::{BoundedVec, DispatchError::BadOrigin};

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
		let total_supply: u128 = 1_000_000;
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
		let mut owner = Some(ALICE);
		assert_eq!(ReadResult::OwnerOf::<Test>(owner.clone()).encode(), owner.encode());
		owner = None;
		assert_eq!(ReadResult::OwnerOf::<Test>(owner.clone()).encode(), owner.encode());
	}

	#[test]
	fn get_attribute() {
		let mut attribute = Some("some attribute".as_bytes().to_vec());
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
			owner: ALICE,
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

	#[test]
	fn item_metadata_works() {
		let mut data = Some("some metadata".as_bytes().to_vec());
		assert_eq!(ReadResult::ItemMetadata::<Test>(data.clone()).encode(), data.encode());
		data = None;
		assert_eq!(ReadResult::ItemMetadata::<Test>(data.clone()).encode(), data.encode());
	}
}

#[test]
fn transfer() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let dest = BOB;

		let (collection, item) = nfts::create_collection_mint(owner, ITEM);
		for origin in vec![root(), none()] {
			assert_noop!(NonFungibles::transfer(origin, collection, item, dest), BadOrigin);
		}
		// Successfully burn an existing new collection item.
		let balance_before_transfer = AccountBalance::<Test>::get(collection, &dest);
		assert_ok!(NonFungibles::transfer(signed(owner), collection, ITEM, dest));
		let balance_after_transfer = AccountBalance::<Test>::get(collection, &dest);
		assert_eq!(AccountBalance::<Test>::get(collection, &owner), 0);
		assert_eq!(balance_after_transfer - balance_before_transfer, 1);
		System::assert_last_event(
			Event::Transfer { collection, item, from: Some(owner), to: Some(dest), price: None }
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
		let balance_before_mint = AccountBalance::<Test>::get(collection, owner);
		assert_ok!(NonFungibles::mint(
			signed(owner),
			owner,
			collection,
			ITEM,
			MintWitness { mint_price: None, owned_item: None }
		));
		let balance_after_mint = AccountBalance::<Test>::get(collection, owner);
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
		let owner = ALICE;

		// Successfully burn an existing new collection item.
		let (collection, item) = nfts::create_collection_mint(owner, ITEM);
		assert_ok!(NonFungibles::burn(signed(owner), collection, ITEM));
		System::assert_last_event(
			Event::Transfer { collection, item, from: Some(owner), to: None, price: None }.into(),
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
		assert_ok!(NonFungibles::approve(signed(owner), collection, Some(item), operator, true));
		System::assert_last_event(
			Event::Approval { collection, item: Some(item), owner, operator, approved: true }
				.into(),
		);
		// Successfully transfer the item by the delegated account `operator`.
		assert_ok!(Nfts::transfer(signed(operator), collection, item, operator));
	});
}

#[test]
fn cancel_approval_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let operator = BOB;
		let (collection, item) = nfts::create_collection_mint_and_approve(owner, ITEM, operator);
		// Successfully cancel the transfer approval of `operator` by `owner`.
		assert_ok!(NonFungibles::approve(signed(owner), collection, Some(item), operator, false));
		// Failed to transfer the item by `operator` without permission.
		assert_noop!(
			Nfts::transfer(signed(operator), collection, item, operator),
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
			assert_ok!(Nfts::mint(signed(owner), collection, i, owner, None));
		});
		assert_noop!(
			Nfts::mint(signed(owner), collection, 42, owner, None),
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
		let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
		let value = BoundedVec::truncate_from("some value".as_bytes().to_vec());
		let mut result: Option<BoundedVec<u8, <Test as pallet_nfts::Config>::ValueLimit>> = None;
		// No attribute set.
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item,
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
				item,
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
	new_test_ext().execute_with(|| {
		let (collection, item) = nfts::create_collection_mint(ALICE, ITEM);
		let value = BoundedVec::truncate_from("some metadata".as_bytes().to_vec());
		assert_ok!(NonFungibles::set_metadata(signed(ALICE), collection, item, value.clone()));
		assert_eq!(
			NonFungibles::read(ItemMetadata { collection, item }).encode(),
			Some(value).encode()
		);
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let (collection, item) = nfts::create_collection_mint(ALICE, ITEM);
		assert_ok!(NonFungibles::set_metadata(
			signed(ALICE),
			collection,
			item,
			BoundedVec::truncate_from("some metadata".as_bytes().to_vec())
		));
		assert_ok!(NonFungibles::clear_metadata(signed(ALICE), collection, item));
		assert_eq!(
			NonFungibles::read(ItemMetadata { collection, item }).encode(),
			ReadResult::<Test>::ItemMetadata(None).encode()
		);
	});
}

#[test]
fn clear_attribute_works() {
	new_test_ext().execute_with(|| {
		let (collection, item) = nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(NonFungibles::read(NextCollectionId).encode(), Some(1).encode());
		let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
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
				item,
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
		// Successfully approve delegate to set attributes.
		assert_ok!(Nfts::approve_item_attributes(signed(ALICE), collection, item, BOB));
		assert_ok!(Nfts::set_attribute(
			signed(BOB),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::Account(BOB),
			attribute.clone(),
			value.clone()
		));
		let result: Option<BoundedVec<u8, <Test as pallet_nfts::Config>::ValueLimit>> = Some(value);
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item,
				namespace: pallet_nfts::AttributeNamespace::Account(BOB),
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
		// Successfully approve delegate to set attributes.
		assert_ok!(Nfts::approve_item_attributes(signed(ALICE), collection, item, BOB));
		assert_ok!(Nfts::set_attribute(
			signed(BOB),
			collection,
			Some(item),
			pallet_nfts::AttributeNamespace::Account(BOB),
			attribute.clone(),
			value.clone()
		));
		assert_ok!(Nfts::cancel_item_attributes_approval(
			signed(ALICE),
			collection,
			item,
			BOB,
			pallet_nfts::CancelAttributesApprovalWitness { account_attributes: 1 }
		));
		assert_noop!(
			Nfts::set_attribute(
				signed(BOB),
				collection,
				Some(item),
				pallet_nfts::AttributeNamespace::Account(BOB),
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
			assert_ok!(Nfts::mint(signed(owner), collection, i, owner, None));
			assert_eq!(
				NonFungibles::read(TotalSupply(collection)).encode(),
				((i + 1) as u128).encode()
			);
			assert_eq!(
				NonFungibles::read(TotalSupply(collection)).encode(),
				(Nfts::collection_items(collection).unwrap_or_default() as u128).encode()
			);
		});
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let next_collection_id = pallet_nfts::NextCollectionId::<Test>::get().unwrap_or_default();
		assert_ok!(NonFungibles::create(
			signed(owner),
			next_collection_id,
			owner,
			CollectionConfig {
				max_supply: None,
				mint_settings: MintSettings::default(),
				settings: CollectionSettings::all_enabled()
			},
		));
		assert_eq!(Nfts::collection_owner(next_collection_id), Some(owner));
	});
}

#[test]
fn destroy_works() {
	new_test_ext().execute_with(|| {
		let collection = nfts::create_collection(ALICE);
		assert_ok!(NonFungibles::destroy(
			signed(ALICE),
			collection,
			DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
		));
		assert_eq!(Nfts::collection_owner(collection), None);
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
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let collection = nfts::create_collection(owner);
		(0..10).into_iter().for_each(|i| {
			assert_ok!(Nfts::mint(signed(owner), collection, i, owner, None));
			assert_eq!(
				NonFungibles::read(BalanceOf { collection, owner }).encode(),
				(i + 1).encode()
			);
			assert_eq!(
				NonFungibles::read(BalanceOf { collection, owner }).encode(),
				AccountBalance::<Test>::get(collection, owner).encode()
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
			NonFungibles::read(Allowance { collection, item: Some(item), owner, operator })
				.encode(),
			true.encode()
		);
		assert_eq!(
			NonFungibles::read(Allowance { collection, item: Some(item), owner, operator })
				.encode(),
			Nfts::check_allowance(&collection, &Some(item), &owner, &operator)
				.is_ok()
				.encode()
		);
	});
}

fn signed(account_id: AccountId) -> RuntimeOrigin {
	RuntimeOrigin::signed(account_id)
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
		owner: AccountId,
		item: ItemIdOf<Test>,
		operator: AccountId,
	) -> (u32, u32) {
		let (collection, item) = create_collection_mint(owner, item);
		assert_ok!(Nfts::approve_transfer(signed(owner), collection, Some(item), operator, None));
		(collection, item)
	}

	pub(super) fn create_collection_mint(owner: AccountId, item: ItemIdOf<Test>) -> (u32, u32) {
		let collection = create_collection(owner);
		assert_ok!(Nfts::mint(signed(owner), collection, item, owner, None));
		(collection, item)
	}

	pub(super) fn create_collection(owner: AccountId) -> u32 {
		let next_id = next_collection_id();
		assert_ok!(Nfts::create(
			signed(owner),
			owner,
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

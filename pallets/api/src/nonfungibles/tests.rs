use codec::Encode;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::WithPostDispatchInfo,
	sp_runtime::{traits::Zero, BoundedVec, DispatchError::BadOrigin},
	weights::Weight,
};
use pallet_nfts::{CollectionSetting, MintWitness, WeightInfo as NftsWeightInfoTrait};

use crate::{
	mock::*,
	nonfungibles::{
		AccountBalanceOf, AttributeNamespace, AttributeOf, CancelAttributesApprovalWitness,
		CollectionConfig, CollectionConfigOf, CollectionIdOf, CollectionSettings, Config,
		DestroyWitness, ItemIdOf, MintSettings, NextCollectionIdOf, NftsErrorOf, NftsInstanceOf,
		NftsWeightInfoOf, Read::*, ReadResult, WeightInfo as WeightInfoTrait,
	},
	Read,
};

const COLLECTION: u32 = 0;
const ITEM: u32 = 1;

type CollectionApprovals = pallet_nfts::CollectionApprovals<Test, NftsInstanceOf<Test>>;
type Event = crate::nonfungibles::Event<Test>;
type NftsError = NftsErrorOf<Test>;
type NftsWeightInfo = NftsWeightInfoOf<Test>;
type WeightInfo = <Test as Config>::WeightInfo;

mod approve {
	use super::*;

	#[test]
	fn approve_works() {
		new_test_ext().execute_with(|| {
			let collection = COLLECTION;
			let item = ITEM;
			let operator = BOB;
			let owner = ALICE;

			// Check error works for `Nfts::approve_transfer()`.
			assert_noop!(
				NonFungibles::approve(signed(owner), collection, operator, Some(item), true, None),
				NftsError::UnknownItem.with_weight(NftsWeightInfo::approve_transfer())
			);
			nfts::create_collection_and_mint(owner, owner, item);
			// Successfully approve `operator` to transfer the collection item.
			assert_ok!(NonFungibles::approve(
				signed(owner),
				collection,
				operator,
				Some(item),
				true,
				None
			));
			assert_ok!(Nfts::check_approval_permission(
				&collection,
				&Some(item),
				&owner,
				&operator
			));
			System::assert_last_event(
				Event::Approval { collection, item: Some(item), owner, operator, approved: true }
					.into(),
			);
		});
	}

	#[test]
	fn approve_collection_works() {
		new_test_ext().execute_with(|| {
			let collection = COLLECTION;
			let item = ITEM;
			let operator = BOB;
			let owner = ALICE;

			// Check error works for `Nfts::approve_collection_transfer()`.
			assert_noop!(
				NonFungibles::approve(signed(owner), collection, operator, None, true, None),
				NftsError::NoItemOwned.with_weight(NftsWeightInfo::approve_collection_transfer())
			);
			nfts::create_collection_and_mint(owner, owner, item);
			// Successfully approve `operator` to transfer all collection items owned by `owner`.
			assert_ok!(NonFungibles::approve(
				signed(owner),
				collection,
				operator,
				None,
				true,
				None
			));
			assert_ok!(Nfts::check_approval_permission(&collection, &None, &owner, &operator));
			System::assert_last_event(
				Event::Approval { collection, item: None, owner, operator, approved: true }.into(),
			);
		});
	}

	#[test]
	fn cancel_approval_works() {
		new_test_ext().execute_with(|| {
			let collection = COLLECTION;
			let item = ITEM;
			let operator = BOB;
			let owner = ALICE;

			// Check error works for `Nfts::cancel_approval()`.
			assert_noop!(
				NonFungibles::approve(signed(owner), collection, operator, Some(item), false, None),
				NftsError::UnknownItem.with_weight(NftsWeightInfo::cancel_approval())
			);
			nfts::create_collection_mint_and_approve(owner, owner, item, operator);
			// Successfully cancel the transfer approval of `operator` by `owner`.
			assert_ok!(NonFungibles::approve(
				signed(owner),
				collection,
				operator,
				Some(item),
				false,
				None
			));
			assert_eq!(
				Nfts::check_approval_permission(&collection, &Some(item), &owner, &operator),
				Err(NftsError::NoPermission.into())
			);
		});
	}

	#[test]
	fn cancel_collection_approval_works() {
		new_test_ext().execute_with(|| {
			let collection = COLLECTION;
			let item = ITEM;
			let operator = BOB;
			let owner = ALICE;

			// Check error works for `Nfts::cancel_collection_approval()`.
			assert_noop!(
				NonFungibles::approve(signed(owner), collection, operator, None, false, None),
				NftsError::NotDelegate.with_weight(NftsWeightInfo::cancel_collection_approval())
			);
			nfts::create_collection_and_mint(owner, owner, item);
			assert_ok!(Nfts::approve_collection_transfer(
				signed(owner),
				collection,
				operator,
				None
			));
			// Successfully cancel the transfer collection approval of `operator` by `owner`.
			assert_ok!(NonFungibles::approve(
				signed(owner),
				collection,
				operator,
				None,
				false,
				None
			));
			assert_eq!(
				Nfts::check_approval_permission(&collection, &None, &owner, &operator),
				Err(NftsError::NoPermission.into())
			);
		});
	}
}

mod transfer {
	use super::*;

	#[test]
	fn transfer_works() {
		new_test_ext().execute_with(|| {
			let collection = COLLECTION;
			let dest = BOB;
			let item = ITEM;
			let owner = ALICE;

			// Throw `NftsError::UnknownItem` if no item found.
			assert_noop!(
				NonFungibles::transfer(signed(dest), collection, dest, item),
				NftsError::UnknownItem
			);
			nfts::create_collection_and_mint(owner, owner, item);
			// Check error works for `Nfts::transfer()`.
			assert_noop!(
				NonFungibles::transfer(signed(dest), collection, dest, item),
				NftsError::NoPermission
			);
			// Successfully transfer a collection item.
			let owner_balance_before_transfer = nfts::balance_of(collection, &owner);
			let dest_balance_before_transfer = nfts::balance_of(collection, &dest);
			assert_ok!(NonFungibles::transfer(signed(owner), collection, dest, item));
			let owner_balance_after_transfer = nfts::balance_of(collection, &owner);
			let dest_balance_after_transfer = nfts::balance_of(collection, &dest);
			// Check that `to` has received the collection item from `from`.
			assert_eq!(owner_balance_after_transfer, owner_balance_before_transfer - 1);
			assert_eq!(dest_balance_after_transfer, dest_balance_before_transfer + 1);
			System::assert_last_event(
				Event::Transfer { collection, item, from: Some(owner), to: Some(dest) }.into(),
			);
		});
	}

	#[test]
	fn approved_transfer_works() {
		new_test_ext().execute_with(|| {
			let collection = COLLECTION;
			let dest = CHARLIE;
			let item = ITEM;
			let operator = BOB;
			let owner = ALICE;

			nfts::create_collection_and_mint(owner, owner, item);
			// Approve `operator` to transfer all `collection` items owned by the `owner`.
			assert_ok!(Nfts::approve_collection_transfer(
				signed(owner),
				collection,
				operator,
				None
			));
			// Successfully transfer a collection item.
			let owner_balance_before_transfer = nfts::balance_of(collection, &owner);
			let dest_balance_before_transfer = nfts::balance_of(collection, &dest);
			assert_ok!(NonFungibles::transfer(signed(operator), collection, dest, item));
			let owner_balance_after_transfer = nfts::balance_of(collection, &owner);
			let dest_balance_after_transfer = nfts::balance_of(collection, &dest);
			// Check that `to` has received the collection item from `from`.
			assert_eq!(owner_balance_after_transfer, owner_balance_before_transfer - 1);
			assert_eq!(dest_balance_after_transfer, dest_balance_before_transfer + 1);
			System::assert_last_event(
				Event::Transfer { collection, item, from: Some(owner), to: Some(dest) }.into(),
			);
		});
	}
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let admin = ALICE;
		let mut config = CollectionConfig {
			max_supply: None,
			mint_settings: MintSettings::default(),
			settings: CollectionSettings::all_enabled(),
		};
		let collection = COLLECTION;
		let creator = ALICE;

		// Origin checks.
		for origin in vec![root(), none()] {
			assert_noop!(NonFungibles::create(origin, admin, config), BadOrigin);
		}

		// Check error works for `Nfts::create()`.
		config.disable_setting(CollectionSetting::DepositRequired);
		assert_noop!(NonFungibles::create(signed(creator), admin, config), NftsError::WrongSetting);
		config.enable_setting(CollectionSetting::DepositRequired);
		// Successfully create a collection.
		assert_ok!(NonFungibles::create(signed(creator), admin, config));
		assert_eq!(Nfts::collection_owner(collection), Some(creator));
		System::assert_last_event(Event::Created { id: collection, creator, admin }.into());
	});
}

#[test]
fn destroy_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let witness = DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 };

		// Check error works for `Nfts::destroy()`.
		assert_noop!(
			NonFungibles::destroy(signed(ALICE), collection, witness),
			NftsError::UnknownCollection
		);
		nfts::create_collection(ALICE);
		// Successfully destroy a collection.
		assert_ok!(NonFungibles::destroy(signed(ALICE), collection, witness));
		assert_eq!(Nfts::collection_owner(collection), None);
	});
}

#[test]
fn set_attribute_works() {
	new_test_ext().execute_with(|| {
		let attribute = BoundedVec::truncate_from("some attribute".into());
		let collection = COLLECTION;
		let item = ITEM;
		let owner = ALICE;
		let value = BoundedVec::truncate_from("some value".into());

		// Check error works for `Nfts::set_attribute()`.
		assert_noop!(
			NonFungibles::set_attribute(
				signed(owner),
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				attribute.clone(),
				value.clone()
			),
			NftsError::UnknownCollection
		);
		nfts::create_collection_and_mint(owner, owner, item);
		// Successfully set attribute.
		assert_ok!(NonFungibles::set_attribute(
			signed(owner),
			collection,
			Some(item),
			AttributeNamespace::CollectionOwner,
			attribute.clone(),
			value.clone()
		));
		System::assert_last_event(
			Event::AttributeSet {
				collection,
				item: Some(item),
				key: attribute.to_vec(),
				data: value.to_vec(),
			}
			.into(),
		);
		assert_eq!(
			nfts::get_attribute(
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				attribute
			),
			Some(value.into())
		);
	});
}

#[test]
fn clear_attribute_works() {
	new_test_ext().execute_with(|| {
		let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
		let collection = COLLECTION;
		let item = ITEM;
		let owner = ALICE;
		let value = BoundedVec::truncate_from("some value".as_bytes().to_vec());

		// Check error works for `Nfts::clear_attribute()`.
		assert_noop!(
			Nfts::set_attribute(
				signed(owner),
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				attribute.clone(),
				value.clone()
			),
			NftsError::UnknownCollection
		);
		nfts::create_collection_and_mint(owner, owner, item);
		assert_ok!(Nfts::set_attribute(
			signed(owner),
			collection,
			Some(item),
			AttributeNamespace::CollectionOwner,
			attribute.clone(),
			value.clone()
		));
		// Successfully clear an attribute.
		assert_ok!(NonFungibles::clear_attribute(
			signed(owner),
			collection,
			Some(item),
			AttributeNamespace::CollectionOwner,
			attribute.clone(),
		));
		assert!(nfts::get_attribute(
			collection,
			Some(item),
			AttributeNamespace::CollectionOwner,
			attribute
		)
		.is_none());
	});
}

#[test]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let item = ITEM;
		let metadata = BoundedVec::truncate_from("some metadata".into());
		let owner = ALICE;

		// Check error works for `Nfts::set_metadata()`.
		assert_noop!(
			NonFungibles::set_metadata(signed(owner), collection, item, metadata.clone()),
			NftsError::NoPermission
		);
		nfts::create_collection_and_mint(owner, owner, item);
		// Successfully set the metadata.
		assert_ok!(NonFungibles::set_metadata(signed(owner), collection, item, metadata.clone()));
		assert_eq!(Nfts::item_metadata(collection, item), Some(metadata));
	});
}

#[test]
fn clear_metadata_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let item = ITEM;
		let metadata = BoundedVec::truncate_from("some metadata".into());
		let owner = ALICE;

		// Check error works for `Nfts::clear_metadata()`.
		assert_noop!(
			NonFungibles::clear_metadata(signed(owner), collection, item),
			NftsError::NoPermission
		);
		nfts::create_collection_and_mint(owner, owner, item);
		assert_ok!(Nfts::set_metadata(signed(owner), collection, item, metadata));
		// Successfully clear the metadata.
		assert_ok!(NonFungibles::clear_metadata(signed(owner), collection, item));
		assert!(Nfts::item_metadata(collection, item).is_none());
	});
}

#[test]
fn set_max_supply_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let owner = ALICE;
		let max_supply = 10;

		// Check error works for `Nfts::set_max_supply()`.
		assert_noop!(
			NonFungibles::set_max_supply(signed(owner), collection, max_supply),
			NftsError::NoConfig
		);
		nfts::create_collection(owner);
		// Successfully set the max supply for the collection.
		assert_ok!(NonFungibles::set_max_supply(signed(owner), collection, max_supply));
		(0..max_supply).into_iter().for_each(|i| {
			assert_ok!(Nfts::mint(signed(owner), collection, i, owner, None));
		});
		// Throws `MaxSupplyReached` error if number of minted items is over the max supply.
		assert_noop!(
			Nfts::mint(signed(owner), collection, 42, owner, None),
			NftsError::MaxSupplyReached
		);
		// Override the max supply.
		assert_ok!(NonFungibles::set_max_supply(signed(owner), collection, max_supply * 2));
		assert_ok!(Nfts::mint(signed(owner), collection, 42, owner, None));
	});
}

#[test]
fn approve_item_attribute_works() {
	new_test_ext().execute_with(|| {
		let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
		let collection = COLLECTION;
		let delegate = BOB;
		let item = ITEM;
		let owner = ALICE;
		let value = BoundedVec::truncate_from("some value".as_bytes().to_vec());

		// Check error works for `Nfts::approve_item_attributes()`.
		assert_noop!(
			NonFungibles::approve_item_attributes(signed(owner), collection, item, delegate),
			NftsError::UnknownItem
		);
		nfts::create_collection_and_mint(owner, owner, item);
		// Successfully approve delegate to set attributes.
		assert_ok!(NonFungibles::approve_item_attributes(
			signed(owner),
			collection,
			item,
			delegate
		));
		assert_ok!(Nfts::set_attribute(
			signed(delegate),
			collection,
			Some(item),
			AttributeNamespace::Account(delegate),
			attribute,
			value
		));
	});
}

#[test]
fn cancel_item_attribute_approval_works() {
	new_test_ext().execute_with(|| {
		let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
		let collection = COLLECTION;
		let delegate = BOB;
		let item = ITEM;
		let owner = ALICE;
		let value = BoundedVec::truncate_from("some value".as_bytes().to_vec());

		// Check error works for `Nfts::cancel_item_attribute_approval()`.
		assert_noop!(
			NonFungibles::cancel_item_attributes_approval(
				signed(owner),
				collection,
				item,
				delegate,
				CancelAttributesApprovalWitness { account_attributes: 1 }
			),
			NftsError::UnknownItem
		);

		nfts::create_collection_and_mint(owner, owner, item);
		assert_ok!(Nfts::approve_item_attributes(signed(owner), collection, item, delegate));
		// Successfully cancel item attribute approval.
		assert_ok!(Nfts::cancel_item_attributes_approval(
			signed(owner),
			collection,
			item,
			delegate,
			CancelAttributesApprovalWitness { account_attributes: 1 }
		));
		assert_noop!(
			Nfts::set_attribute(
				signed(delegate),
				collection,
				Some(item),
				AttributeNamespace::Account(delegate),
				attribute,
				value
			),
			NftsError::NoPermission
		);
	});
}

#[test]
fn clear_all_transfer_approvals_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let delegates = 10..20;
		let item = ITEM;
		let owner = ALICE;

		// Check error works for `Nfts::clear_all_transfer_approvals()`.
		assert_noop!(
			NonFungibles::clear_all_transfer_approvals(signed(owner), collection, item),
			NftsError::UnknownCollection
		);

		nfts::create_collection_and_mint(owner, owner, item);
		delegates.clone().for_each(|delegate| {
			assert_ok!(Nfts::approve_transfer(signed(owner), collection, item, delegate, None));
		});
		// Successfully clear all transfer approvals.
		assert_ok!(NonFungibles::clear_all_transfer_approvals(signed(owner), collection, item));
		delegates.for_each(|delegate| {
			assert!(Nfts::check_approval_permission(&collection, &Some(item), &owner, &delegate)
				.is_err());
		});
	});
}

#[test]
fn clear_collection_approvals_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let delegates = 10..20;
		let owner = ALICE;
		let approvals = (delegates.end - delegates.start) as u32;

		// Check error works for `Nfts::clear_collection_approvals()`.
		assert_noop!(
			NonFungibles::clear_collection_approvals(none(), collection, 1),
			BadOrigin.with_weight(NftsWeightInfo::clear_collection_approvals(0))
		);
		nfts::create_collection_and_mint(owner, owner, ITEM);
		delegates.clone().for_each(|delegate| {
			assert_ok!(Nfts::approve_collection_transfer(
				signed(owner),
				collection,
				delegate,
				None
			));
		});
		// Partially clear collection approvals.
		assert_eq!(
			NonFungibles::clear_collection_approvals(signed(owner), collection, 1),
			Ok(Some(NftsWeightInfo::clear_collection_approvals(1)).into())
		);
		assert_eq!(
			CollectionApprovals::iter_prefix((collection, owner,)).count(),
			(approvals - 1) as usize
		);
		// Successfully clear all collection approvals.
		assert_eq!(
			NonFungibles::clear_collection_approvals(signed(owner), collection, approvals),
			Ok(Some(NftsWeightInfo::clear_collection_approvals(approvals - 1)).into())
		);
		assert!(CollectionApprovals::iter_prefix((collection, owner,)).count().is_zero());
	});
}

#[test]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let item = ITEM;
		let owner = ALICE;
		let witness = MintWitness { mint_price: None, owned_item: None };

		// Check error works for `Nfts::mint()`.
		assert_noop!(
			NonFungibles::mint(signed(owner), collection, owner, item, Some(witness.clone())),
			NftsError::NoConfig
		);
		// Successfully mint a new collection item.
		nfts::create_collection(owner);
		let balance_before_mint = nfts::balance_of(collection, &owner);
		assert_ok!(NonFungibles::mint(signed(owner), collection, owner, item, Some(witness)));
		let balance_after_mint = nfts::balance_of(collection, &owner);
		assert_eq!(balance_after_mint, balance_before_mint + 1);
		System::assert_last_event(
			Event::Transfer { collection, item, from: None, to: Some(owner) }.into(),
		);
	});
}

#[test]
fn burn_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let owner = ALICE;
		let item = ITEM;

		// Throw `NftsError::UnknownItem` if no owner found for the item.
		assert_noop!(NonFungibles::burn(signed(owner), collection, item), NftsError::UnknownItem);
		nfts::create_collection_and_mint(owner, owner, item);
		// Check error works for `Nfts::burn()`.
		assert_noop!(NonFungibles::burn(signed(BOB), collection, item), NftsError::NoPermission);
		// Successfully burn a collection item.
		let balance_before_burn = nfts::balance_of(collection, &owner);
		assert_ok!(NonFungibles::burn(signed(owner), collection, item));
		let balance_after_burn = nfts::balance_of(collection, &owner);
		assert_eq!(balance_after_burn, balance_before_burn - 1);
		System::assert_last_event(
			Event::Transfer { collection, item, from: Some(owner), to: None }.into(),
		);
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let collection = nfts::create_collection(owner);

		assert_eq!(
			NonFungibles::read(BalanceOf { collection, owner }),
			ReadResult::BalanceOf(Default::default())
		);
		(0..10).into_iter().for_each(|i| {
			assert_ok!(Nfts::mint(signed(owner), collection, i, owner, None));
			assert_eq!(
				NonFungibles::read(BalanceOf { collection, owner }),
				ReadResult::BalanceOf(i + 1)
			);
			assert_eq!(
				NonFungibles::read(BalanceOf { collection, owner }).encode(),
				nfts::balance_of(collection, &owner).encode()
			);
		});
	});
}

#[test]
fn owner_of_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let item = ITEM;
		let owner = ALICE;

		assert_eq!(NonFungibles::read(OwnerOf { collection, item }), ReadResult::OwnerOf(None));
		nfts::create_collection_and_mint(owner, owner, item);
		assert_eq!(
			NonFungibles::read(OwnerOf { collection, item }),
			ReadResult::OwnerOf(Some(owner))
		);
		assert_eq!(
			NonFungibles::read(OwnerOf { collection, item }).encode(),
			Nfts::owner(collection, item).encode()
		);
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let item = ITEM;
		let operator = BOB;
		let owner = ALICE;

		nfts::create_collection_mint_and_approve(owner, owner, item, operator);
		assert_eq!(
			NonFungibles::read(Allowance { collection, owner, operator, item: Some(item) }),
			ReadResult::Allowance(true)
		);
		assert_eq!(
			NonFungibles::read(Allowance { collection, owner, operator, item: Some(item) })
				.encode(),
			Nfts::check_approval_permission(&collection, &Some(item), &owner, &operator)
				.is_ok()
				.encode()
		);
	});
}

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let collection = nfts::create_collection(owner);

		assert_eq!(NonFungibles::read(TotalSupply(collection)), ReadResult::TotalSupply(0));
		(0..10).into_iter().for_each(|i| {
			assert_ok!(Nfts::mint(signed(owner), collection, i, owner, None));
			assert_eq!(
				NonFungibles::read(TotalSupply(collection)),
				ReadResult::TotalSupply((i + 1).into())
			);
			assert_eq!(
				NonFungibles::read(TotalSupply(collection)).encode(),
				(Nfts::collection_items(collection).unwrap_or_default() as u128).encode()
			);
		});
	});
}

mod get_attribute {
	use super::*;

	#[test]
	fn get_attribute_works() {
		new_test_ext().execute_with(|| {
			let attribute = BoundedVec::truncate_from("some attribute".into());
			let collection = COLLECTION;
			let item = ITEM;
			let metadata = "some value".as_bytes().to_vec();
			let owner = ALICE;

			nfts::create_collection_and_mint(owner, owner, item);
			// No attribute set.
			assert_eq!(
				NonFungibles::read(GetAttribute {
					collection,
					item: Some(item),
					namespace: AttributeNamespace::CollectionOwner,
					key: attribute.clone()
				}),
				ReadResult::GetAttribute(None)
			);
			// Successfully get an existing attribute.
			assert_ok!(Nfts::set_attribute(
				signed(owner),
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				attribute.clone(),
				BoundedVec::truncate_from(metadata.clone()),
			));
			assert_eq!(
				NonFungibles::read(GetAttribute {
					collection,
					item: Some(item),
					namespace: AttributeNamespace::CollectionOwner,
					key: attribute.clone()
				}),
				ReadResult::GetAttribute(Some(metadata))
			);
			assert_eq!(
				NonFungibles::read(GetAttribute {
					collection,
					item: Some(item),
					namespace: AttributeNamespace::CollectionOwner,
					key: attribute.clone()
				})
				.encode(),
				nfts::get_attribute(
					collection,
					Some(item),
					AttributeNamespace::CollectionOwner,
					attribute
				)
				.encode()
			);
		});
	}

	#[test]
	fn get_collection_attribute_works() {
		new_test_ext().execute_with(|| {
			let attribute = BoundedVec::truncate_from("some attribute".into());
			let collection = COLLECTION;
			let metadata = "some value".as_bytes().to_vec();
			let owner = ALICE;

			nfts::create_collection(owner);
			// No attribute set.
			assert_eq!(
				NonFungibles::read(GetAttribute {
					collection,
					item: None,
					namespace: AttributeNamespace::CollectionOwner,
					key: attribute.clone()
				}),
				ReadResult::GetAttribute(None)
			);
			// Successfully get an existing attribute.
			assert_ok!(Nfts::set_attribute(
				signed(owner),
				collection,
				None,
				AttributeNamespace::CollectionOwner,
				attribute.clone(),
				BoundedVec::truncate_from(metadata.clone()),
			));
			assert_eq!(
				NonFungibles::read(GetAttribute {
					collection,
					item: None,
					namespace: AttributeNamespace::CollectionOwner,
					key: attribute.clone()
				}),
				ReadResult::GetAttribute(Some(metadata))
			);
			assert_eq!(
				NonFungibles::read(GetAttribute {
					collection,
					item: None,
					namespace: AttributeNamespace::CollectionOwner,
					key: attribute.clone()
				})
				.encode(),
				nfts::get_attribute(
					collection,
					None,
					AttributeNamespace::CollectionOwner,
					attribute
				)
				.encode()
			);
		});
	}
}

#[test]
fn next_collection_id_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(NonFungibles::read(NextCollectionId), ReadResult::NextCollectionId(Some(0)));
		nfts::create_collection_and_mint(ALICE, ALICE, ITEM);
		assert_eq!(NonFungibles::read(NextCollectionId), ReadResult::NextCollectionId(Some(1)));
		assert_eq!(
			NonFungibles::read(NextCollectionId).encode(),
			Some(NextCollectionIdOf::<Test>::get().unwrap_or_default()).encode(),
		);
	});
}

#[test]
fn item_metadata_works() {
	new_test_ext().execute_with(|| {
		let collection = COLLECTION;
		let item = ITEM;
		let metadata = "some metadata".as_bytes().to_vec();
		let owner = ALICE;

		// Read item metadata of an unknown collection.
		assert_eq!(
			NonFungibles::read(ItemMetadata { collection, item }),
			ReadResult::ItemMetadata(None)
		);
		nfts::create_collection_and_mint(owner, owner, item);
		// Successfully set the metadata of an item.
		assert_ok!(NonFungibles::set_metadata(
			signed(owner),
			collection,
			item,
			BoundedVec::truncate_from(metadata.clone())
		));
		assert_eq!(
			NonFungibles::read(ItemMetadata { collection, item }),
			ReadResult::ItemMetadata(Some(metadata))
		);
		assert_eq!(
			NonFungibles::read(ItemMetadata { collection, item }).encode(),
			Nfts::item_metadata(collection, item).encode()
		);
	});
}

// Helper functions for interacting with pallet-nfts.
mod nfts {
	use super::*;
	use crate::nonfungibles::AttributeNamespaceOf;

	pub(super) fn balance_of(collection: CollectionIdOf<Test>, owner: &AccountId) -> u32 {
		AccountBalanceOf::<Test>::get(collection, &owner)
			.map(|(balance, _)| balance)
			.unwrap_or_default()
	}

	pub(super) fn create_collection(owner: AccountId) -> u32 {
		let next_id = NextCollectionIdOf::<Test>::get().unwrap_or_default();
		assert_ok!(Nfts::create(
			signed(owner),
			owner,
			collection_config_with_all_settings_enabled()
		));
		next_id
	}

	pub(super) fn create_collection_and_mint(
		owner: AccountId,
		mint_to: AccountId,
		item: ItemIdOf<Test>,
	) -> (u32, u32) {
		let collection = create_collection(owner);
		assert_ok!(Nfts::mint(signed(owner), collection, item, mint_to, None));
		(collection, item)
	}

	pub(super) fn create_collection_mint_and_approve(
		owner: AccountId,
		mint_to: AccountId,
		item: ItemIdOf<Test>,
		operator: AccountId,
	) {
		let (collection, item) = create_collection_and_mint(owner, mint_to, item);
		assert_ok!(Nfts::approve_transfer(signed(owner), collection, item, operator, None));
		assert_ok!(Nfts::check_approval_permission(&collection, &Some(item), &owner, &operator));
	}

	pub(super) fn collection_config_with_all_settings_enabled() -> CollectionConfigOf<Test> {
		CollectionConfig {
			settings: CollectionSettings::all_enabled(),
			max_supply: None,
			mint_settings: MintSettings::default(),
		}
	}

	pub(super) fn get_attribute(
		collection: CollectionIdOf<Test>,
		maybe_item: Option<ItemIdOf<Test>>,
		namespace: AttributeNamespaceOf<Test>,
		key: BoundedVec<u8, <Test as pallet_nfts::Config<NftsInstanceOf<Test>>>::KeyLimit>,
	) -> Option<Vec<u8>> {
		AttributeOf::<Test>::get((collection, maybe_item, namespace, key))
			.map(|attribute| attribute.0.into())
	}
}

mod ensure_codec_indexes {
	use super::{Encode, *};
	use crate::{mock::RuntimeCall::NonFungibles, nonfungibles};

	#[test]
	fn ensure_dispatchable_indexes() {
		use nonfungibles::Call::*;

		[
			(
				approve {
					collection: Default::default(),
					operator: Default::default(),
					item: Default::default(),
					approved: Default::default(),
					deadline: Default::default(),
				},
				3u8,
				"approve",
			),
			(
				transfer {
					to: Default::default(),
					collection: Default::default(),
					item: Default::default(),
				},
				4,
				"transfer",
			),
			(create { admin: Default::default(), config: Default::default() }, 9, "create"),
			(
				destroy {
					collection: Default::default(),
					witness: DestroyWitness {
						item_metadatas: Default::default(),
						item_configs: Default::default(),
						attributes: Default::default(),
					},
				},
				10,
				"destroy",
			),
			(
				set_attribute {
					collection: Default::default(),
					item: Default::default(),
					namespace: AttributeNamespace::CollectionOwner,
					key: Default::default(),
					value: Default::default(),
				},
				11,
				"set_attribute",
			),
			(
				clear_attribute {
					collection: Default::default(),
					item: Default::default(),
					namespace: AttributeNamespace::CollectionOwner,
					key: Default::default(),
				},
				12,
				"clear_attribute",
			),
			(
				set_metadata {
					collection: Default::default(),
					item: Default::default(),
					data: Default::default(),
				},
				13,
				"set_metadata",
			),
			(
				clear_metadata { collection: Default::default(), item: Default::default() },
				14,
				"clear_metadata",
			),
			(
				set_max_supply { collection: Default::default(), max_supply: Default::default() },
				15,
				"set_max_supply",
			),
			(
				approve_item_attributes {
					collection: Default::default(),
					item: Default::default(),
					delegate: Default::default(),
				},
				16,
				"approve_item_attributes",
			),
			(
				cancel_item_attributes_approval {
					collection: Default::default(),
					item: Default::default(),
					delegate: Default::default(),
					witness: CancelAttributesApprovalWitness {
						account_attributes: Default::default(),
					},
				},
				17,
				"cancel_item_attributes_approval",
			),
			(
				clear_all_transfer_approvals {
					collection: Default::default(),
					item: Default::default(),
				},
				18,
				"clear_all_transfer_approvals",
			),
			(
				clear_collection_approvals {
					collection: Default::default(),
					limit: Default::default(),
				},
				19,
				"clear_collection_approvals",
			),
			(
				mint {
					collection: Default::default(),
					to: Default::default(),
					item: Default::default(),
					witness: Some(MintWitness {
						owned_item: Default::default(),
						mint_price: Default::default(),
					}),
				},
				20,
				"mint",
			),
			(burn { collection: Default::default(), item: Default::default() }, 21, "burn"),
		]
		.iter()
		.for_each(|(variant, expected_index, name)| {
			assert_eq!(
				NonFungibles(variant.to_owned()).encode()[1],
				*expected_index,
				"{name} dispatchable index changed"
			);
		})
	}

	#[test]
	fn ensure_read_variant_indexes() {
		[
			(
				BalanceOf::<Test> { collection: Default::default(), owner: Default::default() },
				0u8,
				"BalanceOf",
			),
			(
				OwnerOf::<Test> { collection: Default::default(), item: Default::default() },
				1,
				"OwnerOf",
			),
			(
				Allowance::<Test> {
					collection: Default::default(),
					item: Default::default(),
					owner: Default::default(),
					operator: Default::default(),
				},
				2,
				"Allowance",
			),
			(TotalSupply::<Test>(Default::default()), 5, "TotalSupply"),
			(
				GetAttribute::<Test> {
					collection: Default::default(),
					item: Default::default(),
					namespace: AttributeNamespace::CollectionOwner,
					key: Default::default(),
				},
				6,
				"GetAttribute",
			),
			(NextCollectionId, 7, "NextCollectionId"),
			(
				ItemMetadata { collection: Default::default(), item: Default::default() },
				8,
				"ItemMetadata",
			),
		]
		.iter()
		.for_each(|(variant, expected_index, name)| {
			assert_eq!(variant.encode()[0], *expected_index, "{name} variant index changed");
		})
	}
}

mod read_weights {
	use super::*;

	struct ReadWeightInfo {
		balance_of: Weight,
		owner_of: Weight,
		allowance: Weight,
		total_supply: Weight,
		get_attribute: Weight,
		next_collection_id: Weight,
		item_metadata: Weight,
	}

	impl ReadWeightInfo {
		fn new() -> Self {
			Self {
				balance_of: NonFungibles::weight(&BalanceOf {
					collection: COLLECTION,
					owner: ALICE,
				}),
				owner_of: NonFungibles::weight(&OwnerOf { collection: COLLECTION, item: ITEM }),
				allowance: NonFungibles::weight(&Allowance {
					collection: COLLECTION,
					item: Some(ITEM),
					owner: ALICE,
					operator: BOB,
				}),
				total_supply: NonFungibles::weight(&TotalSupply(COLLECTION)),
				get_attribute: NonFungibles::weight(&GetAttribute {
					collection: COLLECTION,
					item: Some(ITEM),
					namespace: AttributeNamespace::CollectionOwner,
					key: BoundedVec::default(),
				}),
				next_collection_id: NonFungibles::weight(&NextCollectionId),
				item_metadata: NonFungibles::weight(&ItemMetadata {
					collection: COLLECTION,
					item: ITEM,
				}),
			}
		}
	}

	#[test]
	fn ensure_read_matches_benchmarks() {
		let ReadWeightInfo {
			balance_of,
			owner_of,
			allowance,
			total_supply,
			get_attribute,
			next_collection_id,
			item_metadata,
		} = ReadWeightInfo::new();

		assert_eq!(balance_of, WeightInfo::balance_of());
		assert_eq!(owner_of, WeightInfo::owner_of());
		assert_eq!(allowance, WeightInfo::allowance());
		assert_eq!(total_supply, WeightInfo::total_supply());
		assert_eq!(get_attribute, WeightInfo::get_attribute());
		assert_eq!(next_collection_id, WeightInfo::next_collection_id());
		assert_eq!(item_metadata, WeightInfo::item_metadata());
	}

	// Proof size is based on `MaxEncodedLen`, not hardware.
	// This test ensures that the data structure sizes do not change with upgrades.
	#[test]
	fn ensure_expected_proof_size_does_not_change() {
		let ReadWeightInfo {
			balance_of,
			owner_of,
			allowance,
			total_supply,
			get_attribute,
			next_collection_id,
			item_metadata,
		} = ReadWeightInfo::new();

		// These values come from `weights.rs`.
		assert_eq!(balance_of.proof_size(), 3585);
		assert_eq!(owner_of.proof_size(), 4326);
		assert_eq!(allowance.proof_size(), 4326);
		assert_eq!(total_supply.proof_size(), 3549);
		assert_eq!(get_attribute.proof_size(), 3944);
		assert_eq!(next_collection_id.proof_size(), 1489);
		assert_eq!(item_metadata.proof_size(), 3812);
	}
}

mod encoding_read_result {
	use super::*;

	#[test]
	fn balance_of() {
		let balance: u32 = 100;
		assert_eq!(ReadResult::BalanceOf::<Test>(balance).encode(), balance.encode());
	}

	#[test]
	fn owner_of() {
		let mut owner = Some(ALICE);
		assert_eq!(ReadResult::OwnerOf::<Test>(owner.clone()).encode(), owner.encode());
		owner = None;
		assert_eq!(ReadResult::OwnerOf::<Test>(owner.clone()).encode(), owner.encode());
	}

	#[test]
	fn allowance() {
		let allowance = false;
		assert_eq!(ReadResult::Allowance::<Test>(allowance).encode(), allowance.encode());
	}

	#[test]
	fn total_supply() {
		let total_supply: u128 = 1_000_000;
		assert_eq!(ReadResult::TotalSupply::<Test>(total_supply).encode(), total_supply.encode());
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

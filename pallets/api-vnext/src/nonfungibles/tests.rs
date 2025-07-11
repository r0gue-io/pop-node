use frame_support::{
	assert_noop, assert_ok,
	pallet_prelude::Zero,
	sp_runtime::{traits::BadOrigin, AccountId32},
	traits::Get,
};
use pallet_nfts::{
	CollectionApprovals, CollectionConfig, CollectionSetting, CollectionSettings, MintSettings,
	WeightInfo,
};

use super::*;
use crate::mock::{Nfts, *};

const COLLECTION: u32 = 0;
const ITEM: u32 = 1;

type AccountBalanceOf = pallet_nfts::AccountBalance<Test>;
type AttributeNamespaceOf = AttributeNamespace<AccountIdOf<Test>>;
type AttributeOf = pallet_nfts::Attribute<Test>;
type ED = ExistentialDeposit;
type NextCollectionIdOf = pallet_nfts::NextCollectionId<Test>;
type NftsError = pallet_nfts::Error<Test>;
type NftsWeightInfo = <Test as pallet_nfts::Config>::WeightInfo;
// type WeightInfo = <Test as Config>::WeightInfo;

mod approve {
	use super::*;

	#[test]
	fn approve_works() {
		let collection = COLLECTION;
		let item = ITEM;
		let operator = BOB;
		let owner = ALICE;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), 10_000_000), (operator.clone(), ED::get())])
			.build()
			.execute_with(|| {
				// Check error works for `Nfts::approve_transfer()`.
				assert_noop!(
					approve::<Test, ()>(
						signed(owner.clone()),
						collection,
						operator.clone(),
						Some(item),
						true,
						None
					),
					// TODO: Handle weight
					NftsError::UnknownItem.with_weight(Weight::default())
				);
				nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
				// Successfully approve `operator` to transfer the collection item.
				assert_ok!(approve::<Test, ()>(
					signed(owner.clone()),
					collection,
					operator.clone(),
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
			});
	}

	#[test]
	fn approve_collection_works() {
		let collection = COLLECTION;
		let item = ITEM;
		let operator = BOB;
		let owner = ALICE;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), 10_000_000), (operator.clone(), ED::get())])
			.build()
			.execute_with(|| {
				// Check error works for `Nfts::approve_collection_transfer()`.
				assert_noop!(
					approve::<Test, ()>(
						signed(owner.clone()),
						collection,
						operator.clone(),
						None,
						true,
						None
					),
					NftsError::NoItemOwned.with_weight(Weight::default())
				);
				nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
				// Successfully approve `operator` to transfer all collection items owned by
				// `owner`.
				assert_ok!(approve::<Test, ()>(
					signed(owner.clone()),
					collection,
					operator.clone(),
					None,
					true,
					None
				));
				assert_ok!(Nfts::check_approval_permission(&collection, &None, &owner, &operator));
			});
	}

	#[test]
	fn cancel_approval_works() {
		let collection = COLLECTION;
		let item = ITEM;
		let operator = BOB;
		let owner = ALICE;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), 10_000_000), (operator.clone(), ED::get())])
			.build()
			.execute_with(|| {
				// Check error works for `Nfts::cancel_approval()`.
				assert_noop!(
					approve::<Test, ()>(
						signed(owner.clone()),
						collection,
						operator.clone(),
						Some(item),
						false,
						None
					),
					NftsError::UnknownItem.with_weight(Weight::default())
				);
				nfts::create_collection_mint_and_approve(
					owner.clone(),
					owner.clone(),
					item,
					operator.clone(),
				);
				// Successfully cancel the transfer approval of `operator` by `owner`.
				assert_ok!(approve::<Test, ()>(
					signed(owner.clone()),
					collection,
					operator.clone(),
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
		let collection = COLLECTION;
		let item = ITEM;
		let operator = BOB;
		let owner = ALICE;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), 10_000_000), (operator.clone(), ED::get())])
			.build()
			.execute_with(|| {
				// Check error works for `Nfts::cancel_collection_approval()`.
				assert_noop!(
					approve::<Test, ()>(
						signed(owner.clone()),
						collection,
						operator.clone(),
						None,
						false,
						None
					),
					// TODO: Handle weight calculation
					NftsError::NotDelegate.with_weight(Weight::default())
				);
				nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
				assert_ok!(Nfts::approve_collection_transfer(
					signed(owner.clone()),
					collection,
					operator.clone().into(),
					None
				));
				// Successfully cancel the transfer collection approval of `operator` by `owner`.
				assert_ok!(approve::<Test, ()>(
					signed(owner.clone()),
					collection,
					operator.clone(),
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
	use frame_support::assert_ok;

	use super::*;

	#[test]
	fn transfer_works() {
		let collection = COLLECTION;
		let dest = BOB;
		let item = ITEM;
		let owner = ALICE;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), 10_000_000), (dest.clone(), ED::get())])
			.build()
			.execute_with(|| {
				nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
				// Throw `NftsError::UnknownItem` if no item found.
				assert_noop!(
					transfer::<Test, ()>(signed(dest.clone()), collection, dest.clone(), ITEM + 1),
					NftsError::UnknownItem
				);
				// Check error works for `Nfts::transfer()`.
				assert_noop!(
					transfer::<Test, ()>(signed(dest.clone()), collection, dest.clone(), item),
					NftsError::NoPermission
				);
				// Successfully transfer a collection item.
				let owner_balance_before_transfer = nfts::balance_of(collection, &owner);
				let dest_balance_before_transfer = nfts::balance_of(collection, &dest);
				assert_ok!(transfer::<Test, ()>(
					signed(owner.clone()),
					collection,
					dest.clone(),
					item
				));
				let owner_balance_after_transfer = nfts::balance_of(collection, &owner);
				let dest_balance_after_transfer = nfts::balance_of(collection, &dest);
				// Check that `to` has received the collection item from `from`.
				assert_eq!(owner_balance_after_transfer, owner_balance_before_transfer - 1);
				assert_eq!(dest_balance_after_transfer, dest_balance_before_transfer + 1);
			});
	}

	#[test]
	fn approved_transfer_works() {
		let collection = COLLECTION;
		let dest = CHARLIE;
		let item = ITEM;
		let operator = BOB;
		let owner = ALICE;
		ExtBuilder::new()
			.with_balances(vec![
				(owner.clone(), 10_000_000),
				(operator.clone(), 10_000_000),
				(dest.clone(), 10_000_000),
			])
			.build()
			.execute_with(|| {
				nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
				// Approve `operator` to transfer all `collection` items owned by the `owner`.
				assert_ok!(Nfts::approve_collection_transfer(
					signed(owner.clone()),
					collection,
					operator.clone().into(),
					None
				));
				// Successfully transfer a collection item.
				let owner_balance_before_transfer = nfts::balance_of(collection, &owner);
				let dest_balance_before_transfer = nfts::balance_of(collection, &dest);
				assert_ok!(transfer::<Test, ()>(
					signed(operator.clone()),
					collection,
					dest.clone(),
					item
				));
				let owner_balance_after_transfer = nfts::balance_of(collection, &owner);
				let dest_balance_after_transfer = nfts::balance_of(collection, &dest);
				// Check that `to` has received the collection item from `from`.
				assert_eq!(owner_balance_after_transfer, owner_balance_before_transfer - 1);
				assert_eq!(dest_balance_after_transfer, dest_balance_before_transfer + 1);
			});
	}
}

#[test]
fn create_works() {
	let admin = ALICE;
	let mut config = CollectionConfig {
		max_supply: None,
		mint_settings: MintSettings::default(),
		settings: CollectionSettings::all_enabled(),
	};
	let collection = COLLECTION;
	let creator = ALICE;
	ExtBuilder::new()
		.with_balances(vec![(admin.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			// Origin checks.
			for origin in vec![root(), none()] {
				assert_noop!(create::<Test, ()>(origin, admin.clone(), config.clone()), BadOrigin);
			}

			// Check error works for `Nfts::create()`.
			config.disable_setting(CollectionSetting::DepositRequired);
			assert_noop!(
				create::<Test, ()>(signed(creator.clone()), admin.clone(), config.clone()),
				NftsError::WrongSetting
			);
			config.enable_setting(CollectionSetting::DepositRequired);
			// Successfully create a collection.
			assert_ok!(create::<Test, ()>(signed(creator.clone()), admin, config));
			assert_eq!(Nfts::collection_owner(collection), Some(creator));
		});
}

#[test]
fn destroy_works() {
	let collection = COLLECTION;
	let owner = ALICE;
	let witness = DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 };
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::destroy()`.
			assert_noop!(
				destroy::<Test, ()>(signed(owner.clone()), collection, witness),
				NftsError::UnknownCollection
			);
			nfts::create_collection(owner.clone());
			// Successfully destroy a collection.
			assert_ok!(destroy::<Test, ()>(signed(owner), collection, witness));
			assert_eq!(Nfts::collection_owner(collection), None);
		});
}

#[test]
fn set_attribute_works() {
	let attribute = BoundedVec::truncate_from("some attribute".into());
	let collection = COLLECTION;
	let item = ITEM;
	let owner = ALICE;
	let value = BoundedVec::truncate_from("some value".into());
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::clear_attribute()`.
			assert_noop!(
				Nfts::set_attribute(
					signed(owner.clone()),
					collection,
					Some(item),
					AttributeNamespace::CollectionOwner,
					attribute.clone(),
					value.clone()
				),
				NftsError::UnknownCollection
			);
			nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
			assert_ok!(Nfts::set_attribute(
				signed(owner.clone()),
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				attribute.clone(),
				value.clone()
			));
			// Successfully clear an attribute.
			assert_ok!(clear_attribute::<Test, ()>(
				signed(owner.clone()),
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
fn clear_attribute_works() {
	let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
	let collection = COLLECTION;
	let item = ITEM;
	let owner = ALICE;
	let value = BoundedVec::truncate_from("some value".as_bytes().to_vec());
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::clear_attribute()`.
			assert_noop!(
				Nfts::set_attribute(
					signed(owner.clone()),
					collection,
					Some(item),
					AttributeNamespace::CollectionOwner,
					attribute.clone(),
					value.clone()
				),
				NftsError::UnknownCollection
			);
			nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
			assert_ok!(Nfts::set_attribute(
				signed(owner.clone()),
				collection,
				Some(item),
				AttributeNamespace::CollectionOwner,
				attribute.clone(),
				value.clone()
			));
			// Successfully clear an attribute.
			assert_ok!(clear_attribute::<Test, ()>(
				signed(owner.clone()),
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
	let collection = COLLECTION;
	let item = ITEM;
	let metadata = BoundedVec::truncate_from("some metadata".into());
	let owner = ALICE;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::set_metadata()`.
			assert_noop!(
				set_metadata::<Test, ()>(signed(owner.clone()), collection, item, metadata.clone()),
				NftsError::NoPermission
			);
			nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
			// Successfully set the metadata.
			assert_ok!(set_metadata::<Test, ()>(
				signed(owner.clone()),
				collection,
				item,
				metadata.clone()
			));
			assert_eq!(Nfts::item_metadata(collection, item), Some(metadata));
		});
}

#[test]
fn clear_metadata_works() {
	let collection = COLLECTION;
	let item = ITEM;
	let metadata = BoundedVec::truncate_from("some metadata".into());
	let owner = ALICE;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::clear_metadata()`.
			assert_noop!(
				clear_metadata::<Test, ()>(signed(owner.clone()), collection, item),
				NftsError::NoPermission
			);
			nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
			assert_ok!(Nfts::set_metadata(signed(owner.clone()), collection, item, metadata));
			// Successfully clear the metadata.
			assert_ok!(clear_metadata::<Test, ()>(signed(owner), collection, item));
			assert!(Nfts::item_metadata(collection, item).is_none());
		});
}

#[test]
fn set_max_supply_works() {
	let collection = COLLECTION;
	let owner = ALICE;
	let max_supply = 10;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::set_max_supply()`.
			assert_noop!(
				set_max_supply::<Test, ()>(signed(owner.clone()), collection, max_supply),
				NftsError::NoConfig
			);
			nfts::create_collection(owner.clone());
			// Successfully set the max supply for the collection.
			assert_ok!(set_max_supply::<Test, ()>(signed(owner.clone()), collection, max_supply));
			(0..max_supply).into_iter().for_each(|i| {
				assert_ok!(Nfts::mint(
					signed(owner.clone()),
					collection,
					i,
					owner.clone().into(),
					None
				));
			});
			// Throws `MaxSupplyReached` error if number of minted items is over the max supply.
			assert_noop!(
				Nfts::mint(signed(owner.clone()), collection, 42, owner.clone().into(), None),
				NftsError::MaxSupplyReached
			);
			// Override the max supply.
			assert_ok!(set_max_supply::<Test, ()>(
				signed(owner.clone()),
				collection,
				max_supply * 2
			));
			assert_ok!(Nfts::mint(signed(owner.clone()), collection, 42, owner.into(), None));
		});
}

#[test]
fn approve_item_attribute_works() {
	let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
	let collection = COLLECTION;
	let delegate = BOB;
	let item = ITEM;
	let owner = ALICE;
	let value = BoundedVec::truncate_from("some value".as_bytes().to_vec());
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000), (delegate.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::approve_item_attributes()`.
			assert_noop!(
				approve_item_attributes::<Test, ()>(
					signed(owner.clone()),
					collection,
					item,
					delegate.clone()
				),
				NftsError::UnknownItem
			);
			nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
			// Successfully approve delegate to set attributes.
			assert_ok!(approve_item_attributes::<Test, ()>(
				signed(owner.clone()),
				collection,
				item,
				delegate.clone()
			));
			assert_ok!(Nfts::set_attribute(
				signed(delegate.clone()),
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
	let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
	let collection = COLLECTION;
	let delegate = BOB;
	let item = ITEM;
	let owner = ALICE;
	let value = BoundedVec::truncate_from("some value".as_bytes().to_vec());
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000), (delegate.clone(), ED::get())])
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::cancel_item_attribute_approval()`.
			assert_noop!(
				cancel_item_attributes_approval::<Test, ()>(
					signed(owner.clone()),
					collection,
					item,
					delegate.clone(),
					CancelAttributesApprovalWitness { account_attributes: 1 }
				),
				NftsError::UnknownItem
			);

			nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
			assert_ok!(Nfts::approve_item_attributes(
				signed(owner.clone()),
				collection,
				item,
				delegate.clone().into()
			));
			// Successfully cancel item attribute approval.
			assert_ok!(Nfts::cancel_item_attributes_approval(
				signed(owner),
				collection,
				item,
				delegate.clone().into(),
				CancelAttributesApprovalWitness { account_attributes: 1 }
			));
			assert_noop!(
				Nfts::set_attribute(
					signed(delegate.clone()),
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
	let collection = COLLECTION;
	let delegates = 10..20;
	let delegate_balances = delegates
		.clone()
		.into_iter()
		.map(|d| (AccountId32::new([d; 32]), ED::get()))
		.collect::<Vec<_>>();
	let item = ITEM;
	let owner = ALICE;
	ExtBuilder::new()
		.with_balances(vec![vec![(owner.clone(), 10_000_000)], delegate_balances].concat())
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::clear_all_transfer_approvals()`.
			assert_noop!(
				clear_all_transfer_approvals::<Test, ()>(signed(owner.clone()), collection, item),
				NftsError::UnknownCollection
			);

			nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
			delegates.clone().for_each(|delegate| {
				assert_ok!(Nfts::approve_transfer(
					signed(owner.clone()),
					collection,
					item,
					AccountId32::new([delegate; 32]).into(),
					None
				));
			});
			// Successfully clear all transfer approvals.
			assert_ok!(clear_all_transfer_approvals::<Test, ()>(
				signed(owner.clone()),
				collection,
				item
			));
			delegates.for_each(|delegate| {
				assert!(Nfts::check_approval_permission(
					&collection,
					&Some(item),
					&owner,
					&AccountId32::new([delegate; 32]).into()
				)
				.is_err());
			});
		});
}

#[test]
fn clear_collection_approvals_works() {
	let collection = COLLECTION;
	let delegates = 10..20;
	let delegate_balances = delegates
		.clone()
		.into_iter()
		.map(|d| (AccountId32::new([d; 32]), ED::get()))
		.collect::<Vec<_>>();
	let owner = ALICE;
	let approvals = (delegates.end - delegates.start) as u32;
	ExtBuilder::new()
		.with_balances(vec![vec![(owner.clone(), 10_000_000)], delegate_balances].concat())
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::clear_collection_approvals()`.
			assert_noop!(
				clear_collection_approvals::<Test, ()>(none(), collection, 1),
				BadOrigin.with_weight(NftsWeightInfo::clear_collection_approvals(0))
			);
			nfts::create_collection_and_mint(owner.clone(), owner.clone(), ITEM);
			delegates.clone().for_each(|delegate| {
				assert_ok!(Nfts::approve_collection_transfer(
					signed(owner.clone()),
					collection,
					AccountId32::new([delegate; 32]).into(),
					None
				));
			});
			// Partially clear collection approvals.
			assert_eq!(
				clear_collection_approvals::<Test, ()>(signed(owner.clone()), collection, 1),
				Ok(Some(NftsWeightInfo::clear_collection_approvals(1)).into())
			);
			assert_eq!(
				CollectionApprovals::<Test, ()>::iter_prefix((collection, owner.clone(),)).count(),
				(approvals - 1) as usize
			);
			// Successfully clear all collection approvals.
			assert_eq!(
				clear_collection_approvals::<Test, ()>(
					signed(owner.clone()),
					collection,
					approvals
				),
				Ok(Some(NftsWeightInfo::clear_collection_approvals(approvals - 1)).into())
			);
			assert!(CollectionApprovals::<Test, ()>::iter_prefix((collection, owner,))
				.count()
				.is_zero());
		});
}

#[test]
fn mint_works() {
	let collection = COLLECTION;
	let item = ITEM;
	let owner = ALICE;
	let witness = MintWitness { mint_price: None, owned_item: None };
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			// Check error works for `Nfts::mint()`.
			assert_noop!(
				mint::<Test, ()>(
					signed(owner.clone()),
					collection,
					owner.clone(),
					item,
					Some(witness.clone())
				),
				NftsError::NoConfig
			);
			// Successfully mint a new collection item.
			nfts::create_collection(owner.clone());
			let balance_before_mint = nfts::balance_of(collection, &owner);
			assert_ok!(mint::<Test, ()>(
				signed(owner.clone()),
				collection,
				owner.clone(),
				item,
				Some(witness)
			));
			let balance_after_mint = nfts::balance_of(collection, &owner);
			assert_eq!(balance_after_mint, balance_before_mint + 1);
		});
}

#[test]
fn burn_works() {
	let collection = COLLECTION;
	let owner = ALICE;
	let item = ITEM;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000), (BOB, ED::get())])
		.build()
		.execute_with(|| {
			// Throw `NftsError::UnknownItem` if no owner found for the item.
			assert_noop!(
				burn::<Test, ()>(signed(owner.clone()), collection, item),
				NftsError::UnknownItem
			);
			nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
			// Check error works for `Nfts::burn()`.
			assert_noop!(burn::<Test, ()>(signed(BOB), collection, item), NftsError::NoPermission);
			// Successfully burn a collection item.
			let balance_before_burn = nfts::balance_of(collection, &owner);
			assert_ok!(burn::<Test, ()>(signed(owner.clone()), collection, item));
			let balance_after_burn = nfts::balance_of(collection, &owner);
			assert_eq!(balance_after_burn, balance_before_burn - 1);
		});
}

#[test]
fn balance_of_works() {
	let owner = ALICE;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			let collection = nfts::create_collection(owner.clone());
			assert_eq!(balance_of::<Test, ()>(collection, owner.clone()), 0);
			(0..10).into_iter().for_each(|i| {
				assert_ok!(Nfts::mint(
					signed(owner.clone()),
					collection,
					i,
					owner.clone().into(),
					None
				));
				assert_eq!(balance_of::<Test, ()>(collection, owner.clone()), i + 1);
				assert_eq!(
					balance_of::<Test, ()>(collection, owner.clone()),
					nfts::balance_of(collection, &owner)
				);
			});
		});
}

#[test]
fn owner_of_works() {
	let collection = COLLECTION;
	let item = ITEM;
	let owner = ALICE;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			assert_eq!(owner_of::<Test, ()>(collection, item), None);
			nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
			assert_eq!(owner_of::<Test, ()>(collection, item), Some(owner));
			assert_eq!(owner_of::<Test, ()>(collection, item), Nfts::owner(collection, item));
		});
}

#[test]
fn allowance_works() {
	let collection = COLLECTION;
	let item = ITEM;
	let operator = BOB;
	let owner = ALICE;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000), (operator.clone(), ED::get())])
		.build()
		.execute_with(|| {
			nfts::create_collection_mint_and_approve(
				owner.clone(),
				owner.clone(),
				item,
				operator.clone(),
			);
			assert!(allowance::<Test, ()>(collection, owner.clone(), operator.clone(), Some(item)));
			assert_eq!(
				allowance::<Test, ()>(collection, owner.clone(), operator.clone(), Some(item)),
				Nfts::check_approval_permission(&collection, &Some(item), &owner, &operator)
					.is_ok()
			);
		});
}

#[test]
fn total_supply_works() {
	let owner = ALICE;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			let collection = nfts::create_collection(owner.clone());
			assert_eq!(total_supply::<Test, ()>(collection), 0);
			(0..10).into_iter().for_each(|i| {
				assert_ok!(Nfts::mint(
					signed(owner.clone()),
					collection,
					i,
					owner.clone().into(),
					None
				));
				assert_eq!(total_supply::<Test, ()>(collection), i + 1);
				assert_eq!(
					total_supply::<Test, ()>(collection),
					Nfts::collection_items(collection).unwrap_or_default()
				);
			});
		});
}

mod get_attribute {
	use super::*;

	#[test]
	fn get_attribute_works() {
		let attribute = BoundedVec::truncate_from("some attribute".into());
		let collection = COLLECTION;
		let item = ITEM;
		let metadata = "some value".as_bytes().to_vec();
		let owner = ALICE;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), 10_000_000)])
			.build()
			.execute_with(|| {
				nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
				// No attribute set.
				assert_eq!(
					get_attribute::<Test, ()>(
						collection,
						Some(item),
						AttributeNamespace::CollectionOwner,
						attribute.clone()
					),
					None
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
					get_attribute::<Test, ()>(
						collection,
						Some(item),
						AttributeNamespace::CollectionOwner,
						attribute.clone()
					),
					Some(metadata)
				);
				assert_eq!(
					get_attribute::<Test, ()>(
						collection,
						Some(item),
						AttributeNamespace::CollectionOwner,
						attribute.clone()
					),
					nfts::get_attribute(
						collection,
						Some(item),
						AttributeNamespace::CollectionOwner,
						attribute
					)
				);
			});
	}

	#[test]
	fn get_collection_attribute_works() {
		let attribute = BoundedVec::truncate_from("some attribute".into());
		let collection = COLLECTION;
		let metadata = "some value".as_bytes().to_vec();
		let owner = ALICE;
		ExtBuilder::new()
			.with_balances(vec![(owner.clone(), 10_000_000)])
			.build()
			.execute_with(|| {
				nfts::create_collection(owner.clone());
				// No attribute set.
				assert_eq!(
					get_attribute::<Test, ()>(
						collection,
						None,
						AttributeNamespace::CollectionOwner,
						attribute.clone()
					),
					None
				);
				// Successfully get an existing attribute.
				assert_ok!(Nfts::set_attribute(
					signed(owner.clone()),
					collection,
					None,
					AttributeNamespace::CollectionOwner,
					attribute.clone(),
					BoundedVec::truncate_from(metadata.clone()),
				));
				assert_eq!(
					get_attribute::<Test, ()>(
						collection,
						None,
						AttributeNamespace::CollectionOwner,
						attribute.clone()
					),
					Some(metadata)
				);
				assert_eq!(
					get_attribute::<Test, ()>(
						collection,
						None,
						AttributeNamespace::CollectionOwner,
						attribute.clone()
					),
					nfts::get_attribute(
						collection,
						None,
						AttributeNamespace::CollectionOwner,
						attribute
					)
				);
			});
	}
}

#[test]
fn next_collection_id_works() {
	let owner = ALICE;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			assert_eq!(next_collection_id::<Test, ()>(), Some(0));
			nfts::create_collection_and_mint(ALICE, ALICE, ITEM);
			assert_eq!(next_collection_id::<Test, ()>(), Some(1));
			assert_eq!(
				next_collection_id::<Test, ()>(),
				Some(NextCollectionIdOf::get().unwrap_or_default())
			);
		});
}

#[test]
fn item_metadata_works() {
	let collection = COLLECTION;
	let item = ITEM;
	let metadata = "some metadata".as_bytes().to_vec();
	let owner = ALICE;
	ExtBuilder::new()
		.with_balances(vec![(owner.clone(), 10_000_000)])
		.build()
		.execute_with(|| {
			// Read item metadata of an unknown collection.
			assert_eq!(item_metadata::<Test, ()>(collection, item), None);
			nfts::create_collection_and_mint(owner.clone(), owner.clone(), item);
			// Successfully set the metadata of an item.
			assert_ok!(set_metadata::<Test, ()>(
				signed(owner),
				collection,
				item,
				BoundedVec::truncate_from(metadata.clone())
			));
			assert_eq!(item_metadata::<Test, ()>(collection, item), Some(metadata));
			assert_eq!(
				item_metadata::<Test, ()>(collection, item).map(|m| BoundedVec::truncate_from(m)),
				Nfts::item_metadata(collection, item)
			);
		});
}

mod nfts {
	use frame_support::assert_ok;

	use super::*;

	pub(super) fn balance_of(collection: CollectionIdOf<Test>, owner: &AccountId) -> u32 {
		AccountBalanceOf::get(collection, &owner)
			.map(|(balance, _)| balance)
			.unwrap_or_default()
	}

	pub(super) fn create_collection(owner: AccountId) -> u32 {
		let next_id = NextCollectionIdOf::get().unwrap_or_default();
		assert_ok!(Nfts::create(
			signed(owner.clone()),
			owner.into(),
			collection_config_with_all_settings_enabled()
		));
		next_id
	}

	pub(super) fn create_collection_and_mint(
		owner: AccountId,
		mint_to: AccountId,
		item: ItemIdOf<Test>,
	) -> (u32, u32) {
		let collection = create_collection(owner.clone());
		assert_ok!(Nfts::mint(signed(owner), collection, item, mint_to.into(), None));
		(collection, item)
	}

	pub(super) fn create_collection_mint_and_approve(
		owner: AccountId,
		mint_to: AccountId,
		item: ItemIdOf<Test>,
		operator: AccountId,
	) {
		let (collection, item) = create_collection_and_mint(owner.clone(), mint_to, item);
		assert_ok!(Nfts::approve_transfer(
			signed(owner.clone().into()),
			collection,
			item,
			operator.clone().into(),
			None
		));
		assert_ok!(Nfts::check_approval_permission(
			&collection,
			&Some(item),
			&owner.into(),
			&operator.into()
		));
	}

	pub(super) fn collection_config_with_all_settings_enabled() -> CollectionConfigFor<Test> {
		CollectionConfig {
			settings: CollectionSettings::all_enabled(),
			max_supply: None,
			mint_settings: MintSettings::default(),
		}
	}

	pub(super) fn get_attribute(
		collection: CollectionIdOf<Test>,
		maybe_item: Option<ItemIdOf<Test>>,
		namespace: AttributeNamespaceOf,
		key: BoundedVec<u8, <Test as pallet_nfts::Config<()>>::KeyLimit>,
	) -> Option<Vec<u8>> {
		AttributeOf::get((collection, maybe_item, namespace, key))
			.map(|attribute| attribute.0.into())
	}
}

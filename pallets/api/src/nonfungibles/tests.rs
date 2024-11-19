use codec::Encode;
use frame_support::{
	assert_noop, assert_ok,
	sp_runtime::{BoundedVec, DispatchError::BadOrigin},
};
use pallet_nfts::WeightInfo as NftsWeightInfoTrait;

use crate::{
	mock::*,
	nonfungibles::{
		AccountBalanceOf, AttributeNamespace, AttributeOf, BlockNumberFor,
		CancelAttributesApprovalWitness, CollectionConfig, CollectionIdOf, CollectionOf,
		CollectionSettings, DestroyWitness, ItemIdOf, MintSettings, MintWitness,
		NextCollectionIdOf, NftsInstanceOf, NftsWeightInfoOf, Read::*, ReadResult,
	},
	Read,
};

const COLLECTION: u32 = 0;
const ITEM: u32 = 1;

type NftsError = pallet_nfts::Error<Test, NftsInstanceOf<Test>>;
type Event = crate::nonfungibles::Event<Test>;

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
		let balance_before_transfer = AccountBalanceOf::<Test>::get(collection, &dest);
		assert_ok!(NonFungibles::transfer(signed(owner), collection, ITEM, dest));
		let balance_after_transfer = AccountBalanceOf::<Test>::get(collection, &dest);
		assert_eq!(AccountBalanceOf::<Test>::get(collection, &owner), 0);
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
		let balance_before_mint = AccountBalanceOf::<Test>::get(collection, owner);
		assert_ok!(NonFungibles::mint(
			signed(owner),
			owner,
			collection,
			ITEM,
			MintWitness { mint_price: None, owned_item: None }
		));
		let balance_after_mint = AccountBalanceOf::<Test>::get(collection, owner);
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

		// "UnknownItem" error is returned if collection item is not created.
		assert_noop!(NonFungibles::burn(signed(owner), COLLECTION, ITEM), NftsError::UnknownItem);
		// Successfully burn an existing new collection item.
		let (collection, item) = nfts::create_collection_mint(owner, ITEM);
		let balance_before_burn = AccountBalanceOf::<Test>::get(collection, owner);
		assert_ok!(NonFungibles::burn(signed(owner), collection, ITEM));
		let balance_after_burn = AccountBalanceOf::<Test>::get(collection, owner);
		assert_eq!(balance_after_burn, balance_before_burn - 1);
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
		// Successfully approve `operator` to transfer the collection item.
		assert_eq!(
			NonFungibles::approve(signed(owner), collection, Some(item), operator, true),
			Ok(Some(NftsWeightInfoOf::<Test>::approve_transfer()).into())
		);
		assert_ok!(Nfts::check_approval(&collection, &Some(item), &owner, &operator));
		System::assert_last_event(
			Event::Approval { collection, item: Some(item), owner, operator, approved: true }
				.into(),
		);
		// Successfully transfer the item by the delegated account `operator`.
		assert_ok!(Nfts::transfer(signed(operator), collection, item, operator));
	});
}

#[test]
fn approve_collection_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let operator = BOB;
		let (collection, item) = nfts::create_collection_mint(owner, ITEM);
		// Successfully approve `operator` to transfer all items within the collection.
		assert_eq!(
			NonFungibles::approve(signed(owner), collection, None, operator, true),
			Ok(Some(NftsWeightInfoOf::<Test>::approve_collection_transfer()).into())
		);
		assert_ok!(Nfts::check_approval(&collection, &None, &owner, &operator));
		System::assert_last_event(
			Event::Approval { collection, item: None, owner, operator, approved: true }.into(),
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
		assert_eq!(
			NonFungibles::approve(signed(owner), collection, Some(item), operator, false),
			Ok(Some(NftsWeightInfoOf::<Test>::cancel_approval()).into())
		);
		assert_eq!(
			Nfts::check_approval(&collection, &Some(item), &owner, &operator),
			Err(NftsError::NoPermission.into())
		);
		// Failed to transfer the item by `operator` without permission.
		assert_noop!(
			Nfts::transfer(signed(operator), collection, item, operator),
			NftsError::NoPermission
		);
	});
}

#[test]
fn cancel_collection_approval_works() {
	new_test_ext().execute_with(|| {
		let owner = ALICE;
		let operator = BOB;
		let (collection, item) = nfts::create_collection_mint(owner, ITEM);
		// Successfully cancel the transfer collection approval of `operator` by `owner`.
		assert_ok!(Nfts::approve_collection_transfer(signed(owner), collection, operator, None));
		assert_eq!(
			NonFungibles::approve(signed(owner), collection, None, operator, false),
			Ok(Some(NftsWeightInfoOf::<Test>::cancel_collection_approval()).into())
		);
		assert_eq!(
			Nfts::check_approval(&collection, &None, &owner, &operator),
			Err(NftsError::NoPermission.into())
		);
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
		let result: Option<
			BoundedVec<u8, <Test as pallet_nfts::Config<NftsInstanceOf<Test>>>::ValueLimit>,
		> = None;
		assert_ok!(Nfts::set_attribute(
			signed(ALICE),
			collection,
			Some(item),
			AttributeNamespace::CollectionOwner,
			attribute.clone(),
			BoundedVec::truncate_from("some value".as_bytes().to_vec())
		));
		// Successfully clear an attribute.
		assert_ok!(Nfts::clear_attribute(
			signed(ALICE),
			collection,
			Some(item),
			AttributeNamespace::CollectionOwner,
			attribute.clone(),
		));
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item,
				namespace: AttributeNamespace::CollectionOwner,
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
			AttributeNamespace::Account(BOB),
			attribute.clone(),
			value.clone()
		));
		let result: Option<
			BoundedVec<u8, <Test as pallet_nfts::Config<NftsInstanceOf<Test>>>::ValueLimit>,
		> = Some(value);
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item,
				namespace: AttributeNamespace::Account(BOB),
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
			AttributeNamespace::Account(BOB),
			attribute.clone(),
			value.clone()
		));
		assert_ok!(Nfts::cancel_item_attributes_approval(
			signed(ALICE),
			collection,
			item,
			BOB,
			CancelAttributesApprovalWitness { account_attributes: 1 }
		));
		assert_noop!(
			Nfts::set_attribute(
				signed(BOB),
				collection,
				Some(item),
				AttributeNamespace::Account(BOB),
				attribute.clone(),
				value.clone()
			),
			NftsError::NoPermission
		);
	});
}

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		let creator = ALICE;
		let admin = ALICE;
		let next_collection_id = NextCollectionIdOf::<Test>::get().unwrap_or_default();
		for origin in vec![root(), none()] {
			assert_noop!(
				NonFungibles::create(
					origin,
					admin,
					CollectionConfig {
						max_supply: None,
						mint_settings: MintSettings::default(),
						settings: CollectionSettings::all_enabled()
					},
				),
				BadOrigin
			);
		}
		assert_ok!(NonFungibles::create(
			signed(creator),
			admin,
			CollectionConfig {
				max_supply: None,
				mint_settings: MintSettings::default(),
				settings: CollectionSettings::all_enabled()
			},
		));
		assert_eq!(Nfts::collection_owner(next_collection_id), Some(creator));
		System::assert_last_event(Event::Created { id: next_collection_id, creator, admin }.into());
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
		assert_ok!(NonFungibles::destroy(signed(ALICE), collection, witness));
		assert_eq!(Nfts::collection_owner(collection), None);
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
				AccountBalanceOf::<Test>::get(collection, owner).encode()
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
			NonFungibles::read(Allowance { collection, item: Some(item), owner, operator }),
			ReadResult::Allowance(true)
		);
		assert_eq!(
			NonFungibles::read(Allowance { collection, item: Some(item), owner, operator })
				.encode(),
			Nfts::check_approval(&collection, &Some(item), &owner, &operator)
				.is_ok()
				.encode()
		);
	});
}

#[test]
fn owner_of_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			NonFungibles::read(OwnerOf { collection: COLLECTION, item: ITEM }),
			ReadResult::OwnerOf(None)
		);
		nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(
			NonFungibles::read(OwnerOf { collection: COLLECTION, item: ITEM }),
			ReadResult::OwnerOf(Some(ALICE))
		);
		assert_eq!(
			NonFungibles::read(OwnerOf { collection: COLLECTION, item: ITEM }).encode(),
			Nfts::owner(COLLECTION, ITEM).encode()
		);
	});
}

#[test]
fn get_attribute_works() {
	new_test_ext().execute_with(|| {
		let (collection, item) = nfts::create_collection_mint(ALICE, ITEM);
		let attribute = BoundedVec::truncate_from("some attribute".as_bytes().to_vec());
		let raw_value = "some value".as_bytes().to_vec();
		let value = BoundedVec::truncate_from(raw_value.clone());
		let namespace = AttributeNamespace::CollectionOwner;
		// No attribute set.
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item,
				namespace: namespace.clone(),
				key: attribute.clone()
			}),
			ReadResult::GetAttribute(None)
		);
		// Successfully get an existing attribute.
		assert_ok!(Nfts::set_attribute(
			signed(ALICE),
			collection,
			Some(item),
			namespace.clone(),
			attribute.clone(),
			value,
		));
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item,
				namespace: namespace.clone(),
				key: attribute.clone()
			}),
			ReadResult::GetAttribute(Some(raw_value))
		);
		assert_eq!(
			NonFungibles::read(GetAttribute {
				collection,
				item,
				namespace: namespace.clone(),
				key: attribute.clone()
			})
			.encode(),
			AttributeOf::<Test>::get((collection, Some(item), namespace, attribute))
				.map(|result| result.0)
				.encode()
		);
	});
}

#[test]
fn collection_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(NonFungibles::read(Collection(COLLECTION)), ReadResult::Collection(None),);
		nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(
			NonFungibles::read(Collection(COLLECTION)),
			ReadResult::Collection(CollectionOf::<Test>::get(COLLECTION)),
		);
		assert_eq!(
			NonFungibles::read(Collection(COLLECTION)).encode(),
			CollectionOf::<Test>::get(COLLECTION).encode(),
		);
	});
}

#[test]
fn item_metadata_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			NonFungibles::read(ItemMetadata { collection: COLLECTION, item: ITEM }),
			ReadResult::ItemMetadata(None)
		);
		nfts::create_collection_mint(ALICE, ITEM);
		let value = "some metadata".as_bytes().to_vec();
		assert_ok!(NonFungibles::set_metadata(
			signed(ALICE),
			COLLECTION,
			ITEM,
			BoundedVec::truncate_from(value.clone())
		));
		assert_eq!(
			NonFungibles::read(ItemMetadata { collection: COLLECTION, item: ITEM }),
			ReadResult::ItemMetadata(Some(value))
		);
		assert_eq!(
			NonFungibles::read(ItemMetadata { collection: COLLECTION, item: ITEM }).encode(),
			Nfts::item_metadata(COLLECTION, ITEM).encode()
		);
	});
}

#[test]
fn next_collection_id_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(NonFungibles::read(NextCollectionId), ReadResult::NextCollectionId(Some(0)));
		nfts::create_collection_mint(ALICE, ITEM);
		assert_eq!(NonFungibles::read(NextCollectionId), ReadResult::NextCollectionId(Some(1)));
		assert_eq!(
			NonFungibles::read(NextCollectionId).encode(),
			Some(NextCollectionIdOf::<Test>::get().unwrap_or_default()).encode(),
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

// Helper functions for interacting with pallet-nfts.
mod nfts {
	use super::*;

	pub(super) fn create_collection_mint_and_approve(
		owner: AccountId,
		item: ItemIdOf<Test>,
		operator: AccountId,
	) -> (u32, u32) {
		let (collection, item) = create_collection_mint(owner, item);
		assert_ok!(Nfts::approve_transfer(signed(owner), collection, item, operator, None));
		(collection, item)
	}

	pub(super) fn create_collection_mint(owner: AccountId, item: ItemIdOf<Test>) -> (u32, u32) {
		let collection = create_collection(owner);
		assert_ok!(Nfts::mint(signed(owner), collection, item, owner, None));
		(collection, item)
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

	pub(super) fn collection_config_with_all_settings_enabled(
	) -> CollectionConfig<Balance, BlockNumberFor<Test>, CollectionIdOf<Test>> {
		CollectionConfig {
			settings: CollectionSettings::all_enabled(),
			max_supply: None,
			mint_settings: MintSettings::default(),
		}
	}
}

mod read_weights {
	use frame_support::weights::Weight;

	use super::*;
	use crate::nonfungibles::{weights::WeightInfo, Config};

	struct ReadWeightInfo {
		total_supply: Weight,
		balance_of: Weight,
		allowance: Weight,
		owner_of: Weight,
		get_attribute: Weight,
		collection: Weight,
		next_collection_id: Weight,
		item_metadata: Weight,
	}

	impl ReadWeightInfo {
		fn new() -> Self {
			Self {
				total_supply: NonFungibles::weight(&TotalSupply(COLLECTION)),
				balance_of: NonFungibles::weight(&BalanceOf {
					collection: COLLECTION,
					owner: ALICE,
				}),
				allowance: NonFungibles::weight(&Allowance {
					collection: COLLECTION,
					item: Some(ITEM),
					owner: ALICE,
					operator: BOB,
				}),
				owner_of: NonFungibles::weight(&OwnerOf { collection: COLLECTION, item: ITEM }),
				get_attribute: NonFungibles::weight(&GetAttribute {
					collection: COLLECTION,
					item: ITEM,
					namespace: AttributeNamespace::CollectionOwner,
					key: BoundedVec::default(),
				}),
				collection: NonFungibles::weight(&Collection(COLLECTION)),
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
			allowance,
			balance_of,
			collection,
			get_attribute,
			item_metadata,
			next_collection_id,
			owner_of,
			total_supply,
		} = ReadWeightInfo::new();

		assert_eq!(total_supply, <Test as Config>::WeightInfo::total_supply());
		assert_eq!(balance_of, <Test as Config>::WeightInfo::balance_of());
		assert_eq!(allowance, <Test as Config>::WeightInfo::allowance());
		assert_eq!(owner_of, <Test as Config>::WeightInfo::owner_of());
		assert_eq!(get_attribute, <Test as Config>::WeightInfo::get_attribute());
		assert_eq!(collection, <Test as Config>::WeightInfo::collection());
		assert_eq!(next_collection_id, <Test as Config>::WeightInfo::next_collection_id());
		assert_eq!(item_metadata, <Test as Config>::WeightInfo::item_metadata());
	}

	// These types read from the `Collection` storage.
	#[test]
	fn ensure_collection_variants_match() {
		let ReadWeightInfo { total_supply, collection, .. } = ReadWeightInfo::new();

		assert_eq!(total_supply, collection);
	}

	// Proof size is based on `MaxEncodedLen`, not hardware.
	// This test ensures that the data structure sizes do not change with upgrades.
	#[test]
	fn ensure_expected_proof_size_does_not_change() {
		let ReadWeightInfo {
			allowance,
			balance_of,
			collection,
			get_attribute,
			item_metadata,
			next_collection_id,
			owner_of,
			total_supply,
		} = ReadWeightInfo::new();

		// These values come from `weights.rs`.
		assert_eq!(total_supply.proof_size(), 3557);
		assert_eq!(balance_of.proof_size(), 3529);
		assert_eq!(allowance.proof_size(), 4326);
		assert_eq!(owner_of.proof_size(), 4326);
		assert_eq!(get_attribute.proof_size(), 3944);
		assert_eq!(collection.proof_size(), 3557);
		assert_eq!(next_collection_id.proof_size(), 1489);
		assert_eq!(item_metadata.proof_size(), 3812);
	}
}

mod ensure_codec_indexes {
	use super::{Encode, *};
	use crate::{mock::RuntimeCall::NonFungibles, nonfungibles};

	#[test]
	fn ensure_read_variant_indexes() {
		[
			(TotalSupply::<Test>(Default::default()), 0u8, "TotalSupply"),
			(
				BalanceOf::<Test> { collection: Default::default(), owner: Default::default() },
				1,
				"BalanceOf",
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
			(
				OwnerOf::<Test> { collection: Default::default(), item: Default::default() },
				5,
				"OwnerOf",
			),
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
			(Collection::<Test>(Default::default()), 9, "Collection"),
			(NextCollectionId, 10, "NextCollectionId"),
			(
				ItemMetadata { collection: Default::default(), item: Default::default() },
				11,
				"ItemMetadata",
			),
		]
		.iter()
		.for_each(|(variant, expected_index, name)| {
			assert_eq!(variant.encode()[0], *expected_index, "{name} variant index changed");
		})
	}

	#[test]
	fn ensure_dispatchable_indexes() {
		use nonfungibles::Call::*;

		[
			(
				transfer {
					collection: Default::default(),
					item: Default::default(),
					to: Default::default(),
				},
				3u8,
				"transfer",
			),
			(
				approve {
					collection: Default::default(),
					item: Default::default(),
					operator: Default::default(),
					approved: Default::default(),
				},
				4,
				"approve",
			),
			(create { admin: Default::default(), config: Default::default() }, 7, "create"),
			(
				destroy {
					collection: Default::default(),
					witness: DestroyWitness {
						item_metadatas: Default::default(),
						item_configs: Default::default(),
						attributes: Default::default(),
					}
				},
				8,
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
				12,
				"set_attribute",
			),
			(
				clear_attribute {
					collection: Default::default(),
					item: Default::default(),
					namespace: AttributeNamespace::CollectionOwner,
					key: Default::default(),
				},
				13,
				"clear_attribute",
			),
			(
				set_metadata {
					collection: Default::default(),
					item: Default::default(),
					data: Default::default(),
				},
				14,
				"set_metadata",
			),
			(
				clear_metadata { collection: Default::default(), item: Default::default() },
				15,
				"clear_metadata",
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
				set_max_supply { collection: Default::default(), max_supply: Default::default() },
				18,
				"set_max_supply",
			),
			(
				mint {
					to: Default::default(),
					collection: Default::default(),
					item: Default::default(),
					witness: MintWitness {
						owned_item: Default::default(),
						mint_price: Default::default(),
					},
				},
				19,
				"mint",
			),
			(burn { collection: Default::default(), item: Default::default() }, 20, "burn"),
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
}

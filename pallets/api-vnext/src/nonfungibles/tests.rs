use frame_support::assert_noop;
use pallet_nfts::CollectionConfig;

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
type WeightInfo = <Test as Config>::WeightInfo;

mod approve {
	use frame_support::assert_ok;

	use super::*;

	#[test]
	fn approve_works() {
		let collection = COLLECTION;
		let item = ITEM;
		let operator = BOB;
		let owner = ALICE;
		ExtBuilder::new()
			.with_balances(vec![(ALICE, 10_000_000)])
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
}

// Helper functions for interacting with pallet-nfts.
mod nfts {
	use frame_support::assert_ok;
	use pallet_nfts::{CollectionSettings, MintSettings};

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

use super::*;

pub(super) fn balance_of(
	addr: &AccountId32,
	collection: CollectionId,
	owner: AccountId32,
) -> Result<u32, Error> {
	let params = [collection.encode(), owner.encode()].concat();
	let result = do_bare_call("balance_of", &addr, params);
	decoded::<Result<u32, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn owner_of(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
) -> Result<Option<AccountId32>, Error> {
	let params = [collection.encode(), item.encode()].concat();
	let result = do_bare_call("owner_of", &addr, params);
	decoded::<Result<Option<AccountId32>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn allowance(
	addr: &AccountId32,
	collection: CollectionId,
	owner: AccountId32,
	operator: AccountId32,
	item: Option<ItemId>,
) -> Result<bool, Error> {
	let params = [collection.encode(), owner.encode(), operator.encode(), item.encode()].concat();
	let result = do_bare_call("allowance", &addr, params);
	decoded::<Result<bool, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn approve(
	addr: &AccountId32,
	collection: CollectionId,
	operator: AccountId32,
	item: Option<ItemId>,
	approved: bool,
) -> Result<(), Error> {
	let params =
		[collection.encode(), operator.encode(), item.encode(), approved.encode()].concat();
	let result = do_bare_call("approve", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn transfer(
	addr: &AccountId32,
	collection: CollectionId,
	to: AccountId32,
	item: ItemId,
) -> Result<(), Error> {
	let params = [collection.encode(), to.encode(), item.encode()].concat();
	let result = do_bare_call("transfer", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn total_supply(addr: &AccountId32, collection: CollectionId) -> Result<u128, Error> {
	let result = do_bare_call("total_supply", addr, collection.encode());
	decoded::<Result<u128, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn get_attribute(
	addr: &AccountId32,
	collection: CollectionId,
	item: Option<ItemId>,
	namespace: AttributeNamespace,
	key: Vec<u8>,
) -> Result<Option<Vec<u8>>, Error> {
	let params = [
		collection.encode(),
		item.encode(),
		namespace.encode(),
		AttributeKey::truncate_from(key).encode(),
	]
	.concat();
	let result = do_bare_call("get_attribute", &addr, params);
	decoded::<Result<Option<AttributeValue>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
		.map(|value| value.map(|v| v.to_vec()))
}

pub(super) fn next_collection_id(addr: &AccountId32) -> Result<CollectionId, Error> {
	let result = do_bare_call("next_collection_id", &addr, vec![]);
	decoded::<Result<CollectionId, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn item_metadata(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
) -> Result<Option<Vec<u8>>, Error> {
	let params = [collection.encode(), item.encode()].concat();
	let result = do_bare_call("item_metadata", &addr, params);
	decoded::<Result<Option<Vec<u8>>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
		.map(|value| value.map(|v| v.to_vec()))
}

pub(super) fn create(
	addr: &AccountId32,
	admin: AccountId32,
	config: CollectionConfig,
) -> Result<(), Error> {
	let params = [admin.encode(), config.encode()].concat();
	let result = do_bare_call("create", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn destroy(
	addr: &AccountId32,
	collection: CollectionId,
	witness: DestroyWitness,
) -> Result<(), Error> {
	let params = [collection.encode(), witness.encode()].concat();
	let result = do_bare_call("destroy", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn set_attribute(
	addr: &AccountId32,
	collection: CollectionId,
	item: Option<ItemId>,
	namespace: AttributeNamespace,
	key: Vec<u8>,
	value: Vec<u8>,
) -> Result<(), Error> {
	let params = [
		collection.encode(),
		item.encode(),
		namespace.encode(),
		AttributeKey::truncate_from(key).encode(),
		AttributeValue::truncate_from(value).encode(),
	]
	.concat();
	let result = do_bare_call("set_attribute", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn clear_attribute(
	addr: &AccountId32,
	collection: CollectionId,
	item: Option<ItemId>,
	namespace: AttributeNamespace,
	key: Vec<u8>,
) -> Result<(), Error> {
	let params = [collection.encode(), item.encode(), namespace.encode(), key.encode()].concat();
	let result = do_bare_call("clear_attribute", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn set_metadata(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
	data: Vec<u8>,
) -> Result<(), Error> {
	let params = [collection.encode(), item.encode(), data.encode()].concat();
	let result = do_bare_call("set_metadata", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn clear_metadata(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
) -> Result<(), Error> {
	let params = [collection.encode(), item.encode()].concat();
	let result = do_bare_call("clear_metadata", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn set_max_supply(
	addr: &AccountId32,
	collection: CollectionId,
	max_supply: u32,
) -> Result<(), Error> {
	let params = [collection.encode(), max_supply.encode()].concat();
	let result = do_bare_call("set_max_supply", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn approve_item_attributes(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
	delegate: AccountId32,
) -> Result<(), Error> {
	let params = [collection.encode(), item.encode(), delegate.encode()].concat();
	let result = do_bare_call("approve_item_attributes", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn cancel_item_attributes_approval(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
	delegate: AccountId32,
	witness: CancelAttributesApprovalWitness,
) -> Result<(), Error> {
	let params = [collection.encode(), item.encode(), delegate.encode(), witness.encode()].concat();
	let result = do_bare_call("cancel_item_attributes_approval", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn clear_all_transfer_approvals(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
) -> Result<(), Error> {
	let params = [collection.encode(), item.encode()].concat();
	let result = do_bare_call("clear_all_transfer_approvals", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn clear_collection_approvals(
	addr: &AccountId32,
	collection: CollectionId,
	limit: u32,
) -> Result<(), Error> {
	let params = [collection.encode(), limit.encode()].concat();
	let result = do_bare_call("clear_collection_approvals", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn mint(
	addr: &AccountId32,
	collection: CollectionId,
	to: AccountId32,
	item: ItemId,
	witness: Option<MintWitness>,
) -> Result<(), Error> {
	let params = [collection.encode(), to.encode(), item.encode(), witness.encode()].concat();
	let result = do_bare_call("mint", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn burn(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
) -> Result<(), Error> {
	let params = [collection.encode(), item.encode()].concat();
	let result = do_bare_call("burn", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) mod nfts {
	use super::*;

	pub(crate) fn balance_of(collection: CollectionId, owner: AccountId32) -> u32 {
		AccountBalance::get(collection, owner)
			.map(|(balance, _)| balance)
			.unwrap_or_default()
	}

	pub(crate) fn burn(collection: CollectionId, item: ItemId, owner: &AccountId32) {
		assert_ok!(Nfts::burn(RuntimeOrigin::signed(owner.clone()), collection, item));
	}

	pub(super) fn collection_config_with_all_settings_enabled(
	) -> pallet_nfts::CollectionConfig<u128, BlockNumber, CollectionId> {
		pallet_nfts::CollectionConfig {
			settings: pallet_nfts::CollectionSettings::all_enabled(),
			max_supply: Some(u32::MAX),
			mint_settings: pallet_nfts::MintSettings::default(),
		}
	}

	pub(crate) fn create_collection_and_mint_to(
		owner: &AccountId32,
		admin: &AccountId32,
		to: &AccountId32,
		item: ItemId,
	) -> (CollectionId, ItemId) {
		let collection = create_collection(owner, admin);
		mint(owner, collection, item, to);
		(collection, item)
	}

	pub(crate) fn create_collection_mint_and_approve(
		owner: &AccountId32,
		admin: &AccountId32,
		item: ItemId,
		to: &AccountId32,
		operator: &AccountId32,
	) -> (CollectionId, ItemId) {
		let (collection, item) = create_collection_and_mint_to(&owner, admin, to, item);
		assert_ok!(Nfts::approve_transfer(
			RuntimeOrigin::signed(to.clone()),
			collection,
			item,
			operator.clone().into(),
			Some(BlockNumber::MAX)
		));
		(collection, item)
	}

	pub(crate) fn create_collection(owner: &AccountId32, admin: &AccountId32) -> CollectionId {
		let next_id = NextCollectionId::get().unwrap_or_default();
		assert_ok!(Nfts::create(
			RuntimeOrigin::signed(owner.clone()),
			admin.clone().into(),
			collection_config_with_all_settings_enabled()
		));
		next_id
	}

	pub(crate) fn max_supply(collection: CollectionId) -> Option<u32> {
		pallet_nfts::CollectionConfigOf::<Runtime, NftsInstance>::get(collection)
			.map(|config| config.max_supply)
			.unwrap_or_default()
	}

	pub(crate) fn mint(
		owner: &AccountId32,
		collection: CollectionId,
		item: ItemId,
		to: &AccountId32,
	) -> ItemId {
		assert_ok!(Nfts::mint(
			RuntimeOrigin::signed(owner.clone()),
			collection,
			item,
			to.clone().into(),
			None
		));
		item
	}

	pub(crate) fn default_mint_settings() -> MintSettings {
		MintSettings {
			mint_type: MintType::Issuer,
			price: Some(Balance::MAX),
			start_block: Some(BlockNumber::MIN),
			end_block: Some(BlockNumber::MAX),
			default_item_settings: ItemSettings::all_enabled(),
		}
	}

	pub(crate) fn default_collection_config() -> CollectionConfig {
		CollectionConfig {
			max_supply: Some(u32::MAX),
			mint_settings: default_mint_settings(),
			settings: CollectionSettings::all_enabled(),
		}
	}
}

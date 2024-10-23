use super::*;

fn do_bare_call(function: &str, addr: &AccountId32, params: Vec<u8>) -> ExecReturnValue {
	let function = function_selector(function);
	let params = [function, params].concat();
	bare_call(addr.clone(), params, 0).expect("should work")
}

// TODO - issue #263 - why result.data[1..]
pub(super) fn decoded<T: Decode>(result: ExecReturnValue) -> Result<T, ExecReturnValue> {
	<T>::decode(&mut &result.data[1..]).map_err(|_| result)
}

pub(super) fn total_supply(addr: &AccountId32, collection: CollectionId) -> Result<u128, Error> {
	let result = do_bare_call("total_supply", addr, collection.encode());
	decoded::<Result<u128, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

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

pub(super) fn transfer(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
	to: AccountId32,
) -> Result<(), Error> {
	let params = [collection.encode(), item.encode(), to.encode()].concat();
	let result = do_bare_call("transfer", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn approve(
	addr: &AccountId32,
	collection: CollectionId,
	item: Option<ItemId>,
	operator: AccountId32,
	approved: bool,
) -> Result<(), Error> {
	let params =
		[collection.encode(), item.encode(), operator.encode(), approved.encode()].concat();
	let result = do_bare_call("approve", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn owner_of(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
) -> Result<AccountId32, Error> {
	let params = [collection.encode(), item.encode()].concat();
	let result = do_bare_call("owner_of", &addr, params);
	decoded::<Result<AccountId32, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn get_attribute(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
	namespace: AttributeNamespace,
	key: Vec<u8>,
) -> Result<Vec<u8>, Error> {
	let params = [collection.encode(), item.encode(), namespace.encode(), key.encode()].concat();
	let result = do_bare_call("get_attribute", &addr, params);
	decoded::<Result<Vec<u8>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn create(
	addr: &AccountId32,
	admin: AccountId32,
	config: CreateCollectionConfig,
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

pub(super) fn collection(
	addr: &AccountId32,
	collection: CollectionId,
) -> Result<Option<CollectionDetails>, Error> {
	let result = do_bare_call("collection", &addr, collection.encode());
	decoded::<Result<Option<CollectionDetails>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn set_attribute(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
	namespace: AttributeNamespace,
	key: Vec<u8>,
	value: Vec<u8>,
) -> Result<(), Error> {
	let params =
		[collection.encode(), item.encode(), namespace.encode(), key.encode(), value.encode()]
			.concat();
	let result = do_bare_call("set_attribute", &addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn clear_attribute(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
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

pub(super) fn item_metadata(
	addr: &AccountId32,
	collection: CollectionId,
	item: ItemId,
) -> Result<Option<Vec<u8>>, Error> {
	let params = [collection.encode(), item.encode()].concat();
	let result = do_bare_call("item_metadata", &addr, params);
	decoded::<Result<Option<Vec<u8>>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) mod nfts {
	use super::*;

	pub(crate) fn create_collection_and_mint_to(
		owner: &AccountId32,
		admin: &AccountId32,
		to: &AccountId32,
		item: ItemId,
	) -> (CollectionId, ItemId) {
		let collection = create_collection(owner, admin);
		mint(collection, item, owner, to);
		(collection, item)
	}

	pub(crate) fn create_collection_mint_and_approve(
		owner: &AccountId32,
		admin: &AccountId32,
		item: ItemId,
		to: &AccountId32,
		operator: &AccountId32,
	) -> (u32, u32) {
		let (collection, item) = create_collection_and_mint_to(&owner.clone(), admin, to, item);
		assert_ok!(Nfts::approve_transfer(
			RuntimeOrigin::signed(to.clone()),
			collection,
			Some(item),
			operator.clone().into(),
			None
		));
		(collection, item)
	}

	pub(crate) fn create_collection(owner: &AccountId32, admin: &AccountId32) -> CollectionId {
		let next_id = next_collection_id();
		assert_ok!(Nfts::create(
			RuntimeOrigin::signed(owner.clone()),
			owner.clone().into(),
			collection_config_with_all_settings_enabled()
		));
		next_id
	}

	pub(super) fn next_collection_id() -> u32 {
		pallet_nfts::NextCollectionId::<Runtime>::get().unwrap_or_default()
	}

	pub(crate) fn mint(
		collection: CollectionId,
		item: ItemId,
		owner: &AccountId32,
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

	pub(crate) fn balance_of(collection: CollectionId, owner: AccountId32) -> u32 {
		pallet_nfts::AccountBalance::<Runtime>::get(collection, owner)
	}

	pub(super) fn collection_config_with_all_settings_enabled(
	) -> CollectionConfig<u128, BlockNumber, CollectionId> {
		CollectionConfig {
			settings: pallet_nfts::CollectionSettings::all_enabled(),
			max_supply: None,
			mint_settings: pallet_nfts::MintSettings::default(),
		}
	}
}

pub(super) fn instantiate_and_create_nonfungible(
	contract: &str,
	admin: AccountId32,
	config: CreateCollectionConfig,
) -> Result<AccountId32, Error> {
	let function = function_selector("new");
	let input = [function, admin.encode(), config.encode()].concat();
	let wasm_binary = std::fs::read(contract).expect("could not read .wasm file");
	let result = Contracts::bare_instantiate(
		ALICE,
		INIT_VALUE,
		GAS_LIMIT,
		None,
		Code::Upload(wasm_binary),
		input,
		vec![],
		DEBUG_OUTPUT,
		CollectEvents::Skip,
	)
	.result
	.expect("should work");
	let address = result.account_id;
	let result = result.result;
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
		.map(|_| address)
}

/// Get the last event from pallet contracts.
pub(super) fn last_contract_event() -> Vec<u8> {
	let events = System::read_events_for_pallet::<pallet_contracts::Event<Runtime>>();
	let contract_events = events
		.iter()
		.filter_map(|event| match event {
			pallet_contracts::Event::<Runtime>::ContractEmitted { data, .. } =>
				Some(data.as_slice()),
			_ => None,
		})
		.collect::<Vec<&[u8]>>();
	contract_events.last().unwrap().to_vec()
}

/// Decodes a byte slice into an `AccountId` as defined in `primitives`.
///
/// This is used to resolve type mismatches between the `AccountId` in the integration tests and the
/// contract environment.
pub fn account_id_from_slice(s: &[u8; 32]) -> pop_api::primitives::AccountId {
	pop_api::primitives::AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
}

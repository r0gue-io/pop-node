use super::*;

fn do_bare_call(function: &str, addr: AccountId32, params: Vec<u8>) -> ExecReturnValue {
	let function = function_selector(function);
	let params = [function, params].concat();
	bare_call(addr, params, 0).expect("should work")
}

pub(super) fn decoded<T: Decode>(result: ExecReturnValue) -> Result<T, ExecReturnValue> {
	<T>::decode(&mut &result.data[1..]).map_err(|_| result)
}

pub(super) fn total_supply(addr: AccountId32, asset_id: AssetId) -> Result<Balance, Error> {
	let result = do_bare_call("total_supply", addr, asset_id.encode());
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn balance_of(
	addr: AccountId32,
	asset_id: AssetId,
	owner: AccountId32,
) -> Result<Balance, Error> {
	let params = [asset_id.encode(), owner.encode()].concat();
	let result = do_bare_call("balance_of", addr, params);
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn allowance(
	addr: AccountId32,
	asset_id: AssetId,
	owner: AccountId32,
	spender: AccountId32,
) -> Result<Balance, Error> {
	let params = [asset_id.encode(), owner.encode(), spender.encode()].concat();
	let result = do_bare_call("allowance", addr, params);
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_name(addr: AccountId32, asset_id: AssetId) -> Result<Vec<u8>, Error> {
	let result = do_bare_call("token_name", addr, asset_id.encode());
	decoded::<Result<Vec<u8>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_symbol(addr: AccountId32, asset_id: AssetId) -> Result<Vec<u8>, Error> {
	let result = do_bare_call("token_symbol", addr, asset_id.encode());
	decoded::<Result<Vec<u8>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_decimals(addr: AccountId32, asset_id: AssetId) -> Result<u8, Error> {
	let result = do_bare_call("token_decimals", addr, asset_id.encode());
	decoded::<Result<u8, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_exists(addr: AccountId32, asset_id: AssetId) -> Result<bool, Error> {
	let result = do_bare_call("token_exists", addr, asset_id.encode());
	decoded::<Result<bool, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn transfer(
	addr: AccountId32,
	asset_id: AssetId,
	to: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let params = [asset_id.encode(), to.encode(), value.encode()].concat();
	let result = do_bare_call("transfer", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn transfer_from(
	addr: AccountId32,
	asset_id: AssetId,
	from: AccountId32,
	to: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let data: Vec<u8> = vec![];
	let params =
		[asset_id.encode(), from.encode(), to.encode(), value.encode(), data.encode()].concat();
	let result = do_bare_call("transfer_from", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn approve(
	addr: AccountId32,
	asset_id: AssetId,
	spender: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let params = [asset_id.encode(), spender.encode(), value.encode()].concat();
	let result = do_bare_call("approve", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn increase_allowance(
	addr: AccountId32,
	asset_id: AssetId,
	spender: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let params = [asset_id.encode(), spender.encode(), value.encode()].concat();
	let result = do_bare_call("increase_allowance", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn decrease_allowance(
	addr: AccountId32,
	asset_id: AssetId,
	spender: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let params = [asset_id.encode(), spender.encode(), value.encode()].concat();
	let result = do_bare_call("decrease_allowance", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn create(
	addr: AccountId32,
	asset_id: AssetId,
	admin: AccountId32,
	min_balance: Balance,
) -> Result<(), Error> {
	let params = [asset_id.encode(), admin.encode(), min_balance.encode()].concat();
	let result = do_bare_call("create", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn start_destroy(addr: AccountId32, asset_id: AssetId) -> Result<(), Error> {
	let result = do_bare_call("start_destroy", addr, asset_id.encode());
	match decoded::<Result<(), Error>>(result) {
		Ok(x) => x,
		Err(result) => panic!("Contract reverted: {:?}", result),
	}
}

pub(super) fn set_metadata(
	addr: AccountId32,
	asset_id: AssetId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) -> Result<(), Error> {
	let params = [asset_id.encode(), name.encode(), symbol.encode(), decimals.encode()].concat();
	let result = do_bare_call("set_metadata", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn clear_metadata(addr: AccountId32, asset_id: AssetId) -> Result<(), Error> {
	let result = do_bare_call("clear_metadata", addr, asset_id.encode());
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn mint(
	addr: AccountId32,
	asset_id: AssetId,
	account: AccountId32,
	amount: Balance,
) -> Result<(), Error> {
	let params = [asset_id.encode(), account.encode(), amount.encode()].concat();
	let result = do_bare_call("mint", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn burn(
	addr: AccountId32,
	asset_id: AssetId,
	account: AccountId32,
	amount: Balance,
) -> Result<(), Error> {
	let params = [asset_id.encode(), account.encode(), amount.encode()].concat();
	let result = do_bare_call("burn", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn create_asset(owner: AccountId32, asset_id: AssetId, min_balance: Balance) -> AssetId {
	assert_ok!(Assets::create(
		RuntimeOrigin::signed(owner.clone()),
		asset_id.into(),
		owner.into(),
		min_balance
	));
	asset_id
}

pub(super) fn mint_asset(
	owner: AccountId32,
	asset_id: AssetId,
	to: AccountId32,
	value: Balance,
) -> AssetId {
	assert_ok!(Assets::mint(
		RuntimeOrigin::signed(owner.clone()),
		asset_id.into(),
		to.into(),
		value
	));
	asset_id
}

pub(super) fn create_asset_and_mint_to(
	owner: AccountId32,
	asset_id: AssetId,
	to: AccountId32,
	value: Balance,
) -> AssetId {
	create_asset(owner.clone(), asset_id, 1);
	mint_asset(owner, asset_id, to, value)
}

// Create an asset, mints to, and approves spender.
pub(super) fn create_asset_mint_and_approve(
	owner: AccountId32,
	asset_id: AssetId,
	to: AccountId32,
	mint: Balance,
	spender: AccountId32,
	approve: Balance,
) -> AssetId {
	create_asset_and_mint_to(owner.clone(), asset_id, to.clone(), mint);
	assert_ok!(Assets::approve_transfer(
		RuntimeOrigin::signed(to.into()),
		asset_id.into(),
		spender.into(),
		approve,
	));
	asset_id
}

// Freeze an asset.
pub(super) fn freeze_asset(owner: AccountId32, asset_id: AssetId) {
	assert_ok!(Assets::freeze_asset(RuntimeOrigin::signed(owner.into()), asset_id.into()));
}

// Thaw an asset.
pub(super) fn thaw_asset(owner: AccountId32, asset_id: AssetId) {
	assert_ok!(Assets::thaw_asset(RuntimeOrigin::signed(owner.into()), asset_id.into()));
}

// Start destroying an asset.
pub(super) fn start_destroy_asset(owner: AccountId32, asset_id: AssetId) {
	assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(owner.into()), asset_id.into()));
}

// Create an asset and set metadata.
pub(super) fn create_asset_and_set_metadata(
	owner: AccountId32,
	asset_id: AssetId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) -> AssetId {
	assert_ok!(Assets::create(
		RuntimeOrigin::signed(owner.clone()),
		asset_id.into(),
		owner.clone().into(),
		100
	));
	set_metadata_asset(owner, asset_id, name, symbol, decimals);
	asset_id
}

// Set metadata of an asset.
pub(super) fn set_metadata_asset(
	owner: AccountId32,
	asset_id: AssetId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) {
	assert_ok!(Assets::set_metadata(
		RuntimeOrigin::signed(owner.into()),
		asset_id.into(),
		name,
		symbol,
		decimals
	));
}

pub(super) fn token_name_asset(asset_id: AssetId) -> Vec<u8> {
	<pallet_assets::Pallet<Runtime, TrustBackedAssetsInstance> as MetadataInspect<AccountId32>>::name(
        asset_id,
    )
}

pub(super) fn token_symbol_asset(asset_id: AssetId) -> Vec<u8> {
	<pallet_assets::Pallet<Runtime, TrustBackedAssetsInstance> as MetadataInspect<AccountId32>>::symbol(
        asset_id,
    )
}

pub(super) fn token_decimals_asset(asset_id: AssetId) -> u8 {
	<pallet_assets::Pallet<Runtime, TrustBackedAssetsInstance> as MetadataInspect<AccountId32>>::decimals(
        asset_id,
    )
}

pub(super) fn instantiate_and_create_fungible(
	contract: &str,
	asset_id: AssetId,
	min_balance: Balance,
) -> Result<AccountId32, Error> {
	let function = function_selector("new");
	let input = [function, asset_id.encode(), min_balance.encode()].concat();
	let (wasm_binary, _) =
		load_wasm_module::<Runtime>(contract).expect("could not read .wasm file");
	let result = Contracts::bare_instantiate(
		ALICE,
		INIT_VALUE,
		GAS_LIMIT,
		None,
		Code::Upload(wasm_binary),
		input,
		vec![],
		DEBUG_OUTPUT,
		CollectEvents::UnsafeCollect,
	)
	.result
	.expect("should work");
	let account_id = result.clone().account_id;
	let result = decoded::<Result<(), Error>>(result.clone().result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result));
	match result {
		Ok(_) => Ok(account_id),
		Err(error) => Err(error),
	}
}

/// Get the latest event from pallet contracts.
pub(super) fn latest_contract_event() -> Vec<u8> {
	let events = System::read_events_for_pallet::<pallet_contracts::Event<Runtime>>();
	let contract_events = events
		.iter()
		.filter_map(|event| match event {
			pallet_contracts::Event::<Runtime>::ContractEmitted { data, .. } => {
				Some(data.as_slice())
			},
			_ => None,
		})
		.collect::<Vec<&[u8]>>();
	contract_events.last().unwrap().to_vec()
}

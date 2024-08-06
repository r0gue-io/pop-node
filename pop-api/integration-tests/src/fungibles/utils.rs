use super::*;

pub(super) fn decoded<T: Decode>(result: ExecReturnValue) -> Result<T, ExecReturnValue> {
	<T>::decode(&mut &result.data[1..]).map_err(|_| result)
}

pub(super) fn total_supply(addr: AccountId32, asset_id: AssetId) -> Result<Balance, Error> {
	let function = function_selector("total_supply");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn balance_of(
	addr: AccountId32,
	asset_id: AssetId,
	owner: AccountId32,
) -> Result<Balance, Error> {
	let function = function_selector("balance_of");
	let params = [function, asset_id.encode(), owner.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn allowance(
	addr: AccountId32,
	asset_id: AssetId,
	owner: AccountId32,
	spender: AccountId32,
) -> Result<Balance, Error> {
	let function = function_selector("allowance");
	let params = [function, asset_id.encode(), owner.encode(), spender.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_name(addr: AccountId32, asset_id: AssetId) -> Result<Vec<u8>, Error> {
	let function = function_selector("token_name");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<Vec<u8>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_symbol(addr: AccountId32, asset_id: AssetId) -> Result<Vec<u8>, Error> {
	let function = function_selector("token_symbol");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<Vec<u8>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_decimals(addr: AccountId32, asset_id: AssetId) -> Result<u8, Error> {
	let function = function_selector("token_decimals");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<u8, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn asset_exists(addr: AccountId32, asset_id: AssetId) -> Result<bool, Error> {
	let function = function_selector("asset_exists");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<bool, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn transfer(
	addr: AccountId32,
	asset_id: AssetId,
	to: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let function = function_selector("transfer");
	let params = [function, asset_id.encode(), to.encode(), value.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
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
	let function = function_selector("transfer_from");
	let data: Vec<u8> = vec![];
	let params =
		[function, asset_id.encode(), from.encode(), to.encode(), value.encode(), data.encode()]
			.concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn approve(
	addr: AccountId32,
	asset_id: AssetId,
	spender: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let function = function_selector("approve");
	let params = [function, asset_id.encode(), spender.encode(), value.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn increase_allowance(
	addr: AccountId32,
	asset_id: AssetId,
	spender: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let function = function_selector("increase_allowance");
	let params = [function, asset_id.encode(), spender.encode(), value.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn decrease_allowance(
	addr: AccountId32,
	asset_id: AssetId,
	spender: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let function = function_selector("decrease_allowance");
	let params = [function, asset_id.encode(), spender.encode(), value.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn create(
	addr: AccountId32,
	asset_id: AssetId,
	admin: AccountId32,
	min_balance: Balance,
) -> Result<(), Error> {
	let function = function_selector("create");
	let params = [function, asset_id.encode(), admin.encode(), min_balance.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn start_destroy(addr: AccountId32, asset_id: AssetId) -> Result<(), Error> {
	let function = function_selector("start_destroy");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
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
	let function = function_selector("set_metadata");
	let params =
		[function, asset_id.encode(), name.encode(), symbol.encode(), decimals.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn clear_metadata(addr: AccountId32, asset_id: AssetId) -> Result<(), Error> {
	let function = function_selector("clear_metadata");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn mint(
	addr: AccountId32,
	asset_id: AssetId,
	account: AccountId32,
	amount: Balance,
) -> Result<(), Error> {
	let function = function_selector("mint");
	let params = [function, asset_id.encode(), account.encode(), amount.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn burn(
	addr: AccountId32,
	asset_id: AssetId,
	account: AccountId32,
	amount: Balance,
) -> Result<(), Error> {
	let function = function_selector("burn");
	let params = [function, asset_id.encode(), account.encode(), amount.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
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
) -> Result<(), Error> {
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
		CollectEvents::Skip,
	)
	.result
	.expect("should work")
	.result;
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

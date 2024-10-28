use super::*;
use crate::sp_core::H160;
use pallet_revive::AccountId32Mapper;
use pallet_revive::AddressMapper;

fn do_bare_call(function: &str, addr: &H160, params: Vec<u8>) -> ExecReturnValue {
	let function = function_selector(function);
	let params = [function, params].concat();
	bare_call(addr.clone(), params, 0).expect("should work")
}

// TODO - issue #263 - why result.data[1..]
pub(super) fn decoded<T: Decode>(result: ExecReturnValue) -> Result<T, ExecReturnValue> {
	<T>::decode(&mut &result.data[1..]).map_err(|_| result)
}

pub(super) fn total_supply(addr: &H160, token_id: TokenId) -> Result<Balance, Error> {
	let result = do_bare_call("total_supply", addr, token_id.encode());
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn balance_of(
	addr: &H160,
	token_id: TokenId,
	owner: AccountId32,
) -> Result<Balance, Error> {
	let params = [token_id.encode(), owner.encode()].concat();
	let result = do_bare_call("balance_of", &addr, params);
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn allowance(
	addr: &H160,
	token_id: TokenId,
	owner: AccountId32,
	spender: AccountId32,
) -> Result<Balance, Error> {
	let params = [token_id.encode(), owner.encode(), spender.encode()].concat();
	let result = do_bare_call("allowance", &addr, params);
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_name(addr: &H160, token_id: TokenId) -> Result<Option<Vec<u8>>, Error> {
	let result = do_bare_call("token_name", addr, token_id.encode());
	decoded::<Result<Option<Vec<u8>>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_symbol(addr: &H160, token_id: TokenId) -> Result<Option<Vec<u8>>, Error> {
	let result = do_bare_call("token_symbol", addr, token_id.encode());
	decoded::<Result<Option<Vec<u8>>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_decimals(addr: &H160, token_id: TokenId) -> Result<u8, Error> {
	let result = do_bare_call("token_decimals", addr, token_id.encode());
	decoded::<Result<u8, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn token_exists(addr: &H160, token_id: TokenId) -> Result<bool, Error> {
	let result = do_bare_call("token_exists", addr, token_id.encode());
	decoded::<Result<bool, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn transfer(
	addr: &H160,
	token_id: TokenId,
	to: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let params = [token_id.encode(), to.encode(), value.encode()].concat();
	let result = do_bare_call("transfer", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn transfer_from(
	addr: &H160,
	token_id: TokenId,
	from: AccountId32,
	to: AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let data: Vec<u8> = vec![];
	let params =
		[token_id.encode(), from.encode(), to.encode(), value.encode(), data.encode()].concat();
	let result = do_bare_call("transfer_from", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn approve(
	addr: &H160,
	token_id: TokenId,
	spender: &AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let params = [token_id.encode(), spender.encode(), value.encode()].concat();
	let result = do_bare_call("approve", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn increase_allowance(
	addr: &H160,
	token_id: TokenId,
	spender: &AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let params = [token_id.encode(), spender.encode(), value.encode()].concat();
	let result = do_bare_call("increase_allowance", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn decrease_allowance(
	addr: &H160,
	token_id: TokenId,
	spender: &AccountId32,
	value: Balance,
) -> Result<(), Error> {
	let params = [token_id.encode(), spender.encode(), value.encode()].concat();
	let result = do_bare_call("decrease_allowance", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn create(
	addr: &H160,
	token_id: TokenId,
	admin: &AccountId32,
	min_balance: Balance,
) -> Result<(), Error> {
	let params = [token_id.encode(), admin.encode(), min_balance.encode()].concat();
	let result = do_bare_call("create", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn start_destroy(addr: &H160, token_id: TokenId) -> Result<(), Error> {
	let result = do_bare_call("start_destroy", addr, token_id.encode());
	match decoded::<Result<(), Error>>(result) {
		Ok(x) => x,
		Err(result) => panic!("Contract reverted: {:?}", result),
	}
}

pub(super) fn set_metadata(
	addr: &H160,
	token_id: TokenId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) -> Result<(), Error> {
	let params = [token_id.encode(), name.encode(), symbol.encode(), decimals.encode()].concat();
	let result = do_bare_call("set_metadata", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn clear_metadata(addr: &H160, token_id: TokenId) -> Result<(), Error> {
	let result = do_bare_call("clear_metadata", addr, token_id.encode());
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn mint(
	addr: &H160,
	token_id: TokenId,
	account: &AccountId32,
	amount: Balance,
) -> Result<(), Error> {
	let params = [token_id.encode(), account.encode(), amount.encode()].concat();
	let result = do_bare_call("mint", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

pub(super) fn burn(
	addr: &H160,
	token_id: TokenId,
	account: &AccountId32,
	amount: Balance,
) -> Result<(), Error> {
	let params = [token_id.encode(), account.encode(), amount.encode()].concat();
	let result = do_bare_call("burn", addr, params);
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

// Helper functions for interacting with pallet-assets.
pub(super) mod assets {
	use super::*;

	type AssetId = TokenId;

	pub(crate) fn create(owner: &AccountId32, asset_id: AssetId, min_balance: Balance) -> AssetId {
		assert_ok!(Assets::create(
			RuntimeOrigin::signed(owner.clone()),
			asset_id.into(),
			owner.clone().into(),
			min_balance
		));
		asset_id
	}

	pub(crate) fn mint(
		owner: &AccountId32,
		asset_id: AssetId,
		to: &AccountId32,
		value: Balance,
	) -> AssetId {
		assert_ok!(Assets::mint(
			RuntimeOrigin::signed(owner.clone()),
			asset_id.into(),
			to.clone().into(),
			value
		));
		asset_id
	}

	pub(crate) fn create_and_mint_to(
		owner: &AccountId32,
		asset_id: AssetId,
		to: &AccountId32,
		value: Balance,
	) -> AssetId {
		create(owner, asset_id, 1);
		mint(owner, asset_id, to, value)
	}

	// Create an asset, mints to, and approves spender.
	pub(crate) fn create_mint_and_approve(
		owner: &AccountId32,
		asset_id: AssetId,
		to: &AccountId32,
		mint: Balance,
		spender: &AccountId32,
		approve: Balance,
	) -> AssetId {
		create_and_mint_to(owner, asset_id, to, mint);
		assert_ok!(Assets::approve_transfer(
			RuntimeOrigin::signed(to.clone().into()),
			asset_id.into(),
			spender.clone().into(),
			approve,
		));
		asset_id
	}

	// Freeze an asset.
	pub(crate) fn freeze(owner: &AccountId32, asset_id: AssetId) {
		assert_ok!(Assets::freeze_asset(
			RuntimeOrigin::signed(owner.clone().into()),
			asset_id.into()
		));
	}

	// Thaw an asset.
	pub(crate) fn thaw(owner: &AccountId32, asset_id: AssetId) {
		assert_ok!(Assets::thaw_asset(
			RuntimeOrigin::signed(owner.clone().into()),
			asset_id.into()
		));
	}

	// Start destroying an asset.
	pub(crate) fn start_destroy(owner: &AccountId32, asset_id: AssetId) {
		assert_ok!(Assets::start_destroy(
			RuntimeOrigin::signed(owner.clone().into()),
			asset_id.into()
		));
	}

	// Create an asset and set metadata.
	pub(crate) fn create_and_set_metadata(
		owner: &AccountId32,
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
		set_metadata(owner, asset_id, name, symbol, decimals);
		asset_id
	}

	// Set metadata of an asset.
	pub(crate) fn set_metadata(
		owner: &AccountId32,
		asset_id: AssetId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) {
		assert_ok!(Assets::set_metadata(
			RuntimeOrigin::signed(owner.clone().into()),
			asset_id.into(),
			name,
			symbol,
			decimals
		));
	}
}

pub(super) fn instantiate_and_create_fungible(
	contract: &str,
	token_id: TokenId,
	min_balance: Balance,
) -> Result<AccountId32, Error> {
	let function = function_selector("new");
	let input = [function, token_id.encode(), min_balance.encode()].concat();
	let wasm_binary = std::fs::read(contract).expect("could not read .wasm file");
	let result = Revive::bare_instantiate(
		RuntimeOrigin::signed(ALICE),
		INIT_VALUE,
		GAS_LIMIT,
		1 * 1_000_000_000_000,
		Code::Upload(wasm_binary),
		input,
		None,
		DEBUG_OUTPUT,
		CollectEvents::Skip,
	)
	.result
	.expect("should work");
	let address = result.addr;
	let result = result.result;
	decoded::<Result<(), Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
		.map(|_| AccountId32Mapper::<Runtime>::to_account_id(&address))
}

/// Get the last event from pallet contracts.
pub(super) fn last_contract_event() -> Vec<u8> {
	let events = System::read_events_for_pallet::<pallet_revive::Event<Runtime>>();
	let contract_events = events
		.iter()
		.filter_map(|event| match event {
			pallet_revive::Event::<Runtime>::ContractEmitted { data, .. } => Some(data.as_slice()),
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

use super::*;
use pop_primitives::error::{
	ArithmeticError::*,
	Error::{self, *},
	TokenError::*,
};

const ASSET_ID: AssetId = 1;
const CONTRACT: &str = "contracts/fungibles/target/ink/fungibles.wasm";

/// 1. PSP-22 Interface:
/// - total_supply
/// - balance_of
/// - allowance
/// - transfer
/// - transfer_from
/// - approve
/// - increase_allowance
/// - decrease_allowance

#[test]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(total_supply(addr.clone(), ASSET_ID), Ok(Assets::total_supply(ASSET_ID)));
		assert_eq!(total_supply(addr.clone(), ASSET_ID), Ok(0));

		// Tokens in circulation.
		create_asset_and_mint_to(addr.clone(), ASSET_ID, BOB, 100);
		assert_eq!(total_supply(addr.clone(), ASSET_ID), Ok(Assets::total_supply(ASSET_ID)));
		assert_eq!(total_supply(addr, ASSET_ID), Ok(100));
	});
}

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(balance_of(addr.clone(), ASSET_ID, BOB), Ok(Assets::balance(ASSET_ID, BOB)));
		assert_eq!(balance_of(addr.clone(), ASSET_ID, BOB), Ok(0));

		// Tokens in circulation.
		create_asset_and_mint_to(addr.clone(), ASSET_ID, BOB, 100);
		assert_eq!(balance_of(addr.clone(), ASSET_ID, BOB), Ok(Assets::balance(ASSET_ID, BOB)));
		assert_eq!(balance_of(addr, ASSET_ID, BOB), Ok(100));
	});
}

#[test]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(
			allowance(addr.clone(), ASSET_ID, BOB, ALICE),
			Ok(Assets::allowance(ASSET_ID, &BOB, &ALICE))
		);
		assert_eq!(allowance(addr.clone(), ASSET_ID, BOB, ALICE), Ok(0));

		// Tokens in circulation.
		create_asset_mint_and_approve(addr.clone(), ASSET_ID, BOB, 100, ALICE, 50);
		assert_eq!(
			allowance(addr.clone(), ASSET_ID, BOB, ALICE),
			Ok(Assets::allowance(ASSET_ID, &BOB, &ALICE))
		);
		assert_eq!(allowance(addr, ASSET_ID, BOB, ALICE), Ok(50));
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;
		// Asset does not exist.
		assert_eq!(transfer(addr.clone(), 1, BOB, amount), Err(Module { index: 52, error: 3 }));
		// Create asset with Alice as owner and mint `amount` to contract address.
		let asset = create_asset_and_mint_to(ALICE, 1, addr.clone(), amount);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(ALICE, asset);
		assert_eq!(
			transfer(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: 16 })
		);
		thaw_asset(ALICE, asset);
		// Not enough balance.
		assert_eq!(
			transfer(addr.clone(), asset, BOB, amount + 1 * UNIT),
			Err(Module { index: 52, error: 0 })
		);
		// Not enough balance due to ED.
		assert_eq!(transfer(addr.clone(), asset, BOB, amount), Err(Module { index: 52, error: 0 }));
		// Successful transfer.
		let balance_before_transfer = Assets::balance(asset, &BOB);
		assert_ok!(transfer(addr.clone(), asset, BOB, amount / 2));
		let balance_after_transfer = Assets::balance(asset, &BOB);
		assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
		// Transfer asset to account that does not exist.
		assert_eq!(transfer(addr.clone(), asset, FERDIE, amount / 4), Err(Token(CannotCreate)));
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(ALICE, asset);
		assert_eq!(
			transfer(addr.clone(), asset, BOB, amount / 4),
			Err(Module { index: 52, error: 16 })
		);
	});
}

#[test]
fn transfer_from_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;
		// Asset does not exist.
		assert_eq!(
			transfer_from(addr.clone(), 1, ALICE, BOB, amount / 2),
			Err(Module { index: 52, error: 3 }),
		);
		// Create asset with Alice as owner and mint `amount` to contract address.
		let asset = create_asset_and_mint_to(ALICE, 1, ALICE, amount);
		// Unapproved transfer.
		assert_eq!(
			transfer_from(addr.clone(), asset, ALICE, BOB, amount / 2),
			Err(Module { index: 52, error: 10 })
		);
		assert_ok!(Assets::approve_transfer(
			RuntimeOrigin::signed(ALICE.into()),
			asset.into(),
			addr.clone().into(),
			amount + 1 * UNIT,
		));
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(ALICE, asset);
		assert_eq!(
			transfer_from(addr.clone(), asset, ALICE, BOB, amount),
			Err(Module { index: 52, error: 16 }),
		);
		thaw_asset(ALICE, asset);
		// Not enough balance.
		assert_eq!(
			transfer_from(addr.clone(), asset, ALICE, BOB, amount + 1 * UNIT),
			Err(Module { index: 52, error: 0 }),
		);
		// Successful transfer.
		let balance_before_transfer = Assets::balance(asset, &BOB);
		assert_ok!(transfer_from(addr.clone(), asset, ALICE, BOB, amount / 2));
		let balance_after_transfer = Assets::balance(asset, &BOB);
		assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
	});
}

#[test]
fn approve_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, 0, vec![]);
		let amount: Balance = 100 * UNIT;
		// Asset does not exist.
		assert_eq!(approve(addr.clone(), 0, BOB, amount), Err(Module { index: 52, error: 3 }));
		let asset = create_asset_and_mint_to(ALICE, 0, addr.clone(), amount);
		assert_eq!(approve(addr.clone(), asset, BOB, amount), Err(ConsumerRemaining));
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![1]);
		// Create asset with Alice as owner and mint `amount` to contract address.
		let asset = create_asset_and_mint_to(ALICE, 1, addr.clone(), amount);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(ALICE, asset);
		assert_eq!(approve(addr.clone(), asset, BOB, amount), Err(Module { index: 52, error: 16 }));
		thaw_asset(ALICE, asset);
		// Successful approvals:
		assert_eq!(0, Assets::allowance(asset, &addr, &BOB));
		assert_ok!(approve(addr.clone(), asset, BOB, amount));
		assert_eq!(Assets::allowance(asset, &addr, &BOB), amount);
		// Non-additive, sets new value.
		assert_ok!(approve(addr.clone(), asset, BOB, amount / 2));
		assert_eq!(Assets::allowance(asset, &addr, &BOB), amount / 2);
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(ALICE, asset);
		assert_eq!(approve(addr.clone(), asset, BOB, amount), Err(Module { index: 52, error: 16 }));
	});
}

#[test]
fn increase_allowance_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, 0, vec![]);
		let amount: Balance = 100 * UNIT;
		// Asset does not exist.
		assert_eq!(
			increase_allowance(addr.clone(), 0, BOB, amount),
			Err(Module { index: 52, error: 3 })
		);
		let asset = create_asset_and_mint_to(ALICE, 0, addr.clone(), amount);
		assert_eq!(increase_allowance(addr.clone(), asset, BOB, amount), Err(ConsumerRemaining));
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![1]);
		// Create asset with Alice as owner and mint `amount` to contract address.
		let asset = create_asset_and_mint_to(ALICE, 1, addr.clone(), amount);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(ALICE, asset);
		assert_eq!(
			increase_allowance(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: 16 })
		);
		thaw_asset(ALICE, asset);
		// Successful approvals:
		assert_eq!(0, Assets::allowance(asset, &addr, &BOB));
		assert_ok!(increase_allowance(addr.clone(), asset, BOB, amount));
		assert_eq!(Assets::allowance(asset, &addr, &BOB), amount);
		// Additive.
		assert_ok!(increase_allowance(addr.clone(), asset, BOB, amount));
		assert_eq!(Assets::allowance(asset, &addr, &BOB), amount * 2);
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(ALICE, asset);
		assert_eq!(
			increase_allowance(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: 16 })
		);
	});
}

#[test]
fn decrease_allowance_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;
		// Asset does not exist.
		assert_eq!(
			decrease_allowance(addr.clone(), 0, BOB, amount),
			Err(Module { index: 52, error: 3 }),
		);
		// Create asset and mint `amount` to contract address, then approve Bob to spend `amount`.
		let asset =
			create_asset_mint_and_approve(addr.clone(), 0, addr.clone(), amount, BOB, amount);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(addr.clone(), asset);
		assert_eq!(
			decrease_allowance(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: 16 }),
		);
		thaw_asset(addr.clone(), asset);
		// Successfully decrease allowance.
		let allowance_before = Assets::allowance(asset, &addr, &BOB);
		assert_ok!(decrease_allowance(addr.clone(), 0, BOB, amount / 2 - 1 * UNIT));
		let allowance_after = Assets::allowance(asset, &addr, &BOB);
		assert_eq!(allowance_before - allowance_after, amount / 2 - 1 * UNIT);
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(addr.clone(), asset);
		assert_eq!(
			decrease_allowance(addr.clone(), asset, BOB, amount),
			Err(Module { index: 52, error: 16 }),
		);
	});
}

/// 2. PSP-22 Metadata Interface:
/// - token_name
/// - token_symbol
/// - token_decimals

#[test]
fn token_metadata_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		let name: Vec<u8> = vec![11, 12, 13];
		let symbol: Vec<u8> = vec![21, 22, 23];
		let decimals: u8 = 69;

		// Token does not exist.
		assert_eq!(token_name(addr.clone(), ASSET_ID), Ok(token_name_asset(ASSET_ID)));
		assert_eq!(token_name(addr.clone(), ASSET_ID), Ok(Vec::<u8>::new()));
		assert_eq!(token_symbol(addr.clone(), ASSET_ID), Ok(token_symbol_asset(ASSET_ID)));
		assert_eq!(token_symbol(addr.clone(), ASSET_ID), Ok(Vec::<u8>::new()));
		assert_eq!(token_decimals(addr.clone(), ASSET_ID), Ok(token_decimals_asset(ASSET_ID)));
		assert_eq!(token_decimals(addr.clone(), ASSET_ID), Ok(0));

		create_asset_and_set_metadata(
			addr.clone(),
			ASSET_ID,
			name.clone(),
			symbol.clone(),
			decimals,
		);
		assert_eq!(token_name(addr.clone(), ASSET_ID), Ok(token_name_asset(ASSET_ID)));
		assert_eq!(token_name(addr.clone(), ASSET_ID), Ok(name));
		assert_eq!(token_symbol(addr.clone(), ASSET_ID), Ok(token_symbol_asset(ASSET_ID)));
		assert_eq!(token_symbol(addr.clone(), ASSET_ID), Ok(symbol));
		assert_eq!(token_decimals(addr.clone(), ASSET_ID), Ok(token_decimals_asset(ASSET_ID)));
		assert_eq!(token_decimals(addr.clone(), ASSET_ID), Ok(decimals));
	});
}

// #[test]
// #[ignore]
// fn asset_exists_works() {
// 	new_test_ext().execute_with(|| {
// 		let _ = env_logger::try_init();
// 		let addr =
// 			instantiate(CONTRACT, INIT_VALUE, vec![]);
//
// 		// No tokens in circulation.
// 		assert_eq!(Assets::asset_exists(ASSET_ID), asset_exists(addr.clone(), ASSET_ID));
//
// 		// Tokens in circulation.
// 		create_asset(addr.clone(), ASSET_ID, 1);
// 		assert_eq!(Assets::asset_exists(ASSET_ID), asset_exists(addr, ASSET_ID));
// 	});
// }

// #[test]
// #[ignore]
// fn mint_works() {
// 	new_test_ext().execute_with(|| {
// 		let _ = env_logger::try_init();
// 		let addr =
// 			instantiate(CONTRACT, INIT_VALUE, vec![]);
// 		let amount: Balance = 100 * UNIT;
//
// 		// Asset does not exist.
// 		assert_eq!(
// 			decoded::<Error>(transfer_from(addr.clone(), 1, None, Some(BOB), amount, &[0u8])),
// 			Token(UnknownAsset)
// 		);
// 		let asset = create_asset(ALICE, 1, 2);
// 		// Minting can only be done by the owner.
// 		assert_eq!(
// 			decoded::<Error>(transfer_from(addr.clone(), asset, None, Some(BOB), amount, &[0u8])),
// 			Ok(Module { index: 52, error: 2 }),
// 		);
// 		// Minimum balance of an asset can not be zero.
// 		assert_eq!(
// 			decoded::<Error>(transfer_from(addr.clone(), asset, None, Some(BOB), 1, &[0u8])),
// 			Token(BelowMinimum)
// 		);
// 		let asset = create_asset(addr.clone(), 2, 2);
// 		// Asset is not live, i.e. frozen or being destroyed.
// 		freeze_asset(addr.clone(), asset);
// 		assert_eq!(
// 			decoded::<Error>(transfer_from(addr.clone(), asset, None, Some(BOB), amount, &[0u8])),
// 			Ok(Module { index: 52, error: 16 }),
// 		);
// 		thaw_asset(addr.clone(), asset);
// 		// Successful mint.
// 		let balance_before_mint = Assets::balance(asset, &BOB);
// 		let result = transfer_from(addr.clone(), asset, None, Some(BOB), amount, &[0u8]);
// 		assert!(!result.did_revert(), "Contract reverted!");
// 		let balance_after_mint = Assets::balance(asset, &BOB);
// 		assert_eq!(balance_after_mint, balance_before_mint + amount);
// 		// Can not mint more tokens than Balance::MAX.
// 		assert_eq!(
// 			decoded::<Error>(transfer_from(
// 				addr.clone(),
// 				asset,
// 				None,
// 				Some(BOB),
// 				Balance::MAX,
// 				&[0u8]
// 			)),
// 			Arithmetic(Overflow)
// 		);
// 		// Asset is not live, i.e. frozen or being destroyed.
// 		start_destroy_asset(addr.clone(), asset);
// 		assert_eq!(
// 			decoded::<Error>(transfer_from(addr.clone(), asset, None, Some(BOB), amount, &[0u8])),
// 			Ok(Module { index: 52, error: 16 }),
// 		);
// 	});
// }

// #[test]
// #[ignore]
// fn create_works() {
// 	new_test_ext().execute_with(|| {
// 		let _ = env_logger::try_init();
// 		// Instantiate a contract without balance (relay token).
// 		let addr = instantiate(CONTRACT, 0, vec![0]);
// 		// No balance to pay for fees.
// 		assert_eq!(
// 			decoded::<Error>(create(addr.clone(), ASSET_ID, addr.clone(), 1)),
// 			Ok(Module { index: 10, error: 2 }),
// 		);
// 		// Instantiate a contract without balance (relay token).
// 		let addr = instantiate(CONTRACT, 100, vec![2]);
// 		// No balance to pay the deposit.
// 		assert_eq!(
// 			decoded::<Error>(create(addr.clone(), ASSET_ID, addr.clone(), 1)),
// 			Ok(Module { index: 10, error: 2 }),
// 		);
// 		// Instantiate a contract with balance.
// 		let addr =
// 			instantiate(CONTRACT, INIT_VALUE, vec![1]);
// 		assert_eq!(
// 			decoded::<Error>(create(addr.clone(), ASSET_ID, BOB, 0)),
// 			Ok(Module { index: 52, error: 7 }),
// 		);
// 		create_asset(ALICE, ASSET_ID, 1);
// 		// Asset ID is already taken.
// 		assert_eq!(
// 			decoded::<Error>(create(addr.clone(), ASSET_ID, BOB, 1)),
// 			Ok(Module { index: 52, error: 5 }),
// 		);
// 		// The minimal balance for an asset must be non zero.
// 		let new_asset = 2;
// 		let result = create(addr.clone(), new_asset, BOB, 1);
// 		assert!(!result.did_revert(), "Contract reverted!");
// 	});
// }

// #[test]
// #[ignore]
// fn set_metadata_works() {
// 	new_test_ext().execute_with(|| {
// 		let _ = env_logger::try_init();
// 		let addr =
// 			instantiate(CONTRACT, INIT_VALUE, vec![]);
//
// 		create_asset(addr.clone(), ASSET_ID, 1);
// 		let result = set_metadata(addr.clone(), ASSET_ID, vec![12], vec![12], 12);
// 		assert!(!result.did_revert(), "Contract reverted!");
// 	});
// }

fn decoded<T: Decode>(result: ExecReturnValue) -> Result<T, ExecReturnValue> {
	<T>::decode(&mut &result.data[1..]).map_err(|_| result)
}

// Call total_supply contract message.
fn total_supply(addr: AccountId32, asset_id: AssetId) -> Result<Balance, Error> {
	let function = function_selector("total_supply");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

// Call balance_of contract message.
fn balance_of(addr: AccountId32, asset_id: AssetId, owner: AccountId32) -> Result<Balance, Error> {
	let function = function_selector("balance_of");
	let params = [function, asset_id.encode(), owner.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<Balance, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

// Call allowance contract message.
fn allowance(
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

// Call token_name contract message.
fn token_name(addr: AccountId32, asset_id: AssetId) -> Result<Vec<u8>, Error> {
	let function = function_selector("token_name");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<Vec<u8>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

// Call token_symbol contract message.
fn token_symbol(addr: AccountId32, asset_id: AssetId) -> Result<Vec<u8>, Error> {
	let function = function_selector("token_symbol");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<Vec<u8>, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

// Call token_decimals contract message.
fn token_decimals(addr: AccountId32, asset_id: AssetId) -> Result<u8, Error> {
	let function = function_selector("token_decimals");
	let params = [function, asset_id.encode()].concat();
	let result = bare_call(addr, params, 0).expect("should work");
	decoded::<Result<u8, Error>>(result.clone())
		.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
}

fn transfer(
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

fn transfer_from(
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

fn approve(
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

fn increase_allowance(
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

fn decrease_allowance(
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

// fn asset_exists(addr: AccountId32, asset_id: AssetId) -> bool {
// 	let function = function_selector("asset_exists");
// 	let params = [function, asset_id.encode()].concat();
// 	let result = bare_call(addr, params, 0).expect("should work");
// 	decoded::<bool>(result)
// }
//
// fn create(
// 	addr: AccountId32,
// 	asset_id: AssetId,
// 	admin: AccountId32,
// 	min_balance: Balance,
// ) -> ExecReturnValue {
// 	let function = function_selector("create");
// 	let params = [function, asset_id.encode(), admin.encode(), min_balance.encode()].concat();
// 	bare_call(addr, params, 0).expect("should work")
// }
//
// fn set_metadata(
// 	addr: AccountId32,
// 	asset_id: AssetId,
// 	name: Vec<u8>,
// 	symbol: Vec<u8>,
// 	decimals: u8,
// ) -> ExecReturnValue {
// 	let function = function_selector("set_metadata");
// 	let params =
// 		[function, asset_id.encode(), name.encode(), symbol.encode(), decimals.encode()].concat();
// 	bare_call(addr, params, 0).expect("should work")
// }

fn create_asset(owner: AccountId32, asset_id: AssetId, min_balance: Balance) -> AssetId {
	assert_ok!(Assets::create(
		RuntimeOrigin::signed(owner.clone()),
		asset_id.into(),
		owner.into(),
		min_balance
	));
	asset_id
}

fn mint_asset(owner: AccountId32, asset_id: AssetId, to: AccountId32, value: Balance) -> AssetId {
	assert_ok!(Assets::mint(
		RuntimeOrigin::signed(owner.clone()),
		asset_id.into(),
		to.into(),
		value
	));
	asset_id
}

fn create_asset_and_mint_to(
	owner: AccountId32,
	asset_id: AssetId,
	to: AccountId32,
	value: Balance,
) -> AssetId {
	create_asset(owner.clone(), asset_id, 1);
	mint_asset(owner, asset_id, to, value)
}

// Create an asset, mints to, and approves spender.
fn create_asset_mint_and_approve(
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
fn freeze_asset(owner: AccountId32, asset_id: AssetId) {
	assert_ok!(Assets::freeze_asset(RuntimeOrigin::signed(owner.into()), asset_id.into()));
}

// Thaw an asset.
fn thaw_asset(owner: AccountId32, asset_id: AssetId) {
	assert_ok!(Assets::thaw_asset(RuntimeOrigin::signed(owner.into()), asset_id.into()));
}

// Start destroying an asset.
fn start_destroy_asset(owner: AccountId32, asset_id: AssetId) {
	assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(owner.into()), asset_id.into()));
}

// Create an asset and set metadata.
fn create_asset_and_set_metadata(
	owner: AccountId32,
	asset_id: AssetId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) {
	assert_ok!(Assets::create(
		RuntimeOrigin::signed(owner.clone()),
		asset_id.into(),
		owner.clone().into(),
		100
	));
	set_metadata_asset(owner, asset_id, name, symbol, decimals);
}

// Set metadata of an asset.
fn set_metadata_asset(
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

fn token_name_asset(asset_id: AssetId) -> Vec<u8> {
	<pallet_assets::Pallet<Runtime, TrustBackedAssetsInstance> as MetadataInspect<AccountId32>>::name(
		asset_id,
	)
}

fn token_symbol_asset(asset_id: AssetId) -> Vec<u8> {
	<pallet_assets::Pallet<Runtime, TrustBackedAssetsInstance> as MetadataInspect<AccountId32>>::symbol(
		asset_id,
	)
}

fn token_decimals_asset(asset_id: AssetId) -> u8 {
	<pallet_assets::Pallet<Runtime, TrustBackedAssetsInstance> as MetadataInspect<AccountId32>>::decimals(
		asset_id,
	)
}

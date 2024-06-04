#![cfg(test)]

use super::*;
use pallet_contracts::debug::ExecResult;

#[derive(Decode, Encode, Debug, Eq, PartialEq)]
enum FungiblesError {
	/// The amount to mint is less than the existential deposit.
	BelowMinimum,
	/// Unspecified dispatch error, providing the index and optionally its error index.
	DispatchError { index: u8, error: Option<u8> },
	/// Not enough allowance to fulfill a request is available.
	InsufficientAllowance,
	/// Not enough balance to fulfill a request is available.
	InsufficientBalance,
	/// The asset ID is already taken.
	InUse,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
	/// Unspecified pallet error, providing pallet index and error index.
	ModuleError { pallet: u8, error: u16 },
	/// The signing account has no permission to do the operation.
	NoPermission,
	/// The given asset ID is unknown.
	Unknown,
}

const ASSET_ID: u32 = 1;

fn decoded<T: Decode>(result: ExecReturnValue) -> T {
	<T>::decode(&mut &result.data[2..]).unwrap()
}

fn allowance(
	addr: AccountId32,
	asset_id: u32,
	owner: AccountId32,
	spender: AccountId32,
) -> ExecReturnValue {
	let function = function_selector("allowance");
	let params = [function, asset_id.encode(), owner.encode(), spender.encode()].concat();
	do_bare_call(addr, params, 0).expect("should work")
}

// Call balance_of contract message.
fn balance_of(addr: AccountId32, asset_id: u32, owner: AccountId32) -> ExecReturnValue {
	let function = function_selector("balance_of");
	let params = [function, asset_id.encode(), owner.encode()].concat();
	do_bare_call(addr, params, 0).expect("should work")
}

// Call total_supply contract message.
fn total_supply(addr: AccountId32, asset_id: u32) -> Balance {
	let function = function_selector("total_supply");
	let params = [function, asset_id.encode()].concat();
	let result = do_bare_call(addr, params, 0).expect("should work");
	decoded::<Balance>(result)
}

fn asset_exists(addr: AccountId32, asset_id: u32) -> ExecReturnValue {
	let function = function_selector("asset_exists");
	let params = [function, asset_id.encode()].concat();
	do_bare_call(addr, params, 0).expect("should work")
}

fn create(
	addr: AccountId32,
	asset_id: u32,
	admin: AccountId32,
	min_balance: Balance,
) -> ExecReturnValue {
	let function = function_selector("create");
	let params = [function, asset_id.encode(), admin.encode(), min_balance.encode()].concat();
	do_bare_call(addr, params, 0).expect("should work")
}

fn set_metadata(
	addr: AccountId32,
	asset_id: u32,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) -> ExecReturnValue {
	let function = function_selector("set_metadata");
	let params =
		[function, asset_id.encode(), name.encode(), symbol.encode(), decimals.encode()].concat();
	do_bare_call(addr, params, 0).expect("should work")
}

fn transfer_from(
	addr: AccountId32,
	asset_id: u32,
	_from: Option<AccountId32>,
	to: Option<AccountId32>,
	value: Balance,
	_data: &[u8],
) -> ExecReturnValue {
	// let function = function_selector("transfer_from");
	// let params =
	// 	[function, asset_id.encode(), from.encode(), to.encode(), value.encode(), data.encode()]
	// 		.concat();
	// do_bare_call(addr, params, 0)
	let function = function_selector("mint");
	let params = [function, asset_id.encode(), to.unwrap().encode(), value.encode()].concat();
	do_bare_call(addr, params, 0).expect("should work")
}

// Create an asset and mint to owner.
fn create_asset(asset_id: u32, owner: AccountId32, min_balance: Balance) {
	assert_eq!(
		Assets::create(
			RuntimeOrigin::signed(owner.clone()),
			asset_id.into(),
			owner.into(),
			min_balance
		),
		Ok(())
	);
}

// Create an asset and mint to owner.
fn create_asset_and_mint_to(asset_id: u32, owner: AccountId32, to: AccountId32, value: Balance) {
	create_asset(asset_id, owner.clone(), 1);
	assert_eq!(
		Assets::mint(RuntimeOrigin::signed(owner.into()), asset_id.into(), to.into(), value,),
		Ok(())
	);
}

// Create an asset, mints to, and approves spender.
fn create_asset_mint_and_approve(
	asset_id: u32,
	owner: AccountId32,
	to: AccountId32,
	mint: Balance,
	spender: AccountId32,
	approve: Balance,
) {
	create_asset_and_mint_to(asset_id, owner.clone(), to.clone(), mint);
	assert_eq!(
		Assets::approve_transfer(
			RuntimeOrigin::signed(to.into()),
			asset_id.into(),
			spender.into(),
			approve,
		),
		Ok(())
	);
}

#[test]
#[ignore]
fn total_supply_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(
			"../../pop-api/examples/fungibles/target/ink/fungibles.wasm",
			INIT_VALUE,
			vec![],
		);

		// No tokens in circulation.
		assert_eq!(Assets::total_supply(ASSET_ID), total_supply(addr.clone(), ASSET_ID));

		// Tokens in circulation.
		create_asset_and_mint_to(ASSET_ID, addr.clone(), BOB, 100);
		assert_eq!(Assets::total_supply(ASSET_ID), total_supply(addr, ASSET_ID));
	});
}

#[test]
#[ignore]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(
			"../../pop-api/examples/fungibles/target/ink/fungibles.wasm",
			INIT_VALUE,
			vec![],
		);

		// No tokens in circulation.
		assert_eq!(
			Assets::balance(ASSET_ID, BOB).encode(),
			balance_of(addr.clone(), ASSET_ID, BOB).data[2..]
		);

		// Tokens in circulation.
		create_asset_and_mint_to(ASSET_ID, addr.clone(), BOB, 100);
		assert_eq!(
			Assets::balance(ASSET_ID, BOB).encode(),
			balance_of(addr, ASSET_ID, BOB).data[2..]
		);
	});
}

#[test]
#[ignore]
fn allowance_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(
			"../../pop-api/examples/fungibles/target/ink/fungibles.wasm",
			INIT_VALUE,
			vec![],
		);

		// No tokens in circulation.
		assert_eq!(
			Assets::allowance(ASSET_ID, &BOB, &ALICE).encode(),
			allowance(addr.clone(), ASSET_ID, BOB, ALICE).data[2..]
		);

		// Tokens in circulation.
		create_asset_mint_and_approve(ASSET_ID, addr.clone(), BOB, 100, ALICE, 50);
		assert_eq!(
			Assets::allowance(ASSET_ID, &BOB, &ALICE).encode(),
			allowance(addr, ASSET_ID, BOB, ALICE).data[2..]
		);
	});
}

#[test]
#[ignore]
fn asset_exists_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(
			"../../pop-api/examples/fungibles/target/ink/fungibles.wasm",
			INIT_VALUE,
			vec![],
		);

		// No tokens in circulation.
		assert_eq!(
			Assets::asset_exists(ASSET_ID).encode(),
			asset_exists(addr.clone(), ASSET_ID).data[2..]
		);

		// Tokens in circulation.
		create_asset(ASSET_ID, addr.clone(), 1);
		assert_eq!(Assets::asset_exists(ASSET_ID).encode(), asset_exists(addr, ASSET_ID).data[2..]);
	});
}

// Todo - errors:
// - Badorigin: contract is always signed
// - Lookup: is a valid AccountId due to the contract
// - reserve(): Overflow, LiquidityRestrictions; frozen
// - Callback
// - StorageDepositLimitExhausted
#[test]
#[ignore]
fn create_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let new_asset = 2;
		let addr =
			instantiate("../../pop-api/examples/fungibles/target/ink/fungibles.wasm", 0, vec![0]);

		assert_eq!(
			decoded::<FungiblesError>(create(addr.clone(), ASSET_ID, addr.clone(), 1)),
			FungiblesError::InsufficientBalance
		);
		let addr = instantiate(
			"../../pop-api/examples/fungibles/target/ink/fungibles.wasm",
			INIT_VALUE,
			vec![1],
		);
		create_asset(ASSET_ID, ALICE, 1);
		assert_eq!(
			decoded::<FungiblesError>(create(addr.clone(), ASSET_ID, BOB, 1)),
			FungiblesError::InUse
		);
		assert_eq!(
			decoded::<FungiblesError>(create(addr.clone(), new_asset, BOB, 0)),
			FungiblesError::MinBalanceZero
		);
		assert!(!create(addr.clone(), new_asset, BOB, 1).did_revert(), "Contract reverted!");
	});
}

#[test]
#[ignore]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(
			"../../pop-api/examples/fungibles/target/ink/fungibles.wasm",
			INIT_VALUE,
			vec![],
		);

		create_asset(ASSET_ID, addr.clone(), 1);

		let result = set_metadata(addr.clone(), ASSET_ID, vec![12], vec![12], 12);
		assert!(!result.did_revert(), "Contract reverted!");
	});
}

// todo: errors:
// - AssetNotLive: when frozen or being destroyed
// - TokenErrors: https://github.com/paritytech/polkadot-sdk/blob/3977f389cce4a00fd7100f95262e0563622b9aa4/substrate/frame/assets/src/functions.rs#L125
#[test]
#[ignore]
fn mint_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(
			"../../pop-api/examples/fungibles/target/ink/fungibles.wasm",
			INIT_VALUE,
			vec![],
		);
		let amount: Balance = 100 * UNIT;

		assert_eq!(
			decoded::<FungiblesError>(transfer_from(
				addr.clone(),
				ASSET_ID,
				None,
				Some(BOB),
				amount,
				&[0u8]
			)),
			FungiblesError::Unknown
		);
		create_asset(ASSET_ID, ALICE, 2);
		assert_eq!(
			decoded::<FungiblesError>(transfer_from(
				addr.clone(),
				ASSET_ID,
				None,
				Some(BOB),
				amount,
				&[0u8]
			)),
			FungiblesError::NoPermission
		);
		assert_eq!(
			decoded::<FungiblesError>(transfer_from(
				addr.clone(),
				ASSET_ID,
				None,
				Some(BOB),
				1,
				&[0u8]
			)),
			FungiblesError::BelowMinimum
		);
		let asset = 2;
		create_asset(asset, addr.clone(), 2);
		let bob_balance_before_mint = Assets::balance(asset, &BOB);
		let result = transfer_from(addr.clone(), asset, None, Some(BOB), 100 * UNIT, &[0u8]);
		assert!(!result.did_revert(), "Contract reverted!");
		let bob_balance_after_mint = Assets::balance(asset, &BOB);
		assert_eq!(bob_balance_after_mint, bob_balance_before_mint + amount);
	});
}

#[test]
#[ignore]
fn transfer_works() {}

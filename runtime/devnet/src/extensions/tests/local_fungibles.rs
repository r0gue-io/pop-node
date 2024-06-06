#![cfg(test)]

use super::*;

use pop_api::v0::assets::fungibles::FungiblesError;

const ASSET_ID: AssetId = 1;

fn decoded<T: Decode>(result: ExecReturnValue) -> T {
	<T>::decode(&mut &result.data[2..]).unwrap()
}

fn allowance(
	addr: AccountId32,
	asset_id: AssetId,
	owner: AccountId32,
	spender: AccountId32,
) -> Balance {
	let function = function_selector("allowance");
	let params = [function, asset_id.encode(), owner.encode(), spender.encode()].concat();
	let result = do_bare_call(addr, params, 0).expect("should work");
	decoded::<Balance>(result)
}

// Call balance_of contract message.
fn balance_of(addr: AccountId32, asset_id: AssetId, owner: AccountId32) -> Balance {
	let function = function_selector("balance_of");
	let params = [function, asset_id.encode(), owner.encode()].concat();
	let result = do_bare_call(addr, params, 0).expect("should work");
	decoded::<Balance>(result)
}

// Call total_supply contract message.
fn total_supply(addr: AccountId32, asset_id: AssetId) -> Balance {
	let function = function_selector("total_supply");
	let params = [function, asset_id.encode()].concat();
	let result = do_bare_call(addr, params, 0).expect("should work");
	decoded::<Balance>(result)
}

fn asset_exists(addr: AccountId32, asset_id: AssetId) -> bool {
	let function = function_selector("asset_exists");
	let params = [function, asset_id.encode()].concat();
	let result = do_bare_call(addr, params, 0).expect("should work");
	decoded::<bool>(result)
}

fn create(
	addr: AccountId32,
	asset_id: AssetId,
	admin: AccountId32,
	min_balance: Balance,
) -> ExecReturnValue {
	let function = function_selector("create");
	let params = [function, asset_id.encode(), admin.encode(), min_balance.encode()].concat();
	do_bare_call(addr, params, 0).expect("should work")
}

fn set_metadata(
	addr: AccountId32,
	asset_id: AssetId,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
) -> ExecReturnValue {
	let function = function_selector("set_metadata");
	let params =
		[function, asset_id.encode(), name.encode(), symbol.encode(), decimals.encode()].concat();
	do_bare_call(addr, params, 0).expect("should work")
}

fn transfer(
	addr: AccountId32,
	asset_id: AssetId,
	to: AccountId32,
	value: Balance,
) -> ExecReturnValue {
	let function = function_selector("transfer");
	let params = [function, asset_id.encode(), to.encode(), value.encode()].concat();
	do_bare_call(addr, params, 0).expect("should work")
}

fn transfer_from(
	addr: AccountId32,
	asset_id: AssetId,
	from: Option<AccountId32>,
	to: Option<AccountId32>,
	value: Balance,
	data: &[u8],
) -> ExecReturnValue {
	let function = function_selector("transfer_from");
	let params =
		[function, asset_id.encode(), from.encode(), to.encode(), value.encode(), data.encode()]
			.concat();
	do_bare_call(addr, params, 0).expect("should work")
}

// Create an asset and mint to owner.
fn create_asset(asset_id: AssetId, owner: AccountId32, min_balance: Balance) -> AssetId {
	assert_eq!(
		Assets::create(
			RuntimeOrigin::signed(owner.clone()),
			asset_id.into(),
			owner.into(),
			min_balance
		),
		Ok(())
	);
	asset_id
}

// Create an asset and mint to owner.
fn create_asset_and_mint_to(
	asset_id: AssetId,
	owner: AccountId32,
	to: AccountId32,
	value: Balance,
) -> AssetId {
	create_asset(asset_id, owner.clone(), 1);
	assert_eq!(
		Assets::mint(RuntimeOrigin::signed(owner.into()), asset_id.into(), to.into(), value,),
		Ok(())
	);
	asset_id
}

// Create an asset, mints to, and approves spender.
fn create_asset_mint_and_approve(
	asset_id: AssetId,
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

// Freeze an asset.
fn freeze_asset(asset_id: AssetId, owner: AccountId32) {
	assert_eq!(Assets::freeze_asset(RuntimeOrigin::signed(owner.into()), asset_id.into()), Ok(()));
}

// Thaw an asset.
fn thaw_asset(asset_id: AssetId, owner: AccountId32) {
	assert_eq!(Assets::thaw_asset(RuntimeOrigin::signed(owner.into()), asset_id.into()), Ok(()));
}

// Start destroying an asset.
fn start_destroy_asset(asset_id: AssetId, owner: AccountId32) {
	assert_eq!(Assets::start_destroy(RuntimeOrigin::signed(owner.into()), asset_id.into()), Ok(()));
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
		assert_eq!(Assets::balance(ASSET_ID, BOB), balance_of(addr.clone(), ASSET_ID, BOB));

		// Tokens in circulation.
		create_asset_and_mint_to(ASSET_ID, addr.clone(), BOB, 100);
		assert_eq!(Assets::balance(ASSET_ID, BOB), balance_of(addr, ASSET_ID, BOB));
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
			Assets::allowance(ASSET_ID, &BOB, &ALICE),
			allowance(addr.clone(), ASSET_ID, BOB, ALICE)
		);

		// Tokens in circulation.
		create_asset_mint_and_approve(ASSET_ID, addr.clone(), BOB, 100, ALICE, 50);
		assert_eq!(
			Assets::allowance(ASSET_ID, &BOB, &ALICE),
			allowance(addr, ASSET_ID, BOB, ALICE)
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
		assert_eq!(Assets::asset_exists(ASSET_ID), asset_exists(addr.clone(), ASSET_ID));

		// Tokens in circulation.
		create_asset(ASSET_ID, addr.clone(), 1);
		assert_eq!(Assets::asset_exists(ASSET_ID), asset_exists(addr, ASSET_ID));
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
// - TokenErrors
// - Arithmetic
// - https://github.com/paritytech/polkadot-sdk/blob/3977f389cce4a00fd7100f95262e0563622b9aa4/substrate/frame/assets/src/functions.rs#L125
#[test]
#[ignore]
fn transfer_from_mint_works() {
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
				1,
				None,
				Some(BOB),
				amount,
				&[0u8]
			)),
			FungiblesError::Unknown
		);
		let asset = create_asset(1, ALICE, 2);
		assert_eq!(
			decoded::<FungiblesError>(transfer_from(
				addr.clone(),
				asset,
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
				asset,
				None,
				Some(BOB),
				1,
				&[0u8]
			)),
			FungiblesError::BelowMinimum
		);
		let asset = create_asset(2, addr.clone(), 2);
		freeze_asset(asset, addr.clone());
		assert_eq!(
			decoded::<FungiblesError>(transfer_from(
				addr.clone(),
				asset,
				None,
				Some(BOB),
				amount,
				&[0u8]
			)),
			FungiblesError::AssetNotLive
		);
		thaw_asset(asset, addr.clone());
		let bob_balance_before_mint = Assets::balance(asset, &BOB);
		let result = transfer_from(addr.clone(), asset, None, Some(BOB), amount, &[0u8]);
		assert!(!result.did_revert(), "Contract reverted!");
		let bob_balance_after_mint = Assets::balance(asset, &BOB);
		assert_eq!(bob_balance_after_mint, bob_balance_before_mint + amount);
		start_destroy_asset(asset, addr.clone());
		assert_eq!(
			decoded::<FungiblesError>(transfer_from(
				addr.clone(),
				asset,
				None,
				Some(BOB),
				amount,
				&[0u8]
			)),
			FungiblesError::AssetNotLive
		);
	});
}

// Todo: error:
// - Frozen: account is frozen, who do you freeze an account?
// - https://github.com/paritytech/polkadot-sdk/blob/2460cddf57660a88844d201f769eb17a7accce5a/substrate/frame/assets/src/functions.rs#L161
// - ArithmeticError: Underflow, Overflow
// - https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/assets/src/functions.rs#L125
// -
#[test]
#[ignore]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate(
			"../../pop-api/examples/fungibles/target/ink/fungibles.wasm",
			INIT_VALUE,
			vec![],
		);
		let amount: Balance = 100 * UNIT;

		assert_eq!(
			decoded::<FungiblesError>(transfer(addr.clone(), 1, BOB, amount,)),
			FungiblesError::Unknown
		);
		let asset = create_asset_and_mint_to(1, ALICE, addr.clone(), amount);
		freeze_asset(asset, ALICE);
		assert_eq!(
			decoded::<FungiblesError>(transfer(addr.clone(), asset, BOB, amount,)),
			FungiblesError::AssetNotLive
		);
		thaw_asset(asset, ALICE);
		assert_eq!(
			decoded::<FungiblesError>(transfer(addr.clone(), asset, BOB, amount + 1 * UNIT)),
			FungiblesError::InsufficientBalance
		);
		// Errors due to ED. Could be Belowminimum
		assert_eq!(
			decoded::<FungiblesError>(transfer(addr.clone(), asset, BOB, amount)),
			FungiblesError::InsufficientBalance
		);
		let bob_balance_before_mint = Assets::balance(asset, &BOB);
		let result = transfer(addr.clone(), asset, BOB, amount / 2);
		assert!(!result.did_revert(), "Contract reverted!");
		let bob_balance_after_mint = Assets::balance(asset, &BOB);
		assert_eq!(bob_balance_after_mint, bob_balance_before_mint + amount / 2);
		start_destroy_asset(asset, ALICE);
		assert_eq!(
			decoded::<FungiblesError>(transfer(addr.clone(), asset, BOB, amount / 4)),
			FungiblesError::AssetNotLive
		);
	});
}

// Todo - errors:
// - Badorigin: contract is always signed
// - Lookup: is a valid AccountId due to the contract
// - Many errors can occur from calling a dispatchable. All the DispatchErrors are handled by the
// pop api but not all the possible errors for each dipatchable are tested. How should I approach
// this?
#![cfg(test)]

use super::*;

use pop_api::{
	error::{ArithmeticError::*, PopApiError::*, TokenError::*},
	v0::assets::use_cases::fungibles::FungiblesError::*,
};

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

fn create_asset(owner: AccountId32, asset_id: AssetId, min_balance: Balance) -> AssetId {
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

fn mint_asset(owner: AccountId32, asset_id: AssetId, to: AccountId32, value: Balance) -> AssetId {
	assert_eq!(
		Assets::mint(RuntimeOrigin::signed(owner.clone()), asset_id.into(), to.into(), value),
		Ok(())
	);
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
) {
	create_asset_and_mint_to(owner.clone(), asset_id, to.clone(), mint);
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
		create_asset_and_mint_to(addr.clone(), ASSET_ID, BOB, 100);
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
		create_asset_and_mint_to(addr.clone(), ASSET_ID, BOB, 100);
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
		create_asset_mint_and_approve(addr.clone(), ASSET_ID, BOB, 100, ALICE, 50);
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
		create_asset(addr.clone(), ASSET_ID, 1);
		assert_eq!(Assets::asset_exists(ASSET_ID), asset_exists(addr, ASSET_ID));
	});
}

#[test]
#[ignore]
fn create_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let new_asset = 2;
		// Instantiate a contract without balance (relay token).
		let addr =
			instantiate("../../pop-api/examples/fungibles/target/ink/fungibles.wasm", 0, vec![0]);
		// No balance to pay for fees.
		assert_eq!(
			decoded::<PopApiError>(create(addr.clone(), ASSET_ID, addr.clone(), 1)),
			UseCaseError(NoBalance)
		);
		// Instantiate a contract without balance (relay token).
		let addr =
			instantiate("../../pop-api/examples/fungibles/target/ink/fungibles.wasm", 100, vec![2]);
		// TODO: make sure it has enough for the fees but not for the deposit.
		// No balance to pay fe deposit.
		assert_eq!(
			decoded::<PopApiError>(create(addr.clone(), ASSET_ID, addr.clone(), 1)),
			UseCaseError(NoBalance)
		);
		// Instantiate a contract with balance.
		let addr = instantiate(
			"../../pop-api/examples/fungibles/target/ink/fungibles.wasm",
			INIT_VALUE,
			vec![1],
		);
		create_asset(ALICE, ASSET_ID, 1);
		// Asset ID is already taken.
		assert_eq!(
			decoded::<PopApiError>(create(addr.clone(), ASSET_ID, BOB, 1)),
			UseCaseError(InUse)
		);
		// The minimal balance for an asset must be non zero.
		assert_eq!(
			decoded::<PopApiError>(create(addr.clone(), new_asset, BOB, 0)),
			UseCaseError(MinBalanceZero)
		);
		let result = create(addr.clone(), new_asset, BOB, 1);
		assert!(!result.did_revert(), "Contract reverted!");
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

		create_asset(addr.clone(), ASSET_ID, 1);
		let result = set_metadata(addr.clone(), ASSET_ID, vec![12], vec![12], 12);
		assert!(!result.did_revert(), "Contract reverted!");
	});
}

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

		// Asset does not exist.
		assert_eq!(
			decoded::<PopApiError>(transfer_from(addr.clone(), 1, None, Some(BOB), amount, &[0u8])),
			Token(UnknownAsset)
		);
		let asset = create_asset(ALICE, 1, 2);
		// Minting can only be done by the owner.
		assert_eq!(
			decoded::<PopApiError>(transfer_from(
				addr.clone(),
				asset,
				None,
				Some(BOB),
				amount,
				&[0u8]
			)),
			UseCaseError(NoPermission)
		);
		// Minimum balance of an asset can not be zero.
		assert_eq!(
			decoded::<PopApiError>(transfer_from(addr.clone(), asset, None, Some(BOB), 1, &[0u8])),
			Token(BelowMinimum)
		);
		let asset = create_asset(addr.clone(), 2, 2);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(asset, addr.clone());
		assert_eq!(
			decoded::<PopApiError>(transfer_from(
				addr.clone(),
				asset,
				None,
				Some(BOB),
				amount,
				&[0u8]
			)),
			UseCaseError(AssetNotLive)
		);
		thaw_asset(asset, addr.clone());
		// Successful mint.
		let bob_balance_before_mint = Assets::balance(asset, &BOB);
		let result = transfer_from(addr.clone(), asset, None, Some(BOB), amount, &[0u8]);
		assert!(!result.did_revert(), "Contract reverted!");
		let bob_balance_after_mint = Assets::balance(asset, &BOB);
		assert_eq!(bob_balance_after_mint, bob_balance_before_mint + amount);
		// Can not mint more tokens than Balance::MAX.
		assert_eq!(
			decoded::<PopApiError>(transfer_from(
				addr.clone(),
				asset,
				None,
				Some(BOB),
				Balance::MAX,
				&[0u8]
			)),
			Arithmetic(Overflow)
		);
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(asset, addr.clone());
		assert_eq!(
			decoded::<PopApiError>(transfer_from(
				addr.clone(),
				asset,
				None,
				Some(BOB),
				amount,
				&[0u8]
			)),
			UseCaseError(AssetNotLive)
		);
	});
}

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

		// Asset does not exist.
		assert_eq!(
			decoded::<PopApiError>(transfer(addr.clone(), 1, BOB, amount,)),
			UseCaseError(Unknown)
		);
		// Create asset with Alice as owner and mint `amount` to contract address.
		let asset = create_asset_and_mint_to(ALICE, 1, addr.clone(), amount);
		// Asset is not live, i.e. frozen or being destroyed.
		freeze_asset(asset, ALICE);
		assert_eq!(
			decoded::<PopApiError>(transfer(addr.clone(), asset, BOB, amount,)),
			UseCaseError(AssetNotLive)
		);
		thaw_asset(asset, ALICE);
		// Not enough balance.
		assert_eq!(
			decoded::<PopApiError>(transfer(addr.clone(), asset, BOB, amount + 1 * UNIT)),
			UseCaseError(InsufficientBalance)
		);
		// Not enough balance due to ED.
		assert_eq!(
			decoded::<PopApiError>(transfer(addr.clone(), asset, BOB, amount)),
			UseCaseError(InsufficientBalance)
		);
		// Successful transfer.
		let bob_balance_before_mint = Assets::balance(asset, &BOB);
		let result = transfer(addr.clone(), asset, BOB, amount / 2);
		assert!(!result.did_revert(), "Contract reverted!");
		let bob_balance_after_mint = Assets::balance(asset, &BOB);
		assert_eq!(bob_balance_after_mint, bob_balance_before_mint + amount / 2);
		// Transfer asset to account that does not exist.
		assert_eq!(
			decoded::<PopApiError>(transfer(addr.clone(), asset, FERDIE, amount / 4)),
			Token(CannotCreate)
		);
		// Asset is not live, i.e. frozen or being destroyed.
		start_destroy_asset(asset, ALICE);
		assert_eq!(
			decoded::<PopApiError>(transfer(addr.clone(), asset, BOB, amount / 4)),
			UseCaseError(AssetNotLive)
		);
	});
}

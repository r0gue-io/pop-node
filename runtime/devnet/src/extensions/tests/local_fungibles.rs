#![cfg(test)]

use super::*;
use pallet_contracts::debug::ExecResult;

#[derive(Decode, Encode, Debug, Eq, PartialEq)]
enum FungiblesError {
	// AssetsError(Error),
	// /// The origin of the call doesn't have the right permission.
	// BadOrigin,
	// /// Custom error type for cases in which an implementation adds its own restrictions.
	// Custom(String),
	/// Not enough balance to fulfill a request is available.
	InsufficientBalance,
	/// Not enough allowance to fulfill a request is available.
	InsufficientAllowance,
	/// The asset status is not the expected status.
	IncorrectStatus,
	/// The asset ID is already taken.
	InUse,
	/// Minimum balance should be non-zero.
	MinBalanceZero,
	/// The signing account has no permission to do the operation.
	NoPermission,
	// /// Safe transfer check fails (e.g. if the receiving contract does not accept tokens).
	// SafeTransferCheckFailed(String),
	/// The given asset ID is unknown.
	Unknown,
	/// Recipient's address is zero.
	ZeroRecipientAddress,
	/// Sender's address is zero.
	ZeroSenderAddress,
	UndefinedError,
}

const ASSET_ID: u32 = 1;

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
fn total_supply(addr: AccountId32, asset_id: u32) -> ExecReturnValue {
	let function = function_selector("total_supply");
	let params = [function, asset_id.encode()].concat();
	do_bare_call(addr, params, 0).expect("should work")
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
	min_balance: u128,
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
	value: u128,
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
fn create_asset(asset_id: u32, owner: AccountId32) {
	assert_eq!(
		Assets::create(RuntimeOrigin::signed(owner.clone()), asset_id.into(), owner.into(), 1),
		Ok(())
	);
}

// Create an asset and mint to owner.
fn create_asset_and_mint_to(asset_id: u32, owner: AccountId32, to: AccountId32, value: u128) {
	create_asset(asset_id, owner.clone());
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
	mint: u128,
	spender: AccountId32,
	approve: u128,
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
		let addr =
			instantiate("../../pop-api/examples/fungibles/target/ink/fungibles.wasm", INIT_VALUE);

		// No tokens in circulation.
		assert_eq!(
			Assets::total_supply(ASSET_ID).encode(),
			total_supply(addr.clone(), ASSET_ID).data[2..]
		);

		// Tokens in circulation.
		create_asset_and_mint_to(ASSET_ID, addr.clone(), BOB, 100);
		assert_eq!(Assets::total_supply(ASSET_ID).encode(), total_supply(addr, ASSET_ID).data[2..]);
	});
}

#[test]
#[ignore]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr =
			instantiate("../../pop-api/examples/fungibles/target/ink/fungibles.wasm", INIT_VALUE);

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
		let addr =
			instantiate("../../pop-api/examples/fungibles/target/ink/fungibles.wasm", INIT_VALUE);

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
		let addr =
			instantiate("../../pop-api/examples/fungibles/target/ink/fungibles.wasm", INIT_VALUE);

		// No tokens in circulation.
		assert_eq!(
			Assets::asset_exists(ASSET_ID).encode(),
			asset_exists(addr.clone(), ASSET_ID).data[2..]
		);

		// Tokens in circulation.
		create_asset(ASSET_ID, addr.clone());
		assert_eq!(Assets::asset_exists(ASSET_ID).encode(), asset_exists(addr, ASSET_ID).data[2..]);
	});
}

fn decode_error(result: ExecReturnValue) -> FungiblesError {
	FungiblesError::decode(&mut &result.data[2..]).unwrap()
}

#[test]
#[ignore]
fn create_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr = instantiate("../../pop-api/examples/fungibles/target/ink/fungibles.wasm", 0);
		let new_asset = 2;

		assert_eq!(
			decode_error(create(addr.clone(), new_asset, BOB, 1)),
			FungiblesError::UndefinedError
		);
		// Todo: errors Badorigin, Lookup, reserve(), Callback
		// create_asset(ASSET_ID, ALICE);
		// // Error `InUse`.
		// assert_eq!(decode_error(create(addr.clone(), ASSET_ID, BOB, 1)), FungiblesError::InUse);
		// // Error `MinBalanceZero`.
		// assert_eq!(
		// 	decode_error(create(addr.clone(), new_asset, BOB, 0)),
		// 	FungiblesError::MinBalanceZero
		// );
		// assert!(
		// 	!create(addr.clone(), new_asset, BOB, 1).did_revert(),
		// 	"Contract should have been reverted!"
		// );
	});
}

#[test]
#[ignore]
fn set_metadata_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr =
			instantiate("../../pop-api/examples/fungibles/target/ink/fungibles.wasm", INIT_VALUE);

		create_asset(ASSET_ID, addr.clone());

		let result = set_metadata(addr.clone(), ASSET_ID, vec![12], vec![12], 12);
		assert!(!result.did_revert(), "Contract should have been reverted!");
	});
}

#[test]
#[ignore]
fn transfer_from_aka_mint_works() {
	new_test_ext().execute_with(|| {
		let _ = env_logger::try_init();
		let addr =
			instantiate("../../pop-api/examples/fungibles/target/ink/fungibles.wasm", INIT_VALUE);

		let amount: u128 = 100 * UNIT;
		// Create asset with contract as owner.
		create_asset(ASSET_ID, addr.clone());
		// Check Bob's asset balance before minting through contract.
		let bob_balance_before = Assets::balance(ASSET_ID, &BOB);

		let result = transfer_from(addr.clone(), ASSET_ID, None, Some(BOB), 100 * UNIT, &[0u8]);
		assert!(!result.did_revert(), "Contract reverted!");

		let bob_balance_after = Assets::balance(ASSET_ID, &BOB);
		assert_eq!(bob_balance_after, bob_balance_before + amount);
	});
}

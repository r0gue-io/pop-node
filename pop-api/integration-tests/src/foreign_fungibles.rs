use pop_api::fungibles::events::Transfer;
use pop_primitives::{ArithmeticError::*, Error, Error::*, TokenError::*};
use xcm::latest::{Junctions, Location};

use super::*;
use crate::foreign_fungibles::utils::*;
type TokenId = Location;
const TOKEN_ID: TokenId = Location { parents: 1, interior: Junctions::Here };
const CONTRACT: &str = "contracts/foreign_fungibles/target/ink/foreign_fungibles.wasm";

#[test]
fn balance_of_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);

		// No tokens in circulation.
		assert_eq!(balance_of(&addr, TOKEN_ID, BOB), Ok(ForeignAssets::balance(TOKEN_ID, BOB)));
		assert_eq!(balance_of(&addr, TOKEN_ID, BOB), Ok(0));

		// Tokens in circulation.
		assets::create_and_mint_to(&addr, TOKEN_ID, &BOB, 100);
		assert_eq!(balance_of(&addr, TOKEN_ID, BOB), Ok(ForeignAssets::balance(TOKEN_ID, BOB)));
		assert_eq!(balance_of(&addr, TOKEN_ID, BOB), Ok(100));
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		let addr = instantiate(CONTRACT, INIT_VALUE, vec![]);
		let amount: Balance = 100 * UNIT;

		// Token does not exist.
		assert_eq!(
			transfer(&addr, TOKEN_ID, BOB, amount),
			Err(Module { index: 53, error: [3, 0] })
		);
		// Create token with Alice as owner and mint `amount` to contract address.
		let token = assets::create_and_mint_to(&ALICE, TOKEN_ID, &addr, amount);
		// Token is not live, i.e. frozen or being destroyed.
		assets::freeze(&ALICE, token.clone());
		assert_eq!(
			transfer(&addr, token.clone(), BOB, amount),
			Err(Module { index: 53, error: [16, 0] })
		);
		assets::thaw(&ALICE, token.clone());
		// Not enough balance.
		assert_eq!(
			transfer(&addr, token.clone(), BOB, amount + 1 * UNIT),
			Err(Module { index: 53, error: [0, 0] })
		);
		// Not enough balance due to ED.
		assert_eq!(
			transfer(&addr, token.clone(), BOB, amount),
			Err(Module { index: 53, error: [0, 0] })
		);
		// Successful transfer.
		let balance_before_transfer = ForeignAssets::balance(token.clone(), &BOB);
		assert_ok!(transfer(&addr, token.clone(), BOB, amount / 2));
		let balance_after_transfer = ForeignAssets::balance(token.clone(), &BOB);
		assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
		// Successfully emit event.
		let from = account_id_from_slice(addr.as_ref());
		let to = account_id_from_slice(BOB.as_ref());
		let expected = Transfer { from: Some(from), to: Some(to), value: amount / 2 }.encode();
		assert_eq!(last_contract_event(), expected.as_slice());
		// Transfer token to account that does not exist.
		assert_eq!(transfer(&addr, token.clone(), FERDIE, amount / 4), Err(Token(CannotCreate)));
		// Token is not live, i.e. frozen or being destroyed.
		assets::start_destroy(&ALICE, token.clone());
		assert_eq!(
			transfer(&addr, token.clone(), BOB, amount / 4),
			Err(Module { index: 53, error: [16, 0] })
		);
	});
}

mod utils {
	use super::*;

	fn do_bare_call(function: &str, addr: &AccountId32, params: Vec<u8>) -> ExecReturnValue {
		let function = function_selector(function);
		let params = [function, params].concat();
		bare_call(addr.clone(), params, 0).expect("should work")
	}

	pub(super) fn decoded<T: Decode>(result: ExecReturnValue) -> Result<T, ExecReturnValue> {
		<T>::decode(&mut &result.data[1..]).map_err(|_| result)
	}

	pub fn account_id_from_slice(s: &[u8; 32]) -> pop_api::primitives::AccountId {
		pop_api::primitives::AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
	}

	pub(super) fn balance_of(
		addr: &AccountId32,
		token_id: TokenId,
		owner: AccountId32,
	) -> Result<Balance, Error> {
		let params = [token_id.encode(), owner.encode()].concat();
		let result = do_bare_call("balance_of", &addr, params);
		decoded::<Result<Balance, Error>>(result.clone())
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

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

	pub(super) fn transfer(
		addr: &AccountId32,
		token_id: TokenId,
		to: AccountId32,
		value: Balance,
	) -> Result<(), Error> {
		let params = [token_id.encode(), to.encode(), value.encode()].concat();
		let result = do_bare_call("transfer", addr, params);
		decoded::<Result<(), Error>>(result.clone())
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}
}

mod assets {
	use super::*;

	type AssetId = TokenId;

	pub(crate) fn create(owner: &AccountId32, asset_id: AssetId, min_balance: Balance) -> AssetId {
		assert_ok!(ForeignAssets::create(
			RuntimeOrigin::signed(owner.clone()),
			asset_id.clone().into(),
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
		assert_ok!(ForeignAssets::mint(
			RuntimeOrigin::signed(owner.clone()),
			asset_id.clone().into(),
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
		create(owner, asset_id.clone(), 1);
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
		create_and_mint_to(owner, asset_id.clone(), to, mint);
		assert_ok!(ForeignAssets::approve_transfer(
			RuntimeOrigin::signed(to.clone().into()),
			asset_id.clone().into(),
			spender.clone().into(),
			approve,
		));
		asset_id
	}

	// Freeze an asset.
	pub(crate) fn freeze(owner: &AccountId32, asset_id: AssetId) {
		assert_ok!(ForeignAssets::freeze_asset(
			RuntimeOrigin::signed(owner.clone().into()),
			asset_id.into()
		));
	}

	// Thaw an asset.
	pub(crate) fn thaw(owner: &AccountId32, asset_id: AssetId) {
		assert_ok!(ForeignAssets::thaw_asset(
			RuntimeOrigin::signed(owner.clone().into()),
			asset_id.into()
		));
	}

	// Start destroying an asset.
	pub(crate) fn start_destroy(owner: &AccountId32, asset_id: AssetId) {
		assert_ok!(ForeignAssets::start_destroy(
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
		assert_ok!(ForeignAssets::create(
			RuntimeOrigin::signed(owner.clone()),
			asset_id.clone().into(),
			owner.clone().into(),
			100
		));
		set_metadata(owner, asset_id.clone(), name, symbol, decimals);
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
		assert_ok!(ForeignAssets::set_metadata(
			RuntimeOrigin::signed(owner.clone().into()),
			asset_id.into(),
			name,
			symbol,
			decimals
		));
	}
}

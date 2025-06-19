use frame_support::{
	pallet_prelude::Encode,
	traits::fungibles::{approvals::Inspect as _, metadata::Inspect as _, Inspect as _},
};
use pop_api::fungibles::{Approval, Transfer};
use pop_primitives::TokenId;
use sp_io::hashing::twox_256;

use super::*;

const CONTRACT: &str = "contracts/fungibles/target/ink/fungibles.polkavm";

#[test]
fn total_supply_works() {
	let token = 1;
	let endowment = 100;
	ExtBuilder::new()
		.with_assets(vec![(token, ALICE, false, 1)])
		.with_asset_balances(vec![(token, BOB, endowment)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(0);

			// Tokens in circulation.
			assert_eq!(contract.total_supply(token), Assets::total_supply(token).into());
			assert_eq!(contract.total_supply(token), endowment.into());

			// No tokens in circulation.
			let token = TokenId::MAX;
			assert_eq!(contract.total_supply(token), Assets::total_supply(token).into());
			assert_eq!(contract.total_supply(token), 0.into());
		});
}

#[test]
fn balance_of_works() {
	let token = 1;
	let owner = BOB;
	let endowment = 100;
	ExtBuilder::new()
		.with_assets(vec![(token, ALICE, false, 1)])
		.with_asset_balances(vec![(token, owner.clone(), endowment)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(0);

			// Tokens in circulation.
			assert_eq!(
				contract.balance_of(token, to_address(&owner)),
				Assets::balance(token, &owner).into()
			);
			assert_eq!(contract.balance_of(token, to_address(&owner)), endowment.into());

			// No tokens in circulation.
			let token = TokenId::MAX;
			assert_eq!(
				contract.balance_of(token, to_address(&owner)),
				Assets::balance(token, &owner).into()
			);
			assert_eq!(contract.balance_of(token, to_address(&owner)), 0.into());
		});
}

#[test]
fn allowance_works() {
	let token = 1;
	let owner = BOB;
	let spender = ALICE;
	let allowance = 50;
	ExtBuilder::new()
		.with_assets(vec![(token, ALICE, false, 1)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(0);

			// Tokens in circulation.
			approve(&owner, token, &spender, allowance);
			assert_eq!(
				contract.allowance(token, to_address(&owner), to_address(&spender)),
				Assets::allowance(token, &owner, &spender).into()
			);
			assert_eq!(
				contract.allowance(token, to_address(&owner), to_address(&spender)),
				allowance.into()
			);

			// No tokens in circulation.
			let token = TokenId::MAX;
			assert_eq!(
				contract.allowance(token, to_address(&owner), to_address(&spender)),
				Assets::allowance(token, &owner, &spender).into()
			);
			assert_eq!(
				contract.allowance(token, to_address(&owner), to_address(&spender)),
				0.into()
			);
		});
}

#[test]
fn transfer_works() {
	let token = 1;
	let owner = ALICE;
	let amount: Balance = 100 * UNIT;
	let to = BOB;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			let mut contract = Contract::new(0);

			// Token does not exist.
			// assert_eq!(transfer(&addr, 1, BOB, amount), Err(Module { index: 52, error: [3, 0]
			// }));
			// Mint `amount` to contract address.
			mint(&owner, token, &contract.account_id(), amount);
			// Token is not live, i.e. frozen or being destroyed.
			freeze(&owner, token);
			// assert_eq!(
			// 	transfer(&addr, token, BOB, amount),
			// 	Err(Module { index: 52, error: [16, 0] })
			// );
			thaw(&owner, token);
			// Not enough balance.
			// assert_eq!(
			// 	transfer(&addr, token, BOB, amount + 1 * UNIT),
			// 	Err(Module { index: 52, error: [0, 0] })
			// );
			// Not enough balance due to ED.
			// assert_eq!(
			// 	transfer(&addr, token, BOB, amount),
			// 	Err(Module { index: 52, error: [0, 0] })
			// );
			// Successful transfer.
			let balance_before_transfer = Assets::balance(token, &BOB);
			contract.transfer(&owner, token, to_address(&to), (amount / 2).into());
			let balance_after_transfer = Assets::balance(token, &BOB);
			assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
			// Successfully emit event.
			let from = contract.address;
			let to = to_address(&to);
			let expected = Transfer { from, to, value: (amount / 2).into() }.encode();
			assert_eq!(contract.last_event(), expected);
			// Transfer token to account that does not exist.
			// assert_eq!(
			// 	contract.transfer(&owner, token, to_address(&CHARLIE), (amount / 4).into()),
			// 	Err(Token(CannotCreate))
			// );
			// Token is not live, i.e. frozen or being destroyed.
			start_destroy(&owner, token);
			// assert_eq!(
			// 	contract.transfer(&addr, token, BOB, amount / 4),
			// 	Err(Module { index: 52, error: [16, 0] })
			// );
		});
}

#[test]
fn transfer_from_works() {
	let token = 1;
	let owner = ALICE;
	let amount: Balance = 100 * UNIT;
	let to = BOB;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.with_asset_balances(vec![(token, owner.clone(), amount)])
		.build()
		.execute_with(|| {
			let mut contract = Contract::new(0);

			// Token does not exist.
			// assert_eq!(
			// 	transfer_from(&addr, 1, ALICE, BOB, amount / 2),
			// 	Err(Module { index: 52, error: [3, 0] }),
			// );
			// Unapproved transfer.
			// assert_eq!(
			// 	transfer_from(&addr, token, ALICE, BOB, amount / 2),
			// 	Err(Module { index: 52, error: [10, 0] })
			// );
			// Approve the contract to transfer on behalf of owner.
			approve(&owner, token, &contract.account_id(), amount + 1 * UNIT);
			// Token is not live, i.e. frozen or being destroyed.
			freeze(&owner, token);
			// assert_eq!(
			// 	transfer_from(&addr, token, ALICE, BOB, amount),
			// 	Err(Module { index: 52, error: [16, 0] }),
			// );
			thaw(&owner, token);
			// Not enough balance.
			// assert_eq!(
			// 	transfer_from(&addr, token, ALICE, BOB, amount + 1 * UNIT),
			// 	Err(Module { index: 52, error: [0, 0] }),
			// );
			// Successful transfer.
			let balance_before_transfer = Assets::balance(token, &BOB);
			contract.transfer_from(
				&owner,
				token,
				to_address(&owner),
				to_address(&to),
				(amount / 2).into(),
			);
			let balance_after_transfer = Assets::balance(token, &BOB);
			assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
			// Successfully emit event.
			let from = to_address(&owner);
			let to = to_address(&to);
			let expected = Transfer { from, to, value: (amount / 2).into() }.encode();
			assert_eq!(contract.last_event(), expected);
		});
}

#[test]
fn approve_works() {
	let token = 1;
	let owner = ALICE;
	let amount: Balance = 100 * UNIT;
	let delegate = BOB;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.with_asset_balances(vec![(token, owner.clone(), amount)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(0);

			// Token does not exist.
			// assert_eq!(contract.approve(&addr, TokenId::MAX, &BOB, amount), Err(Module { index:
			// 52, error: [3, 0] }));
			// assert_eq!(contract.approve(&addr, token, &BOB, amount), Err(ConsumerRemaining));
			let mut contract = Contract::new(INIT_VALUE);
			// Mint `amount` to contract address.
			mint(&owner, token, &contract.account_id(), amount);
			// Token is not live, i.e. frozen or being destroyed.
			freeze(&owner, token);
			// assert_eq!(
			// 	contract.approve(&addr, token, &BOB, amount),
			// 	Err(Module { index: 52, error: [16, 0] })
			// );
			thaw(&owner, token);
			// Successful approvals.
			assert_eq!(0, Assets::allowance(token, &contract.account_id(), &delegate));
			contract.approve(&contract.account_id(), token, to_address(&delegate), amount.into());
			assert_eq!(Assets::allowance(token, &contract.account_id(), &delegate), amount);
			// Successfully emit event.
			let spender = to_address(&delegate);
			let expected =
				Approval { owner: contract.address, spender, value: amount.into() }.encode();
			assert_eq!(contract.last_event(), expected);
			// Non-additive, sets new value.
			contract.approve(&contract.account_id(), token, spender, (amount / 2).into());
			assert_eq!(Assets::allowance(token, &contract.account_id(), &delegate), amount / 2);
			// Successfully emit event.
			let expected =
				Approval { owner: contract.address, spender, value: (amount / 2).into() }.encode();
			assert_eq!(contract.last_event(), expected);
			// Token is not live, i.e. frozen or being destroyed.
			start_destroy(&owner, token);
			// assert_eq!(
			// 	approve(&addr, token, &BOB, amount),
			// 	Err(Module { index: 52, error: [16, 0] })
			// );
		});
}

#[test]
fn increase_allowance_works() {
	let token = 1;
	let owner = ALICE;
	let amount: Balance = 100 * UNIT;
	let delegate = BOB;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			// Instantiate a contract without balance - test `ConsumerRemaining.
			let contract = Contract::new(0);
			// Token does not exist.
			// assert_eq!(
			// 	increase_allowance(&addr, 0, &BOB, amount),
			// 	Err(Module { index: 52, error: [3, 0] })
			// );
			mint(&owner, token, &contract.account_id(), amount);
			// assert_eq!(contract.increase_allowance(&owner, token, &delegate, amount),
			// Err(ConsumerRemaining));

			// Instantiate a contract with balance.
			let mut contract = Contract::new(INIT_VALUE);
			// Create token with Alice as owner and mint `amount` to contract address.
			mint(&owner, token, &contract.account_id(), amount);
			// Token is not live, i.e. frozen or being destroyed.
			freeze(&owner, token);
			// assert_eq!(
			// 	contract.increase_allowance(&addr, token, &BOB, amount),
			// 	Err(Module { index: 52, error: [16, 0] })
			// );
			thaw(&owner, token);
			// Successful approvals:
			assert_eq!(0, Assets::allowance(token, &contract.account_id(), &delegate));
			assert_eq!(
				contract.increase_allowance(
					&contract.account_id(),
					token,
					to_address(&delegate),
					amount.into()
				),
				amount.into()
			);
			assert_eq!(Assets::allowance(token, &contract.account_id(), &delegate), amount);
			// Additive.
			assert_eq!(
				contract.increase_allowance(
					&contract.account_id(),
					token,
					to_address(&delegate),
					amount.into()
				),
				(amount * 2).into()
			);
			assert_eq!(Assets::allowance(token, &contract.account_id(), &delegate), amount * 2);
			// Token is not live, i.e. frozen or being destroyed.
			start_destroy(&owner, token);
			// assert_eq!(
			// 	contract.increase_allowance(&addr, token, &BOB, amount),
			// 	Err(Module { index: 52, error: [16, 0] })
			// );
		});
}

#[test]
fn decrease_allowance_works() {
	let token = 1;
	let owner = ALICE;
	let amount: Balance = 100 * UNIT;
	let delegate = BOB;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			let mut contract = Contract::new(INIT_VALUE);

			// Mint `amount` to contract address, then approve delegate to spend `amount`.
			mint(&owner, token, &contract.account_id(), amount);
			approve(&contract.account_id(), token, &delegate, amount);
			// Token is not live, i.e. frozen or being destroyed.
			freeze(&owner, token);
			// assert_eq!(
			// 	decrease_allowance(&addr, token, &BOB, amount),
			// 	Err(Module { index: 52, error: [16, 0] }),
			// );
			thaw(&owner, token);
			// "Unapproved" error is returned if the current allowance is less than `value`.
			// assert_eq!(
			// 	decrease_allowance(&addr, token, &BOB, amount * 2),
			// 	Err(Module { index: 52, error: [10, 0] }),
			// );
			// Successfully decrease allowance.
			let amount = amount / 2 - 1 * UNIT;
			let allowance_before = Assets::allowance(token, &contract.account_id(), &delegate);
			assert_eq!(
				contract.decrease_allowance(&owner, token, to_address(&delegate), amount.into()),
				(allowance_before - amount).into()
			);
			let allowance_after = Assets::allowance(token, &contract.account_id(), &delegate);
			assert_eq!(allowance_before - allowance_after, amount);
			// Token is not live, i.e. frozen or being destroyed.
			start_destroy(&owner, token);
			// assert_eq!(
			// 	contract.decrease_allowance(&addr, token, &delegate, 1 * UNIT),
			// 	Err(Module { index: 52, error: [16, 0] }),
			// );
		});
}

#[test]
fn metadata_works() {
	let token = 1;
	let name = "name".to_string();
	let symbol = "symbol".to_string();
	let decimals: u8 = 69;
	ExtBuilder::new()
		.with_assets(vec![(token, ALICE, false, 1)])
		.with_asset_metadata(vec![(
			token,
			name.as_bytes().to_vec(),
			symbol.as_bytes().to_vec(),
			decimals,
		)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(0);

			// Existing token.
			assert_eq!(contract.name(token).as_bytes(), Assets::name(token).as_slice());
			assert_eq!(contract.name(token), name);
			assert_eq!(contract.symbol(token).as_bytes(), Assets::symbol(token).as_slice());
			assert_eq!(contract.symbol(token), symbol);
			assert_eq!(contract.decimals(token), Assets::decimals(token));
			assert_eq!(contract.decimals(token), decimals);

			// Token does not exist.
			let token = TokenId::MAX;
			assert_eq!(contract.name(token), String::default());
			assert_eq!(contract.symbol(token), String::default());
			assert_eq!(contract.decimals(token), 0);
		});
}

#[test]
#[ignore]
fn create_works() {
	todo!()
}

// Testing a contract that creates a token in the constructor.
#[test]
#[ignore]
fn instantiate_and_create_fungible_works() {
	todo!()
}

#[test]
#[ignore]
fn start_destroy_works() {
	todo!()
}

#[test]
#[ignore]
fn set_metadata_works() {
	todo!()
}

#[test]
#[ignore]
fn clear_metadata_works() {
	todo!()
}

#[test]
fn exists_works() {
	let token = 1;
	ExtBuilder::new()
		.with_assets(vec![(token, ALICE, false, 1)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(0);

			// Tokens in circulation.
			assert_eq!(contract.exists(token), Assets::asset_exists(token));

			// No tokens in circulation.
			let token = TokenId::MAX;
			assert_eq!(contract.exists(token), Assets::asset_exists(token));
		});
}

#[test]
#[ignore]
fn mint_works() {
	todo!()
}

#[test]
#[ignore]
fn burn_works() {
	todo!()
}

// A simple, strongly typed wrapper for the contract.
struct Contract {
	address: H160,
}

impl Contract {
	// Create a new instance of the contract through on-chain instantiation.
	fn new(value: Balance) -> Self {
		let salt = twox_256(&value.to_le_bytes());
		let address = instantiate(CONTRACT, value, Some(salt));
		Self { address }
	}

	fn allowance(&self, token: TokenId, owner: H160, spender: H160) -> U256 {
		let owner = alloy::Address::from(owner.0);
		let spender = alloy::Address::from(spender.0);
		U256::from_little_endian(
			self.call::<alloy::U256>(
				ALICE,
				keccak_selector("allowance(uint32,address,address)"),
				(token, owner, spender).abi_encode(),
				0,
			)
			.unwrap()
			.as_le_slice(),
		)
	}

	fn approve(&mut self, origin: &AccountId, token: TokenId, spender: H160, value: U256) {
		let spender = alloy::Address::from(spender.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		self.call(
			origin.clone(),
			keccak_selector("approve(uint32,address,uint256)"),
			(token, spender, value).abi_encode(),
			0,
		)
		.unwrap()
	}

	fn balance_of(&self, token: TokenId, owner: H160) -> U256 {
		let owner = alloy::Address::from(owner.0);
		U256::from_little_endian(
			self.call::<alloy::U256>(
				ALICE,
				keccak_selector("balanceOf(uint32,address)"),
				(token, owner).abi_encode(),
				0,
			)
			.unwrap()
			.as_le_slice(),
		)
	}

	fn burn(&mut self, origin: &AccountId, token: TokenId, account: H160, value: U256) {
		let account = alloy::Address::from(account.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		self.call(
			origin.clone(),
			keccak_selector("burn(uint32,address,uint256)"),
			(token, account, value).abi_encode(),
			0,
		)
		.unwrap()
	}

	fn clear_metadata(&mut self, origin: &AccountId, token: TokenId) {
		self.call(
			origin.clone(),
			keccak_selector("clearMetadata(uint32)"),
			(token,).abi_encode(),
			0,
		)
		.unwrap()
	}

	fn create(&mut self, origin: &AccountId, admin: H160, min_balance: U256) -> TokenId {
		let admin = alloy::Address::from(admin.0);
		let min_balance = alloy::U256::from_be_bytes(min_balance.to_big_endian());
		self.call(
			origin.clone(),
			keccak_selector("create(address,uint256)"),
			(admin, min_balance).abi_encode(),
			0,
		)
		.unwrap()
	}

	fn decimals(&self, token: TokenId) -> u8 {
		self.call::<u16>(ALICE, keccak_selector("decimals(uint32)"), (token,).abi_encode(), 0)
			.unwrap() as u8
	}

	fn decrease_allowance(
		&mut self,
		origin: &AccountId,
		token: TokenId,
		spender: H160,
		value: U256,
	) -> U256 {
		let spender = alloy::Address::from(spender.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		U256::from_little_endian(
			self.call::<alloy::U256>(
				origin.clone(),
				keccak_selector("decreaseAllowance(uint32,address,uint256)"),
				(token, spender, value).abi_encode(),
				0,
			)
			.unwrap()
			.as_le_slice(),
		)
	}

	fn exists(&self, token: TokenId) -> bool {
		self.call(ALICE, keccak_selector("exists(uint32)"), (token,).abi_encode(), 0)
			.unwrap()
	}

	fn increase_allowance(
		&mut self,
		origin: &AccountId,
		token: TokenId,
		spender: H160,
		value: U256,
	) -> U256 {
		let spender = alloy::Address::from(spender.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		U256::from_little_endian(
			self.call::<alloy::U256>(
				origin.clone(),
				keccak_selector("increaseAllowance(uint32,address,uint256)"),
				(token, spender, value).abi_encode(),
				0,
			)
			.unwrap()
			.as_le_slice(),
		)
	}

	fn mint(&mut self, origin: &AccountId, token: TokenId, account: H160, value: U256) {
		let account = alloy::Address::from(account.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		self.call(
			origin.clone(),
			keccak_selector("mint(uint32,address,uint256)"),
			(token, account, value).abi_encode(),
			0,
		)
		.unwrap()
	}

	fn name(&self, token: TokenId) -> String {
		self.call(ALICE, keccak_selector("name(uint32)"), (token,).abi_encode(), 0)
			.unwrap()
	}

	fn set_metadata(
		&mut self,
		origin: &AccountId,
		token: TokenId,
		name: String,
		symbol: String,
		decimals: u8,
	) {
		self.call(
			origin.clone(),
			keccak_selector("setMetadata(uint32,string,string,uint8)"),
			(token, name, symbol, decimals as u16).abi_encode(),
			0,
		)
		.unwrap()
	}

	fn start_destroy(&mut self, origin: &AccountId, token: TokenId) {
		self.call(origin.clone(), keccak_selector("startDestroy(uint32)"), (token,).abi_encode(), 0)
			.unwrap()
	}

	fn symbol(&self, token: TokenId) -> String {
		self.call(ALICE, keccak_selector("symbol(uint32)"), (token,).abi_encode(), 0)
			.unwrap()
	}

	fn total_supply(&self, token: TokenId) -> U256 {
		U256::from_little_endian(
			self.call::<alloy::U256>(
				ALICE,
				keccak_selector("totalSupply(uint32)"),
				(token,).abi_encode(),
				0,
			)
			.unwrap()
			.as_le_slice(),
		)
	}

	fn transfer(&mut self, origin: &AccountId, token: TokenId, to: H160, value: U256) {
		let to = alloy::Address::from(to.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		self.call(
			origin.clone(),
			keccak_selector("transfer(uint32,address,uint256)"),
			(token, to, value).abi_encode(),
			0,
		)
		.unwrap()
	}

	fn transfer_from(
		&mut self,
		origin: &AccountId,
		token: TokenId,
		from: H160,
		to: H160,
		value: U256,
	) {
		let from = alloy::Address::from(from.0);
		let to = alloy::Address::from(to.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		self.call(
			origin.clone(),
			keccak_selector("transferFrom(uint32,address,address,uint256)"),
			(token, from, to, value).abi_encode(),
			0,
		)
		.unwrap()
	}

	fn account_id(&self) -> AccountId {
		to_account_id(&self.address)
	}

	fn call<T: SolValue + From<<T::SolType as SolType>::RustType>>(
		&self,
		origin: AccountId,
		selector: [u8; 4],
		params: Vec<u8>,
		value: Balance,
	) -> Result<T, ()> {
		let origin = RuntimeOrigin::signed(origin);
		let dest = self.address.clone();
		let data = [selector.as_slice(), params.as_slice()].concat();
		let result = bare_call(origin, dest, value, GAS_LIMIT, STORAGE_DEPOSIT_LIMIT, data)
			.expect("should work");
		match result.did_revert() {
			true => {
				println!("{:?}", result.data);
				todo!("error conversion")
			},
			false => Ok(decode::<T>(&result.data)),
		}
	}

	fn last_event(&self) -> Vec<u8> {
		last_contract_event(&self.address)
	}
}

fn approve(origin: &AccountId, id: TokenId, delegate: &AccountId, amount: Balance) {
	assert_ok!(Assets::approve_transfer(
		RuntimeOrigin::signed(origin.clone()),
		id.into(),
		delegate.clone().into(),
		amount,
	));
}

fn freeze(origin: &AccountId, id: TokenId) {
	assert_ok!(Assets::freeze_asset(RuntimeOrigin::signed(origin.clone()), id.into()));
}

fn mint(origin: &AccountId, id: TokenId, beneficiary: &AccountId, amount: Balance) {
	assert_ok!(Assets::mint(
		RuntimeOrigin::signed(origin.clone()),
		id.into(),
		beneficiary.clone().into(),
		amount,
	));
}

fn start_destroy(origin: &AccountId, id: TokenId) {
	assert_ok!(Assets::start_destroy(RuntimeOrigin::signed(origin.clone()), id.into()));
}

fn thaw(origin: &AccountId, id: TokenId) {
	assert_ok!(Assets::thaw_asset(RuntimeOrigin::signed(origin.clone()), id.into()));
}

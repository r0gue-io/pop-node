use frame_support::{
	pallet_prelude::Encode,
	traits::fungibles::{
		approvals::Inspect as _, metadata::Inspect as _, roles::Inspect as _, Inspect as _,
	},
};
use pallet_api_vnext::fungibles::precompiles::IFungibles::{
	allowanceCall, approveCall, balanceOfCall, burnCall, clearMetadataCall, createCall,
	decimalsCall, decreaseAllowanceCall, existsCall, increaseAllowanceCall, mintCall, nameCall,
	setMetadataCall, startDestroyCall, symbolCall, totalSupplyCall, transferCall, transferFromCall,
};
use pop_api::fungibles::events::*;
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
			let contract = Contract::new(&BOB, 0);

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
	let owner = ALICE;
	let endowment = 100;
	ExtBuilder::new()
		.with_assets(vec![(token, BOB, false, 1)])
		.with_asset_balances(vec![(token, owner.clone(), endowment)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(&CHARLIE, 0);

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
	let owner = ALICE;
	let spender = BOB;
	let allowance = 50;
	ExtBuilder::new()
		.with_assets(vec![(token, CHARLIE, false, 1)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(&CHARLIE, 0);

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
			let mut contract = Contract::new(&owner, 0);

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
			let balance_before_transfer = Assets::balance(token, &to);
			contract.transfer(token, to_address(&to), (amount / 2).into());
			let balance_after_transfer = Assets::balance(token, &to);
			assert_eq!(balance_after_transfer, balance_before_transfer + amount / 2);
			// Successfully emit event.
			let from = contract.address;
			let to = to_address(&to);
			let expected = Transfer { from, to, value: (amount / 2).into() }.encode();
			assert_eq!(contract.last_event(), expected);
			// Transfer token to account that does not exist.
			// assert_eq!(
			// 	contract.transfer(token, to_address(&CHARLIE), (amount / 4).into()),
			// 	Err(Token(CannotCreate))
			// );
			// Token is not live, i.e. frozen or being destroyed.
			start_destroy(&owner, token);
			// assert_eq!(
			// 	contract.transfer(token, BOB, amount / 4),
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
			let mut contract = Contract::new(&owner, 0);

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
			let balance_before_transfer = Assets::balance(token, &to);
			contract.transfer_from(token, to_address(&owner), to_address(&to), (amount / 2).into());
			let balance_after_transfer = Assets::balance(token, &to);
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
	let spender = BOB;
	let amount: Balance = 100 * UNIT;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		//.with_asset_balances(vec![(token, owner.clone(), amount)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(&owner, 0);

			// Token does not exist.
			// assert_eq!(contract.approve(&addr, TokenId::MAX, &BOB, amount), Err(Module { index:
			// 52, error: [3, 0] }));
			// assert_eq!(contract.approve(&addr, token, &BOB, amount), Err(ConsumerRemaining));
			let mut contract = Contract::new(&owner, INIT_VALUE);
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
			assert_eq!(0, Assets::allowance(token, &contract.account_id(), &spender));
			contract.approve(token, to_address(&spender), amount.into());
			assert_eq!(Assets::allowance(token, &contract.account_id(), &spender), amount);
			// Successfully emit event.
			let spender = to_address(&spender);
			let expected =
				Approval { owner: contract.address, spender, value: amount.into() }.encode();
			assert_eq!(contract.last_event(), expected);
			// Non-additive, sets new value.
			contract.approve( token, spender, (amount / 2).into());
			assert_eq!(
				Assets::allowance(token, &contract.account_id(), &to_account_id(&spender)),
				amount / 2
			);
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
	let spender = BOB;
	let amount: Balance = 100 * UNIT;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			// Instantiate a contract without balance - test `ConsumerRemaining.
			let contract = Contract::new(&owner, 0);
			// Token does not exist.
			// assert_eq!(
			// 	increase_allowance(&addr, 0, &BOB, amount),
			// 	Err(Module { index: 52, error: [3, 0] })
			// );
			mint(&owner, token, &contract.account_id(), amount);
			// assert_eq!(contract.increase_allowance(&owner, token, &delegate, amount),
			// Err(ConsumerRemaining));

			// Instantiate a contract with balance.
			let mut contract = Contract::new(&owner, INIT_VALUE);
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
			assert_eq!(0, Assets::allowance(token, &contract.account_id(), &spender));
			assert_eq!(
				contract.increase_allowance(token, to_address(&spender), amount.into()),
				amount.into()
			);
			assert_eq!(Assets::allowance(token, &contract.account_id(), &spender), amount);
			// Additive.
			assert_eq!(
				contract.increase_allowance(token, to_address(&spender), amount.into()),
				(amount * 2).into()
			);
			assert_eq!(Assets::allowance(token, &contract.account_id(), &spender), amount * 2);
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
	let spender = BOB;
	let amount: Balance = 100 * UNIT;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			let mut contract = Contract::new(&owner, INIT_VALUE);

			// Mint `amount` to contract address, then approve delegate to spend `amount`.
			mint(&owner, token, &contract.account_id(), amount);
			approve(&contract.account_id(), token, &spender, amount);
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
			let allowance_before = Assets::allowance(token, &contract.account_id(), &spender);
			assert_eq!(
				contract.decrease_allowance(token, to_address(&spender), amount.into()),
				(allowance_before - amount).into()
			);
			let allowance_after = Assets::allowance(token, &contract.account_id(), &spender);
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
	let owner = ALICE;
	let name = "name".to_string();
	let symbol = "symbol".to_string();
	let decimals: u8 = 69;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.with_asset_metadata(vec![(token, name.clone().into(), symbol.clone().into(), decimals)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(&owner, 0);

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
fn create_works() {
	let owner = ALICE;
	ExtBuilder::new().build().execute_with(|| {
		// Instantiate a contract without balance for fees.
		let contract = Contract::new(&owner, 0);
		// No balance to pay for fees.
		// assert_eq!(contract.create(&addr, TOKEN_ID, &addr, 1), Err(Module { index: 10, error: [2,
		// 0] }),);

		// Instantiate a contract with insufficient balance for deposit.
		let contract = Contract::new(&owner, 100);
		// No balance to pay the deposit.
		// assert_eq!(contract.create(&addr, TOKEN_ID, &addr, 1), Err(Module { index: 10, error: [2,
		// 0] }),);

		// Instantiate a contract with enough balance.
		let mut contract = Contract::new(&owner, INIT_VALUE);
		// }),); The minimal balance for a token must be non zero.
		// assert_eq!(contract.create(&addr, &admin, 0), Err(Module { index: 52, error: [7, 0] }),);
		// Create token successfully.
		let admin = to_address(&owner);
		let token = contract.create(admin, 1.into());
		assert_eq!(Assets::owner(token), Some(contract.account_id()));
		// Successfully emit event.
		let expected = Created { id: token, creator: contract.address, admin }.encode();
		assert_eq!(contract.last_event(), expected);
		// Token ID is already taken.
		// assert_eq!(
		// 	contract.create(&addr, TOKEN_ID, &BOB, 1),
		// 	Err(Module { index: 52, error: [5, 0] }),
		// );
	});
}

// Testing a contract that creates a token in the constructor.
#[test]
#[ignore]
fn instantiate_and_create_fungible_works() {
	todo!()
}

#[test]
fn start_destroy_works() {
	let owner = ALICE;
	ExtBuilder::new().build().execute_with(|| {
		let mut contract = Contract::new(&owner, INIT_VALUE);

		// Token does not exist.
		// assert_eq!(
		// 	contract.start_destroy(&creator, TokenId::MAX),
		// 	Err(Module { index: 52, error: [3, 0] }),
		// );
		// No Permission.
		// assert_eq!(
		// 	contract.start_destroy(&creator, token),
		// 	Err(Module { index: 52, error: [2, 0] }),
		// );
		let token = contract.create(to_address(&owner), 1.into());
		contract.start_destroy(token);
		// Successfully emit event.
		let expected = DestroyStarted { token }.encode();
		assert_eq!(contract.last_event(), expected);
	});
}

#[test]
fn set_metadata_works() {
	let owner = ALICE;
	let name = "name".to_string();
	let symbol = "symbol".to_string();
	let decimals: u8 = 69;
	ExtBuilder::new().build().execute_with(|| {
		let mut contract = Contract::new(&owner, INIT_VALUE);

		// Token does not exist.
		// assert_eq!(
		// 	set_metadata(&addr, TOKEN_ID, vec![0], vec![0], 0u8),
		// 	Err(Module { index: 52, error: [3, 0] }),
		// );
		// No Permission.
		// assert_eq!(
		// 	set_metadata(&addr, token, vec![0], vec![0], 0u8),
		// 	Err(Module { index: 52, error: [2, 0] }),
		// );
		let token = contract.create(to_address(&owner), 1.into());
		// Token is not live, i.e. frozen or being destroyed.
		freeze(&owner, token);
		// assert_eq!(
		// 	set_metadata(&addr, TOKEN_ID, vec![0], vec![0], 0u8),
		// 	Err(Module { index: 52, error: [16, 0] }),
		// );
		thaw(&owner, token);
		// TODO: calling the below with a vector of length `100_000` errors in pallet contracts
		//  `OutputBufferTooSmall. Added to security analysis issue #131 to revisit.
		// Set bad metadata - too large values.
		// assert_eq!(
		// 	set_metadata(&addr, TOKEN_ID, vec![0; 1000], vec![0; 1000], 0u8),
		// 	Err(Module { index: 52, error: [9, 0] }),
		// );
		// Set metadata successfully.
		contract.set_metadata(token, name.clone(), symbol.clone(), decimals);
		assert_eq!(
			(&contract.name(token), &contract.symbol(token), &contract.decimals(token)),
			(&name, &symbol, &decimals)
		);
		// Successfully emit event.
		let expected = MetadataSet { token, name, symbol, decimals }.encode();
		assert_eq!(contract.last_event(), expected);
		// Token is not live, i.e. frozen or being destroyed.
		start_destroy(&contract.account_id(), token);
		// assert_eq!(
		// 	set_metadata(&addr, TOKEN_ID, vec![0], vec![0], 0),
		// 	Err(Module { index: 52, error: [16, 0] }),
		// );
	});
}

#[test]
fn clear_metadata_works() {
	let token = 0;
	let owner = ALICE;
	let name = "name".to_string();
	let symbol = "symbol".to_string();
	let decimals: u8 = 69;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			let mut contract = Contract::new(&owner, INIT_VALUE);

			// Token does not exist.
			// assert_eq!(contract.clear_metadata(TokenId::MAX), Err(Module { index: 52, error: [3,
			// 0] }),);
			// No Permission.
			// assert_eq!(contract.clear_metadata(token), Err(Module { index: 52, error: [2, 0]
			// }),);
			let token = contract.create(to_address(&owner), 1.into());
			// Token is not live, i.e. frozen or being destroyed.
			freeze(&owner, token);
			// assert_eq!(
			// 	contract.clear_metadata(&addr, token),
			// 	Err(Module { index: 52, error: [16, 0] }),
			// );
			thaw(&owner, token);
			// No metadata set.
			// assert_eq!(
			// 	contract.clear_metadata(&addr, token),
			// 	Err(Module { index: 52, error: [3, 0] }),
			// );
			contract.set_metadata(token, name, symbol, decimals);
			// Clear metadata successfully.
			contract.clear_metadata(token);
			// Successfully emit event.
			let expected = MetadataCleared { token }.encode();
			assert_eq!(contract.last_event(), expected);
			// Token is not live, i.e. frozen or being destroyed.
			start_destroy(&contract.account_id(), token);
			// assert_eq!(
			// 	contract.set_metadata(&addr, TOKEN_ID, vec![0], vec![0], decimals),
			// 	Err(Module { index: 52, error: [16, 0] }),
			// );
		});
}

#[test]
fn exists_works() {
	let token = 1;
	let owner = ALICE;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			let contract = Contract::new(&owner, 0);

			// Tokens in circulation.
			assert_eq!(contract.exists(token), Assets::asset_exists(token));

			// No tokens in circulation.
			let token = TokenId::MAX;
			assert_eq!(contract.exists(token), Assets::asset_exists(token));
		});
}

#[test]
fn mint_works() {
	let token = 0;
	let owner = ALICE;
	let account = BOB;
	let amount: Balance = 100 * UNIT;
	ExtBuilder::new()
		.with_assets(vec![(token, owner.clone(), false, 1)])
		.build()
		.execute_with(|| {
			let mut contract = Contract::new(&owner, INIT_VALUE);

			// Token does not exist.
			// assert_eq!(mint(&addr, TokenId::MAX, &BOB, amount), Err(Token(UnknownAsset)));
			// Minting can only be done by the owner.
			// assert_eq!(
			// 	contract.mint(token, &to_address(&account), 1.into()),
			// 	Err(Module { index: 52, error: [2, 0] })
			// );
			// Contract must be admin in order to be able to mint.
			let token = contract.create(contract.address, 2.into());
			// Minimum balance of a token can not be zero.
			// assert_eq!(contract.mint(token, &account, 1.into()), Err(Token(BelowMinimum)));
			// Token is not live, i.e. frozen or being destroyed.
			freeze(&contract.account_id(), token);
			// assert_eq!(
			// 	contract.mint(token, to_address(&account), amount),
			// 	Err(Module { index: 52, error: [16, 0] })
			// );
			thaw(&contract.account_id(), token);
			// Successful mint.
			let balance_before_mint = Assets::balance(token, &account);
			contract.mint(token, to_address(&account), amount.into());
			let balance_after_mint = Assets::balance(token, &account);
			assert_eq!(balance_after_mint, balance_before_mint + amount);
			// Account can not hold more tokens than Balance::MAX.
			// assert_eq!(
			// 	contract.mint(token, to_address(&account), Balance::MAX.into()),
			// 	Err(Arithmetic(Overflow))
			// );
			// Token is being destroyed.
			start_destroy(&contract.account_id(), token);
			// assert_eq!(
			// 	contract.mint(token, to_address(&account), amount.into()),
			// 	Err(Token(UnknownAsset))
			// );
		});
}

#[test]
#[ignore]
fn burn_works() {
	todo!()
}

// A simple, strongly typed wrapper for the contract.
struct Contract {
	address: H160,
	creator: AccountId,
}

impl Contract {
	// Create a new instance of the contract through on-chain instantiation.
	fn new(origin: &AccountId, value: Balance) -> Self {
		let salt = twox_256(&value.to_le_bytes());
		let address =
			instantiate(RuntimeOrigin::signed(origin.clone()), CONTRACT, value, Some(salt));
		Self { address, creator: origin.clone() }
	}

	fn allowance(&self, token: TokenId, owner: H160, spender: H160) -> U256 {
		let owner = alloy::Address::from(owner.0);
		let spender = alloy::Address::from(spender.0);
		let call = allowanceCall { token, owner, spender };
		U256::from_little_endian(self.call(&self.creator, call, 0).unwrap().as_le_slice())
	}

	fn approve(&mut self, token: TokenId, spender: H160, value: U256) {
		let spender = alloy::Address::from(spender.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		let call = approveCall { token, spender, value };
		self.call(&self.creator, call, 0).unwrap();
	}

	fn balance_of(&self, token: TokenId, owner: H160) -> U256 {
		let owner = alloy::Address::from(owner.0);
		let call = balanceOfCall { token, owner };
		U256::from_little_endian(self.call(&self.creator, call, 0).unwrap().as_le_slice())
	}

	fn burn(&mut self, token: TokenId, account: H160, value: U256) {
		let account = alloy::Address::from(account.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		let call = burnCall { token, account, value };
		self.call(&self.creator, call, 0).unwrap();
	}

	fn clear_metadata(&mut self, token: TokenId) {
		let call = clearMetadataCall { token };
		self.call(&self.creator, call, 0).unwrap();
	}

	fn create(&mut self, admin: H160, min_balance: U256) -> TokenId {
		let admin = alloy::Address::from(admin.0);
		let min_balance = alloy::U256::from_be_bytes(min_balance.to_big_endian());
		let call = createCall { admin, minBalance: min_balance };
		self.call(&self.creator, call, 0).unwrap()
	}

	fn decimals(&self, token: TokenId) -> u8 {
		let call = decimalsCall { token };
		self.call(&self.creator, call, 0).unwrap()
	}

	fn decrease_allowance(&mut self, token: TokenId, spender: H160, value: U256) -> U256 {
		let spender = alloy::Address::from(spender.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		let call = decreaseAllowanceCall { token, spender, value };
		U256::from_little_endian(self.call(&self.creator, call, 0).unwrap().as_le_slice())
	}

	fn exists(&self, token: TokenId) -> bool {
		let call = existsCall { token };
		self.call(&self.creator, call, 0).unwrap()
	}

	fn increase_allowance(&mut self, token: TokenId, spender: H160, value: U256) -> U256 {
		let spender = alloy::Address::from(spender.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		let call = increaseAllowanceCall { token, spender, value };
		U256::from_little_endian(self.call(&self.creator, call, 0).unwrap().as_le_slice())
	}

	fn mint(&mut self, token: TokenId, account: H160, value: U256) {
		let account = alloy::Address::from(account.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		let call = mintCall { token, account, value };
		self.call(&self.creator, call, 0).unwrap();
	}

	fn name(&self, token: TokenId) -> String {
		let call = nameCall { token };
		self.call(&self.creator, call, 0).unwrap()
	}

	fn set_metadata(&mut self, token: TokenId, name: String, symbol: String, decimals: u8) {
		let call = setMetadataCall { token, name, symbol, decimals };
		self.call(&self.creator, call, 0).unwrap();
	}

	fn start_destroy(&mut self, token: TokenId) {
		let call = startDestroyCall { token };
		self.call(&self.creator, call, 0).unwrap();
	}

	fn symbol(&self, token: TokenId) -> String {
		let call = symbolCall { token };
		self.call(&self.creator, call, 0).unwrap()
	}

	fn total_supply(&self, token: TokenId) -> U256 {
		let call = totalSupplyCall { token };
		U256::from_little_endian(self.call(&self.creator, call, 0).unwrap().as_le_slice())
	}

	fn transfer(&mut self, token: TokenId, to: H160, value: U256) {
		let to = alloy::Address::from(to.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		let call = transferCall { token, to, value };
		self.call(&self.creator, call, 0).unwrap();
	}

	fn transfer_from(&mut self, token: TokenId, from: H160, to: H160, value: U256) {
		let from = alloy::Address::from(from.0);
		let to = alloy::Address::from(to.0);
		let value = alloy::U256::from_be_bytes(value.to_big_endian());
		let call = transferFromCall { token, from, to, value };
		self.call(&self.creator, call, 0).unwrap();
	}

	fn account_id(&self) -> AccountId {
		to_account_id(&self.address)
	}

	fn call<T: SolCall>(
		&self,
		origin: &AccountId,
		call: T,
		value: Balance,
	) -> Result<T::Return, ()> {
		let origin = RuntimeOrigin::signed(origin.clone());
		let dest = self.address.clone();
		let data = call.abi_encode();
		let result = bare_call(origin, dest, value, GAS_LIMIT, STORAGE_DEPOSIT_LIMIT, data)
			.expect("should work");
		match result.did_revert() {
			true => todo!("error conversion: {:?}", String::from_utf8_lossy(&result.data)),
			false => Ok(T::abi_decode_returns(&result.data).expect("unable to decode")),
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

use drink::{
	assert_ok,
	devnet::{
		account_id_from_slice,
		error::{
			v0::{assert_runtime_err, RuntimeError},
			ArithmeticError::*,
			Assets,
			AssetsError::*,
			Error::*,
		},
		AccountId, Balance, Runtime,
	},
	session::Session,
	utils::{call, last_contract_event},
	AssetsAPI, TestExternalities, NO_SALT,
};
use ink::scale::Encode;
use pop_api::{
	primitives::TokenId,
	v0::fungibles::events::{Approval, Created, Transfer},
};

use super::*;

const UNIT: Balance = 10_000_000_000;
const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
const INIT_VALUE: Balance = 100 * UNIT;
const ALICE: AccountId = AccountId::new([1u8; 32]);
const BOB: AccountId = AccountId::new([2_u8; 32]);
const CHARLIE: AccountId = AccountId::new([3_u8; 32]);
const AMOUNT: Balance = MIN_BALANCE * 4;
const MIN_BALANCE: Balance = 10_000;
const TOKEN: TokenId = 1;

// The contract bundle provider.
//
// See https://github.com/r0gue-io/pop-drink/blob/main/drink/test-macro/src/lib.rs for more information.
#[drink::contract_bundle_provider]
enum BundleProvider {}

/// Sandbox environment for Pop Devnet Runtime.
pub struct Pop {
	ext: TestExternalities,
}

impl Default for Pop {
	fn default() -> Self {
		// Initialising genesis state, providing accounts with an initial balance.
		let balances: Vec<(AccountId, u128)> =
			vec![(ALICE, INIT_AMOUNT), (BOB, INIT_AMOUNT), (CHARLIE, INIT_AMOUNT)];
		let ext = BlockBuilder::<Runtime>::new_ext(balances);
		Self { ext }
	}
}

// Implement core functionalities for the `Pop` sandbox.
drink::impl_sandbox!(Pop, Runtime, ALICE);

// Deployment and constructor method tests.

#[drink::test(sandbox = Pop)]
fn new_constructor_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	// Token exists after the deployment.
	assert!(session.sandbox().asset_exists(&TOKEN));
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Created {
			id: TOKEN,
			creator: account_id_from_slice(&contract),
			admin: account_id_from_slice(&contract),
		}
		.encode()
		.as_slice()
	);
}

#[drink::test(sandbox = Pop)]
fn new_constructor_fails_with_used_token() {
	let _ = env_logger::try_init();
	// Fails to deploy contract with an used token ID.
	let token = TOKEN + 1;
	assert_ok!(session.sandbox().create(&token, &ALICE, MIN_BALANCE));
	// `pallet-assets` returns `InUse` error.
	assert_eq!(
		deploy(&mut session, "existing", vec![TOKEN.to_string()]),
		Err(PSP22Error::Custom(String::from("Token does not exist")))
	);
}

#[drink::test(sandbox = Pop)]
fn existing_constructor_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Successfully deploy contract with an existing token ID.
	let actor = session.get_actor();
	assert_ok!(session.sandbox().create(&TOKEN, &actor, MIN_BALANCE));
	assert_ok!(deploy(&mut session, "existing", vec![TOKEN.to_string()]));
	assert!(session.sandbox().asset_exists(&TOKEN));
}

#[drink::test(sandbox = Pop)]
fn existing_constructor_fails_with_non_existing_token(&mut session: Session) {
	let _ = env_logger::try_init();
	// Fails to deploy contract with a non-existing token ID.
	assert_eq!(
		deploy(&mut session, "existing", vec![TOKEN.to_string()]),
		Err(PSP22Error::Custom(String::from("Token does not exist")))
	);
}

// 1. PSP-22 Interface:
// - total_supply
// - balance_of
// - allowance
// - transfer
// - transfer_from
// - approve
// - increase_allowance
// - decrease_allowance

#[drink::test(sandbox = Pop)]
fn total_supply_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	assert_ok!(deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()],));
	// No tokens in circulation.
	assert_eq!(total_supply(&mut session), 0);
	assert_eq!(total_supply(&mut session), session.sandbox().total_supply(&TOKEN));
	// Tokens in circulation.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_eq!(total_supply(&mut session), AMOUNT);
	assert_eq!(total_supply(&mut session), session.sandbox().total_supply(&TOKEN));
}

#[drink::test(sandbox = Pop)]
fn balance_of_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	assert_ok!(deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]));
	// No tokens in circulation.
	assert_eq!(balance_of(&mut session, ALICE), 0);
	assert_eq!(balance_of(&mut session, ALICE), session.sandbox().balance_of(&TOKEN, &ALICE));
	// Tokens in circulation.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_eq!(balance_of(&mut session, ALICE), AMOUNT);
	assert_eq!(balance_of(&mut session, ALICE), session.sandbox().balance_of(&TOKEN, &ALICE));
}

#[drink::test(sandbox = Pop)]
fn allowance_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	// No tokens in circulation.
	assert_eq!(allowance(&mut session, contract.clone(), ALICE), 0);
	// Tokens in circulation.
	assert_ok!(session.sandbox().approve(&TOKEN, &contract.clone(), &ALICE, AMOUNT / 2));
	assert_eq!(allowance(&mut session, contract, ALICE), AMOUNT / 2);
}

#[drink::test(sandbox = Pop)]
fn transfer_fails_with_no_account() {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// `pallet-assets` returns `NoAccount` error.
	assert_runtime_err!(
		transfer(&mut session, ALICE, AMOUNT),
		RuntimeError::Module(Assets(NoAccount))
	);
}

#[drink::test(sandbox = Pop)]
fn transfer_noop_works(&mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// No-op if `value` is zero, returns success and no events are emitted.
	assert_ok!(transfer(&mut session, ALICE, 0));
	assert_eq!(last_contract_event(&session), None);
	// No-op if the caller and `to` is the same address, returns success and no events are emitted.
	assert_ok!(transfer(&mut session, contract.clone(), AMOUNT));
	assert_eq!(last_contract_event(&session), None);
}

#[drink::test(sandbox = Pop)]
fn transfer_fails_with_insufficient_balance() {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	// Failed with `InsufficientBalance`.
	assert_eq!(transfer(&mut session, BOB, AMOUNT + 1), Err(PSP22Error::InsufficientBalance));
}

#[drink::test(sandbox = Pop)]
fn transfer_fails_with_token_not_live() {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_runtime_err!(
		transfer(&mut session, BOB, AMOUNT / 2),
		RuntimeError::Module(Assets(AssetNotLive))
	);
}

#[drink::test(sandbox = Pop)]
fn transfer_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	let value = AMOUNT / 4;
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	// Successfully transfer.
	assert_ok!(transfer(&mut session, BOB, value));
	assert_eq!(session.sandbox().balance_of(&TOKEN, &contract), AMOUNT - value);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &BOB), value);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Transfer {
			from: Some(account_id_from_slice(&contract)),
			to: Some(account_id_from_slice(&BOB)),
			value,
		}
		.encode()
		.as_slice()
	);
}

#[drink::test(sandbox = Pop)]
fn transfer_from_noop_works() {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// No-op if `value` is zero, returns success and no events are emitted.
	assert_ok!(transfer_from(&mut session, ALICE, BOB, 0));
	assert_eq!(last_contract_event(&session), None);
	// No-op if the `from` and `to` is the same address, returns success and no events are emitted.
	assert_ok!(transfer_from(&mut session, ALICE, ALICE, AMOUNT));
	assert_eq!(last_contract_event(&session), None);
}

#[drink::test(sandbox = Pop)]
fn transfer_from_fails_with_insufficient_balance() {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &ALICE, &contract.clone(), AMOUNT * 2));
	// Not enough balance. Failed with `InsufficientBalance`.
	assert_eq!(
		transfer_from(&mut session, ALICE, contract.clone(), AMOUNT + 1),
		Err(PSP22Error::InsufficientBalance)
	);
}

#[drink::test(sandbox = Pop)]
fn transfer_from_fails_with_insufficient_allowance() {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &ALICE, &contract.clone(), AMOUNT));
	// Unapproved transfer. Failed with `InsufficientAllowance`.
	assert_eq!(
		transfer_from(&mut session, ALICE, contract.clone(), AMOUNT + 1),
		Err(PSP22Error::InsufficientAllowance)
	);
}

#[drink::test(sandbox = Pop)]
fn transfer_from_fails_with_token_not_live() {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &ALICE, &contract.clone(), AMOUNT));
	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_runtime_err!(
		transfer_from(&mut session, ALICE, BOB, AMOUNT / 2),
		RuntimeError::Module(Assets(AssetNotLive))
	);
}

#[drink::test(sandbox = Pop)]
fn transfer_from_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	let value = AMOUNT / 2;
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &ALICE, &contract.clone(), AMOUNT));
	// Successful transfer.
	assert_ok!(transfer_from(&mut session, ALICE, BOB, value));
	assert_eq!(session.sandbox().allowance(&TOKEN, &ALICE, &contract.clone()), value);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), value);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &BOB), value);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(&ALICE),
			spender: account_id_from_slice(&contract),
			value,
		}
		.encode()
		.as_slice()
	);
}

#[drink::test(sandbox = Pop)]
fn approve_noop_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// No-op if the caller and `spender` is the same address, returns success and no events are
	// emitted.
	assert_ok!(approve(&mut session, contract.clone(), AMOUNT));
	assert_eq!(last_contract_event(&session), None);
}

#[drink::test(sandbox = Pop)]
fn approve_fails_with_token_not_live(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_runtime_err!(
		approve(&mut session, ALICE, AMOUNT),
		RuntimeError::Module(Assets(AssetNotLive))
	);
}

#[drink::test(sandbox = Pop)]
fn approve_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	let value = AMOUNT / 2;
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	// Successfully approve.
	assert_ok!(approve(&mut session, ALICE, value));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), value);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(&contract),
			spender: account_id_from_slice(&ALICE),
			value,
		}
		.encode()
		.as_slice()
	);
	// Non-additive, sets new value.
	assert_ok!(approve(&mut session, ALICE, value - 1));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), value - 1);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(&contract),
			spender: account_id_from_slice(&ALICE),
			value: value - 1,
		}
		.encode()
		.as_slice()
	);
}

#[drink::test(sandbox = Pop)]
fn increase_allowance_noop_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// No-op if the caller and `spender` is the same address, returns success and no events are
	// emitted.
	assert_ok!(increase_allowance(&mut session, contract.clone(), AMOUNT));
	assert_eq!(last_contract_event(&session), None);
	// No-op if the `value` is zero, returns success and no events are emitted.
	assert_ok!(increase_allowance(&mut session, contract.clone(), 0));
	assert_eq!(last_contract_event(&session), None);
}

#[drink::test(sandbox = Pop)]
fn increase_allowance_fails_with_token_not_live(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_runtime_err!(
		increase_allowance(&mut session, ALICE, AMOUNT),
		RuntimeError::Module(Assets(AssetNotLive))
	);
}

#[drink::test(sandbox = Pop)]
fn increase_allowance_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	let value = AMOUNT / 2;
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &contract.clone(), &ALICE, AMOUNT));
	// Successfully approve.
	assert_ok!(increase_allowance(&mut session, ALICE, value));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT + value);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(&contract),
			spender: account_id_from_slice(&ALICE),
			value: AMOUNT + value,
		}
		.encode()
		.as_slice()
	);
	// Additive.
	assert_ok!(increase_allowance(&mut session, ALICE, value));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT + value * 2);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(&contract),
			spender: account_id_from_slice(&ALICE),
			value: AMOUNT + value * 2,
		}
		.encode()
		.as_slice()
	);
}

#[drink::test(sandbox = Pop)]
fn decrease_allowance_noop_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// No-op if the caller and `spender` is the same address, returns success and no events are
	// emitted.
	assert_ok!(decrease_allowance(&mut session, contract.clone(), AMOUNT));
	assert_eq!(last_contract_event(&session), None);
}

#[drink::test(sandbox = Pop)]
fn decrease_allowance_fails_with_insufficient_allowance(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Failed with `InsufficientAllowance`.
	assert_eq!(
		decrease_allowance(&mut session, ALICE, AMOUNT),
		Err(PSP22Error::InsufficientAllowance)
	);
}

#[drink::test(sandbox = Pop)]
fn decrease_allowance_fails_with_token_not_live(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &contract.clone(), &ALICE, AMOUNT));
	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_runtime_err!(
		decrease_allowance(&mut session, ALICE, AMOUNT),
		RuntimeError::Module(Assets(AssetNotLive))
	);
}

#[drink::test(sandbox = Pop)]
fn decrease_allowance_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	let value = 1;
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &contract.clone(), &ALICE, AMOUNT));
	// Successfully approve.
	assert_ok!(decrease_allowance(&mut session, ALICE, value));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT - value);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(&contract),
			spender: account_id_from_slice(&ALICE),
			value: AMOUNT - value,
		}
		.encode()
		.as_slice()
	);
	// Additive.
	assert_ok!(decrease_allowance(&mut session, ALICE, value));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT - value * 2);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(&contract),
			spender: account_id_from_slice(&ALICE),
			value: AMOUNT - value * 2,
		}
		.encode()
		.as_slice()
	);
}

// 2. PSP-22 Metadata Interface:
// - token_name
// - token_symbol
// - token_decimals

#[drink::test(sandbox = Pop)]
fn token_metadata(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	let name: String = String::from("Paseo Token");
	let symbol: String = String::from("PAS");
	let decimals: u8 = 69;
	// Token does not exist.
	assert_eq!(token_name(&mut session), None);
	assert_eq!(token_symbol(&mut session), None);
	assert_eq!(token_decimals(&mut session), 0);
	// Set token metadata.
	let actor = session.get_actor();
	assert_ok!(session.sandbox().set_metadata(
		Some(actor),
		&TOKEN,
		name.clone().into(),
		symbol.clone().into(),
		decimals
	));
	assert_eq!(token_name(&mut session), Some(name));
	assert_eq!(token_symbol(&mut session), Some(symbol));
	assert_eq!(token_decimals(&mut session), decimals);
}

// 3. PSP-22 Mintable Interface:
// - mint

#[drink::test(sandbox = Pop)]
fn mint_fails_with_no_permission(mut session: Session) {
	let _ = env_logger::try_init();
	// No permission to mint.
	// Create a new token owned by `BOB`.
	assert_ok!(session.sandbox().create(&(TOKEN + 1), &BOB, MIN_BALANCE));
	assert_ok!(deploy(&mut session, "existing", vec![(TOKEN + 1).to_string()]));
	// `pallet-assets` returns `NoPermission` error.
	assert_runtime_err!(
		mint(&mut session, BOB, AMOUNT),
		RuntimeError::Module(Assets(NoPermission))
	);
}

#[drink::test(sandbox = Pop)]
fn mint_noop_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// No-op if minted value is zero, returns success and no events are emitted.
	assert_ok!(mint(&mut session, ALICE, 0));
	assert_eq!(last_contract_event(&session), None);
}

#[drink::test(sandbox = Pop)]
fn mint_fails_with_arithmetic_overflow(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	assert_ok!(mint(&mut session, ALICE, AMOUNT));
	// Total supply increased by `value` exceeds maximal value of `u128` type.
	assert_runtime_err!(
		mint(&mut session, ALICE, u128::MAX),
		RuntimeError::Raw(Arithmetic(Overflow))
	);
}

#[drink::test(sandbox = Pop)]
fn mint_fails_with_token_not_live(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_runtime_err!(
		mint(&mut session, ALICE, AMOUNT),
		RuntimeError::Module(Assets(AssetNotLive))
	);
}

#[drink::test(sandbox = Pop)]
fn mint_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	let value = AMOUNT;
	// Successfully mint tokens.
	assert_ok!(mint(&mut session, ALICE, value));
	assert_eq!(session.sandbox().total_supply(&TOKEN), value);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), value);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Transfer { from: None, to: Some(account_id_from_slice(&ALICE)), value }
			.encode()
			.as_slice()
	);
}

// 4. PSP-22 Burnable Interface:
// - burn

#[drink::test(sandbox = Pop)]
fn burn_fails_with_no_permission(mut session: Session) {
	let _ = env_logger::try_init();
	// No permission to burn.
	// Create a new token owned by `BOB`.
	assert_ok!(session.sandbox().create(&(TOKEN + 1), &BOB, MIN_BALANCE));
	assert_ok!(session.sandbox().mint_into(&(TOKEN + 1), &BOB, AMOUNT));
	assert_ok!(deploy(&mut session, "existing", vec![(TOKEN + 1).to_string()]));
	// `pallet-assets` returns `NoPermission` error.
	assert_runtime_err!(
		burn(&mut session, BOB, AMOUNT),
		RuntimeError::Module(Assets(NoPermission))
	);
}

#[drink::test(sandbox = Pop)]
fn burn_noop_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// No-op if burned value is zero, returns success and no events are emitted.
	assert_ok!(burn(&mut session, ALICE, 0));
	assert_eq!(last_contract_event(&session), None);
}

#[drink::test(sandbox = Pop)]
fn burn_fails_with_insufficient_balance(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	// Failed with `InsufficientBalance`.
	assert_eq!(burn(&mut session, ALICE, AMOUNT), Err(PSP22Error::InsufficientBalance));
}

#[drink::test(sandbox = Pop)]
fn burn_fails_with_token_not_live(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `IncorrectStatus` error.
	assert_runtime_err!(
		burn(&mut session, ALICE, AMOUNT),
		RuntimeError::Module(Assets(IncorrectStatus))
	);
}

#[drink::test(sandbox = Pop)]
fn burn_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract =
		deploy(&mut session, "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()]).unwrap();
	session.set_actor(contract.clone());
	let value = 1;
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	// Successfully burn tokens.
	assert_ok!(burn(&mut session, ALICE, value));
	assert_eq!(session.sandbox().total_supply(&TOKEN), AMOUNT - value);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), AMOUNT - value);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Transfer { from: Some(account_id_from_slice(&ALICE)), to: None, value }
			.encode()
			.as_slice()
	);
}

// Deploy the contract with `NO_SALT and `INIT_VALUE`.
fn deploy(
	session: &mut Session<Pop>,
	method: &str,
	input: Vec<String>,
) -> Result<AccountId, PSP22Error> {
	drink::utils::deploy::<Pop, PSP22Error>(
		session,
		// The local contract (i.e. `fungibles`).
		BundleProvider::local().unwrap(),
		method,
		input,
		NO_SALT,
		Some(INIT_VALUE),
	)
}

// A set of helper methods to test the contract calls.

fn total_supply(session: &mut Session<Pop>) -> Balance {
	call::<Pop, Balance, PSP22Error>(session, "Psp22::total_supply", vec![], None).unwrap()
}

fn balance_of(session: &mut Session<Pop>, owner: AccountId) -> Balance {
	call::<Pop, Balance, PSP22Error>(session, "Psp22::balance_of", vec![owner.to_string()], None)
		.unwrap()
}

fn allowance(session: &mut Session<Pop>, owner: AccountId, spender: AccountId) -> Balance {
	call::<Pop, Balance, PSP22Error>(
		session,
		"Psp22::allowance",
		vec![owner.to_string(), spender.to_string()],
		None,
	)
	.unwrap()
}

fn transfer(session: &mut Session<Pop>, to: AccountId, amount: Balance) -> Result<(), PSP22Error> {
	call::<Pop, (), PSP22Error>(
		session,
		"Psp22::transfer",
		vec![to.to_string(), amount.to_string(), serde_json::to_string::<[u8; 0]>(&[]).unwrap()],
		None,
	)
}

fn transfer_from(
	session: &mut Session<Pop>,
	from: AccountId,
	to: AccountId,
	amount: Balance,
) -> Result<(), PSP22Error> {
	call::<Pop, (), PSP22Error>(
		session,
		"Psp22::transfer_from",
		vec![
			from.to_string(),
			to.to_string(),
			amount.to_string(),
			serde_json::to_string::<[u8; 0]>(&[]).unwrap(),
		],
		None,
	)
}

fn approve(
	session: &mut Session<Pop>,
	spender: AccountId,
	value: Balance,
) -> Result<(), PSP22Error> {
	call::<Pop, (), PSP22Error>(
		session,
		"Psp22::approve",
		vec![spender.to_string(), value.to_string()],
		None,
	)
}

fn increase_allowance(
	session: &mut Session<Pop>,
	spender: AccountId,
	value: Balance,
) -> Result<(), PSP22Error> {
	call::<Pop, (), PSP22Error>(
		session,
		"Psp22::increase_allowance",
		vec![spender.to_string(), value.to_string()],
		None,
	)
}

fn decrease_allowance(
	session: &mut Session<Pop>,
	spender: AccountId,
	value: Balance,
) -> Result<(), PSP22Error> {
	call::<Pop, (), PSP22Error>(
		session,
		"Psp22::decrease_allowance",
		vec![spender.to_string(), value.to_string()],
		None,
	)
}

fn token_name(session: &mut Session<Pop>) -> Option<String> {
	call::<Pop, Option<String>, PSP22Error>(session, "Psp22Metadata::token_name", vec![], None)
		.unwrap()
}

fn token_symbol(session: &mut Session<Pop>) -> Option<String> {
	call::<Pop, Option<String>, PSP22Error>(session, "Psp22Metadata::token_symbol", vec![], None)
		.unwrap()
}

fn token_decimals(session: &mut Session<Pop>) -> u8 {
	call::<Pop, u8, PSP22Error>(session, "Psp22Metadata::token_decimals", vec![], None).unwrap()
}

fn mint(session: &mut Session<Pop>, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
	call::<Pop, (), PSP22Error>(
		session,
		"Psp22Mintable::mint",
		vec![account.to_string(), amount.to_string()],
		None,
	)
}

fn burn(session: &mut Session<Pop>, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
	call::<Pop, (), PSP22Error>(
		session,
		"Psp22Burnable::burn",
		vec![account.to_string(), amount.to_string()],
		None,
	)
}

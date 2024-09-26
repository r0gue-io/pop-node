use super::*;
use drink::{
	sandbox_api::assets_api::AssetsAPI,
	session::{ContractBundle, Session},
};
use frame_support::assert_ok;
use pop_api::{
	primitives::{
		ArithmeticError::Overflow,
		Error::{Arithmetic, Module},
		TokenId,
	},
	v0::fungibles::events::{Approval, Created, Transfer},
};
use pop_sandbox::{Balance, DevnetSandbox as Sandbox, ALICE, BOB};
use scale::Encode;
use std::sync::LazyLock;
use utils::*;

#[drink::contract_bundle_provider]
enum BundleProvider {}

const AMOUNT: Balance = MIN_BALANCE * 4;
const MIN_BALANCE: Balance = 10_000;
const TOKEN: TokenId = 1;

static CONTRACT: LazyLock<ContractBundle> = LazyLock::new(|| BundleProvider::local().unwrap());

/// Deployment and constructor method tests.

#[drink::test(sandbox = Sandbox)]
fn new_constructor_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy(
		&mut session,
		CONTRACT.clone(),
		"new",
		vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
	)
	.unwrap();
	// Token exists after the deployment.
	assert!(session.sandbox().asset_exists(&TOKEN));
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Created {
			id: TOKEN,
			creator: account_id_from_slice(contract.as_ref()),
			admin: account_id_from_slice(contract.as_ref()),
		}
		.encode()
		.as_slice()
	);
}

#[drink::test(sandbox = Sandbox)]
fn existing_constructor_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Fails to deploy contract with a non-existing token ID.
	assert_eq!(
		deploy(&mut session, CONTRACT.clone(), "existing", vec![TOKEN.to_string()]),
		Err(PSP22Error::Custom(String::from("Unknown")))
	);

	// Successfully deploy contract with an existing token ID.
	let actor = session.get_actor();
	session.sandbox().create(&TOKEN, &actor, MIN_BALANCE).unwrap();
	deploy(&mut session, CONTRACT.clone(), "existing", vec![TOKEN.to_string()]).unwrap();
	assert!(session.sandbox().asset_exists(&TOKEN));
}

/// 1. PSP-22 Interface:
/// - total_supply
/// - balance_of
/// - allowance
/// - transfer
/// - transfer_from
/// - approve
/// - increase_allowance
/// - decrease_allowance

#[drink::test(sandbox = Sandbox)]
fn total_supply_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy(&mut session, CONTRACT.clone(), "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()])
		.unwrap();

	// No tokens in circulation.
	assert_eq!(total_supply(&mut session), 0);
	assert_eq!(total_supply(&mut session), session.sandbox().total_supply(&TOKEN));

	// Tokens in circulation.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_ok!(session.sandbox().mint_into(&TOKEN, &BOB, AMOUNT * 2));
	assert_eq!(total_supply(&mut session), AMOUNT * 3);
	assert_eq!(total_supply(&mut session), session.sandbox().total_supply(&TOKEN));
}

#[drink::test(sandbox = Sandbox)]
fn balance_of_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy(&mut session, CONTRACT.clone(), "new", vec![TOKEN.to_string(), MIN_BALANCE.to_string()])
		.unwrap();

	// No tokens in circulation.
	assert_eq!(balance_of(&mut session, ALICE), 0);
	assert_eq!(balance_of(&mut session, ALICE), session.sandbox().balance_of(&TOKEN, &ALICE));

	// Tokens in circulation.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_eq!(balance_of(&mut session, ALICE), AMOUNT);
	assert_eq!(balance_of(&mut session, ALICE), session.sandbox().balance_of(&TOKEN, &ALICE));
}

#[drink::test(sandbox = Sandbox)]
fn allowance_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy(
		&mut session,
		CONTRACT.clone(),
		"new",
		vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
	)
	.unwrap();

	// No tokens in circulation.
	assert_eq!(allowance(&mut session, contract.clone(), ALICE), 0);

	// Tokens in circulation.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &contract.clone(), &ALICE, AMOUNT / 2));
	assert_eq!(allowance(&mut session, contract, ALICE), AMOUNT / 2);
}

#[drink::test(sandbox = Sandbox)]
fn transfer_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy(
		&mut session,
		CONTRACT.clone(),
		"new",
		vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
	)
	.unwrap();
	session.set_actor(contract.clone());
	// `pallet-assets` returns `NoAccount` error.
	assert_eq!(
		transfer(&mut session, ALICE, AMOUNT / 4),
		Err(PSP22Error::Custom(
			vec_u8_to_u32(Module { index: 52, error: [1, 0] }.encode()).unwrap().to_string()
		))
	);

	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().mint_into(&TOKEN, &BOB, AMOUNT));

	// Successfully transfer.
	assert_ok!(transfer(&mut session, BOB, AMOUNT / 4));
	assert_eq!(session.sandbox().balance_of(&TOKEN, &contract), AMOUNT - AMOUNT / 4);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &BOB), AMOUNT + AMOUNT / 4);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Transfer {
			from: Some(account_id_from_slice(contract.as_ref())),
			to: Some(account_id_from_slice(BOB.as_ref())),
			value: AMOUNT / 4,
		}
		.encode()
		.as_slice()
	);

	// No-op if `value` is zero.
	assert_ok!(transfer(&mut session, ALICE, 0));
	assert_eq!(last_contract_event(&session), None); // No event emitted.
	assert_eq!(session.sandbox().balance_of(&TOKEN, &contract), AMOUNT - AMOUNT / 4);

	// No-op if the caller and `to` is the same address, returns success and no events are emitted.
	assert_ok!(transfer(&mut session, contract.clone(), AMOUNT / 4));
	assert_eq!(last_contract_event(&session), None); // No event emitted.
	assert_eq!(session.sandbox().balance_of(&TOKEN, &contract), AMOUNT - AMOUNT / 4);

	// Failed with `InsufficientBalance`.
	assert_eq!(transfer(&mut session, BOB, AMOUNT + 1), Err(PSP22Error::InsufficientBalance));
	assert_eq!(session.sandbox().balance_of(&TOKEN, &contract), AMOUNT - AMOUNT / 4);

	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_eq!(
		transfer(&mut session, BOB, AMOUNT / 4),
		Err(PSP22Error::Custom(
			vec_u8_to_u32(Module { index: 52, error: [16, 0] }.encode())
				.unwrap()
				.to_string()
		))
	);
}

#[drink::test(sandbox = Sandbox)]
fn transfer_from_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy(
		&mut session,
		CONTRACT.clone(),
		"new",
		vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
	)
	.unwrap();
	session.set_actor(contract.clone());
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &ALICE, &contract.clone(), AMOUNT * 2));

	// No-op if `value` is zero.
	assert_ok!(transfer_from(&mut session, ALICE, BOB, 0));
	assert_eq!(last_contract_event(&session), None); // No event emitted.

	// No-op if the `from` and `to` is the same address, returns success and no events are emitted.
	assert_ok!(transfer_from(&mut session, ALICE, ALICE, AMOUNT / 2));
	assert_eq!(last_contract_event(&session), None); // No event emitted.

	// Not enough balance. Failed with `InsufficientBalance`.
	assert_eq!(
		transfer_from(&mut session, ALICE, contract.clone(), AMOUNT + 1),
		Err(PSP22Error::InsufficientBalance)
	);

	// Unapproved transfer. Failed with `InsufficientAllowance`.
	assert_eq!(
		transfer_from(&mut session, ALICE, contract.clone(), AMOUNT * 2 + 1),
		Err(PSP22Error::InsufficientAllowance)
	);

	// Successful transfer.
	assert_ok!(transfer_from(&mut session, ALICE, BOB, AMOUNT / 2));
	assert_eq!(session.sandbox().allowance(&TOKEN, &ALICE, &contract.clone()), AMOUNT + AMOUNT / 2);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &contract), 0);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), AMOUNT / 2);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &BOB), AMOUNT / 2);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(ALICE.as_ref()),
			spender: account_id_from_slice(contract.as_ref()),
			value: AMOUNT + AMOUNT / 2,
		}
		.encode()
		.as_slice()
	);

	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_eq!(
		transfer_from(&mut session, ALICE, BOB, AMOUNT / 2),
		Err(PSP22Error::Custom(
			vec_u8_to_u32(Module { index: 52, error: [16, 0] }.encode())
				.unwrap()
				.to_string()
		))
	);
}

#[drink::test(sandbox = Sandbox)]
fn approve_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy(
		&mut session,
		CONTRACT.clone(),
		"new",
		vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
	)
	.unwrap();
	session.set_actor(contract.clone());
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));

	// Successfully approvals.
	assert_ok!(approve(&mut session, ALICE, AMOUNT / 2));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT / 2);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(contract.clone().as_ref()),
			spender: account_id_from_slice(ALICE.as_ref()),
			value: AMOUNT / 2,
		}
		.encode()
		.as_slice()
	);

	// No-op if the caller and `spender` is the same address, returns success and no events are
	// emitted.
	assert_ok!(approve(&mut session, contract.clone(), AMOUNT / 2));
	assert_eq!(last_contract_event(&session), None); // No event emitted.
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT / 2);

	// Non-additive, sets new value.
	assert_ok!(approve(&mut session, ALICE, AMOUNT / 2 - 1));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT / 2 - 1);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(contract.as_ref()),
			spender: account_id_from_slice(ALICE.as_ref()),
			value: AMOUNT / 2 - 1,
		}
		.encode()
		.as_slice()
	);

	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_eq!(
		approve(&mut session, ALICE, AMOUNT / 2),
		Err(PSP22Error::Custom(
			vec_u8_to_u32(Module { index: 52, error: [16, 0] }.encode())
				.unwrap()
				.to_string()
		))
	);
}

#[drink::test(sandbox = Sandbox)]
fn increase_allowance_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy(
		&mut session,
		CONTRACT.clone(),
		"new",
		vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
	)
	.unwrap();
	session.set_actor(contract.clone());
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &contract.clone(), &ALICE, AMOUNT / 2));

	// Successfully approvals.
	assert_ok!(increase_allowance(&mut session, ALICE, AMOUNT / 2));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(contract.as_ref()),
			spender: account_id_from_slice(ALICE.as_ref()),
			value: AMOUNT,
		}
		.encode()
		.as_slice()
	);

	// No-op if the caller and `spender` is the same address, returns success and no events are
	// emitted.
	assert_ok!(increase_allowance(&mut session, contract.clone(), AMOUNT / 2));
	assert_eq!(last_contract_event(&session), None); // No event emitted.
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT);

	// No-op if the `value` is zero.
	assert_ok!(increase_allowance(&mut session, contract.clone(), 0));
	assert_eq!(last_contract_event(&session), None); // No event emitted.
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT);

	// Additive.
	assert_ok!(increase_allowance(&mut session, ALICE, AMOUNT / 2));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT + AMOUNT / 2);

	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_eq!(
		increase_allowance(&mut session, ALICE, AMOUNT / 2),
		Err(PSP22Error::Custom(
			vec_u8_to_u32(Module { index: 52, error: [16, 0] }.encode())
				.unwrap()
				.to_string()
		))
	);
}

#[drink::test(sandbox = Sandbox)]
fn decrease_allowance_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy(
		&mut session,
		CONTRACT.clone(),
		"new",
		vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
	)
	.unwrap();
	session.set_actor(contract.clone());
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &contract.clone(), &ALICE, AMOUNT / 2));

	// Successfully approvals.
	assert_ok!(decrease_allowance(&mut session, ALICE, 1));
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT / 2 - 1);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Approval {
			owner: account_id_from_slice(contract.as_ref()),
			spender: account_id_from_slice(ALICE.as_ref()),
			value: AMOUNT / 2 - 1,
		}
		.encode()
		.as_slice()
	);

	// No-op if the caller and `spender` is the same address, returns success and no events are
	// emitted.
	assert_ok!(decrease_allowance(&mut session, contract.clone(), 1));
	assert_eq!(last_contract_event(&session), None); // No event emitted.
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT / 2 - 1);

	// Failed with `InsufficientAllowance`.
	assert_eq!(
		decrease_allowance(&mut session, ALICE, AMOUNT),
		Err(PSP22Error::InsufficientAllowance)
	);
	assert_eq!(session.sandbox().allowance(&TOKEN, &contract, &ALICE), AMOUNT / 2 - 1);

	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_eq!(
		decrease_allowance(&mut session, ALICE, 1),
		Err(PSP22Error::Custom(
			vec_u8_to_u32(Module { index: 52, error: [16, 0] }.encode())
				.unwrap()
				.to_string()
		))
	);
}

/// 2. PSP-22 Metadata Interface:
/// - token_name
/// - token_symbol
/// - token_decimals

#[drink::test(sandbox = Sandbox)]
fn token_metadata(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy(
		&mut session,
		CONTRACT.clone(),
		"new",
		vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
	)
	.unwrap();
	let name: String = String::from("Paseo Token");
	let symbol: String = String::from("PAS");
	let decimals: u8 = 69;

	session.set_actor(contract.clone());
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

/// 3. PSP-22 Mintable Interface:
/// - mint

#[drink::test(sandbox = Sandbox)]
fn mint_works(mut session: Session) {
	let _ = env_logger::try_init();
	// No permission to mint.
	assert_ok!(session.sandbox().create(&(TOKEN + 1), &BOB, MIN_BALANCE)); // Create a new token owned by `BOB`.
	deploy(&mut session, CONTRACT.clone(), "existing", vec![(TOKEN + 1).to_string()]).unwrap();
	// `pallet-assets` returns `NoPermission` error.
	assert_eq!(
		mint(&mut session, BOB, AMOUNT),
		Err(PSP22Error::Custom(
			vec_u8_to_u32(Module { index: 52, error: [2, 0] }.encode()).unwrap().to_string()
		))
	);

	// Deploy a new contract.
	let contract = deploy(
		&mut session,
		CONTRACT.clone(),
		"new",
		vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
	)
	.unwrap();
	session.set_actor(contract.clone());

	// Successfully mint tokens.
	assert_ok!(mint(&mut session, ALICE, AMOUNT));
	assert_eq!(session.sandbox().total_supply(&TOKEN), AMOUNT);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), AMOUNT);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Transfer { from: None, to: Some(account_id_from_slice(ALICE.as_ref())), value: AMOUNT }
			.encode()
			.as_slice()
	);

	// No-op if minted value is zero.
	assert_ok!(mint(&mut session, ALICE, 0));
	assert_eq!(last_contract_event(&session), None); // No event emitted.
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), AMOUNT);

	// Total supply increased by `value` exceeds maximal value of `u128` type.
	assert_eq!(
		mint(&mut session, ALICE, u128::MAX),
		Err(PSP22Error::Custom(vec_u8_to_u32(Arithmetic(Overflow).encode()).unwrap().to_string()))
	);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), AMOUNT);

	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `AssetNotLive` error.
	assert_eq!(
		mint(&mut session, ALICE, AMOUNT),
		Err(PSP22Error::Custom(
			vec_u8_to_u32(Module { index: 52, error: [16, 0] }.encode())
				.unwrap()
				.to_string()
		))
	);
}

/// 4. PSP-22 Burnable Interface:
/// - burn

#[drink::test(sandbox = Sandbox)]
fn burn_works(mut session: Session) {
	let _ = env_logger::try_init();
	// No permission to burn.
	assert_ok!(session.sandbox().create(&(TOKEN + 1), &BOB, MIN_BALANCE)); // Create a new token owned by `BOB`.
	assert_ok!(session.sandbox().mint_into(&(TOKEN + 1), &BOB, AMOUNT));
	deploy(&mut session, CONTRACT.clone(), "existing", vec![(TOKEN + 1).to_string()]).unwrap();
	// `pallet-assets` returns `NoPermission` error.
	assert_eq!(
		burn(&mut session, BOB, AMOUNT),
		Err(PSP22Error::Custom(
			vec_u8_to_u32(Module { index: 52, error: [2, 0] }.encode()).unwrap().to_string()
		))
	);

	// Deploy a new contract.
	let contract = deploy(
		&mut session,
		CONTRACT.clone(),
		"new",
		vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
	)
	.unwrap();
	session.set_actor(contract.clone());
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));

	// Successfully burn tokens.
	assert_ok!(burn(&mut session, ALICE, 1));
	assert_eq!(session.sandbox().total_supply(&TOKEN), AMOUNT - 1);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), AMOUNT - 1);
	// Successfully emit event.
	assert_eq!(
		last_contract_event(&session).unwrap(),
		Transfer { from: Some(account_id_from_slice(ALICE.as_ref())), to: None, value: 1 }
			.encode()
			.as_slice()
	);

	// No-op.
	assert_ok!(burn(&mut session, ALICE, 0));
	assert_eq!(last_contract_event(&session), None); // No event emitted.
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), AMOUNT - 1);

	// Failed with `InsufficientBalance`.
	assert_eq!(burn(&mut session, ALICE, AMOUNT * 2), Err(PSP22Error::InsufficientBalance));
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), AMOUNT - 1);

	// Token is not live, i.e. frozen or being destroyed.
	assert_ok!(session.sandbox().start_destroy(&TOKEN));
	// `pallet-assets` returns `IncorrectStatus` error.
	assert_eq!(
		burn(&mut session, ALICE, 1),
		Err(PSP22Error::Custom(
			vec_u8_to_u32(Module { index: 52, error: [17, 0] }.encode())
				.unwrap()
				.to_string()
		))
	);
}

/// A set of helper methods to test the contract calls.
mod utils {
	use drink::session::{error::SessionError, NO_SALT};
	use pop_api::primitives::AccountId;
	use pop_sandbox::{AccountId32, Balance, INIT_VALUE};
	use scale::Decode;

	use super::*;

	/// Deploy test utilities.

	pub(super) fn deploy(
		session: &mut Session<Sandbox>,
		bundle: ContractBundle,
		method: &str,
		inputs: Vec<String>,
	) -> Result<AccountId32, PSP22Error> {
		let result = session.deploy_bundle(bundle, method, &inputs, NO_SALT, Some(INIT_VALUE));
		if result.is_err() {
			let deployment_result = session.record().last_deploy_result().result.clone();
			let error = deployment_result.unwrap().result.data;
			return Err(PSP22Error::decode(&mut &error[2..]).unwrap());
		}
		Ok(result.unwrap())
	}

	/// PSP22 test utilities.

	pub(super) fn total_supply(session: &mut Session<Sandbox>) -> Balance {
		call::<Balance>(session, "Psp22::total_supply", vec![], None).unwrap()
	}

	pub(super) fn balance_of(session: &mut Session<Sandbox>, owner: AccountId32) -> Balance {
		call::<Balance>(session, "Psp22::balance_of", vec![owner.to_string()], None).unwrap()
	}

	pub(super) fn allowance(
		session: &mut Session<Sandbox>,
		owner: AccountId32,
		spender: AccountId32,
	) -> Balance {
		call::<Balance>(
			session,
			"Psp22::allowance",
			vec![owner.to_string(), spender.to_string()],
			None,
		)
		.unwrap()
	}

	pub(super) fn transfer(
		session: &mut Session<Sandbox>,
		to: AccountId32,
		amount: Balance,
	) -> Result<(), PSP22Error> {
		call::<()>(
			session,
			"Psp22::transfer",
			vec![
				to.to_string(),
				amount.to_string(),
				serde_json::to_string::<[u8; 0]>(&[]).unwrap(),
			],
			None,
		)
	}

	pub(super) fn transfer_from(
		session: &mut Session<Sandbox>,
		from: AccountId32,
		to: AccountId32,
		amount: Balance,
	) -> Result<(), PSP22Error> {
		call::<()>(
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

	pub(super) fn approve(
		session: &mut Session<Sandbox>,
		spender: AccountId32,
		value: Balance,
	) -> Result<(), PSP22Error> {
		call::<()>(session, "Psp22::approve", vec![spender.to_string(), value.to_string()], None)
	}

	pub(super) fn increase_allowance(
		session: &mut Session<Sandbox>,
		spender: AccountId32,
		value: Balance,
	) -> Result<(), PSP22Error> {
		call::<()>(
			session,
			"Psp22::increase_allowance",
			vec![spender.to_string(), value.to_string()],
			None,
		)
	}

	pub(super) fn decrease_allowance(
		session: &mut Session<Sandbox>,
		spender: AccountId32,
		value: Balance,
	) -> Result<(), PSP22Error> {
		call::<()>(
			session,
			"Psp22::decrease_allowance",
			vec![spender.to_string(), value.to_string()],
			None,
		)
	}

	/// PSP22Metadata test utilities.

	pub(super) fn token_name(session: &mut Session<Sandbox>) -> Option<String> {
		call::<Option<String>>(session, "Psp22Metadata::token_name", vec![], None).unwrap()
	}

	pub(super) fn token_symbol(session: &mut Session<Sandbox>) -> Option<String> {
		call::<Option<String>>(session, "Psp22Metadata::token_symbol", vec![], None).unwrap()
	}

	pub(super) fn token_decimals(session: &mut Session<Sandbox>) -> u8 {
		call::<u8>(session, "Psp22Metadata::token_decimals", vec![], None).unwrap()
	}

	/// PSP22Mintable test utilities.

	pub(super) fn mint(
		session: &mut Session<Sandbox>,
		account: AccountId32,
		amount: Balance,
	) -> Result<(), PSP22Error> {
		call::<()>(
			session,
			"Psp22Mintable::mint",
			vec![account.to_string(), amount.to_string()],
			None,
		)
	}

	/// PSP22Burnable test utilities.

	pub(super) fn burn(
		session: &mut Session<Sandbox>,
		account: AccountId32,
		amount: Balance,
	) -> Result<(), PSP22Error> {
		call::<()>(
			session,
			"Psp22Burnable::burn",
			vec![account.to_string(), amount.to_string()],
			None,
		)
	}

	/// Call a contract method and decode the returned data.
	pub(super) fn call<T: Decode>(
		session: &mut Session<Sandbox>,
		func_name: &str,
		input: Vec<String>,
		endowment: Option<Balance>,
	) -> Result<T, PSP22Error> {
		match session.call::<String, ()>(func_name, &input, endowment) {
			// If the call is reverted, decode the error into `PSP22Error`.
			Err(SessionError::CallReverted(error)) => {
				Err(PSP22Error::decode(&mut &error[2..])
					.unwrap_or_else(|_| panic!("Decoding failed")))
			},
			// If the call is successful, decode the last returned value.
			Ok(_) => Ok(session
				.record()
				.last_call_return_decoded::<T>()
				.unwrap_or_else(|_| panic!("Expected a return value"))
				.unwrap_or_else(|_| panic!("Decoding failed"))),
			// Catch-all for unexpected results.
			_ => panic!("Expected call to revert or return a value"),
		}
	}

	/// This is used to resolve type mismatches between the `AccountId` in the quasi testing
	/// environment and the contract environment.
	pub(super) fn account_id_from_slice(s: &[u8; 32]) -> AccountId {
		AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
	}

	/// Get the last event from pallet contracts.
	pub(super) fn last_contract_event(session: &Session<Sandbox>) -> Option<Vec<u8>> {
		session.record().last_event_batch().contract_events().last().cloned()
	}

	/// Convert the Vec<u8> to u32.
	pub(super) fn vec_u8_to_u32(vec: Vec<u8>) -> Result<u32, &'static str> {
		let mut padded_vec = vec.to_vec();
		padded_vec.resize(4, 0);

		let array: [u8; 4] = padded_vec.try_into().map_err(|_| "Invalid length")?;
		Ok(u32::from_le_bytes(array))
	}
}

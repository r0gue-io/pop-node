use drink::{
	sandbox_api::assets_api::AssetsAPI,
	session::{error::SessionError, Session},
};
use frame_support::assert_ok;
use pop_api::{
	primitives::TokenId,
	v0::fungibles::events::{Approval, Created, Transfer},
};
use pop_sandbox::{Balance, DevnetSandbox as Sandbox, ALICE, BOB};
use scale::Encode;
use utils::*;

use super::*;

const TOKEN_ID: TokenId = 1;
const TOKEN_MIN_BALANCE: Balance = 10_000;

#[drink::contract_bundle_provider]
enum BundleProvider {}

#[drink::test(sandbox = Sandbox)]
fn new_constructor_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;

	// Token exists after the deployment.
	assert!(session.sandbox().asset_exists(&TOKEN_ID));
	let expected = Created {
		id: TOKEN_ID,
		creator: account_id_from_slice(contract.as_ref()),
		admin: account_id_from_slice(contract.as_ref()),
	}
	.encode();
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice()); // Successfully emit event.
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn existing_constructor_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();

	// Fails to deploy contract with a non-existing token ID.
	assert!(
		deploy_with_existing_constructor(&mut session, BundleProvider::local()?, TOKEN_ID).is_err()
	);
	expect_deployment_reverted(&session, PSP22Error::Custom(String::from("Unknown")));

	// Successfully deploy contract with an existing token ID.
	let actor = session.get_actor();
	session.sandbox().create(&TOKEN_ID, &actor, TOKEN_MIN_BALANCE).unwrap();
	deploy_with_existing_constructor(&mut session, BundleProvider::local()?, TOKEN_ID)?;

	assert!(session.sandbox().asset_exists(&TOKEN_ID));
	assert_eq!(last_contract_event(&session), None); // No event emitted.
	assert_eq!(session.sandbox().total_supply(&TOKEN_ID), 0);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn balance_of_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;

	const AMOUNT: Balance = TOKEN_MIN_BALANCE * 4;
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN_ID, &ALICE, AMOUNT));

	// Successfully return a correct balance.
	assert_eq!(balance_of(&mut session, ALICE), AMOUNT);
	assert_eq!(balance_of(&mut session, ALICE), session.sandbox().balance_of(&TOKEN_ID, &ALICE));
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn mint_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;

	const AMOUNT: Balance = TOKEN_MIN_BALANCE * 4;

	// Successfully mint tokens.
	assert_ok!(mint(&mut session, ALICE, 0));
	assert_eq!(last_contract_event(&session), None); // No event emitted.
	assert_eq!(session.sandbox().total_supply(&TOKEN_ID), 0);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &ALICE), 0);

	session.set_actor(contract.clone());
	// Mint tokens.
	assert_ok!(mint(&mut session, ALICE, AMOUNT));
	let expected =
		Transfer { from: None, to: Some(account_id_from_slice(ALICE.as_ref())), value: AMOUNT }
			.encode();
	// A `Transfer` event is emitted.
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());

	assert_eq!(session.sandbox().total_supply(&TOKEN_ID), AMOUNT);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &ALICE), AMOUNT);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn burn_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;

	const AMOUNT: Balance = TOKEN_MIN_BALANCE * 4;
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN_ID, &ALICE, AMOUNT));

	// No-op.
	assert_ok!(burn(&mut session, ALICE, 0));
	assert_eq!(last_contract_event(&session), None); // No event emitted.

	// Failed with `InsufficientBalance`.
	expect_call_reverted(
		&mut session,
		BURN,
		vec![ALICE.to_string(), (AMOUNT * 2).to_string()],
		PSP22Error::InsufficientBalance,
	);

	// Successfully burn tokens.
	assert_ok!(burn(&mut session, ALICE, 1));
	let expected =
		Transfer { from: Some(account_id_from_slice(ALICE.as_ref())), to: None, value: 1 }.encode();
	// A `Transfer` event is emitted.
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());

	assert_eq!(session.sandbox().total_supply(&TOKEN_ID), AMOUNT - 1);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &ALICE), AMOUNT - 1);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn transfer_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;

	const AMOUNT: Balance = TOKEN_MIN_BALANCE * 4;
	const TRANSFERRED: Balance = TOKEN_MIN_BALANCE;
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN_ID, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().mint_into(&TOKEN_ID, &BOB, AMOUNT));
	// Check balance of accounts.
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &contract.clone()), AMOUNT);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &BOB), AMOUNT);

	session.set_actor(contract.clone());
	// No-op if `value` is zero.
	assert_ok!(transfer(&mut session, ALICE, 0));
	assert_eq!(last_contract_event(&session), None); // No event emitted.

	// No-op if the caller and `to` is the same address, returns success and no events are emitted.
	assert_ok!(transfer(&mut session, contract.clone(), TRANSFERRED));
	assert_eq!(last_contract_event(&session), None); // No event emitted.

	// Failed with `InsufficientBalance`.
	let data = serde_json::to_string::<[u8; 0]>(&[]).unwrap();
	expect_call_reverted(
		&mut session,
		TRANSFER,
		vec![BOB.to_string(), (AMOUNT + 1).to_string(), data],
		PSP22Error::InsufficientBalance,
	);
	// Make sure balance is not changed.
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &contract), AMOUNT);

	// Successfully transfer tokens from `contract` to `account`.
	assert_ok!(transfer(&mut session, BOB, TRANSFERRED));
	let expected = Transfer {
		from: Some(account_id_from_slice(contract.clone().as_ref())),
		to: Some(account_id_from_slice(BOB.as_ref())),
		value: TRANSFERRED,
	}
	.encode();
	// A `Transfer` event is emitted.
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());

	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &contract), AMOUNT - TRANSFERRED);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &BOB), AMOUNT + TRANSFERRED);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn transfer_from_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;

	const AMOUNT: Balance = TOKEN_MIN_BALANCE * 4;
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN_ID, &ALICE, AMOUNT));
	// Approve `contract` to spend `ALICE` tokens.
	assert_ok!(session.sandbox().approve(&TOKEN_ID, &ALICE, &contract.clone(), AMOUNT * 2));
	assert_eq!(session.sandbox().allowance(&TOKEN_ID, &ALICE, &contract.clone()), AMOUNT * 2);
	// Check balance of accounts.
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &contract), 0);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &ALICE), AMOUNT);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &BOB), 0);

	session.set_actor(contract.clone());
	// Failed with `InsufficientAllowance`.
	let data = serde_json::to_string::<[u8; 0]>(&[]).unwrap();
	expect_call_reverted(
		&mut session,
		TRANSFER_FROM,
		vec![ALICE.to_string(), contract.clone().to_string(), (AMOUNT * 2 + 1).to_string(), data],
		PSP22Error::InsufficientAllowance,
	);
	// Make sure balances are unchaged.
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &contract), 0);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &ALICE), AMOUNT);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &BOB), 0);

	session.set_actor(contract.clone());
	// No-op if `value` is zero.
	assert_ok!(transfer_from(&mut session, ALICE, BOB, 0));
	assert_eq!(last_contract_event(&session), None); // No event emitted.

	// No-op if the `from` and `to` is the same address, returns success and no events are emitted.
	assert_ok!(transfer_from(&mut session, ALICE, ALICE, AMOUNT / 2));
	assert_eq!(last_contract_event(&session), None); // No event emitted.

	// Failed with `InsufficientBalance`.
	let data = serde_json::to_string::<[u8; 0]>(&[]).unwrap();
	expect_call_reverted(
		&mut session,
		TRANSFER_FROM,
		vec![ALICE.to_string(), contract.clone().to_string(), (AMOUNT + 1).to_string(), data],
		PSP22Error::InsufficientBalance,
	);
	// Make sure balances are unchaged.
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &contract), 0);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &ALICE), AMOUNT);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &BOB), 0);

	session.set_actor(contract.clone());
	// Successfully transfer from `owner`.
	assert_ok!(transfer_from(&mut session, ALICE, BOB, AMOUNT / 2));
	// Successfully emit event.
	let expected = Approval {
		owner: account_id_from_slice(ALICE.as_ref()),
		spender: account_id_from_slice(contract.clone().as_ref()),
		value: AMOUNT + AMOUNT / 2,
	}
	.encode();
	// An `Approval` event is emitted.
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());
	assert_eq!(
		session.sandbox().allowance(&TOKEN_ID, &ALICE, &contract.clone()),
		AMOUNT + AMOUNT / 2
	);
	// Check balance of accounts.
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &contract), 0);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &ALICE), AMOUNT / 2);
	assert_eq!(session.sandbox().balance_of(&TOKEN_ID, &BOB), AMOUNT / 2);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn allowance_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;

	const AMOUNT: Balance = TOKEN_MIN_BALANCE * 4;
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN_ID, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN_ID, &contract.clone(), &ALICE, AMOUNT / 2));

	// Successfully return a correct allowance.
	assert_eq!(allowance(&mut session, contract, ALICE), AMOUNT / 2);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn approve_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;

	const AMOUNT: Balance = TOKEN_MIN_BALANCE * 4;
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN_ID, &contract.clone(), AMOUNT));

	session.set_actor(contract.clone());
	// No-op if the caller and `spender` is the same address, returns success and no events are
	// emitted.
	assert_ok!(approve(&mut session, contract.clone(), AMOUNT / 2));
	assert_eq!(last_contract_event(&session), None); // No event emitted.

	// Successfully approve.
	assert_ok!(approve(&mut session, ALICE, AMOUNT / 2));
	let expected = Approval {
		owner: account_id_from_slice(contract.clone().as_ref()),
		spender: account_id_from_slice(ALICE.as_ref()),
		value: AMOUNT / 2,
	}
	.encode();
	// An `Approval` event is emitted.
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());
	assert_eq!(session.sandbox().allowance(&TOKEN_ID, &contract, &ALICE), AMOUNT / 2);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn increase_allowance_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;

	const AMOUNT: Balance = TOKEN_MIN_BALANCE * 4;
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN_ID, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN_ID, &contract.clone(), &ALICE, AMOUNT / 2));

	session.set_actor(contract.clone());
	// No-op if the caller and `spender` is the same address, returns success and no events are
	// emitted.
	assert_ok!(increase_allowance(&mut session, contract.clone(), AMOUNT / 2));
	assert_eq!(last_contract_event(&session), None); // No event emitted.

	// Successfully increase allowance.
	assert_ok!(increase_allowance(&mut session, ALICE, AMOUNT / 2));
	let expected = Approval {
		owner: account_id_from_slice(contract.clone().as_ref()),
		spender: account_id_from_slice(ALICE.as_ref()),
		value: AMOUNT,
	}
	.encode();
	// An `Approval` event is emitted.
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());
	assert_eq!(session.sandbox().allowance(&TOKEN_ID, &contract, &ALICE), AMOUNT);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn decrease_allowance_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;

	const AMOUNT: Balance = TOKEN_MIN_BALANCE * 4;
	// Mint tokens.
	assert_ok!(session.sandbox().mint_into(&TOKEN_ID, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN_ID, &contract.clone(), &ALICE, AMOUNT / 2));

	session.set_actor(contract.clone());
	// No-op if the caller and `spender` is the same address, returns success and no events are
	// emitted.
	assert_ok!(decrease_allowance(&mut session, contract.clone(), AMOUNT / 2));
	assert_eq!(last_contract_event(&session), None); // No event emitted.

	// Failed with `InsufficientAllowance`.
	expect_call_reverted(
		&mut session,
		DECREASE_ALLOWANCE,
		vec![ALICE.to_string(), AMOUNT.to_string()],
		PSP22Error::InsufficientAllowance,
	);
	// Make sure allowance is unchaged.
	assert_eq!(session.sandbox().allowance(&TOKEN_ID, &contract, &ALICE), AMOUNT / 2);

	// Successfully decrease allowance.
	assert_ok!(decrease_allowance(&mut session, ALICE, 1));
	// Successfully emit event.
	let expected = Approval {
		owner: account_id_from_slice(contract.clone().as_ref()),
		spender: account_id_from_slice(ALICE.as_ref()),
		value: AMOUNT / 2 - 1,
	}
	.encode();
	// An `Approval` event is emitted.
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());
	assert_eq!(session.sandbox().allowance(&TOKEN_ID, &contract, &ALICE), AMOUNT / 2 - 1);
	Ok(())
}

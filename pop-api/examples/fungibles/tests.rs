use drink::{
	sandbox_api::assets_api::AssetsAPI,
	session::{error::SessionError, Session},
};
use frame_support::assert_ok;
use pop_api::{
	primitives::TokenId,
	v0::fungibles::events::{Approval, Created, Transfer},
};
use pop_sandbox::{Balance, Sandbox, ALICE, BOB};
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
	let contract = account_id_from_slice(contract.as_ref());
	// Successfully emit event.
	let expected = Created { id: TOKEN_ID, creator: contract, admin: contract }.encode();
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn new_existing_constructor_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();

	// Create token.
	let actor = session.get_actor();
	session.sandbox().create(&TOKEN_ID, &actor, TOKEN_MIN_BALANCE).unwrap();
	// Deploy a new contract.
	deploy_with_new_existing_constructor(&mut session, BundleProvider::local()?, TOKEN_ID)?;
	// Token is created successfully.
	assert!(session.sandbox().asset_exists(&TOKEN_ID));
	// No event emitted.
	assert_eq!(last_contract_event(&session), None);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn new_existing_constructor_deployment_fails(
	mut session: Session,
) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();

	let result =
		deploy_with_new_existing_constructor(&mut session, BundleProvider::local()?, TOKEN_ID);
	assert!(result.is_err());
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
	// Mint tokens.
	const AMOUNT: Balance = 12_000;
	assert_ok!(mint(&mut session, ALICE, AMOUNT));
	// Tokens were minted with the right amount.
	assert_eq!(balance_of(&mut session, ALICE), AMOUNT);
	assert_eq!(balance_of(&mut session, ALICE), session.sandbox().balance_of(&TOKEN_ID, &ALICE));
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn mint_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;
	// Mint tokens.
	const AMOUNT: Balance = 12_000;
	assert_ok!(mint(&mut session, ALICE, AMOUNT));
	// Successfully emit event.
	let expected =
		Transfer { from: None, to: Some(account_id_from_slice(ALICE.as_ref())), value: AMOUNT }
			.encode();
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());
	// Tokens were minted with the right amount.
	assert_eq!(total_supply(&mut session), AMOUNT);
	assert_eq!(balance_of(&mut session, ALICE), AMOUNT);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn mint_zero_value_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;
	// Mint tokens.
	assert_ok!(mint(&mut session, ALICE, 0));
	// No event emitted.
	assert_eq!(last_contract_event(&session), None);
	// Tokens were minted with the right amount.
	assert_eq!(total_supply(&mut session), 0);
	assert_eq!(balance_of(&mut session, ALICE), 0);
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
	// Mint tokens.
	const AMOUNT: Balance = 12_000;
	assert_ok!(mint(&mut session, ALICE, AMOUNT));
	// Burn tokens.
	assert_ok!(burn(&mut session, ALICE, 1));
	// Successfully emit event.
	let expected =
		Transfer { from: Some(account_id_from_slice(ALICE.as_ref())), to: None, value: 1 }.encode();
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());

	assert_eq!(total_supply(&mut session), AMOUNT - 1);
	assert_eq!(balance_of(&mut session, ALICE), AMOUNT - 1);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn burn_zero_value_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;
	// Burn tokens.
	assert_ok!(burn(&mut session, ALICE, 0));
	// No event emitted.
	assert_eq!(last_contract_event(&session), None);
	// Tokens were minted with the right amount.
	assert_eq!(total_supply(&mut session), 0);
	assert_eq!(balance_of(&mut session, ALICE), 0);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn burn_fails_with_insufficient_balance(
	mut session: Session,
) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;
	// Mint tokens.
	const AMOUNT: Balance = 12_000;
	assert_ok!(mint(&mut session, ALICE, AMOUNT));
	// Failed with `InsufficientBalance`.
	expect_call_reverted(
		&mut session,
		BURN,
		vec![ALICE.to_string(), (AMOUNT + 1).to_string()],
		PSP22Error::InsufficientBalance,
	);
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
	// Mint tokens.
	const AMOUNT: Balance = 12_000;
	const TRANSFERRED: Balance = 500;
	assert_ok!(mint(&mut session, contract.clone(), AMOUNT));
	assert_ok!(mint(&mut session, BOB, AMOUNT));
	// Transfer tokens from `contract` to `account`.
	session.set_actor(contract.clone());
	assert_ok!(transfer(&mut session, BOB, TRANSFERRED));
	// Successfully emit event.
	let expected = Transfer {
		from: Some(account_id_from_slice(contract.clone().as_ref())),
		to: Some(account_id_from_slice(BOB.as_ref())),
		value: TRANSFERRED,
	}
	.encode();
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());

	assert_eq!(balance_of(&mut session, contract), AMOUNT - TRANSFERRED);
	assert_eq!(balance_of(&mut session, BOB), AMOUNT + TRANSFERRED);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn transfer_zero_value_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;
	assert_ok!(transfer(&mut session, ALICE, 0));
	// No event emitted.
	assert_eq!(last_contract_event(&session), None);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn transfer_fails_with_insufficient_balance(
	mut session: Session,
) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;
	// Mint tokens.
	const AMOUNT: Balance = 12_000;
	assert_ok!(mint(&mut session, contract.clone(), AMOUNT));
	assert_ok!(mint(&mut session, BOB, AMOUNT));

	session.set_actor(contract.clone());
	// Failed with `InsufficientBalance`.
	expect_call_reverted(
		&mut session,
		TRANSFER,
		vec![BOB.to_string(), (AMOUNT + 1).to_string()],
		PSP22Error::InsufficientBalance,
	);
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

	const AMOUNT: Balance = 12_000;
	// Mint tokens.
	assert_ok!(mint(&mut session, contract.clone(), AMOUNT));
	// Successfully apporve.
	session.set_actor(contract.clone());
	assert_ok!(approve(&mut session, ALICE, AMOUNT / 2));
	// Successfully emit event.
	let expected = Approval {
		owner: account_id_from_slice(contract.clone().as_ref()),
		spender: account_id_from_slice(ALICE.as_ref()),
		value: AMOUNT / 2,
	}
	.encode();
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());
	assert_eq!(allowance(&mut session, contract, ALICE), AMOUNT / 2);
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

	const AMOUNT: Balance = 12_000;
	// Mint tokens.
	assert_ok!(mint(&mut session, contract.clone(), AMOUNT));
	// Successfully apporve.
	session.set_actor(contract.clone());
	assert_ok!(approve(&mut session, ALICE, AMOUNT / 2));
	assert_ok!(increase_allowance(&mut session, ALICE, AMOUNT / 2));
	// Successfully emit event.
	let expected = Approval {
		owner: account_id_from_slice(contract.clone().as_ref()),
		spender: account_id_from_slice(ALICE.as_ref()),
		value: AMOUNT,
	}
	.encode();
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());
	assert_eq!(allowance(&mut session, contract, ALICE), AMOUNT);
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

	const AMOUNT: Balance = 12_000;
	// Mint tokens.
	assert_ok!(mint(&mut session, contract.clone(), AMOUNT));
	// Successfully apporve.
	session.set_actor(contract.clone());
	assert_ok!(approve(&mut session, ALICE, AMOUNT / 2));
	assert_ok!(decrease_allowance(&mut session, ALICE, 1));
	// Successfully emit event.
	let expected = Approval {
		owner: account_id_from_slice(contract.clone().as_ref()),
		spender: account_id_from_slice(ALICE.as_ref()),
		value: AMOUNT / 2 - 1,
	}
	.encode();
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());
	assert_eq!(allowance(&mut session, contract, ALICE), AMOUNT / 2 - 1);
	Ok(())
}

// TODO: Unapproved error
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

	const AMOUNT: Balance = 12_000;
	// Mint tokens.
	assert_ok!(mint(&mut session, contract.clone(), AMOUNT));
	// Successfully transfer from `owner`.
	session.set_actor(contract.clone());
	assert_ok!(approve(&mut session, ALICE, AMOUNT / 2));
	assert_eq!(allowance(&mut session, contract.clone(), ALICE), AMOUNT / 2);

	session.set_actor(ALICE);
	assert_ok!(transfer_from(&mut session, contract, BOB, AMOUNT / 2));
	// Successfully emit event.
	let expected = Transfer {
		from: Some(account_id_from_slice(ALICE.as_ref())),
		to: Some(account_id_from_slice(BOB.as_ref())),
		value: AMOUNT / 4,
	}
	.encode();
	assert_eq!(last_contract_event(&session).unwrap(), expected.as_slice());
	Ok(())
}

use drink::{sandbox_api::assets_api::AssetsAPI, session::Session};
use helpers::*;
use pop_api::primitives::TokenId;
use pop_sandbox::{Balance, Sandbox, ALICE, BOB};

use super::*;

const TOKEN_ID: TokenId = 1;
const TOKEN_MIN_BALANCE: Balance = 10_000;

#[drink::contract_bundle_provider]
enum BundleProvider {}

#[drink::test(sandbox = Sandbox)]
fn new_constructor_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;
	// Check if token exists after the deployment.
	assert!(session.sandbox().asset_exists(&TOKEN_ID));
	// Check if `Created` event is emitted.
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
	// Check if token is created successfully.
	assert!(session.sandbox().asset_exists(&TOKEN_ID));
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn mint_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract_address = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;
	// Mint tokens.
	const AMOUNT: Balance = 12_000;
	mint(&mut session, ALICE, AMOUNT)?;
	mint(&mut session, BOB, AMOUNT)?;
	// Check if the tokens were minted with the right amount.
	assert_eq!(total_supply(&mut session)?, AMOUNT * 2);
	assert_eq!(balance_of(&mut session, ALICE)?, AMOUNT);
	assert_eq!(balance_of(&mut session, BOB)?, AMOUNT);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn burn_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract_address = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;
	// Mint tokens.
	const AMOUNT: Balance = 12_000;
	mint(&mut session, ALICE, AMOUNT)?;
	// Burn tokens.
	burn(&mut session, ALICE, 1)?;
	assert_eq!(total_supply(&mut session)?, AMOUNT - 1);
	assert_eq!(balance_of(&mut session, ALICE)?, AMOUNT - 1);
	Ok(())
}

#[drink::test(sandbox = Sandbox)]
fn transfer_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract_address = deploy_with_new_constructor(
		&mut session,
		BundleProvider::local()?,
		TOKEN_ID,
		TOKEN_MIN_BALANCE,
	)?;
	// Mint tokens.
	const AMOUNT: Balance = 12_000;
	const TRANSFERED: Balance = 500;
	mint(&mut session, contract_address.clone(), AMOUNT)?;
	mint(&mut session, BOB, AMOUNT)?;
	// Transfer tokens.
	transfer(&mut session, BOB, TRANSFERED)?;
	assert_eq!(balance_of(&mut session, contract_address)?, AMOUNT - TRANSFERED);
	assert_eq!(balance_of(&mut session, BOB)?, AMOUNT + TRANSFERED);
	Ok(())
}

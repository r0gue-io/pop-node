use drink::{
	frame_support::assert_ok,
	sandbox_api::prelude::AssetsAPI,
	session::{Session, NO_ARGS, NO_SALT},
};
use pop_api::{
	primitives::TokenId,
	v0::fungibles::{self as api},
};
use pop_sandbox::{AccountId32, Balance, Sandbox, ALICE, BOB, INIT_VALUE};
use scale::{Decode, Encode};

use super::*;

const TOKEN_A: TokenId = 1;
const TOKEN_B: TokenId = 2;

#[drink::contract_bundle_provider]
enum BundleProvider {}

use test_methods::*;
// Utility methods to test the contract calls.
mod test_methods {
	use super::*;

	// TODO: do we need this if we use pop_api::primitives::AccountId for below parameters?
	// Decode slice of bytes to `pop-api` AccountId.
	pub(super) fn account_id_from_slice(s: &[u8; 32]) -> pop_api::primitives::AccountId {
		pop_api::primitives::AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
	}

	// Call a contract method and decode the returned data.
	pub(super) fn decoded_call<T: Decode>(
		session: &mut Session<Sandbox>,
		func_name: &str,
		input: Vec<String>,
		endowment: Option<Balance>,
	) -> Result<T, Box<dyn std::error::Error>> {
		session.call(func_name, &input, endowment)??;
		Ok(session.record().last_call_return_decoded::<T>()??)
	}

	// Check if the event emitted correctly.
	pub(super) fn assert_event(session: &mut Session<Sandbox>, event: Vec<u8>) {
		let contract_events = session.record().last_event_batch().contract_events();
		let last_event = contract_events.last().unwrap().to_vec();
		assert_eq!(last_event, event.as_slice());
	}

	pub(super) fn mint(
		session: &mut Session<Sandbox>,
		token: TokenId,
		account: AccountId32,
		amount: Balance,
	) -> Result<(), Box<dyn std::error::Error>> {
		Ok(session.call(
			"mint",
			&vec![token.to_string(), account.to_string(), amount.to_string()],
			None,
		)??)
	}

	pub(super) fn burn(
		session: &mut Session<Sandbox>,
		token: TokenId,
		account: AccountId32,
		amount: Balance,
	) -> Result<(), Box<dyn std::error::Error>> {
		Ok(session.call(
			"burn",
			&vec![token.to_string(), account.to_string(), amount.to_string()],
			None,
		)??)
	}

	pub(super) fn transfer(
		session: &mut Session<Sandbox>,
		token: TokenId,
		to: AccountId32,
		amount: Balance,
	) -> Result<(), Box<dyn std::error::Error>> {
		Ok(session.call(
			"transfer",
			&vec![token.to_string(), to.to_string(), amount.to_string()],
			None,
		)??)
	}

	pub(super) fn token_exist(
		session: &mut Session<Sandbox>,
		token: TokenId,
	) -> Result<PSP22Result<bool>, Box<dyn std::error::Error>> {
		Ok(decoded_call::<PSP22Result<bool>>(session, "token_exists", vec![], None)?)
	}

	pub(super) fn total_supply(
		session: &mut Session<Sandbox>,
		token: TokenId,
	) -> Result<PSP22Result<Balance>, Box<dyn std::error::Error>> {
		Ok(decoded_call::<PSP22Result<Balance>>(
			session,
			"total_supply",
			vec![token.to_string()],
			None,
		)?)
	}

	pub(super) fn balance_of(
		session: &mut Session<Sandbox>,
		token: TokenId,
		owner: AccountId32,
	) -> Result<PSP22Result<Balance>, Box<dyn std::error::Error>> {
		Ok(decoded_call::<PSP22Result<Balance>>(
			session,
			"balance_of",
			vec![token.to_string(), owner.to_string()],
			None,
		)?)
	}
}

// TODO test an error case from api and if present contract errors.
#[drink::test(sandbox = Sandbox)]
fn new_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	let contract_bundle = BundleProvider::local()?;

	// TODO: can not get back the error from `deploy_bundle` and thus not test the error cases in
	//  the contructor. Tackle at the end because might be difficult.

	// Instantiate the contract and create a new token.
	let contract = session.deploy_bundle(
		contract_bundle,
		"new",
		&vec![TOKEN_A.to_string(), ALICE.to_string(), 1_000.to_string()],
		NO_SALT,
		Some(INIT_VALUE),
	)?;

	assert_event(
		&mut session,
		api::events::Created {
			id: TOKEN_A,
			creator: account_id_from_slice(contract.as_ref()),
			admin: account_id_from_slice(ALICE.as_ref()),
		}
		.encode(),
	);

	assert_eq!(token_exist(&mut session, TOKEN_A)?, Ok(true));
	Ok(())
}

// TODO test an error case from api and if present contract errors.
#[drink::test(sandbox = Sandbox)]
fn new_existing_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	let contract_bundle = BundleProvider::local()?;

	// Instantiate the contract to an existing token.
	assert_ok!(session.sandbox.create(&TOKEN_A, &ALICE, 1_000));

	// Instantiate the contract to an existing token.
	let _contract = session.deploy_bundle(
		contract_bundle,
		"new_existing",
		&vec![TOKEN_A.to_string()],
		NO_SALT,
		Some(INIT_VALUE),
	)?;
	// Check that the token is created successfully.
	assert_eq!(session.sandbox.asset_exists(&TOKEN_A), true);
	// TODO: remove below assertion.
	assert_eq!(token_exist(&mut session)?, Ok(true));
	Ok(())
}

// #[drink::test(sandbox = Sandbox)]
// fn test_mint_token_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
// 	let _ = env_logger::try_init();
// 	let contract_bundle = BundleProvider::local()?;
// 	// Deploy a contract.
// 	let contract_address =
// 		session.deploy_bundle(contract_bundle, "new", NO_ARGS, NO_SALT, Some(INIT_VALUE))?;
// 	// Create a new token.
// 	create(&mut session, TOKEN_A, contract_address, 10_000)?;
// 	// Mint tokens.
// 	const AMOUNT: Balance = 12_000;
// 	mint(&mut session, TOKEN_A, ALICE, AMOUNT)?;
// 	mint(&mut session, TOKEN_A, BOB, AMOUNT)?;
// 	// Check if the tokens were minted with the right amount.
// 	assert_eq!(total_supply(&mut session, TOKEN_A)?, Ok(AMOUNT * 2));
// 	assert_eq!(balance_of(&mut session, TOKEN_A, ALICE)?, Ok(AMOUNT));
// 	assert_eq!(balance_of(&mut session, TOKEN_A, BOB)?, Ok(AMOUNT));
// 	Ok(())
// }
//
// #[drink::test(sandbox = Sandbox)]
// fn test_burn_token_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
// 	let _ = env_logger::try_init();
// 	let contract_bundle = BundleProvider::local()?;
// 	// Deploy a contract.
// 	let contract_address =
// 		session.deploy_bundle(contract_bundle, "new", NO_ARGS, NO_SALT, Some(INIT_VALUE))?;
// 	// Create a new token.
// 	create(&mut session, TOKEN_A, contract_address, 10_000)?;
// 	// Mint tokens.
// 	const AMOUNT: Balance = 12_000;
// 	mint(&mut session, TOKEN_A, ALICE, AMOUNT)?;
// 	// Burn tokens.
// 	burn(&mut session, TOKEN_A, ALICE, 1)?;
// 	assert_eq!(total_supply(&mut session, TOKEN_A)?, Ok(AMOUNT - 1));
// 	assert_eq!(balance_of(&mut session, TOKEN_A, ALICE)?, Ok(AMOUNT - 1));
// 	Ok(())
// }
//
// #[drink::test(sandbox = Sandbox)]
// fn test_transfer_token_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
// 	let _ = env_logger::try_init();
// 	let contract_bundle = BundleProvider::local()?;
// 	// Deploy a contract.
// 	let contract_address =
// 		session.deploy_bundle(contract_bundle, "new", NO_ARGS, NO_SALT, Some(INIT_VALUE))?;
// 	// Create a new token.
// 	create(&mut session, TOKEN_A, contract_address.clone(), 10_000)?;
// 	// Mint tokens.
// 	const AMOUNT: Balance = 12_000;
// 	const TRANSFERED: Balance = 500;
// 	mint(&mut session, TOKEN_A, contract_address.clone(), AMOUNT)?;
// 	mint(&mut session, TOKEN_A, BOB, AMOUNT)?;
// 	// Transfer tokens.
// 	transfer(&mut session, TOKEN_A, BOB, TRANSFERED)?;
// 	assert_eq!(balance_of(&mut session, TOKEN_A, contract_address)?, Ok(AMOUNT - TRANSFERED));
// 	assert_eq!(balance_of(&mut session, TOKEN_A, BOB)?, Ok(AMOUNT + TRANSFERED));
// 	Ok(())
// }

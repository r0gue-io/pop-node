use drink::session::{error::SessionError, Session, NO_ARGS, NO_SALT};
use pop_api::{
	primitives::{AccountId, TokenId},
	v0::fungibles::{self as api, PSP22Error},
};
use pop_sandbox::{AccountId32, Balance, Sandbox, ALICE, BOB, INIT_VALUE};
use scale::{Decode, Encode};

use super::*;

const TOKEN_ID: TokenId = 1;

#[drink::contract_bundle_provider]
enum BundleProvider {}

use test_methods::*;
// Utility methods to test the contract calls.
mod test_methods {
	use super::*;

	// Decode slice of bytes to `pop-api` AccountId.
	pub(super) fn account_id_from_slice(s: &[u8; 32]) -> AccountId {
		AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
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

	// Test methods for deployment with constructor function.

	pub(super) fn deploy_with_new_constructor(
		session: &mut Session<Sandbox>,
		id: TokenId,
		min_balance: Balance,
	) -> Result<AccountId32, SessionError> {
		session.deploy_bundle(
			BundleProvider::local()?,
			"new",
			&[id.to_string(), min_balance.to_string()],
			NO_SALT,
			Some(INIT_VALUE),
		)
	}

	// Test methods for `PSP22`.`

	pub(super) fn total_supply(
		session: &mut Session<Sandbox>,
	) -> Result<Balance, Box<dyn std::error::Error>> {
		Ok(decoded_call::<Balance>(session, "Psp22::total_supply", vec![], None)?)
	}

	pub(super) fn balance_of(
		session: &mut Session<Sandbox>,
		owner: AccountId32,
	) -> Result<Balance, Box<dyn std::error::Error>> {
		Ok(decoded_call::<Balance>(session, "Psp22::balance_of", vec![owner.to_string()], None)?)
	}

	pub(super) fn transfer(
		session: &mut Session<Sandbox>,
		to: AccountId32,
		amount: Balance,
	) -> Result<(), Box<dyn std::error::Error>> {
		let data = serde_json::to_string::<[u8; 0]>(&[]).unwrap();
		Ok(session.call(
			"Psp22::transfer",
			&vec![to.to_string(), amount.to_string(), data],
			None,
		)??)
	}

	// Test methods for `PSP22Metadata``.

	pub(super) fn token_name(
		session: &mut Session<Sandbox>,
	) -> Result<String, Box<dyn std::error::Error>> {
		Ok(decoded_call::<String>(session, "Psp22Metadata::token_name", vec![], None)?)
	}

	pub(super) fn token_symbol(
		session: &mut Session<Sandbox>,
	) -> Result<String, Box<dyn std::error::Error>> {
		Ok(decoded_call::<String>(session, "Psp22Metadata::token_symbol", vec![], None)?)
	}

	pub(super) fn token_decimals(
		session: &mut Session<Sandbox>,
	) -> Result<u8, Box<dyn std::error::Error>> {
		Ok(decoded_call::<u8>(session, "Psp22Metadata::token_decimals", vec![], None)?)
	}

	// Test methods for `PSP22Mintable``.

	pub(super) fn mint(
		session: &mut Session<Sandbox>,
		account: AccountId32,
		amount: Balance,
	) -> Result<(), Box<dyn std::error::Error>> {
		Ok(session.call(
			"Psp22Mintable::mint",
			&vec![account.to_string(), amount.to_string()],
			None,
		)??)
	}

	// Test methods for `PSP22MPsp22Burnablentable``.

	pub(super) fn burn(
		session: &mut Session<Sandbox>,
		account: AccountId32,
		amount: Balance,
	) -> Result<(), Box<dyn std::error::Error>> {
		Ok(session.call(
			"Psp22Burnable::burn",
			&vec![account.to_string(), amount.to_string()],
			None,
		)??)
	}
}

#[drink::test(sandbox = Sandbox)]
fn test_mint_token_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract_address = deploy_with_new_constructor(&mut session, 1, 10_000)?;
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
fn test_burn_token_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract_address = deploy_with_new_constructor(&mut session, TOKEN_ID, 10_000)?;
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
fn test_transfer_token_works(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract_address = deploy_with_new_constructor(&mut session, TOKEN_ID, 10_000)?;
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

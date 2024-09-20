// A set of helper methods to test the contract calls.

use drink::{
	session::{bundle::ContractBundle, error::SessionError, Session, NO_SALT},
	DispatchError,
};
use pop_api::primitives::{AccountId, TokenId};
use pop_sandbox::{AccountId32, Balance, Sandbox, INIT_VALUE};
use scale::{Decode, Encode};

use super::*;

/// This is used to resolve type mismatches between the `AccountId` in the quasi testing environment and the
/// contract environment.
pub(super) fn account_id_from_slice(s: &[u8; 32]) -> AccountId {
	AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
}

/// Get the last event from pallet contracts.
pub(super) fn last_contract_event(session: &Session<Sandbox>) -> Option<Vec<u8>> {
	session.record().last_event_batch().contract_events().last().cloned()
}

/// Get the last contract execution result.
pub(super) fn last_contract_error(session: &Session<Sandbox>) -> Option<DispatchError> {
	session.record().last_call_result().result.clone().err()
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
	bundle: ContractBundle,
	id: TokenId,
	min_balance: Balance,
) -> Result<AccountId32, SessionError> {
	session.deploy_bundle(
		bundle,
		"new",
		&[id.to_string(), min_balance.to_string()],
		NO_SALT,
		Some(INIT_VALUE),
	)
}

pub(super) fn deploy_with_new_existing_constructor(
	session: &mut Session<Sandbox>,
	bundle: ContractBundle,
	id: TokenId,
) -> Result<AccountId32, SessionError> {
	session.deploy_bundle(bundle, "new_existing", &[id.to_string()], NO_SALT, Some(INIT_VALUE))
}

// Test methods for `PSP22`.`

pub(super) fn total_supply(session: &mut Session<Sandbox>) -> Balance {
	decoded_call::<Balance>(session, "Psp22::total_supply", vec![], None).unwrap()
}

pub(super) fn balance_of(session: &mut Session<Sandbox>, owner: AccountId32) -> Balance {
	decoded_call::<Balance>(session, "Psp22::balance_of", vec![owner.to_string()], None).unwrap()
}

pub(super) fn transfer(
	session: &mut Session<Sandbox>,
	to: AccountId32,
	amount: Balance,
) -> Result<(), Box<dyn std::error::Error>> {
	let data = serde_json::to_string::<[u8; 0]>(&[]).unwrap();
	Ok(session.call("Psp22::transfer", &vec![to.to_string(), amount.to_string(), data], None)??)
}

// Test methods for `PSP22Metadata``.

pub(super) fn token_name(session: &mut Session<Sandbox>) -> String {
	decoded_call::<String>(session, "Psp22Metadata::token_name", vec![], None).unwrap()
}

pub(super) fn token_symbol(session: &mut Session<Sandbox>) -> String {
	decoded_call::<String>(session, "Psp22Metadata::token_symbol", vec![], None).unwrap()
}

pub(super) fn token_decimals(session: &mut Session<Sandbox>) -> u8 {
	decoded_call::<u8>(session, "Psp22Metadata::token_decimals", vec![], None).unwrap()
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

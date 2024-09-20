// A set of helper methods to test the contract calls.

use drink::{
	session::{bundle::ContractBundle, error::SessionError, Session, NO_SALT},
	DispatchError,
};
use pop_api::primitives::{AccountId, TokenId};
use pop_sandbox::{AccountId32, Balance, Sandbox, INIT_VALUE};
use scale::{Decode, Encode};

use super::*;

// PSP22 functions.
pub const ALLOWANCE: &str = "Psp22::allowance";
pub const BALANCE_OF: &str = "Psp22::balance_of";
pub const TOTAL_SUPPLY: &str = "Psp22::total_supply";
pub const TRANSFER: &str = "Psp22::transfer";
pub const TRANSFER_FROM: &str = "Psp22::transfer_from";
pub const APPROVE: &str = "Psp22::approve";
pub const INCREASE_ALLOWANCE: &str = "Psp22::increase_allowance";
pub const DECREASE_ALLOWANCE: &str = "Psp22::decrease_allowance";
// PSP22Metadata functions.
pub const TOKEN_NAME: &str = "Psp22Metadata::token_name";
pub const TOKEN_SYMBOL: &str = "Psp22Metadata::token_symbol";
pub const TOKEN_DECIMALS: &str = "Psp22Metadata::token_decimals";
// PSP22Mintable functions.
pub const MINT: &str = "Psp22Mintable::mint";
// PSP22Burnable functions.
pub const BURN: &str = "Psp22Burnable::burn";

/// This is used to resolve type mismatches between the `AccountId` in the quasi testing environment and the
/// contract environment.
pub(super) fn account_id_from_slice(s: &[u8; 32]) -> AccountId {
	AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
}

/// Get the last event from pallet contracts.
pub(super) fn last_contract_event(session: &Session<Sandbox>) -> Option<Vec<u8>> {
	session.record().last_event_batch().contract_events().last().cloned()
}

/// Execute a contract method and exepct CallReverted error to be returned.
pub(super) fn expect_call_reverted(
	session: &mut Session<Sandbox>,
	function: &str,
	params: Vec<String>,
	err: PSP22Error,
) {
	let call = session.call::<String, ()>(function, &params, None);
	if let Err(SessionError::CallReverted(error)) = call {
		assert_eq!(error[1..], Err::<(), PSP22Error>(err).encode());
	}
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

// Test methods for `PSP22`.

pub(super) fn total_supply(session: &mut Session<Sandbox>) -> Balance {
	decoded_call::<Balance>(session, TOTAL_SUPPLY, vec![], None).unwrap()
}

pub(super) fn balance_of(session: &mut Session<Sandbox>, owner: AccountId32) -> Balance {
	decoded_call::<Balance>(session, BALANCE_OF, vec![owner.to_string()], None).unwrap()
}

pub(super) fn allowance(
	session: &mut Session<Sandbox>,
	owner: AccountId32,
	spender: AccountId32,
) -> Balance {
	decoded_call::<Balance>(session, ALLOWANCE, vec![owner.to_string(), spender.to_string()], None)
		.unwrap()
}

pub(super) fn transfer(
	session: &mut Session<Sandbox>,
	to: AccountId32,
	amount: Balance,
) -> Result<(), Box<dyn std::error::Error>> {
	let data = serde_json::to_string::<[u8; 0]>(&[]).unwrap();
	Ok(session.call(TRANSFER, &vec![to.to_string(), amount.to_string(), data], None)??)
}

pub(super) fn transfer_from(
	session: &mut Session<Sandbox>,
	from: AccountId32,
	to: AccountId32,
	amount: Balance,
) -> Result<(), Box<dyn std::error::Error>> {
	let data = serde_json::to_string::<[u8; 0]>(&[]).unwrap();
	Ok(session.call(
		TRANSFER_FROM,
		&vec![from.to_string(), to.to_string(), amount.to_string(), data],
		None,
	)??)
}

pub(super) fn approve(
	session: &mut Session<Sandbox>,
	spender: AccountId32,
	value: Balance,
) -> Result<(), Box<dyn std::error::Error>> {
	Ok(session.call(APPROVE, &vec![spender.to_string(), value.to_string()], None)??)
}

pub(super) fn increase_allowance(
	session: &mut Session<Sandbox>,
	spender: AccountId32,
	value: Balance,
) -> Result<(), Box<dyn std::error::Error>> {
	Ok(session.call(INCREASE_ALLOWANCE, &vec![spender.to_string(), value.to_string()], None)??)
}

pub(super) fn decrease_allowance(
	session: &mut Session<Sandbox>,
	spender: AccountId32,
	value: Balance,
) -> Result<(), Box<dyn std::error::Error>> {
	Ok(session.call(DECREASE_ALLOWANCE, &vec![spender.to_string(), value.to_string()], None)??)
}

// Test methods for `PSP22Metadata``.

pub(super) fn token_name(session: &mut Session<Sandbox>) -> String {
	decoded_call::<String>(session, TOKEN_NAME, vec![], None).unwrap()
}

pub(super) fn token_symbol(session: &mut Session<Sandbox>) -> String {
	decoded_call::<String>(session, TOKEN_SYMBOL, vec![], None).unwrap()
}

pub(super) fn token_decimals(session: &mut Session<Sandbox>) -> u8 {
	decoded_call::<u8>(session, TOKEN_DECIMALS, vec![], None).unwrap()
}

// Test methods for `PSP22Mintable``.

pub(super) fn mint(
	session: &mut Session<Sandbox>,
	account: AccountId32,
	amount: Balance,
) -> Result<(), Box<dyn std::error::Error>> {
	Ok(session.call(MINT, &vec![account.to_string(), amount.to_string()], None)??)
}

// Test methods for `PSP22MPsp22Burnablentable``.

pub(super) fn burn(
	session: &mut Session<Sandbox>,
	account: AccountId32,
	amount: Balance,
) -> Result<(), Box<dyn std::error::Error>> {
	Ok(session.call(BURN, &vec![account.to_string(), amount.to_string()], None)??)
}

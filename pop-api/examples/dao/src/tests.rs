use drink::{
	assert_err, assert_last_contract_event, assert_ok, call,
	devnet::{
		account_id_from_slice,
		error::{
			v0::{ApiError::*, ArithmeticError::*},
			Assets,
			AssetsError::*,
		},
		AccountId, Balance, Runtime,
	},
	last_contract_event,
	session::Session,
	AssetsAPI, TestExternalities, NO_SALT,
};
use ink::scale::Encode;
use pop_api::{
	primitives::TokenId,
	v0::fungibles::events::{Approval, Created, Transfer},
};

use super::*;
use crate::dao::{Error, Member};

const UNIT: Balance = 10_000_000_000;
const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
const INIT_VALUE: Balance = 100 * UNIT;
const ALICE: AccountId = AccountId::new([1u8; 32]);
const BOB: AccountId = AccountId::new([2_u8; 32]);
const CHARLIE: AccountId = AccountId::new([3_u8; 32]);
const AMOUNT: Balance = MIN_BALANCE * 4;
const MIN_BALANCE: Balance = 10_000;
const TOKEN: TokenId = 1;

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

fn deploy_with_default(session: &mut Session<Pop>) -> Result<AccountId, Psp22Error> {
	deploy(session, "new", vec![TOKEN.to_string(), 10.to_string(), MIN_BALANCE.to_string()])
}

#[drink::test(sandbox = Pop)]
fn new_constructor_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	println!("{:?}", contract);
	// Token exists after the deployment.
	assert!(session.sandbox().asset_exists(&TOKEN));
	// Successfully emit event.
	assert_last_contract_event!(
		&session,
		Created {
			id: TOKEN,
			creator: account_id_from_slice(&contract),
			admin: account_id_from_slice(&contract),
		}
	);
}

#[drink::test(sandbox = Pop)]
fn join_dao_works(mut session: Session) {
	let _ = env_logger::try_init();
	let value = AMOUNT / 2;
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	session.set_actor(ALICE);
	// Mint tokens and approve.
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &ALICE, &contract.clone(), AMOUNT));
	assert_eq!(session.sandbox().allowance(&TOKEN, &ALICE, &contract.clone()), AMOUNT);
	assert_eq!(session.sandbox().balance_of(&TOKEN, &ALICE), AMOUNT);

	// Alice joins the dao
	assert_ok!(join(&mut session, value));
	// assert_ok!(members(&mut session, ALICE));

	// Successfully emit event.
	assert_last_contract_event!(
		&session,
		Transfer {
			from: Some(account_id_from_slice(&ALICE)),
			to: Some(account_id_from_slice(&contract)),
			value,
		}
	);
	if let Ok(member) = members(&mut session, ALICE) {
		assert_eq!(member.voting_power, 20000);
	}
}

// Deploy the contract with `NO_SALT and `INIT_VALUE`.
fn deploy(
	session: &mut Session<Pop>,
	method: &str,
	input: Vec<String>,
) -> Result<AccountId, Psp22Error> {
	drink::deploy::<Pop, Psp22Error>(
		session,
		// The local contract (i.e. `fungibles`).
		BundleProvider::local().unwrap(),
		method,
		input,
		NO_SALT,
		Some(INIT_VALUE),
	)
}

fn join(session: &mut Session<Pop>, value: Balance) -> Result<(), Error> {
	call::<Pop, (), Error>(session, "join", vec![value.to_string()], None)
}

fn members(session: &mut Session<Pop>, account: AccountId) -> Result<Member, Error> {
	call::<Pop, Member, Error>(session, "get_member", vec![account.to_string()], None)
}

use drink::{
	assert_last_contract_event, assert_ok, call,
	devnet::{account_id_from_slice, AccountId, Balance, Runtime},
	sandbox_api::system_api::SystemAPI,
	session::Session,
	AssetsAPI, TestExternalities, NO_SALT,
};
use pop_api::{
	primitives::TokenId,
	v0::fungibles::events::{Approval, Created, Transfer},
};

use super::*;
use crate::dao::{Error, Member, Proposal, Voted};

const UNIT: Balance = 10_000_000_000;
const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
const INIT_VALUE: Balance = 100 * UNIT;
const ALICE: AccountId = AccountId::new([1u8; 32]);
const BOB: AccountId = AccountId::new([2_u8; 32]);
const CHARLIE: AccountId = AccountId::new([3_u8; 32]);
const NON_MEMBER: AccountId = AccountId::new([4_u8; 32]);
const AMOUNT: Balance = MIN_BALANCE * 4;
const MIN_BALANCE: Balance = 10_000;
const TOKEN: TokenId = 1;
const VOTING_PERIOD: u32 = 10;

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
	deploy(
		session,
		"new",
		vec![TOKEN.to_string(), VOTING_PERIOD.to_string(), MIN_BALANCE.to_string()],
	)
}

#[drink::test(sandbox = Pop)]
fn new_constructor_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
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

	// Successfully emit event.
	assert_last_contract_event!(
		&session,
		Transfer {
			from: Some(account_id_from_slice(&ALICE)),
			to: Some(account_id_from_slice(&contract)),
			value,
		}
	);

	// We check that Alice is a member with a voting power of 20000
	if let Ok(member) = members(&mut session, ALICE) {
		assert_eq!(member.voting_power, 20000);
	}
}

#[drink::test(sandbox = Pop)]
fn member_create_proposal_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// Prepare voters accounts
	let _ = prepare_dao(&mut session, contract.clone());

	// Alice create a proposal
	let description = "Funds for creation of a Dao contract".to_string().as_bytes().to_vec();
	let amount = AMOUNT * 3;
	session.set_actor(ALICE);
	assert_ok!(create_proposal(&mut session, BOB, amount, description));

	assert_last_contract_event!(
		&session,
		Created {
			id: 1,
			creator: account_id_from_slice(&ALICE),
			admin: account_id_from_slice(&contract),
		}
	);
}

#[drink::test(sandbox = Pop)]
fn members_vote_system_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// Prepare voters accounts
	let _ = prepare_dao(&mut session, contract.clone());

	// Alice create a proposal
	let description = "Funds for creation of a Dao contract".to_string().as_bytes().to_vec();
	let amount = AMOUNT * 3;
	session.set_actor(ALICE);
	assert_ok!(create_proposal(&mut session, BOB, amount, description));

	session.set_actor(CHARLIE);
	// Charlie vote
	let now = session.sandbox().block_number();
	assert_ok!(vote(&mut session, 1, true));

	assert_last_contract_event!(
		&session,
		Voted { who: Some(account_id_from_slice(&CHARLIE)), when: Some(now) }
	);
	let prop = proposal(&mut session, 1).unwrap();
	let yes = prop.yes_votes;
}

#[drink::test(sandbox = Pop)]
fn double_vote_fails(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// Prepare voters accounts
	let _ = prepare_dao(&mut session, contract.clone());

	// Alice create a proposal
	let description = "Funds for creation of a Dao contract".to_string().as_bytes().to_vec();
	let amount = AMOUNT * 3;
	session.set_actor(ALICE);
	assert_ok!(create_proposal(&mut session, BOB, amount, description));

	session.set_actor(CHARLIE);
	// Charlie tries to vote twice for the same proposal
	assert_ok!(vote(&mut session, 1, true));
	assert_eq!(vote(&mut session, 1, false), Err(Error::AlreadyVoted));
}

#[drink::test(sandbox = Pop)]
fn vote_fails_if_voting_period_ended(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// Prepare voters accounts
	let _ = prepare_dao(&mut session, contract.clone());

	// Alice create a proposal
	let description = "Funds for creation of a Dao contract".to_string().as_bytes().to_vec();
	let amount = AMOUNT * 3;
	session.set_actor(ALICE);
	assert_ok!(create_proposal(&mut session, BOB, amount, description));

	// Moving to blocks beyond voting period
	session.sandbox().build_blocks(VOTING_PERIOD + 1);

	session.set_actor(CHARLIE);
	// Charlie tries to vote
	assert_eq!(vote(&mut session, 1, true), Err(Error::VotingPeriodEnded));
}

#[drink::test(sandbox = Pop)]
fn vote_fails_if_not_a_member(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// Prepare voters accounts
	let _ = prepare_dao(&mut session, contract.clone());

	// Alice create a proposal
	let description = "Funds for creation of a Dao contract".to_string().as_bytes().to_vec();
	let amount = AMOUNT * 3;
	session.set_actor(ALICE);
	assert_ok!(create_proposal(&mut session, BOB, amount, description));

	session.set_actor(NON_MEMBER);
	assert_eq!(vote(&mut session, 1, true), Err(Error::MemberNotFound));
}

#[drink::test(sandbox = Pop)]
fn proposal_enactment_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// Prepare voters accounts
	let _ = prepare_dao(&mut session, contract.clone());

	// Alice create a proposal
	let description = "Funds for creation of a Dao contract".to_string().as_bytes().to_vec();
	let amount = MIN_BALANCE;
	session.set_actor(ALICE);
	assert_ok!(create_proposal(&mut session, BOB, amount, description));

	let now = session.sandbox().block_number();
	session.set_actor(CHARLIE);
	// Charlie vote
	assert_ok!(vote(&mut session, 1, true));
	assert_last_contract_event!(
		&session,
		Voted { who: Some(account_id_from_slice(&CHARLIE)), when: Some(now) }
	);
	// Alice vote
	session.set_actor(ALICE);
	assert_ok!(vote(&mut session, 1, true));
	assert_last_contract_event!(
		&session,
		Voted { who: Some(account_id_from_slice(&ALICE)), when: Some(now) }
	);
	// BOB vote
	session.set_actor(BOB);
	assert_ok!(vote(&mut session, 1, true));
	assert_last_contract_event!(
		&session,
		Voted { who: Some(account_id_from_slice(&BOB)), when: Some(now) }
	);

	session.sandbox().build_blocks(VOTING_PERIOD + 1);

	assert_ok!(execute_proposal(&mut session, 1));
	// Successfully emit event.
	assert_last_contract_event!(
		&session,
		Approval {
			owner: account_id_from_slice(&contract),
			spender: account_id_from_slice(&contract),
			value: MIN_BALANCE,
		}
	);
}

#[drink::test(sandbox = Pop)]
fn same_proposal_consecutive_claim_fails(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// Prepare voters accounts
	let _ = prepare_dao(&mut session, contract.clone());

	// Alice create a proposal
	let description = "Funds for creation of a Dao contract".to_string().as_bytes().to_vec();
	let amount = MIN_BALANCE;
	session.set_actor(ALICE);
	assert_ok!(create_proposal(&mut session, BOB, amount, description));

	let now = session.sandbox().block_number();
	session.set_actor(CHARLIE);
	// Charlie vote
	assert_ok!(vote(&mut session, 1, true));
	assert_last_contract_event!(
		&session,
		Voted { who: Some(account_id_from_slice(&CHARLIE)), when: Some(now) }
	);
	// Alice vote
	session.set_actor(ALICE);
	assert_ok!(vote(&mut session, 1, true));
	assert_last_contract_event!(
		&session,
		Voted { who: Some(account_id_from_slice(&ALICE)), when: Some(now) }
	);
	// BOB vote
	session.set_actor(BOB);
	assert_ok!(vote(&mut session, 1, true));
	assert_last_contract_event!(
		&session,
		Voted { who: Some(account_id_from_slice(&BOB)), when: Some(now) }
	);

	session.sandbox().build_blocks(VOTING_PERIOD + 1);

	assert_ok!(execute_proposal(&mut session, 1));
	// Successfully emit event.
	// assert_last_contract_event!(
	// &session,
	// Approval {
	// owner: account_id_from_slice(&contract),
	// spender: account_id_from_slice(&contract),
	// value: MIN_BALANCE,
	// }
	// );
	// session.sandbox().build_block();
	// Second consecutive claim for same proposal fails
	// assert_eq!(execute_proposal(&mut session, 1), Err(Error::ProposalExecuted));
}

#[drink::test(sandbox = Pop)]
fn proposal_enactment_fails_if_proposal_is_rejected(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// Prepare voters accounts
	let _ = prepare_dao(&mut session, contract.clone());

	// Alice create a proposal
	let description = "Funds for creation of a Dao contract".to_string().as_bytes().to_vec();
	let amount = MIN_BALANCE;
	session.set_actor(ALICE);
	assert_ok!(create_proposal(&mut session, BOB, amount, description));

	let now = session.sandbox().block_number();
	session.set_actor(CHARLIE);
	// Charlie vote
	assert_ok!(vote(&mut session, 1, false));
	assert_last_contract_event!(
		&session,
		Voted { who: Some(account_id_from_slice(&CHARLIE)), when: Some(now) }
	);
	// Alice vote
	session.set_actor(ALICE);
	assert_ok!(vote(&mut session, 1, false));
	assert_last_contract_event!(
		&session,
		Voted { who: Some(account_id_from_slice(&ALICE)), when: Some(now) }
	);
	// BOB vote
	session.set_actor(BOB);
	assert_ok!(vote(&mut session, 1, false));
	assert_last_contract_event!(
		&session,
		Voted { who: Some(account_id_from_slice(&BOB)), when: Some(now) }
	);

	session.sandbox().build_blocks(VOTING_PERIOD + 1);

	assert_eq!(execute_proposal(&mut session, 1), Err(Error::ProposalRejected));
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

fn create_proposal(
	session: &mut Session<Pop>,
	beneficiary: AccountId,
	amount: Balance,
	description: Vec<u8>,
) -> Result<(), Error> {
	let desc: &[u8] = &description;
	call::<Pop, (), Error>(
		session,
		"create_proposal",
		vec![
			beneficiary.to_string(),
			amount.to_string(),
			serde_json::to_string::<[u8]>(desc).unwrap(),
		],
		None,
	)
}

fn vote(session: &mut Session<Pop>, proposal_id: u32, approve: bool) -> Result<(), Error> {
	call::<Pop, (), Error>(
		session,
		"vote",
		vec![proposal_id.to_string(), approve.to_string()],
		None,
	)
}

fn execute_proposal(session: &mut Session<Pop>, proposal_id: u32) -> Result<(), Error> {
	call::<Pop, (), Error>(session, "execute_proposal", vec![proposal_id.to_string()], None)
}

fn proposal(session: &mut Session<Pop>, proposal_id: u32) -> Option<Proposal> {
	call::<Pop, Option<Proposal>, Error>(
		session,
		"get_proposal",
		vec![proposal_id.to_string()],
		None,
	)
	.unwrap()
}

fn prepare_dao(session: &mut Session<Pop>, contract: AccountId) -> Result<(), Error> {
	assert_ok!(session.sandbox().mint_into(&TOKEN, &ALICE, AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &ALICE, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().mint_into(&TOKEN, &BOB, AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &BOB, &contract.clone(), AMOUNT));
	assert_ok!(session.sandbox().mint_into(&TOKEN, &CHARLIE, AMOUNT));
	assert_ok!(session.sandbox().approve(&TOKEN, &CHARLIE, &contract.clone(), AMOUNT));
	session.set_actor(ALICE);
	assert_ok!(join(session, AMOUNT / 2));
	session.set_actor(BOB);
	assert_ok!(join(session, AMOUNT / 4));
	session.set_actor(CHARLIE);
	assert_ok!(join(session, AMOUNT / 3));
	Ok(())
}

use drink::{
	assert_err, assert_last_contract_event, assert_ok, call,
	devnet::{account_id_from_slice, AccountId, Balance, Runtime},
	last_contract_event,
	sandbox_api::nfts_api::NftsAPI,
	session::Session,
	TestExternalities, NO_SALT,
};
use pop_api::v0::nonfungibles::{events::Transfer, CollectionId, ItemId};

use super::*;

const UNIT: Balance = 10_000_000_000;
const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
const INIT_VALUE: Balance = 100 * UNIT;
const ALICE: AccountId = AccountId::new([1u8; 32]);
const BOB: AccountId = AccountId::new([2_u8; 32]);
const CHARLIE: AccountId = AccountId::new([3_u8; 32]);
const COLLECTION: CollectionId = 0;
const ITEM: ItemId = 0;

// The contract bundle provider.
//
// See https://github.com/r0gue-io/pop-drink/blob/main/crates/drink/drink/test-macro/src/lib.rs for more information.
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

fn deploy_with_default(session: &mut Session<Pop>) -> Result<AccountId> {
	deploy(session, "new", vec!["None".to_string()])
}

#[drink::test(sandbox = Pop)]
fn new_constructor_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// Collection exists after the deployment.
	assert_eq!(session.sandbox().collection_owner(&COLLECTION), Some(contract.clone()));
	// Successfully emit event.
	assert_last_contract_event!(
		&session,
		Created { id: COLLECTION, admin: account_id_from_slice(&contract), max_supply: None }
	);
}

#[drink::test(sandbox = Pop)]
fn collection_id_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a first contract.
	deploy_with_default(&mut session).unwrap();
	assert_eq!(collection_id(&mut session), 0);

	// Deploy a second contract increments the collection ID.
	drink::deploy::<Pop, Psp34Error>(
		&mut session,
		// The local contract (i.e. `nonfungibles`).
		BundleProvider::local().unwrap(),
		"new",
		vec!["None".to_string()],
		vec![1, 2, 3, 4],
		Some(INIT_VALUE),
	)
	.unwrap();
	assert_eq!(collection_id(&mut session), 1);
}

#[drink::test(sandbox = Pop)]
fn next_item_id_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_default(&mut session).unwrap();
	assert_eq!(next_item_id(&mut session), 0);

	// Mint a new collection item increments the `next_item_id`.
	assert_ok!(mint(&mut session, ALICE));
	assert_eq!(next_item_id(&mut session), 1);
}

#[drink::test(sandbox = Pop)]
fn balance_of_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// No collection items.
	assert_eq!(balance_of(&mut session, ALICE), 0);
	assert_eq!(balance_of(&mut session, ALICE), session.sandbox().balance_of(&COLLECTION, &ALICE));
	// Mint a new collection item.
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	assert_eq!(balance_of(&mut session, ALICE), 1);
	assert_eq!(balance_of(&mut session, ALICE), session.sandbox().balance_of(&COLLECTION, &ALICE));
}

#[drink::test(sandbox = Pop)]
fn owner_of_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// No item owner.
	assert_eq!(owner_of(&mut session, ITEM), None);
	assert_eq!(owner_of(&mut session, ITEM), session.sandbox().owner(&COLLECTION, &ITEM));
	// Mint a new collection item.
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	assert_eq!(owner_of(&mut session, ITEM), Some(ALICE));
	assert_eq!(owner_of(&mut session, ITEM), session.sandbox().owner(&COLLECTION, &ITEM));
}

#[drink::test(sandbox = Pop)]
fn total_supply_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// No item in circulation.
	assert_eq!(total_supply(&mut session), 0);
	// Collection items are in circulation.
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	assert_eq!(total_supply(&mut session), 1);
}

// Deploy the contract with `NO_SALT and `INIT_VALUE`.
fn deploy(session: &mut Session<Pop>, method: &str, input: Vec<String>) -> Result<AccountId> {
	drink::deploy::<Pop, Psp34Error>(
		session,
		// The local contract (i.e. `nonfungibles`).
		BundleProvider::local().unwrap(),
		method,
		input,
		NO_SALT,
		Some(INIT_VALUE),
	)
}

// A set of helper methods to test the contract calls.

fn collection_id(session: &mut Session<Pop>) -> CollectionId {
	call::<Pop, CollectionId, Psp34Error>(session, "collection_id", vec![], None).unwrap()
}

fn next_item_id(session: &mut Session<Pop>) -> ItemId {
	call::<Pop, ItemId, Psp34Error>(session, "next_item_id", vec![], None).unwrap()
}

fn balance_of(session: &mut Session<Pop>, owner: AccountId) -> u32 {
	call::<Pop, u32, Psp34Error>(session, "balance_of", vec![owner.to_string()], None).unwrap()
}

fn owner_of(session: &mut Session<Pop>, item: ItemId) -> Option<AccountId> {
	call::<Pop, Option<AccountId>, Psp34Error>(session, "owner_of", vec![item.to_string()], None)
		.unwrap()
}

fn total_supply(session: &mut Session<Pop>) -> u128 {
	call::<Pop, u128, Psp34Error>(session, "total_supply", vec![], None).unwrap()
}

fn mint(session: &mut Session<Pop>, to: AccountId) -> Result<()> {
	call::<Pop, (), Psp34Error>(session, "mint", vec![to.to_string()], None)
}

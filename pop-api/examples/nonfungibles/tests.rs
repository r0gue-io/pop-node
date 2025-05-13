use drink::{
	assert_err, assert_last_contract_event, assert_ok, call,
	devnet::{
		account_id_from_slice,
		error::{v0::Error, Nfts, NftsError::*},
		AccountId, Balance, Runtime,
	},
	sandbox_api::nfts_api::NftsAPI,
	session::Session,
	TestExternalities, Weight, NO_SALT,
};
use pop_api::v0::nonfungibles::{events::Transfer, CollectionId, ItemId};

use super::*;

const UNIT: Balance = 10_000_000_000;
const ALICE: AccountId = AccountId::new([1u8; 32]);
const BOB: AccountId = AccountId::new([2_u8; 32]);
const COLLECTION: CollectionId = 0;
const ITEM: ItemId = 0;
const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
const INIT_VALUE: Balance = 100 * UNIT;

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
		let balances: Vec<(AccountId, u128)> = vec![(ALICE, INIT_AMOUNT), (BOB, INIT_AMOUNT)];
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

	// Mint a new item increments the `next_item_id`.
	assert_ok!(mint(&mut session, ALICE));
	assert_eq!(next_item_id(&mut session), 1);
}

#[drink::test(sandbox = Pop)]
fn balance_of_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	// No items.
	assert_eq!(balance_of(&mut session, ALICE), 0);
	assert_eq!(balance_of(&mut session, ALICE), session.sandbox().balance_of(&COLLECTION, &ALICE));
	// Mint a new item.
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
	// Mint a new item.
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
	assert_eq!(total_supply(&mut session), session.sandbox().total_supply(COLLECTION));
	// Items are in circulation.
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	assert_eq!(total_supply(&mut session), 1);
	assert_eq!(total_supply(&mut session), session.sandbox().total_supply(COLLECTION));
}

#[drink::test(sandbox = Pop)]
fn mint_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_default(&mut session).unwrap();
	// Successfully mint a new item.
	assert_ok!(mint(&mut session, ALICE));
	assert_eq!(session.sandbox().total_supply(COLLECTION), 1);
	assert_eq!(session.sandbox().balance_of(&COLLECTION, &ALICE), 1);
	// Successfully emit event.
	assert_last_contract_event!(
		&session,
		Transfer { from: None, to: Some(account_id_from_slice(&ALICE)), item: ITEM }
	);
}

#[drink::test(sandbox = Pop)]
fn mint_fails_with_unauthorization(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	deploy_with_default(&mut session).unwrap();
	// Failed with `Not the contract owner`.
	session.set_actor(BOB);
	assert_eq!(
		mint(&mut session, ALICE),
		Err(Psp34Error::Custom("Not the contract owner".to_string()))
	);
}

#[drink::test(sandbox = Pop)]
fn mint_fails_with_max_suplly_reached(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract with a max total supply is one.
	assert_ok!(deploy(&mut session, "new", vec!["Some(1)".to_string()]));
	// Failed with `MaxSupplyReached`.
	assert_ok!(mint(&mut session, ALICE));
	assert_err!(mint(&mut session, ALICE), Error::Module(Nfts(MaxSupplyReached)));
}

#[drink::test(sandbox = Pop)]
fn burn_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		contract.clone().into(),
		None
	));
	// Successfully burn an item.
	assert_ok!(burn(&mut session, ITEM));
	assert_eq!(session.sandbox().total_supply(COLLECTION), 0);
	assert_eq!(session.sandbox().balance_of(&COLLECTION, &ALICE), 0);
	// Successfully emit event.
	assert_last_contract_event!(
		&session,
		Transfer { from: Some(account_id_from_slice(&contract)), to: None, item: ITEM }
	);
}

#[drink::test(sandbox = Pop)]
fn burn_fails_with_unauthorization(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		contract.into(),
		None
	));
	// Failed with `Not the contract owner`.
	session.set_actor(BOB);
	assert_eq!(
		burn(&mut session, ITEM),
		Err(Psp34Error::Custom("Not the contract owner".to_string()))
	);
}

#[drink::test(sandbox = Pop)]
fn burn_fails_with_not_approved(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	// Failed with `NotApproved`.
	assert_eq!(burn(&mut session, ITEM), Err(Psp34Error::NotApproved));
}

#[drink::test(sandbox = Pop)]
fn burn_fails_with_invalid_item(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	// Failed with `TokenNotExists`.
	assert_eq!(burn(&mut session, ITEM + 1), Err(Psp34Error::TokenNotExists));
}

#[drink::test(sandbox = Pop)]
fn transfer_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		contract.clone().into(),
		None
	));
	// Successfully transfer an item.
	assert_ok!(transfer(&mut session, ALICE, ITEM));
	assert_eq!(session.sandbox().owner(&COLLECTION, &ITEM), Some(ALICE));
	assert_eq!(session.sandbox().balance_of(&COLLECTION, &ALICE), 1);
	assert_last_contract_event!(
		&session,
		Transfer {
			from: Some(account_id_from_slice(&contract)),
			to: Some(account_id_from_slice(&ALICE)),
			item: ITEM
		}
	);
}

#[drink::test(sandbox = Pop)]
fn transfer_fails_with_unauthorization(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		contract.into(),
		None
	));
	// Failed with `Not the contract owner`.
	session.set_actor(BOB);
	assert_eq!(
		transfer(&mut session, ALICE, ITEM),
		Err(Psp34Error::Custom("Not the contract owner".to_string()))
	);
}

#[drink::test(sandbox = Pop)]
fn transfer_fails_with_not_approved(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	// Failed with `NotApproved`.
	assert_eq!(transfer(&mut session, ALICE, ITEM), Err(Psp34Error::NotApproved));
}

#[drink::test(sandbox = Pop)]
fn transfer_fails_with_invalid_item(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	let contract = deploy_with_default(&mut session).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	// Failed with `TokenNotExists`.
	assert_eq!(transfer(&mut session, ALICE, ITEM + 1), Err(Psp34Error::TokenNotExists));
}

#[drink::test(sandbox = Pop)]
fn destroy_works(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	assert_ok!(deploy_with_default(&mut session));
	// Successfully destroy a collection.
	session.set_gas_limit(Weight::MAX);
	assert_ok!(destroy(
		&mut session,
		DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
	));
	assert_eq!(session.sandbox().collection(&COLLECTION), None);
	// Successfully emit event.
	assert_last_contract_event!(&session, Destroyed { id: COLLECTION });
}

#[drink::test(sandbox = Pop)]
fn destroy_fails_with_unauthorization(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	assert_ok!(deploy_with_default(&mut session));
	// Failed with `Not the contract owner`.
	session.set_gas_limit(Weight::MAX);
	session.set_actor(BOB);
	assert_eq!(
		destroy(&mut session, DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }),
		Err(Psp34Error::Custom("Not the contract owner".to_string()))
	);
}

#[drink::test(sandbox = Pop)]
fn destroy_fails_with_bad_witness(mut session: Session) {
	let _ = env_logger::try_init();
	// Deploy a new contract.
	assert_ok!(deploy_with_default(&mut session));
	// Successfully destroy a collection.
	session.set_gas_limit(Weight::MAX);
	// Failed with `BadWitness`.
	assert_err!(
		destroy(&mut session, DestroyWitness { item_metadatas: 1, item_configs: 1, attributes: 1 }),
		Error::Module(Nfts(BadWitness))
	);
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

fn burn(session: &mut Session<Pop>, item: ItemId) -> Result<()> {
	call::<Pop, (), Psp34Error>(session, "burn", vec![item.to_string()], None)
}

fn transfer(session: &mut Session<Pop>, to: AccountId, item: ItemId) -> Result<()> {
	call::<Pop, (), Psp34Error>(session, "transfer", vec![to.to_string(), item.to_string()], None)
}

fn destroy(session: &mut Session<Pop>, witness: DestroyWitness) -> Result<()> {
	let witness_string = format!("{:?}", witness);
	call::<Pop, (), Psp34Error>(session, "destroy", vec![witness_string], None)
}

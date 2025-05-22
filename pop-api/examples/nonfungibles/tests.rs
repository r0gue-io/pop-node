use drink::{
	assert_err, assert_last_contract_event, assert_ok, call_with_address,
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

#[cfg(debug_assertions)]
compile_error!("Tests must be run using the release profile (--release)");

use super::*;

const UNIT: Balance = 10_000_000_000;
const ALICE: AccountId = AccountId::new([1u8; 32]);
const BOB: AccountId = AccountId::new([2_u8; 32]);
const COLLECTION: CollectionId = 0;
const ITEM: ItemId = 0;
const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
const INIT_VALUE: Balance = 100 * UNIT;

/// Sandbox environment for Pop Devnet Runtime.
pub struct Pop {
	ext: TestExternalities,
}

impl Default for Pop {
	fn default() -> Self {
		let _ = env_logger::try_init();
		// Initialising genesis state, providing accounts with an initial balance.
		let balances: Vec<(AccountId, u128)> = vec![(ALICE, INIT_AMOUNT), (BOB, INIT_AMOUNT)];
		let ext = BlockBuilder::<Runtime>::new_ext(balances);
		Self { ext }
	}
}

// Implement core functionalities for the `Pop` sandbox.
drink::impl_sandbox!(Pop, Runtime, ALICE);

#[drink::test(sandbox = Pop)]
fn new_constructor_works(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	// Collection exists after the deployment.
	assert_eq!(session.sandbox().collection_owner(&COLLECTION).as_ref(), Some(&contract.address));
	// Successfully emits an event.
	assert_last_contract_event!(
		&session,
		Created {
			id: COLLECTION,
			admin: account_id_from_slice(&contract.address),
			max_supply: None
		}
	);
}

#[drink::test(sandbox = Pop)]
fn collection_id_works(mut session: Session) {
	// Deploys a first contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	assert_eq!(contract.collection_id(&mut session), 0);

	// Deploys a second contract which increments the collection ID.
	let contract = Contract::new(&mut session, None, vec![1, 2, 3, 4]).unwrap();
	assert_eq!(contract.collection_id(&mut session), 1);
}

#[drink::test(sandbox = Pop)]
fn next_item_id_works(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	assert_eq!(contract.next_item_id(&mut session), 0);

	// Mint a new item increments the `next_item_id`.
	assert_ok!(contract.mint(&mut session, ALICE));
	assert_eq!(contract.next_item_id(&mut session), 1);
}

#[drink::test(sandbox = Pop)]
fn balance_of_works(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	// No items.
	assert_eq!(contract.balance_of(&mut session, ALICE), 0);
	assert_eq!(
		contract.balance_of(&mut session, ALICE),
		session.sandbox().balance_of(&COLLECTION, &ALICE)
	);
	// Mint a new item.
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	assert_eq!(contract.balance_of(&mut session, ALICE), 1);
	assert_eq!(
		contract.balance_of(&mut session, ALICE),
		session.sandbox().balance_of(&COLLECTION, &ALICE)
	);
}

#[drink::test(sandbox = Pop)]
fn owner_of_works(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	// No item owner.
	assert_eq!(contract.owner_of(&mut session, ITEM), None);
	assert_eq!(contract.owner_of(&mut session, ITEM), session.sandbox().owner(&COLLECTION, &ITEM));
	// Mint a new item.
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	assert_eq!(contract.owner_of(&mut session, ITEM), Some(ALICE));
	assert_eq!(contract.owner_of(&mut session, ITEM), session.sandbox().owner(&COLLECTION, &ITEM));
}

#[drink::test(sandbox = Pop)]
fn total_supply_works(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	// No item in circulation.
	assert_eq!(contract.total_supply(&mut session), 0);
	assert_eq!(
		contract.total_supply(&mut session),
		session.sandbox().total_supply(COLLECTION) as u128
	);
	// Items are in circulation.
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	assert_eq!(contract.total_supply(&mut session), 1);
	assert_eq!(
		contract.total_supply(&mut session),
		session.sandbox().total_supply(COLLECTION) as u128
	);
}

#[drink::test(sandbox = Pop)]
fn mint_works(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	// Successfully mints a new item.
	assert_ok!(contract.mint(&mut session, ALICE));
	assert_eq!(session.sandbox().total_supply(COLLECTION), 1);
	assert_eq!(session.sandbox().balance_of(&COLLECTION, &ALICE), 1);
	// Successfully emits an event.
	assert_last_contract_event!(
		&session,
		Transfer { from: None, to: Some(account_id_from_slice(&ALICE)), item: ITEM }
	);
}

#[drink::test(sandbox = Pop)]
fn mint_fails_with_unauthorization(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	// Fails with `Not the contract owner`.
	session.set_actor(BOB);
	assert_eq!(
		contract.mint(&mut session, ALICE),
		Err(Psp34Error::Custom("Not the contract owner".to_string()))
	);
}

#[drink::test(sandbox = Pop)]
fn mint_fails_with_max_supply_reached(mut session: Session) {
	// Deploys a new contract with a max total supply is one.
	let contract = Contract::new(&mut session, Some(1), vec![]).unwrap();
	assert_ok!(contract.mint(&mut session, ALICE));
	// Fails with `MaxSupplyReached`.
	assert_err!(contract.mint(&mut session, ALICE), Error::Module(Nfts(MaxSupplyReached)));
}

#[drink::test(sandbox = Pop)]
fn burn_works(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		contract.address.clone().into(),
		None
	));
	// Successfully burns an item.
	assert_ok!(contract.burn(&mut session, ITEM));
	assert_eq!(session.sandbox().total_supply(COLLECTION), 0);
	assert_eq!(session.sandbox().balance_of(&COLLECTION, &ALICE), 0);
	// Successfully emits an event.
	assert_last_contract_event!(
		&session,
		Transfer { from: Some(account_id_from_slice(&contract.address)), to: None, item: ITEM }
	);
}

#[drink::test(sandbox = Pop)]
fn burn_fails_with_unauthorization(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		contract.address.clone().into(),
		None
	));
	// Fails with `Not the contract owner`.
	session.set_actor(BOB);
	assert_eq!(
		contract.burn(&mut session, ITEM),
		Err(Psp34Error::Custom("Not the contract owner".to_string()))
	);
}

#[drink::test(sandbox = Pop)]
fn burn_fails_with_not_approved(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	// Fails with `NotApproved`.
	assert_eq!(contract.burn(&mut session, ITEM), Err(Psp34Error::NotApproved));
}

#[drink::test(sandbox = Pop)]
fn burn_fails_with_invalid_item(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	// Fails with `TokenNotExists`.
	assert_eq!(contract.burn(&mut session, ITEM + 1), Err(Psp34Error::TokenNotExists));
}

#[drink::test(sandbox = Pop)]
fn transfer_works(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		contract.address.clone().into(),
		None
	));
	// Successfully transfers an item.
	assert_ok!(contract.transfer(&mut session, ALICE, ITEM));
	assert_eq!(session.sandbox().owner(&COLLECTION, &ITEM), Some(ALICE));
	assert_eq!(session.sandbox().balance_of(&COLLECTION, &ALICE), 1);
	assert_last_contract_event!(
		&session,
		Transfer {
			from: Some(account_id_from_slice(&contract.address)),
			to: Some(account_id_from_slice(&ALICE)),
			item: ITEM
		}
	);
}

#[drink::test(sandbox = Pop)]
fn transfer_fails_with_unauthorization(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		contract.address.clone().into(),
		None
	));
	// Fails with `Not the contract owner`.
	session.set_actor(BOB);
	assert_eq!(
		contract.transfer(&mut session, ALICE, ITEM),
		Err(Psp34Error::Custom("Not the contract owner".to_string()))
	);
}

#[drink::test(sandbox = Pop)]
fn transfer_fails_with_not_approved(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	// Fails with `NotApproved`.
	assert_eq!(contract.transfer(&mut session, ALICE, ITEM), Err(Psp34Error::NotApproved));
}

#[drink::test(sandbox = Pop)]
fn transfer_fails_with_invalid_item(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	assert_ok!(session.sandbox().mint(
		Some(contract.address.clone()),
		COLLECTION,
		ITEM,
		ALICE.into(),
		None
	));
	// Fails with `TokenNotExists`.
	assert_eq!(contract.transfer(&mut session, ALICE, ITEM + 1), Err(Psp34Error::TokenNotExists));
}

#[drink::test(sandbox = Pop)]
fn destroy_works(mut session: Session) {
	// Deploys a new contract.
	assert_ok!(Contract::new(&mut session, None, NO_SALT));
	// Successfully destroys a collection.
	session.set_gas_limit(Weight::MAX);
	let witness_string =
		format!("{:?}", DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 });

	// Error returned "Not enough data to fill buffer" due to contract termination.
	assert!(session.call::<String, ()>("destroy", &[witness_string], None).is_err());
	assert_eq!(session.sandbox().collection(&COLLECTION), None);
	// Successfully emits an event.
	assert_last_contract_event!(&session, Destroyed { id: COLLECTION });
}

#[drink::test(sandbox = Pop)]
fn destroy_fails_with_unauthorization(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	// Fails with `Not the contract owner`.
	session.set_gas_limit(Weight::MAX);
	session.set_actor(BOB);
	assert_eq!(
		contract.destroy(
			&mut session,
			DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 }
		),
		Err(Psp34Error::Custom("Not the contract owner".to_string()))
	);
}

#[drink::test(sandbox = Pop)]
fn destroy_fails_with_bad_witness(mut session: Session) {
	// Deploys a new contract.
	let contract = Contract::new(&mut session, None, NO_SALT).unwrap();
	// Fails with `BadWitness`.
	session.set_gas_limit(Weight::MAX);
	assert_err!(
		contract.destroy(
			&mut session,
			DestroyWitness { item_metadatas: 1, item_configs: 1, attributes: 1 }
		),
		Error::Module(Nfts(BadWitness))
	);
}

// A set of helper methods to test the contract deployment and calls.

#[derive(Debug)]
struct Contract {
	pub address: AccountId,
}

impl Contract {
	// Deploy a new contract.
	fn new(session: &mut Session<Pop>, max_supply: Option<u32>, salt: Vec<u8>) -> Result<Self> {
		// The contract bundle provider.
		//
		// See https://github.com/r0gue-io/pop-drink/blob/main/crates/drink/drink/test-macro/src/lib.rs for more information.
		#[drink::contract_bundle_provider]
		enum BundleProvider {}

		let contract = drink::deploy::<Pop, Psp34Error>(
			session,
			// The local contract (i.e. `nonfungibles`).
			BundleProvider::local().unwrap(),
			"new",
			vec![format!("{:?}", max_supply)],
			salt,
			Some(INIT_VALUE),
		)?;
		Ok(Self { address: contract })
	}

	fn collection_id(&self, session: &mut Session<Pop>) -> CollectionId {
		call_with_address::<Pop, CollectionId, Psp34Error>(
			session,
			self.address.clone(),
			"collection_id",
			vec![],
			None,
		)
		.unwrap()
	}

	fn next_item_id(&self, session: &mut Session<Pop>) -> ItemId {
		call_with_address::<Pop, ItemId, Psp34Error>(
			session,
			self.address.clone(),
			"next_item_id",
			vec![],
			None,
		)
		.unwrap()
	}

	fn balance_of(&self, session: &mut Session<Pop>, owner: AccountId) -> u32 {
		call_with_address::<Pop, u32, Psp34Error>(
			session,
			self.address.clone(),
			"balance_of",
			vec![owner.to_string()],
			None,
		)
		.unwrap()
	}

	fn owner_of(&self, session: &mut Session<Pop>, item: ItemId) -> Option<AccountId> {
		call_with_address::<Pop, Option<AccountId>, Psp34Error>(
			session,
			self.address.clone(),
			"owner_of",
			vec![item.to_string()],
			None,
		)
		.unwrap()
	}

	fn total_supply(&self, session: &mut Session<Pop>) -> u128 {
		call_with_address::<Pop, u128, Psp34Error>(
			session,
			self.address.clone(),
			"total_supply",
			vec![],
			None,
		)
		.unwrap()
	}

	fn mint(&self, session: &mut Session<Pop>, to: AccountId) -> Result<()> {
		call_with_address::<Pop, (), Psp34Error>(
			session,
			self.address.clone(),
			"mint",
			vec![to.to_string()],
			None,
		)
	}

	fn burn(&self, session: &mut Session<Pop>, item: ItemId) -> Result<()> {
		call_with_address::<Pop, (), Psp34Error>(
			session,
			self.address.clone(),
			"burn",
			vec![item.to_string()],
			None,
		)
	}

	fn transfer(&self, session: &mut Session<Pop>, to: AccountId, item: ItemId) -> Result<()> {
		call_with_address::<Pop, (), Psp34Error>(
			session,
			self.address.clone(),
			"transfer",
			vec![to.to_string(), item.to_string()],
			None,
		)
	}

	fn destroy(&self, session: &mut Session<Pop>, witness: DestroyWitness) -> Result<()> {
		let witness_string = format!("{:?}", witness);
		call_with_address::<Pop, (), Psp34Error>(
			session,
			self.address.clone(),
			"destroy",
			vec![witness_string],
			None,
		)
	}
}

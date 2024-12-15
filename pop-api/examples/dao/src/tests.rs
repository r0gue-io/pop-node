use drink::{
	assert_err, assert_last_contract_event, assert_ok, call, deploy,
	devnet::{
		account_id_from_slice,
		error::{
			v0::{ApiError::*, ArithmeticError::*, Error},
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

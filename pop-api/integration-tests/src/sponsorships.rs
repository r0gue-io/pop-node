use frame_support::{dispatch::DispatchInfo, traits::fungible::Inspect};
use pallet_revive::Call;
use pallet_sponsorships::sponsored::Sponsored;
use pallet_transaction_payment::ChargeTransactionPayment;
use pop_api::primitives::Error;
use pop_runtime_devnet::{Balances, RuntimeCall, Sponsorships};
use sp_runtime::{app_crypto::sp_core::H160, traits::SignedExtension};

use super::*;

const CONTRACT: &str = "contracts/sponsorships/target/ink/sponsorships.riscv";

fn create_bare_call(addr: H160, input: Vec<u8>, value: u128) -> RuntimeCall {
	RuntimeCall::Revive(Call::<Runtime>::call {
		dest: addr.into(),
		value: value.into(),
		gas_limit: GAS_LIMIT,
		storage_deposit_limit: 1 * 1_000_000_000_000,
		data: input,
	})
}

#[test]
fn create_sponsorship_works() {
	let beneficiary = BOB;
	new_test_ext().execute_with(|| {
		// ALICE is the deployer and becomes the owner of the contract
		let contract = Contract::new();
		// ALICE signs BOB up to be sponsored by contract for INIT_VALUE.
		assert_ok!(contract.sign_up(&beneficiary, INIT_VALUE));
		assert_eq!(Sponsorships::is_sponsored_by(&beneficiary, &contract.0 .1), Some(INIT_VALUE));
	});
}

#[test]
fn remove_sponsorship_works() {
	let beneficiary = BOB;
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		// ALICE signs BOB up to be sponsored by contract for INIT_VALUE.
		assert_ok!(contract.sign_up(&beneficiary, INIT_VALUE));
		assert_eq!(Sponsorships::is_sponsored_by(&beneficiary, &contract.0 .1), Some(INIT_VALUE));
		// BOB withdraws sponsorship
		assert_ok!(contract.withdraw_sponsorship(&beneficiary));
		assert_eq!(Sponsorships::is_sponsored_by(&beneficiary, &contract.0 .1), None);
	})
}

#[test]
fn beneficiary_can_flip_without_paying_fees() {
	let beneficiary = BOB;
	new_test_ext().execute_with(|| {
		// ALICE is the deployer and becomes the owner of the contract
		let contract = Contract::new();
		// ALICE signs BOB up to be sponsored by contract for INIT_VALUE.
		assert_ok!(contract.sign_up(&beneficiary, INIT_VALUE));
		assert_eq!(Sponsorships::is_sponsored_by(&beneficiary, &contract.0 .1), Some(INIT_VALUE));
		// println!("Sponsored amount pre call: {}",Sponsorships::is_sponsored_by(&beneficiary,
		// &contract.0 .1).unwrap()); capture free balances before calling the contract.
		let beneficiary_balance_pre_call = Balances::balance(&beneficiary);
		let contract_balance_pre_call = Balances::balance(&contract.0 .1);

		let charge_payment: ChargeTransactionPayment<Runtime> =
			ChargeTransactionPayment::<Runtime>::from(0);

		// Instantiate Sponsored signed extension
		let sponsored: Sponsored<Runtime, ChargeTransactionPayment<Runtime>> =
			Sponsored::from(charge_payment);
		let pre = sponsored.pre_dispatch(
			&beneficiary,
			&contract.get_flip_call(),
			&DispatchInfo::default(),
			10, // arbitrary length
		);

		// beneficiary flips
		assert_ok!(contract.flip(&beneficiary));
		println!("EVENTS: {:?}", System::events());
		// beneficiary's free balance has not decreased.
		assert_eq!(beneficiary_balance_pre_call, Balances::balance(&beneficiary));
		// contract's free balance has decreased
		// println!("Contract pre call: {}",contract_free_balance_pre_call);
		// println!("Contract post call: {}",Balances::free_balance(&contract.0 .1));
		println!(
			"Sponsored amount post call: {}",
			Sponsorships::is_sponsored_by(&beneficiary, &contract.0 .1).unwrap()
		);
		// assert!(contract_balance_pre_call > Balances::balance(&contract.0 .1));
		assert!(Sponsorships::is_sponsored_by(&beneficiary, &contract.0 .1).unwrap() < INIT_VALUE);
	});
}

// A simple, strongly typed wrapper for the contract.
struct Contract((H160, AccountId32));

impl Contract {
	fn new() -> Self {
		Self(instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]))
	}

	pub fn sign_up(&self, user: &AccountId32, value: u128) -> Result<(), Error> {
		let result = self.call("sign_up", user.encode(), value);
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	pub fn withdraw_sponsorship(&self, beneficiary: &AccountId32) -> Result<(), Error> {
		let result = self.call_with_signer(
			"withdraw_sponsorship",
			Default::default(),
			0,
			beneficiary.clone(),
		);
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	pub fn flip(&self, signer: &AccountId32) -> Result<(), Error> {
		let result = self.call_with_signer("flip", Default::default(), 0, signer.clone());
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	pub fn get_flip_call(&self) -> RuntimeCall {
		self.create_call("flip", Default::default(), 0)
	}

	fn call(&self, function: &str, params: Vec<u8>, value: u128) -> ExecReturnValue {
		let function = function_selector(function);
		let params = [function, params].concat();
		bare_call(self.0.clone().0, params, value).expect("should work")
	}

	fn call_with_signer(
		&self,
		function: &str,
		params: Vec<u8>,
		value: u128,
		signer: AccountId32,
	) -> ExecReturnValue {
		let function = function_selector(function);
		let params = [function, params].concat();
		bare_call_with_signer(self.0.clone().0, params, value, signer).expect("should work")
	}

	fn create_call(&self, function: &str, params: Vec<u8>, value: u128) -> RuntimeCall {
		let function = function_selector(function);
		let params = [function, params].concat();
		create_bare_call(self.0.clone().0, params, value)
	}
}

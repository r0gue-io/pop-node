use pop_api::primitives::Error;
use pop_runtime_devnet::Sponsorships;
use sp_runtime::app_crypto::sp_core::H160;

use super::*;

const CONTRACT: &str = "contracts/sponsorships/target/ink/sponsorships.riscv";

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
		// BOB removes sponsorship
		assert_ok!(contract.remove_sponsorship(&beneficiary));
		assert_eq!(Sponsorships::is_sponsored_by(&beneficiary, &contract.0 .1), None);
	})
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

	pub fn remove_sponsorship(&self, beneficiary: &AccountId32) -> Result<(), Error> {
		let result =
			self.call_with_signer("remove_sponsorship", Default::default(), 0, beneficiary.clone());
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	pub fn flip(&self, signer: &AccountId32) -> Result<(), Error> {
		let result = self.call_with_signer("flip", Default::default(), 0, signer.clone());
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
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
}

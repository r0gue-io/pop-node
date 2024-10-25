use frame_support::traits::{fungible::Inspect, Currency};
use pop_api::primitives::{Era, Error};
use pop_runtime_devnet::{Balances, Incentives, MINUTES};
use sp_runtime::app_crypto::sp_core::H160;

use super::*;

const CONTRACT: &str = "../examples/incentives/target/ink/incentives.riscv";

#[test]
fn register_works() {
	let beneficiary = BOB;
	new_test_ext().execute_with(|| {
		let contract = Contract::new(&beneficiary);
		assert!(Incentives::is_registered(&contract.0 .1));
	});
}

#[test]
fn claim_works() {
	let beneficiary = BOB;
	let amount = 1_000_000_000;
	const ERA: u32 = 10 * MINUTES;
	new_test_ext().execute_with(|| {
		let contract = Contract::new(&beneficiary);

		// Fund pallet and accrue fees to contract
		Balances::make_free_balance_be(&Incentives::get_pallet_account(), amount);
		for block in 1..=ERA + 1 {
			Incentives::accrue_fees(block, &contract.0 .1, amount / ERA as u128);
		}

		let balance = Balances::balance(&beneficiary);
		assert_ok!(contract.claim_rewards(1));
		assert_eq!(Balances::balance(&beneficiary), balance + amount)
	});
}

// A simple, strongly typed wrapper for the contract.
struct Contract((H160, AccountId32));
impl Contract {
	fn new(beneficiary: &AccountId32) -> Self {
		Self(instantiate(
			CONTRACT,
			INIT_VALUE,
			[function_selector("new"), beneficiary.encode()].concat(),
			vec![],
		))
	}

	fn claim_rewards(&self, era: Era) -> Result<(), Error> {
		let result = self.call("claim_rewards", era.encode(), 0);
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn call(&self, function: &str, params: Vec<u8>, value: u128) -> ExecReturnValue {
		let function = function_selector(function);
		let params = [function, params].concat();
		let result = bare_call(self.0.clone().0, params, value).expect("should work");
		assert!(!result.did_revert(), "calling contract reverted {:?}", result);
		result
	}
}

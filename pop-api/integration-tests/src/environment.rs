use frame_support::traits::fungible::Inspect;
use pop_api::primitives::{Error, Hash, Timestamp};
use pop_runtime_devnet::EXISTENTIAL_DEPOSIT;
use sp_runtime::app_crypto::sp_core::H160;

use super::*;

const CONTRACT: &str = "contracts/environment/target/ink/environment.riscv";

#[test]
fn caller_works() {
	let caller = ALICE;
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert_eq!(contract.caller(), caller);
	});
}

#[test]
fn transferred_value_works() {
	let value = 10_000_000_000;
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert_eq!(contract.transferred_value(value), value);
	});
}

#[test]
fn weight_to_fee_works() {
	let gas = u64::MAX;
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert_eq!(contract.weight_to_fee(gas), 160_000_000);
	});
}

#[test]
fn block_timestamp_works() {
	let timestamp = Timestamp::MAX;
	new_test_ext().execute_with(|| {
		pallet_timestamp::Now::<Runtime>::put(timestamp);
		let contract = Contract::new();
		assert_eq!(contract.block_timestamp(), timestamp);
	});
}

#[test]
fn account_id_works() {
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert_eq!(contract.account_id(), contract.0 .1);
	});
}

#[test]
fn balance_works() {
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert_eq!(contract.balance(), INIT_VALUE);
	});
}

#[test]
fn block_number_works() {
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert_eq!(contract.block_number(), 1);
	});
}

#[test]
fn minimum_balance_works() {
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert_eq!(contract.minimum_balance(), EXISTENTIAL_DEPOSIT);
	});
}

#[test]
#[ignore]
fn invoke_contract_works() {
	todo!()
}

#[test]
#[ignore]
fn invoke_contract_delegate_works() {
	todo!()
}

#[test]
#[ignore]
fn instantiate_contract_works() {
	todo!()
}

#[test]
fn terminate_contract_works() {
	let beneficiary = BOB;
	new_test_ext().execute_with(|| {
		let balance = pallet_balances::Pallet::<Runtime>::balance(&beneficiary);
		let contract = Contract::new();
		let contract_balance = contract.balance();
		contract.terminate_contract(&beneficiary);
		assert!(
			// Storage deposit also goes to beneficiary.
			pallet_balances::Pallet::<Runtime>::balance(&beneficiary) > balance + contract_balance
		);
	});
}

#[test]
fn transfer_works() {
	let destination = BOB;
	let value = 10_000_000_000;
	new_test_ext().execute_with(|| {
		let balance = pallet_balances::Pallet::<Runtime>::balance(&destination);
		let contract = Contract::new();
		assert_ok!(contract.transfer(destination.clone(), value));
		assert_eq!(pallet_balances::Pallet::<Runtime>::balance(&destination), balance + value);
		assert_eq!(
			contract.last_event(),
			Transferred { transferred_value: value, destination }.encode()
		);
	});
}

#[test]
fn is_contract_works() {
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert!(contract.is_contract(&contract.0 .1));
		assert_eq!(contract.is_contract(&ALICE), false);
	});
}

#[test]
fn caller_is_origin_works() {
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert!(contract.caller_is_origin());
	});
}

#[test]
fn code_hash_works() {
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert!(contract.code_hash(&contract.0 .1).is_some());
		assert!(contract.code_hash(&ALICE).is_none());
	});
}

#[test]
fn own_code_hash_works() {
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		assert!(contract.own_code_hash().is_some());
	});
}

#[test]
#[ignore]
fn call_runtime_works() {
	todo!()
}

#[test]
#[ignore]
fn lock_delegate_dependency_works() {
	todo!()
}

#[test]
#[ignore]
fn unlock_delegate_dependency_works() {
	todo!()
}

#[test]
#[ignore]
fn xcm_execute_works() {
	todo!()
}

#[test]
#[ignore]
fn xcm_send_works() {
	todo!()
}

// A simple, strongly typed wrapper for the contract.
struct Contract((H160, AccountId32));
impl Contract {
	fn new() -> Self {
		Self(instantiate(CONTRACT, INIT_VALUE, function_selector("new"), vec![]))
	}

	fn caller(&self) -> AccountId32 {
		let result = self.call("caller", Default::default(), 0);
		<AccountId32>::decode(&mut &result.data[1..]).unwrap()
	}

	fn transferred_value(&self, value: Balance) -> Balance {
		let result = self.call("transferred_value", Default::default(), value);
		<Balance>::decode(&mut &result.data[1..]).unwrap()
	}

	fn weight_to_fee(&self, gas: u64) -> Balance {
		let result = self.call("weight_to_fee", gas.encode(), 0);
		<Balance>::decode(&mut &result.data[1..]).unwrap()
	}

	fn block_timestamp(&self) -> Timestamp {
		let result = self.call("block_timestamp", Default::default(), 0);
		<Timestamp>::decode(&mut &result.data[1..]).unwrap()
	}

	fn account_id(&self) -> AccountId32 {
		let result = self.call("account_id", Default::default(), 0);
		<AccountId32>::decode(&mut &result.data[1..]).unwrap()
	}

	fn balance(&self) -> Balance {
		let result = self.call("balance", Default::default(), 0);
		<Balance>::decode(&mut &result.data[1..]).unwrap()
	}

	fn block_number(&self) -> u32 {
		let result = self.call("block_number", Default::default(), 0);
		<u32>::decode(&mut &result.data[1..]).unwrap()
	}

	fn minimum_balance(&self) -> Balance {
		let result = self.call("minimum_balance", Default::default(), 0);
		<Balance>::decode(&mut &result.data[1..]).unwrap()
	}

	fn terminate_contract(&self, beneficiary: &AccountId32) {
		self.call("terminate_contract", beneficiary.encode(), 0);
	}

	fn transfer(&self, destination: AccountId32, value: Balance) -> Result<(), Error> {
		let result = self.call("transfer", destination.encode(), value);
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn is_contract(&self, account: &AccountId32) -> bool {
		let result = self.call("is_contract", account.encode(), 0);
		<bool>::decode(&mut &result.data[1..]).unwrap()
	}

	fn caller_is_origin(&self) -> bool {
		let result = self.call("caller_is_origin", Default::default(), 0);
		<bool>::decode(&mut &result.data[1..]).unwrap()
	}

	fn code_hash(&self, account: &AccountId32) -> Option<Hash> {
		let result = self.call("code_hash", account.encode(), 0);
		<Option<Hash>>::decode(&mut &result.data[1..]).unwrap()
	}

	fn own_code_hash(&self) -> Option<Hash> {
		let result = self.call("own_code_hash", Default::default(), 0);
		<Option<Hash>>::decode(&mut &result.data[1..]).unwrap()
	}

	fn call(&self, function: &str, params: Vec<u8>, value: u128) -> ExecReturnValue {
		let function = function_selector(function);
		let params = [function, params].concat();
		let result = bare_call(self.0.clone().0, params, value).expect("should work");
		assert!(!result.did_revert(), "calling contract reverted {:?}", result);
		result
	}

	fn last_event(&self) -> Vec<u8> {
		let events = System::read_events_for_pallet::<pallet_revive::Event<Runtime>>();
		let contract_events = events
			.iter()
			.filter_map(|event| match event {
				pallet_revive::Event::<Runtime>::ContractEmitted { contract, data, .. }
					if contract == &self.0 .0 =>
					Some(data.as_slice()),
				_ => None,
			})
			.collect::<Vec<&[u8]>>();
		contract_events.last().unwrap().to_vec()
	}
}

#[derive(Encode)]
pub struct Transferred {
	pub transferred_value: Balance,
	pub destination: AccountId32,
}

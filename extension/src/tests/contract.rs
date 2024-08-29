use std::sync::LazyLock;

use super::utils::{call, initialize_contract, instantiate};
use crate::{
	mock::{self, *},
	ErrorConverter,
};
use codec::Decode;
use frame_system::Call;
use pallet_contracts::StorageDeposit;
use sp_runtime::{
	DispatchError::{self, BadOrigin, Module},
	ModuleError,
};

static CONTRACT: LazyLock<Vec<u8>> =
	LazyLock::new(|| initialize_contract("contract/target/ink/proxy.wasm"));

#[test]
fn dispatch_call_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let call = call(
			contract,
			DispatchCallEverthingFuncId::get(),
			RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() }),
			GAS_LIMIT,
		);
		// Successfully return data.
		let return_value = call.result.unwrap();
		let decoded = <Result<Vec<u8>, u32>>::decode(&mut &return_value.data[..]).unwrap();
		assert!(decoded.unwrap().is_empty());
		// Sucessfully emit event.
		assert!(call.events.unwrap().iter().any(|e| matches!(e.event,
				RuntimeEvent::System(frame_system::Event::<Test>::Remarked { sender, .. })
					if sender == contract)));
		assert_eq!(call.storage_deposit, StorageDeposit::Charge(0));
	});
}

#[test]
fn dispatch_call_filterting_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let call = call(
			contract,
			DispatchCallNothingFuncId::get(),
			RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() }),
			GAS_LIMIT,
		);
		assert_eq!(
			call.result,
			Err(Module(ModuleError {
				index: 0,
				error: [5, 0, 0, 0],
				message: Some("CallFiltered")
			}))
		);
	});
}

#[test]
fn dispatch_call_return_error_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let call = call(
			contract,
			DispatchCallEverthingFuncId::get(),
			// `set_code` requires root origin, expect throwing error.
			RuntimeCall::System(Call::set_code { code: "pop".as_bytes().to_vec() }),
			GAS_LIMIT,
		);
		assert_eq!(call.result.err(), Some(BadOrigin))
	})
}

#[test]
fn read_state_filterting_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let call = call(contract, ReadStateNothingFuncId::get(), RuntimeRead::Ping, GAS_LIMIT); // Successfully return data.
		assert_eq!(
			call.result,
			Err(Module(ModuleError {
				index: 0,
				error: [5, 0, 0, 0],
				message: Some("CallFiltered")
			}))
		);
	});
}

#[test]
fn read_state_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let call = call(contract, ReadStateEverthingFuncId::get(), RuntimeRead::Ping, GAS_LIMIT); // Successfully return data.
		let return_value = call.result.unwrap();
		let decoded = <Result<Vec<u8>, u32>>::decode(&mut &return_value.data[1..]).unwrap();
		let result = Ok("pop".as_bytes().to_vec());
		assert_eq!(decoded, result);
	});
}

#[test]
fn read_state_invalid() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let call = call(contract, ReadStateEverthingFuncId::get(), 99u8, GAS_LIMIT);
		let expected: DispatchError = pallet_contracts::Error::<Test>::DecodingFailed.into();
		// Make sure the error is passed through the error converter.
		let error = <() as ErrorConverter>::convert(expected, &mock::Environment::default()).err();
		assert_eq!(call.result.err(), error);
	})
}

#[test]
fn noop_function_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let call = call(contract, NoopFuncId::get(), (), GAS_LIMIT);
		// Successfully return data.
		let return_value = call.result.unwrap();
		let decoded = <Result<Vec<u8>, u32>>::decode(&mut &return_value.data[..]).unwrap();
		assert!(decoded.unwrap().is_empty());
		assert_eq!(call.storage_deposit, StorageDeposit::Charge(0));
	})
}

#[test]
fn invalid_func_id_fails() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let call = call(contract, INVALID_FUNC_ID, (), GAS_LIMIT);
		let expected: DispatchError = pallet_contracts::Error::<Test>::DecodingFailed.into();
		// Make sure the error is passed through the error converter.
		let error = <() as ErrorConverter>::convert(expected, &mock::Environment::default()).err();
		assert_eq!(call.result.err(), error);
	});
}

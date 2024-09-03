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
		let dispatch_result = call(
			contract,
			DispatchExtFuncId::get(),
			RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() }),
			GAS_LIMIT,
		);
		// Successfully return data.
		let return_value = dispatch_result.result.unwrap();
		let decoded = <Result<Vec<u8>, u32>>::decode(&mut &return_value.data[..]).unwrap();
		assert!(decoded.unwrap().is_empty());
		// Successfully emit event.
		assert!(dispatch_result.events.unwrap().iter().any(|e| matches!(e.event,
				RuntimeEvent::System(frame_system::Event::<Test>::Remarked { sender, .. })
					if sender == contract)));
		assert_eq!(dispatch_result.storage_deposit, StorageDeposit::Charge(0));
	});
}

#[test]
fn dispatch_call_filtering_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let dispatch_result = call(
			contract,
			DispatchContractNoopFuncId::get(),
			RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() }),
			GAS_LIMIT,
		);
		assert_eq!(
			dispatch_result.result,
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
		let dispatch_result = call(
			contract,
			DispatchExtFuncId::get(),
			// `set_code` requires root origin, expect throwing error.
			RuntimeCall::System(Call::set_code { code: "pop".as_bytes().to_vec() }),
			GAS_LIMIT,
		);
		assert_eq!(dispatch_result.result.err(), Some(BadOrigin))
	})
}

#[test]
fn read_state_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		// Successfully return data.
		let read_result = call(contract, ReadExtFuncId::get(), RuntimeRead::Ping, GAS_LIMIT);
		let return_value = read_result.result.unwrap();
		let decoded = <Result<Vec<u8>, u32>>::decode(&mut &return_value.data[1..]).unwrap();
		let result = Ok("pop".as_bytes().to_vec());
		assert_eq!(decoded, result);
	});
}

#[test]
fn read_state_filtering_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		// Successfully return data.
		let read_result =
			call(contract, ReadContractNoopFuncId::get(), RuntimeRead::Ping, GAS_LIMIT);
		assert_eq!(
			read_result.result,
			Err(Module(ModuleError {
				index: 0,
				error: [5, 0, 0, 0],
				message: Some("CallFiltered")
			}))
		);
	});
}

#[test]
fn read_state_invalid() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let read_result = call(contract, ReadExtFuncId::get(), 99u8, GAS_LIMIT);
		let expected: DispatchError = pallet_contracts::Error::<Test>::DecodingFailed.into();
		// Make sure the error is passed through the error converter.
		let error =
			<() as ErrorConverter>::convert(expected, &mock::MockEnvironment::default()).err();
		assert_eq!(read_result.result.err(), error);
	})
}

#[test]
fn noop_function_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let noop_result = call(contract, NoopFuncId::get(), (), GAS_LIMIT);
		// Successfully return data.
		let return_value = noop_result.result.unwrap();
		let decoded = <Result<Vec<u8>, u32>>::decode(&mut &return_value.data[..]).unwrap();
		assert!(decoded.unwrap().is_empty());
		assert_eq!(noop_result.storage_deposit, StorageDeposit::Charge(0));
	})
}

#[test]
fn invalid_func_id_fails() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate(CONTRACT.clone());
		let result = call(contract, INVALID_FUNC_ID, (), GAS_LIMIT);
		let expected: DispatchError = pallet_contracts::Error::<Test>::DecodingFailed.into();
		// Make sure the error is passed through the error converter.
		let error =
			<() as ErrorConverter>::convert(expected, &mock::MockEnvironment::default()).err();
		assert_eq!(result.result.err(), error);
	});
}

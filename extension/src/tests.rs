use core::fmt::Debug;
use std::{path::Path, sync::LazyLock};

use codec::{Decode, Encode};
use frame_support::weights::Weight;
use frame_system::Call;
use pallet_contracts::{Code, CollectEvents, ContractExecResult, Determinism, StorageDeposit};
use sp_runtime::{
	DispatchError::{self, BadOrigin, Module},
	ModuleError,
};

use crate::{
	mock::{self, *},
	ErrorConverter,
};

static CONTRACT: LazyLock<Vec<u8>> =
	LazyLock::new(|| initialize_contract("contract/target/ink/proxy.wasm"));

mod dispatch_call {
	use super::*;

	#[test]
	fn dispatch_call_works() {
		new_test_ext().execute_with(|| {
			// Instantiate a new contract.
			let contract = instantiate();
			let dispatch_result = call(
				contract,
				DispatchContractFuncId::get(),
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
			let contract = instantiate();
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
	fn dispatch_call_returns_error() {
		new_test_ext().execute_with(|| {
			// Instantiate a new contract.
			let contract = instantiate();
			let dispatch_result = call(
				contract,
				DispatchContractFuncId::get(),
				// `set_code` requires root origin, expect throwing error.
				RuntimeCall::System(Call::set_code { code: "pop".as_bytes().to_vec() }),
				GAS_LIMIT,
			);
			assert_eq!(dispatch_result.result.err(), Some(BadOrigin))
		})
	}
}

mod read_state {
	use super::*;

	#[test]
	fn read_state_works() {
		new_test_ext().execute_with(|| {
			// Instantiate a new contract.
			let contract = instantiate();
			// Successfully return data.
			let read_result =
				call(contract, ReadContractFuncId::get(), RuntimeRead::Ping, GAS_LIMIT);
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
			let contract = instantiate();
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
	fn read_state_with_invalid_input_returns_error() {
		new_test_ext().execute_with(|| {
			// Instantiate a new contract.
			let contract = instantiate();
			let read_result = call(contract, ReadExtFuncId::get(), 99u8, GAS_LIMIT);
			let expected: DispatchError = pallet_contracts::Error::<Test>::DecodingFailed.into();
			// Make sure the error is passed through the error converter.
			let error =
				<() as ErrorConverter>::convert(expected, &mock::MockEnvironment::default()).err();
			assert_eq!(read_result.result.err(), error);
		})
	}
}

#[test]
fn noop_function_works() {
	new_test_ext().execute_with(|| {
		// Instantiate a new contract.
		let contract = instantiate();
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
		let contract = instantiate();
		let result = call(contract, INVALID_FUNC_ID, (), GAS_LIMIT);
		let expected: DispatchError = pallet_contracts::Error::<Test>::DecodingFailed.into();
		// Make sure the error is passed through the error converter.
		let error =
			<() as ErrorConverter>::convert(expected, &mock::MockEnvironment::default()).err();
		assert_eq!(result.result.err(), error);
	});
}

/// Initializing a new contract file if it does not exist.
fn initialize_contract(contract_path: &str) -> Vec<u8> {
	if !Path::new(contract_path).exists() {
		use contract_build::*;
		let manifest_path = ManifestPath::new("contract/Cargo.toml").unwrap();
		let args = ExecuteArgs {
			build_artifact: BuildArtifacts::CodeOnly,
			build_mode: BuildMode::Debug,
			manifest_path,
			output_type: OutputType::Json,
			verbosity: Verbosity::Quiet,
			skip_wasm_validation: true,
			..Default::default()
		};
		execute(args).unwrap();
	}
	std::fs::read(contract_path).unwrap()
}

/// Instantiating the contract.
fn instantiate() -> AccountId {
	let result = Contracts::bare_instantiate(
		ALICE,
		0,
		GAS_LIMIT,
		None,
		Code::Upload(CONTRACT.clone()),
		function_selector("new"),
		Default::default(),
		DEBUG_OUTPUT,
		CollectEvents::UnsafeCollect,
	);
	log::debug!("instantiate result: {result:?}");
	let result = result.result.unwrap();
	assert!(!result.result.did_revert());
	result.account_id
}

/// Perform a call to a specified contract.
fn call(
	contract: AccountId,
	func_id: u32,
	input: impl Encode + Debug,
	gas_limit: Weight,
) -> ContractExecResult<Balance, EventRecord> {
	log::debug!("call: func_id={func_id}, input={input:?}");
	let result = Contracts::bare_call(
		ALICE,
		contract,
		0,
		gas_limit,
		None,
		[function_selector("call"), (func_id, input.encode()).encode()].concat(),
		DEBUG_OUTPUT,
		CollectEvents::UnsafeCollect,
		Determinism::Enforced,
	);
	log::debug!("gas consumed: {:?}", result.gas_consumed);
	log::debug!("call result: {result:?}");
	result
}

/// Construct the hashed bytes as a selector of function.
fn function_selector(name: &str) -> Vec<u8> {
	sp_io::hashing::blake2_256(name.as_bytes())[0..4].to_vec()
}

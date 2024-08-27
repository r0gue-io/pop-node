use crate::mock::{self as mock, Test as Runtime, *};
use codec::{Decode, Encode};
use core::fmt::Debug;
use frame_support::{pallet_prelude::Weight, traits::fungible::Inspect};
use frame_system::Call;
use pallet_contracts::{Code, CollectEvents, ContractExecResult, Determinism, WeightInfo};
use pop_chain_extension::{
	ContractWeights, DecodingFailed, ErrorConverter, Extension, RetVal::Converging,
};
use sp_core::Get;
use sp_runtime::{BuildStorage, DispatchError};
use std::{path::Path, sync::LazyLock};

type AccountId = <Test as frame_system::Config>::AccountId;
type Balance = <<Test as pallet_contracts::Config>::Currency as Inspect<
	<Test as frame_system::Config>::AccountId,
>>::Balance;
type EventRecord = frame_system::EventRecord<
	<Test as frame_system::Config>::RuntimeEvent,
	<Test as frame_system::Config>::Hash,
>;

const ALICE: u64 = 1;
const DEBUG_OUTPUT: pallet_contracts::DebugInfo = pallet_contracts::DebugInfo::UnsafeDebug;
const GAS_LIMIT: Weight = Weight::from_parts(100_000_000_000, 3 * 1024 * 1024);
const INIT_AMOUNT: <Runtime as pallet_balances::Config>::Balance = 100_000_000;
const INVALID_FUNC_ID: u32 = 0;

#[test]
fn dispatch_call_works() {
	new_test_ext().execute_with(|| {
		let contract = instantiate();

		let call = call(
			contract,
			DispatchCallFuncId::get(),
			RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() }),
			GAS_LIMIT,
		);

		let return_value = call.result.unwrap();
		let decoded = <Result<Vec<u8>, u32>>::decode(&mut &return_value.data[..]).unwrap();
		assert!(decoded.unwrap().is_empty());

		assert!(call.events.unwrap().iter().any(|e| matches!(e.event,
				RuntimeEvent::System(frame_system::Event::<Test>::Remarked { sender, .. })
					if sender == contract)));
	});
}

#[test]
fn invalid_func_id_fails() {
	new_test_ext().execute_with(|| {
		let contract = instantiate();

		let call = call(contract, INVALID_FUNC_ID, (), GAS_LIMIT);
		let expected: DispatchError = pallet_contracts::Error::<Test>::DecodingFailed.into();
		// TODO: assess whether this error should be passed through the error converter - i.e. is this error type considered 'stable'?
		assert_eq!(call.result, Err(expected))
	});
}

fn new_test_ext() -> sp_io::TestExternalities {
	let _ = env_logger::try_init();

	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Runtime> { balances: vec![(ALICE, INIT_AMOUNT)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

static CONTRACT: LazyLock<Vec<u8>> = LazyLock::new(|| {
	const CONTRACT: &str = "contract/target/ink/proxy.wasm";
	if !Path::new(CONTRACT).exists() {
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
	std::fs::read(CONTRACT).unwrap()
});

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

fn function_selector(name: &str) -> Vec<u8> {
	sp_io::hashing::blake2_256(name.as_bytes())[0..4].to_vec()
}

#[test]
fn extension_call_works() {
	let mut env =
		mock::Environment::new(mock::NoopFuncId::get(), Vec::default(), mock::Ext::default());
	let mut extension = Extension::<mock::Config>::default();
	assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
}

#[test]
fn extension_returns_decoding_failed_for_unknown_function() {
	// no function registered for id 0
	let mut env = mock::Environment::new(0, Vec::default(), mock::Ext::default());
	let mut extension = Extension::<mock::Config>::default();
	assert!(matches!(
		extension.call(&mut env),
		Err(error) if error == pallet_contracts::Error::<mock::Test>::DecodingFailed.into()
	));
}

#[test]
fn extension_call_charges_weight() {
	// specify invalid function
	let mut env = mock::Environment::new(0, [0u8; 42].to_vec(), mock::Ext::default());
	let mut extension = Extension::<mock::Config>::default();
	assert!(extension.call(&mut env).is_err());
	assert_eq!(env.charged(), ContractWeights::<mock::Test>::seal_debug_message(42))
}

#[test]
fn decoding_failed_error_type_works() {
	assert_eq!(
		DecodingFailed::<mock::Test>::get(),
		pallet_contracts::Error::<mock::Test>::DecodingFailed.into()
	)
}

#[test]
fn default_error_conversion_works() {
	let env = mock::Environment::new(0, [0u8; 42].to_vec(), mock::Ext::default());
	assert!(matches!(
		<() as ErrorConverter>::convert(
			DispatchError::BadOrigin,
			&env
		),
		Err(error) if error == DispatchError::BadOrigin
	));
}

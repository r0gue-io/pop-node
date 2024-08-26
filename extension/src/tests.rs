use crate::mock::{Test as Runtime, *};
use codec::{Decode, Encode};
use core::fmt::Debug;
use frame_support::{pallet_prelude::Weight, traits::fungible::Inspect};
use frame_system::Call;
use pallet_contracts::{Code, CollectEvents, ContractExecResult, Determinism};
use sp_runtime::{BuildStorage, DispatchError};

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
#[ignore]
fn wip_extension_charges_weight_for_call() {
	new_test_ext().execute_with(|| {
		let contract = instantiate();
		// Call extension with invalid func id to determine required weight
		let required_weight = call(contract, NoopFuncId::get(), (), Weight::MAX).gas_required;
		// Call with insufficient weight
		let result = call(
			contract,
			NoopFuncId::get(),
			(),
			required_weight.saturating_sub(Weight::from_parts(669158213, 0)),
		);
		let expected: DispatchError = pallet_contracts::Error::<Test>::OutOfGas.into();
		assert_eq!(result.result, Err(expected));
	});
}

#[test]
#[ignore]
fn wip_dispatch_call_charges_weight_for_decoding() {
	new_test_ext().execute_with(|| {
		let contract = instantiate();
		// Call dispatch call function with invalid input to determine required weight
		let required_weight =
			call(contract, DispatchCallFuncId::get(), (), Weight::MAX).gas_required;
		// Call with insufficient weight
		let result =
			call(contract, DispatchCallFuncId::get(), (), required_weight.saturating_sub(1.into()));
		let expected: DispatchError = pallet_contracts::Error::<Test>::OutOfGas.into();
		assert_eq!(result.result, Err(expected));
	});
}

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

#[test]
#[ignore]
fn wip_dispatch_call_handles_invalid_encoded_call() {
	new_test_ext().execute_with(|| {
		let contract = instantiate();

		let call = call(contract, DispatchCallFuncId::get(), (), GAS_LIMIT);

		let return_value = call.result.unwrap();
		let decoded = <Result<Vec<u8>, u32>>::decode(&mut &return_value.data[..]).unwrap();
		assert!(decoded.unwrap().is_empty())
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

fn instantiate() -> AccountId {
	let proxy = std::fs::read("contract/target/ink/proxy.wasm").unwrap();
	let result = Contracts::bare_instantiate(
		ALICE,
		0,
		GAS_LIMIT,
		None,
		Code::Upload(proxy),
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

mod encoding {
	use codec::{Decode, Encode};

	// Test ensuring `func_id()` and `ext_id()` work as expected, i.e. extracting the first two
	// bytes and the last two bytes, respectively, from a 4 byte array.
	#[test]
	fn test_byte_extraction() {
		use rand::Rng;

		// Helper functions
		fn func_id(id: u32) -> u16 {
			(id & 0x0000FFFF) as u16
		}
		fn ext_id(id: u32) -> u16 {
			(id >> 16) as u16
		}

		// Number of test iterations
		let test_iterations = 1_000_000;

		// Create a random number generator
		let mut rng = rand::thread_rng();

		// Run the test for a large number of random 4-byte arrays
		for _ in 0..test_iterations {
			// Generate a random 4-byte array
			let bytes: [u8; 4] = rng.gen();

			// Convert the 4-byte array to a u32 value
			let value = u32::from_le_bytes(bytes);

			// Extract the first two bytes (least significant 2 bytes)
			let first_two_bytes = func_id(value);

			// Extract the last two bytes (most significant 2 bytes)
			let last_two_bytes = ext_id(value);

			// Check if the first two bytes match the expected value
			assert_eq!([bytes[0], bytes[1]], first_two_bytes.to_le_bytes());

			// Check if the last two bytes match the expected value
			assert_eq!([bytes[2], bytes[3]], last_two_bytes.to_le_bytes());
		}
	}

	// Test showing all the different type of variants and its encoding.
	#[test]
	fn encoding_of_enum() {
		#[derive(Debug, PartialEq, Encode, Decode)]
		enum ComprehensiveEnum {
			SimpleVariant,
			DataVariant(u8),
			NamedFields { w: u8 },
			NestedEnum(InnerEnum),
			OptionVariant(Option<u8>),
			VecVariant(Vec<u8>),
			TupleVariant(u8, u8),
			NestedStructVariant(NestedStruct),
			NestedEnumStructVariant(NestedEnumStruct),
		}

		#[derive(Debug, PartialEq, Encode, Decode)]
		enum InnerEnum {
			A,
			B { inner_data: u8 },
			C(u8),
		}

		#[derive(Debug, PartialEq, Encode, Decode)]
		struct NestedStruct {
			x: u8,
			y: u8,
		}

		#[derive(Debug, PartialEq, Encode, Decode)]
		struct NestedEnumStruct {
			inner_enum: InnerEnum,
		}

		// Creating each possible variant for an enum.
		let enum_simple = ComprehensiveEnum::SimpleVariant;
		let enum_data = ComprehensiveEnum::DataVariant(42);
		let enum_named = ComprehensiveEnum::NamedFields { w: 42 };
		let enum_nested = ComprehensiveEnum::NestedEnum(InnerEnum::B { inner_data: 42 });
		let enum_option = ComprehensiveEnum::OptionVariant(Some(42));
		let enum_vec = ComprehensiveEnum::VecVariant(vec![1, 2, 3, 4, 5]);
		let enum_tuple = ComprehensiveEnum::TupleVariant(42, 42);
		let enum_nested_struct =
			ComprehensiveEnum::NestedStructVariant(NestedStruct { x: 42, y: 42 });
		let enum_nested_enum_struct =
			ComprehensiveEnum::NestedEnumStructVariant(NestedEnumStruct {
				inner_enum: InnerEnum::C(42),
			});

		// Encode and print each variant individually to see their encoded values.
		println!("{:?} -> {:?}", enum_simple, enum_simple.encode());
		println!("{:?} -> {:?}", enum_data, enum_data.encode());
		println!("{:?} -> {:?}", enum_named, enum_named.encode());
		println!("{:?} -> {:?}", enum_nested, enum_nested.encode());
		println!("{:?} -> {:?}", enum_option, enum_option.encode());
		println!("{:?} -> {:?}", enum_vec, enum_vec.encode());
		println!("{:?} -> {:?}", enum_tuple, enum_tuple.encode());
		println!("{:?} -> {:?}", enum_nested_struct, enum_nested_struct.encode());
		println!("{:?} -> {:?}", enum_nested_enum_struct, enum_nested_enum_struct.encode());
	}

	#[test]
	fn encoding_decoding_dispatch_error() {
		use sp_runtime::{ArithmeticError, DispatchError, ModuleError, TokenError};

		let error = DispatchError::Module(ModuleError {
			index: 255,
			error: [2, 0, 0, 0],
			message: Some("error message"),
		});
		let encoded = error.encode();
		let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
		assert_eq!(encoded, vec![3, 255, 2, 0, 0, 0]);
		assert_eq!(
			decoded,
			// `message` is skipped for encoding.
			DispatchError::Module(ModuleError { index: 255, error: [2, 0, 0, 0], message: None })
		);

		// Example DispatchError::Token
		let error = DispatchError::Token(TokenError::UnknownAsset);
		let encoded = error.encode();
		let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
		assert_eq!(encoded, vec![7, 4]);
		assert_eq!(decoded, error);

		// Example DispatchError::Arithmetic
		let error = DispatchError::Arithmetic(ArithmeticError::Overflow);
		let encoded = error.encode();
		let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
		assert_eq!(encoded, vec![8, 1]);
		assert_eq!(decoded, error);
	}
}

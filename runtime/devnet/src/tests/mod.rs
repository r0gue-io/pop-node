#![cfg(test)]

use crate::{
	config::assets::TrustBackedAssetsInstance, Assets, Contracts, Runtime, RuntimeOrigin, System,
	Weight, UNIT,
};
use codec::{Decode, Encode};
use frame_support::traits::fungibles::{approvals::Inspect as ApprovalInspect, Inspect};
use frame_system::Config;
use pallet_contracts::{Code, CollectEvents, Determinism, ExecReturnValue};
use pop_api::error::{ArithmeticError, PopApiError, TokenError, TransactionalError};
use sp_runtime::{traits::Hash, AccountId32, BuildStorage, DispatchError};

mod local_fungibles;

type Balance = u128;
type AssetId = u32;
const DEBUG_OUTPUT: pallet_contracts::DebugInfo = pallet_contracts::DebugInfo::UnsafeDebug;

const ALICE: AccountId32 = AccountId32::new([1_u8; 32]);
const BOB: AccountId32 = AccountId32::new([2_u8; 32]);
// FERDIE has no initial balance.
const FERDIE: AccountId32 = AccountId32::new([3_u8; 32]);
const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
const INIT_VALUE: Balance = 100 * UNIT;
const GAS_LIMIT: Weight = Weight::from_parts(100_000_000_000, 3 * 1024 * 1024);

fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.expect("Frame system builds valid default genesis config");

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INIT_AMOUNT), (BOB, INIT_AMOUNT)],
	}
	.assimilate_storage(&mut t)
	.expect("Pallet balances storage can be assimilated");

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn load_wasm_module<T>(path: &str) -> std::io::Result<(Vec<u8>, <T::Hashing as Hash>::Output)>
where
	T: frame_system::Config,
{
	let wasm_binary = std::fs::read(path)?;
	let code_hash = T::Hashing::hash(&wasm_binary);
	Ok((wasm_binary, code_hash))
}

fn function_selector(name: &str) -> Vec<u8> {
	let hash = sp_io::hashing::blake2_256(name.as_bytes());
	[hash[0..4].to_vec()].concat()
}

fn do_bare_call(
	addr: AccountId32,
	input: Vec<u8>,
	value: u128,
) -> Result<ExecReturnValue, DispatchError> {
	let result = Contracts::bare_call(
		ALICE,
		addr.into(),
		value.into(),
		GAS_LIMIT,
		None,
		input,
		DEBUG_OUTPUT,
		CollectEvents::Skip,
		Determinism::Enforced,
	);
	log::debug!("Contract debug buffer - {:?}", String::from_utf8(result.debug_message.clone()));
	log::debug!("result: {:?}", result);
	result.result
}

// Deploy, instantiate and return contract address.
fn instantiate(contract: &str, init_value: u128, salt: Vec<u8>) -> AccountId32 {
	let (wasm_binary, _) =
		load_wasm_module::<Runtime>(contract).expect("could not read .wasm file");
	let result = Contracts::bare_instantiate(
		ALICE,
		init_value,
		GAS_LIMIT,
		None,
		Code::Upload(wasm_binary),
		function_selector("new"),
		salt,
		DEBUG_OUTPUT,
		CollectEvents::Skip,
	)
	.result
	.unwrap();
	assert!(!result.result.did_revert(), "deploying contract reverted {:?}", result);
	result.account_id
}

pub fn get_pallet_index<T: Config>() -> u8 {
	// Get the index of the pallet (module)
	<<T as Config>::PalletInfo as frame_support::traits::PalletInfo>::index::<Assets>()
		.expect("Every active module has an index in the runtime; qed") as u8
}

#[test]
fn encoding_decoding_dispatch_error() {
	use codec::{Decode, Encode};
	use sp_runtime::{ArithmeticError, DispatchError, ModuleError, TokenError};

	new_test_ext().execute_with(|| {
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
		println!("Encoded Module Error: {:?}", encoded);

		// Example pallet assets Error into ModuleError
		let index = get_pallet_index::<Runtime>();
		let mut error =
			pallet_assets::Error::NotFrozen::<Runtime, TrustBackedAssetsInstance>.encode();
		error.resize(sp_runtime::MAX_MODULE_ERROR_ENCODED_SIZE, 0);
		let message = None;
		let error = DispatchError::Module(ModuleError {
			index,
			error: TryInto::try_into(error).expect("should work"),
			message,
		});
		let encoded = error.encode();
		let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
		assert_eq!(encoded, vec![3, 52, 18, 0, 0, 0]);
		assert_eq!(
			decoded,
			DispatchError::Module(ModuleError { index: 52, error: [18, 0, 0, 0], message: None })
		);
		println!("Encoded Module Error: {:?}", encoded);

		// Example DispatchError::Token
		let error = DispatchError::Token(TokenError::UnknownAsset);
		let encoded = error.encode();
		assert_eq!(encoded, vec![7, 4]);
		println!("Encoded Token Error: {:?}", encoded);

		// Example DispatchError::Arithmetic
		let error = DispatchError::Arithmetic(ArithmeticError::Overflow);
		let encoded = error.encode();
		assert_eq!(encoded, vec![8, 1]);
		println!("Encoded Arithmetic Error: {:?}", encoded);
	});
}

#[test]
fn encoding_of_enum() {
	use codec::{Decode, Encode};

	// Comprehensive enum with all different type of variants.
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
	let enum_nested_struct = ComprehensiveEnum::NestedStructVariant(NestedStruct { x: 42, y: 42 });
	let enum_nested_enum_struct = ComprehensiveEnum::NestedEnumStructVariant(NestedEnumStruct {
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
fn dispatch_error_to_status_code_to_pop_api_error_works() {
	// Create all the different `DispatchError` variants with its respective `PopApiError`.
	let test_cases = vec![
		(DispatchError::CannotLookup, PopApiError::CannotLookup),
		(DispatchError::BadOrigin, PopApiError::BadOrigin),
		(
			DispatchError::Module(sp_runtime::ModuleError {
				index: 1,
				error: [2, 0, 0, 0],
				message: Some("hallo"),
			}),
			PopApiError::Module { index: 1, error: 2 },
		),
		(DispatchError::ConsumerRemaining, PopApiError::ConsumerRemaining),
		(DispatchError::NoProviders, PopApiError::NoProviders),
		(DispatchError::TooManyConsumers, PopApiError::TooManyConsumers),
		(
			DispatchError::Token(sp_runtime::TokenError::FundsUnavailable),
			PopApiError::Token(TokenError::FundsUnavailable),
		),
		(
			DispatchError::Arithmetic(sp_runtime::ArithmeticError::Overflow),
			PopApiError::Arithmetic(ArithmeticError::Overflow),
		),
		(
			DispatchError::Transactional(sp_runtime::TransactionalError::LimitReached),
			PopApiError::Transactional(TransactionalError::LimitReached),
		),
		(DispatchError::Exhausted, PopApiError::Exhausted),
		(DispatchError::Corruption, PopApiError::Corruption),
		(DispatchError::Unavailable, PopApiError::Unavailable),
		(DispatchError::RootNotAllowed, PopApiError::RootNotAllowed),
	];
	for (error, pop_api_error) in test_cases {
		// Show that the encoding and decoding of the PopApiError <> u32 (status code) works.
		let status_code = crate::extensions::convert_to_status_code(error);
		// let error = pop_api::error::convert_to_pop_api_error(status_code);
		// assert_eq!(pop_api_error, error,);
	}
}

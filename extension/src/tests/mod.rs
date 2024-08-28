use crate::{
	functions::DefaultConverter,
	mock::{self, NoopFuncId, ReadStateFuncId, RemoveFirstByte, Test},
	ContractWeights, Converter, DecodingFailed, ErrorConverter, Extension, IdentityProcessor,
	Processor,
};
use codec::{Decode, Encode};
use pallet_contracts::chain_extension::RetVal::{Converging, Diverging};
use pallet_contracts::WeightInfo;
use sp_core::Get;
use sp_runtime::DispatchError;

mod contract;
mod utils;

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
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

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
enum InnerEnum {
	A,
	B { inner_data: u8 },
	C(u8),
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct NestedStruct {
	x: u8,
	y: u8,
}

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
struct NestedEnumStruct {
	inner_enum: InnerEnum,
}

mod call {
	use super::*;
	#[test]
	fn extension_call_works() {
		let mut env =
			mock::Environment::new(NoopFuncId::get(), Vec::default(), mock::Ext::default());
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
	fn extension_call_read_state_works() {
		let mut env =
			mock::Environment::new(ReadStateFuncId::get(), [0u8, 1].to_vec(), mock::Ext::default());
		let mut extension = Extension::<mock::Config>::default();
		assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
	}

	#[test]
	fn extension_call_read_state_invalid() {
		let mut env = mock::Environment::new(
			ReadStateFuncId::get(),
			[0u8, 99].to_vec(),
			mock::Ext::default(),
		);
		let mut extension = Extension::<mock::Config>::default();
		// Failed due to the invalid read index.
		assert!(extension.call(&mut env).is_err());
	}
}

mod functions {
	use super::*;
	use core::marker::PhantomData;
	use frame_support::parameter_types;
	use pallet_contracts::chain_extension::{RetVal, ReturnFlags};

	use crate::{environment, matching::WithFuncId, mock::Test, Function, Matches};

	/// A function that returns a success status from a contract call.
	struct ReturnSuccessFunction<M, C>(PhantomData<(M, C)>);
	impl<Matcher: Matches, Config: pallet_contracts::Config> Function
		for ReturnSuccessFunction<Matcher, Config>
	{
		type Config = Config;
		type Error = ();

		fn execute(
			_env: &mut (impl environment::Environment<Config = Config> + crate::BufIn),
		) -> pallet_contracts::chain_extension::Result<RetVal> {
			Ok(Converging(0))
		}
	}
	impl<M: Matches, C> Matches for ReturnSuccessFunction<M, C> {
		fn matches(env: &impl crate::Environment) -> bool {
			M::matches(env)
		}
	}

	/// A function that returns a failure from a contract call.
	struct ReturnFailureFunction<M, C>(PhantomData<(M, C)>);
	impl<Matcher: Matches, Config: pallet_contracts::Config> Function
		for ReturnFailureFunction<Matcher, Config>
	{
		type Config = Config;
		type Error = ();

		fn execute(
			_env: &mut (impl environment::Environment<Config = Config> + crate::BufIn),
		) -> pallet_contracts::chain_extension::Result<RetVal> {
			Ok(RetVal::Diverging { flags: ReturnFlags::REVERT, data: vec![42] })
		}
	}
	impl<M: Matches, C> Matches for ReturnFailureFunction<M, C> {
		fn matches(env: &impl crate::Environment) -> bool {
			M::matches(env)
		}
	}

	/// Registry of chain extension functions.
	type Functions = (
		ReturnSuccessFunction<WithFuncId<ReturnSuccessId>, Test>,
		ReturnFailureFunction<WithFuncId<ReturnFailureId>, Test>,
	);

	parameter_types! {
		pub const ReturnSuccessId : u32 = 0;
		pub const ReturnFailureId : u32 = 1;
		pub const InvalidId : u32 = 99;
	}

	#[test]
	fn execute_success_function_works() {
		let mut env = mock::Environment::new(ReturnSuccessId::get(), vec![], mock::Ext::default());
		assert!(matches!(Functions::execute(&mut env), Ok(Converging(0))));
	}

	#[test]
	fn execute_failure_function_works() {
		let mut env = mock::Environment::new(ReturnFailureId::get(), vec![], mock::Ext::default());
		assert!(matches!(
			Functions::execute(&mut env),
			Ok(Diverging { flags: ReturnFlags::REVERT, data: ref d }) if d == &[42]
		));
	}

	#[test]
	fn execute_invalid_function() {
		let mut env = mock::Environment::new(InvalidId::get(), vec![], mock::Ext::default());
		let error = pallet_contracts::Error::<mock::Test>::DecodingFailed.into();
		let expected = <() as ErrorConverter>::convert(error, &mut env).err();
		assert_eq!(Functions::execute(&mut env).err(), expected);
	}
}

mod error {
	use super::*;

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
			<() as ErrorConverter>::convert(DispatchError::BadOrigin, &env),
			Err(DispatchError::BadOrigin)
		));
	}
}

mod matching {
	use super::*;
	use crate::{matching::WithFuncId, Equals, Matches};
	use sp_core::{ConstU16, ConstU32};

	#[test]
	fn matching_with_func_id_works() {
		let env = mock::Environment::default();
		assert!(WithFuncId::<ConstU32<0>>::matches(&env));
	}

	#[test]
	fn matching_with_func_id_invalid() {
		let env = mock::Environment::new(1, vec![], mock::Ext::default());
		assert!(!WithFuncId::<ConstU32<0>>::matches(&env));
	}

	#[test]
	fn matching_equals_works() {
		let env = mock::Environment::new(
			u32::from_be_bytes([0u8, 1, 0, 2]),
			vec![],
			mock::Ext::default(),
		);
		assert!(Equals::<ConstU16<1>, ConstU16<2>>::matches(&env));
	}

	#[test]
	fn matching_equals_invalid() {
		let env = mock::Environment::new(
			u32::from_be_bytes([0u8, 1, 0, 3]),
			vec![],
			mock::Ext::default(),
		);
		assert!(!Equals::<ConstU16<1>, ConstU16<2>>::matches(&env));
	}

	#[test]
	fn default_converter_works() {
		let env = mock::Environment::default();
		let source = "pop".to_string();
		assert_eq!(DefaultConverter::<String>::convert(source.clone(), &env), source.as_bytes());
	}
}

mod decoding {
	use super::*;
	use crate::decoding::Decode;
	use crate::Decodes;

	#[test]
	fn default_processor_works() {
		let env = mock::Environment::default();
		assert_eq!(<()>::process((), &env), ())
	}

	#[test]
	fn remove_first_byte_processor_works() {
		let env = mock::Environment::default();
		let result = RemoveFirstByte::process(vec![0, 1, 2, 3, 4], &env);
		assert_eq!(result, vec![1, 2, 3, 4])
	}

	#[test]
	fn identity_processor_works() {
		let env = mock::Environment::default();
		let result = IdentityProcessor::process(vec![0, 1, 2, 3, 4], &env);
		assert_eq!(result, vec![0, 1, 2, 3, 4])
	}

	#[test]
	fn decode_with_identity_processor_works() {
		// Creating a set of byte data input and the decoded enum variant.
		vec![
			(vec![0, 0, 0, 0], ComprehensiveEnum::SimpleVariant),
			(vec![1, 42, 0, 0], ComprehensiveEnum::DataVariant(42)),
			(vec![2, 42, 0, 0], ComprehensiveEnum::NamedFields { w: 42 }),
			(vec![3, 0, 0, 0], ComprehensiveEnum::NestedEnum(InnerEnum::A)),
			(vec![3, 1, 42, 0], ComprehensiveEnum::NestedEnum(InnerEnum::B { inner_data: 42 })),
			(vec![3, 2, 42, 0], ComprehensiveEnum::NestedEnum(InnerEnum::C(42))),
			(vec![4, 1, 42, 0], ComprehensiveEnum::OptionVariant(Some(42))),
			(vec![4, 0, 0, 0], ComprehensiveEnum::OptionVariant(None)),
			(vec![5, 12, 1, 2, 3], ComprehensiveEnum::VecVariant(vec![1, 2, 3])),
			(vec![5, 16, 1, 2, 3, 4], ComprehensiveEnum::VecVariant(vec![1, 2, 3, 4])),
			(vec![5, 20, 1, 2, 3, 4, 5], ComprehensiveEnum::VecVariant(vec![1, 2, 3, 4, 5])),
			(vec![6, 42, 43, 0], ComprehensiveEnum::TupleVariant(42, 43)),
			(
				vec![7, 42, 43, 0],
				ComprehensiveEnum::NestedStructVariant(NestedStruct { x: 42, y: 43 }),
			),
			(
				vec![8, 1, 42, 0],
				ComprehensiveEnum::NestedEnumStructVariant(NestedEnumStruct {
					inner_enum: InnerEnum::B { inner_data: 42 },
				}),
			),
		]
		.iter()
		.for_each(|t| {
			let (input, output) = (t.clone().0, t.clone().1);
			println!("input: {:?} -> output: {:?}", input, output);
			let mut env = mock::Environment::new(0, input, mock::Ext::default());
			// Decode `input` to `output` using a provided processor.
			let result =
				Decodes::<ComprehensiveEnum, DecodingFailed<Test>, IdentityProcessor>::decode(
					&mut env,
				);
			assert_eq!(result, Ok(output));
		});
	}

	#[test]
	fn decode_with_remove_first_byte_processor_works() {
		// Creating a set of byte data input and the decoded enum variant.
		vec![
			(vec![0, 0, 0, 0, 0], ComprehensiveEnum::SimpleVariant),
			(vec![0, 1, 42, 0, 0], ComprehensiveEnum::DataVariant(42)),
			(vec![0, 2, 42, 0, 0], ComprehensiveEnum::NamedFields { w: 42 }),
			(vec![0, 3, 0, 0, 0], ComprehensiveEnum::NestedEnum(InnerEnum::A)),
			(vec![0, 3, 1, 42, 0], ComprehensiveEnum::NestedEnum(InnerEnum::B { inner_data: 42 })),
			(vec![0, 3, 2, 42, 0], ComprehensiveEnum::NestedEnum(InnerEnum::C(42))),
			(vec![0, 4, 1, 42, 0], ComprehensiveEnum::OptionVariant(Some(42))),
			(vec![0, 4, 0, 0, 0], ComprehensiveEnum::OptionVariant(None)),
			(vec![0, 5, 12, 1, 2, 3], ComprehensiveEnum::VecVariant(vec![1, 2, 3])),
			(vec![0, 5, 16, 1, 2, 3, 4], ComprehensiveEnum::VecVariant(vec![1, 2, 3, 4])),
			(vec![0, 5, 20, 1, 2, 3, 4, 5], ComprehensiveEnum::VecVariant(vec![1, 2, 3, 4, 5])),
			(vec![0, 6, 42, 43, 0], ComprehensiveEnum::TupleVariant(42, 43)),
			(
				vec![0, 7, 42, 43, 0],
				ComprehensiveEnum::NestedStructVariant(NestedStruct { x: 42, y: 43 }),
			),
			(
				vec![0, 8, 1, 42, 0],
				ComprehensiveEnum::NestedEnumStructVariant(NestedEnumStruct {
					inner_enum: InnerEnum::B { inner_data: 42 },
				}),
			),
		]
		.iter()
		.for_each(|t| {
			let (input, output) = (t.clone().0, t.clone().1);
			println!("input: {:?} -> output: {:?}", input, output);
			let mut env = mock::Environment::new(0, input, mock::Ext::default());
			// Decode `input` to `output` using a provided processor.
			let result =
				Decodes::<ComprehensiveEnum, DecodingFailed<Test>, RemoveFirstByte>::decode(
					&mut env,
				);
			assert_eq!(result, Ok(output));
		});
	}

	#[test]
	fn decode_return_decoding_fail_error() {
		let mut env = mock::Environment::new(0, vec![1, 42, 0, 0], mock::Ext::default());
		let result =
			Decodes::<ComprehensiveEnum, DecodingFailed<Test>, RemoveFirstByte>::decode(&mut env);
		assert_eq!(result, Err(pallet_contracts::Error::<mock::Test>::DecodingFailed.into()));
	}
}

mod encoding {
	use super::*;

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

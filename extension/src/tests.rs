use super::{
	encode_error, DispatchError::*, FuncId, DECODING_FAILED_ERROR, DECODING_FAILED_ERROR_ENCODED,
	UNKNOWN_CALL_ERROR, UNKNOWN_CALL_ERROR_ENCODED,
};
use codec::{Decode, Encode};
use sp_runtime::{
	ArithmeticError::*, DispatchError, ModuleError, TokenError::*, TransactionalError::*,
};

// TODO: #110
// Test ensuring `func_id()` and `ext_id()` work as expected. I.e. extracting the first two
// bytes and the last two bytes from a 4 byte array, respectively.
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

// Assert encoded `DispatchError` with expected encoding.
pub(crate) fn assert_encoding_matches(dispatch_error: DispatchError, expected_encoding: [u8; 4]) {
	let encoding = encode_error(dispatch_error);
	assert_eq!(encoding, expected_encoding);
}

// Assert all unit error possibilities with expected encoding.
#[test]
fn encode_error_unit_variants_works() {
	let test_cases = vec![
		(Other(""), [0, 0, 0, 0]),
		(CannotLookup, [1, 0, 0, 0]),
		(BadOrigin, [2, 0, 0, 0]),
		(ConsumerRemaining, [4, 0, 0, 0]),
		(NoProviders, [5, 0, 0, 0]),
		(TooManyConsumers, [6, 0, 0, 0]),
		(Exhausted, [10, 0, 0, 0]),
		(Corruption, [11, 0, 0, 0]),
		(Unavailable, [12, 0, 0, 0]),
		(RootNotAllowed, [13, 0, 0, 0]),
		(UNKNOWN_CALL_ERROR, UNKNOWN_CALL_ERROR_ENCODED),
		(DECODING_FAILED_ERROR, DECODING_FAILED_ERROR_ENCODED),
	];
	for (dispatch_error, expected_encoding) in test_cases {
		assert_encoding_matches(dispatch_error, expected_encoding);
	}
}

// Assert all single nested error possibilities with expected encoding.
#[test]
fn encode_error_single_nested_variants_works() {
	// TokenError.
	let test_cases = vec![
		(FundsUnavailable, 0),
		(OnlyProvider, 1),
		(BelowMinimum, 2),
		(CannotCreate, 3),
		(UnknownAsset, 4),
		(Frozen, 5),
		(Unsupported, 6),
		(CannotCreateHold, 7),
		(NotExpendable, 8),
		(Blocked, 9),
	];
	for (error, index) in test_cases {
		assert_encoding_matches(Token(error), [7, index, 0, 0]);
	}

	// ArithmeticError.
	let test_cases = vec![(Underflow, 0), (Overflow, 1), (DivisionByZero, 2)];
	for (error, index) in test_cases {
		assert_encoding_matches(Arithmetic(error), [8, index, 0, 0]);
	}

	// TransactionalError.
	let test_cases = vec![(LimitReached, 0), (NoLayer, 1)];
	for (error, index) in test_cases {
		assert_encoding_matches(Transactional(error), [9, index, 0, 0]);
	}
}

// Assert all module error possibilities with expected encoding.
#[test]
fn encode_error_module_error_works() {
	let test_cases = vec![
		(
			Module(ModuleError { index: 1, error: [2, 0, 0, 0], message: Some("hallo") }),
			[3, 1, 2, 0],
		),
		(
			Module(ModuleError { index: 1, error: [2, 3, 0, 0], message: Some("hallo") }),
			[3, 1, 2, 3],
		),
		(
			Module(ModuleError { index: 1, error: [2, 3, 4, 0], message: Some("hallo") }),
			[3, 1, 2, 3],
		),
		(
			Module(ModuleError { index: 1, error: [2, 3, 4, 5], message: Some("hallo") }),
			[3, 1, 2, 3],
		),
		(Module(ModuleError { index: 1, error: [2, 3, 4, 5], message: None }), [3, 1, 2, 3]),
	];
	for (dispatch_error, expected_encoding) in test_cases {
		let encoding = encode_error(dispatch_error);
		assert_eq!(encoding, expected_encoding);
	}
}

#[test]
fn func_id_try_from_works() {
	let test_cases = [
		(0u8, Ok(FuncId::Dispatch)),
		(1, Ok(FuncId::ReadState)),
		(2, Err(UNKNOWN_CALL_ERROR)),
		(3, Err(UNKNOWN_CALL_ERROR)),
		(100, Err(UNKNOWN_CALL_ERROR)),
		(u8::MAX, Err(UNKNOWN_CALL_ERROR)),
	];

	for (input_value, expected_result) in test_cases {
		let actual_result: Result<FuncId, DispatchError> = input_value.try_into();
		assert_eq!(actual_result, expected_result, "Failed on input: {}", input_value);
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
fn encoding_decoding_dispatch_error() {
	let error =
		Module(ModuleError { index: 255, error: [2, 0, 0, 0], message: Some("error message") });
	let encoded = error.encode();
	let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
	assert_eq!(encoded, vec![3, 255, 2, 0, 0, 0]);
	assert_eq!(
		decoded,
		// `message` is skipped for encoding.
		Module(ModuleError { index: 255, error: [2, 0, 0, 0], message: None })
	);

	// Example Token
	let error = Token(UnknownAsset);
	let encoded = error.encode();
	let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
	assert_eq!(encoded, vec![7, 4]);
	assert_eq!(decoded, error);

	// Example Arithmetic
	let error = Arithmetic(Overflow);
	let encoded = error.encode();
	let decoded = DispatchError::decode(&mut &encoded[..]).unwrap();
	assert_eq!(encoded, vec![8, 1]);
	assert_eq!(decoded, error);
}

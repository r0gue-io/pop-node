#[cfg(test)]
mod tests {
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

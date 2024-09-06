use sp_runtime::DispatchError;
use sp_std::vec::Vec;

use super::*;

/// Trait for decoding data read from contract memory.
pub trait Decode {
	/// The output type to be decoded.
	type Output: codec::Decode;
	/// An optional processor, for performing any additional processing on data read from the
	/// contract before decoding.
	type Processor: Processor<Value = Vec<u8>>;
	/// The error to return if decoding fails.
	type Error: Get<DispatchError>;

	/// The log target.
	const LOG_TARGET: &'static str;

	/// Decodes data read from contract memory.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn decode<E: Environment + BufIn>(env: &mut E) -> Result<Self::Output>;
}

/// Trait for processing a value based on additional information available from the environment.
pub trait Processor {
	/// The type of value to be processed.
	type Value;

	/// The log target.
	const LOG_TARGET: &'static str;

	/// Processes the provided value.
	///
	/// # Parameters
	/// - `value` - The value to be processed.
	/// - `env` - The current execution environment.
	fn process(value: Self::Value, env: &impl Environment) -> Self::Value;
}

/// Default processor implementation which just passes through the value unchanged.
pub struct Identity<Value>(PhantomData<Value>);
impl<Value> Processor for Identity<Value> {
	type Value = Value;

	const LOG_TARGET: &'static str = "";

	fn process(value: Self::Value, _env: &impl Environment) -> Self::Value {
		value
	}
}

/// Default implementation for decoding data read from contract memory.
pub struct Decodes<O, W, E, P = Identity<Vec<u8>>, L = ()>(PhantomData<(O, W, E, P, L)>);
impl<
		Output: codec::Decode,
		Weight: WeightInfo,
		Error: Get<DispatchError>,
		ValueProcessor: Processor<Value = Vec<u8>>,
		Logger: LogTarget,
	> Decode for Decodes<Output, Weight, Error, ValueProcessor, Logger>
{
	type Error = Error;
	type Output = Output;
	type Processor = ValueProcessor;

	const LOG_TARGET: &'static str = Logger::LOG_TARGET;

	/// Decodes data read from contract memory.
	///
	/// # Parameters
	/// - `env` - The current execution environment.
	fn decode<E: Environment + BufIn>(env: &mut E) -> Result<Self::Output> {
		// Charge appropriate weight for copying from contract, based on input length, prior to
		// decoding. reference: https://github.com/paritytech/polkadot-sdk/pull/4233/files#:~:text=CopyToContract(len)%20%3D%3E%20T%3A%3AWeightInfo%3A%3Aseal_return(len)%2C
		let len = env.in_len();
		let weight = Weight::seal_return(len);
		let charged = env.charge_weight(weight)?;
		log::debug!(target: Self::LOG_TARGET, "pre-decode weight charged: len={len}, weight={weight}, charged={charged:?}");
		// Read encoded input supplied by contract for buffer.
		let mut input = env.read(len)?;
		log::debug!(target: Self::LOG_TARGET, "input read: input={input:?}");
		// Perform any additional processing required. Any implementation is expected to charge
		// weight as appropriate.
		input = Self::Processor::process(input, env);
		// Finally decode and return.
		Output::decode(&mut &input[..]).map_err(|_| {
			log::error!(target: Self::LOG_TARGET, "decoding failed: unable to decode input into output type. input={input:?}");
			Error::get()
		})
	}
}

/// Error to be returned when decoding fails.
pub struct DecodingFailed<C>(PhantomData<C>);
impl<T: pallet_contracts::Config> Get<DispatchError> for DecodingFailed<T> {
	fn get() -> DispatchError {
		pallet_contracts::Error::<T>::DecodingFailed.into()
	}
}

#[cfg(test)]
mod tests {
	use codec::{Decode as OriginalDecode, Encode};
	use frame_support::assert_ok;

	use super::*;
	use crate::{
		extension::read_from_buffer_weight,
		mock::{MockEnvironment, RemoveFirstByte, Test},
	};

	type EnumDecodes = Decodes<ComprehensiveEnum, ContractWeightsOf<Test>, DecodingFailed<Test>>;

	#[test]
	fn identity_processor_works() {
		let env = MockEnvironment::default();
		assert_eq!(Identity::process(42, &env), 42);
		assert_eq!(Identity::process(vec![0, 1, 2, 3, 4], &env), vec![0, 1, 2, 3, 4]);
	}

	#[test]
	fn remove_first_byte_processor_works() {
		let env = MockEnvironment::default();
		let result = RemoveFirstByte::process(vec![0, 1, 2, 3, 4], &env);
		assert_eq!(result, vec![1, 2, 3, 4])
	}

	#[test]
	fn decode_works() {
		test_cases().into_iter().for_each(|t| {
			let (input, output) = (t.0, t.1);
			println!("input: {:?} -> output: {:?}", input, output);
			let mut env = MockEnvironment::new(0, input);
			// Decode `input` to `output`.
			assert_eq!(EnumDecodes::decode(&mut env), Ok(output));
		});
	}

	#[test]
	fn decode_charges_weight() {
		test_cases().into_iter().for_each(|t| {
			let (input, output) = (t.0, t.1);
			println!("input: {:?} -> output: {:?}", input, output);
			let mut env = MockEnvironment::new(0, input.clone());
			// Decode `input` to `output`.
			assert_ok!(EnumDecodes::decode(&mut env));
			// Decode charges weight based on the length of the input.
			assert_eq!(env.charged(), read_from_buffer_weight(input.len() as u32));
		});
	}

	#[test]
	fn decoding_failed_error_type_works() {
		assert_eq!(
			DecodingFailed::<Test>::get(),
			pallet_contracts::Error::<Test>::DecodingFailed.into()
		)
	}

	#[test]
	fn decode_failure_returns_decoding_failed_error() {
		let input = vec![100];
		let mut env = MockEnvironment::new(0, input.clone());
		let result = EnumDecodes::decode(&mut env);
		assert_eq!(result, Err(pallet_contracts::Error::<Test>::DecodingFailed.into()));
	}

	#[test]
	fn decode_failure_charges_weight() {
		let input = vec![100];
		let mut env = MockEnvironment::new(0, input.clone());
		assert!(EnumDecodes::decode(&mut env).is_err());
		// Decode charges weight based on the length of the input, also when decoding fails.
		assert_eq!(env.charged(), ContractWeightsOf::<Test>::seal_return(input.len() as u32));
	}

	#[derive(Debug, Clone, PartialEq, Encode, OriginalDecode)]
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

	#[derive(Debug, Clone, PartialEq, Encode, OriginalDecode)]
	enum InnerEnum {
		A,
		B { inner_data: u8 },
		C(u8),
	}

	#[derive(Debug, Clone, PartialEq, Encode, OriginalDecode)]
	struct NestedStruct {
		x: u8,
		y: u8,
	}

	#[derive(Debug, Clone, PartialEq, Encode, OriginalDecode)]
	struct NestedEnumStruct {
		inner_enum: InnerEnum,
	}

	// Creating a set of byte data input and the decoded enum variant.
	fn test_cases() -> Vec<(Vec<u8>, ComprehensiveEnum)> {
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
		// DispatchError::Module index is 3
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

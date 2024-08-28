use crate::{
	functions::DefaultConverter,
	mock::{self, NoopFuncId, ReadStateFuncId, RemoveFirstByte},
	ContractWeights, Converter, DecodingFailed, ErrorConverter, Extension, IdentityProcessor,
	Processor,
};
use pallet_contracts::chain_extension::RetVal::Converging;
use pallet_contracts::WeightInfo;
use sp_core::Get;
use sp_runtime::DispatchError;

mod encoding;
mod proxy;
#[cfg(test)]
mod utils;

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
			<() as ErrorConverter>::convert(
				DispatchError::BadOrigin,
				&env
			),
			Err(error) if error == DispatchError::BadOrigin
		));
	}
}

#[test]
fn default_converter_works() {
	let env = mock::Environment::new(0, vec![], mock::Ext::default());
	let source = "pop".to_string();
	assert_eq!(DefaultConverter::<String>::convert(source.clone(), &env), source.as_bytes());
}

mod processor {
	use super::*;
	#[test]
	fn remove_first_byte_processor_works() {
		let env = mock::Environment::new(0, vec![], mock::Ext::default());
		let result = RemoveFirstByte::process(vec![0, 1, 2, 3, 4], &env);
		assert_eq!(result, vec![1, 2, 3, 4])
	}

	#[test]
	fn identity_processor_works() {
		let env = mock::Environment::new(0, vec![], mock::Ext::default());
		let result = IdentityProcessor::process(vec![0, 1, 2, 3, 4], &env);
		assert_eq!(result, vec![0, 1, 2, 3, 4])
	}
}

use crate::{
	mock::{self, NoopFuncId, ReadStateFuncId},
	ContractWeights, DecodingFailed, ErrorConverter, Extension,
};
use pallet_contracts::chain_extension::RetVal::Converging;
use pallet_contracts::WeightInfo;
use sp_core::Get;
use sp_runtime::DispatchError;

mod encoding;
mod proxy;
#[cfg(test)]
mod utils;

#[test]
fn extension_call_works() {
	let mut env = mock::Environment::new(NoopFuncId::get(), Vec::default(), mock::Ext::default());
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

#[test]
fn extension_call_read_state_works() {
	let mut env =
		mock::Environment::new(ReadStateFuncId::get(), [0u8, 1].to_vec(), mock::Ext::default());
	let mut extension = Extension::<mock::Config>::default();
	assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
}

#[test]
fn extension_call_read_state_invalid() {
	let mut env =
		mock::Environment::new(ReadStateFuncId::get(), [0u8, 99].to_vec(), mock::Ext::default());
	let mut extension = Extension::<mock::Config>::default();
	// Failed due to the invalid read index.
	assert!(extension.call(&mut env).is_err());
}

#[test]
fn processor_works() {
	unimplemented!("Test if the provided processor works correctly");
}

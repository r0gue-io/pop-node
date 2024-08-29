use crate::{
	mock::{Config, Environment, Ext, NoopFuncId, ReadStateFuncId, Test},
	ContractWeights, Extension,
};
use pallet_contracts::chain_extension::RetVal::Converging;
use pallet_contracts::WeightInfo;

mod contract;
mod utils;

#[test]
fn extension_call_works() {
	let input = vec![2, 2];
	let mut env = Environment::new(NoopFuncId::get(), input.clone(), Ext::default());
	let mut extension = Extension::<Config>::default();
	assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
	assert_eq!(env.charged(), ContractWeights::<Test>::seal_debug_message(input.len() as u32))
}

#[test]
fn extension_returns_decoding_failed_for_unknown_function() {
	// no function registered for id 0
	let mut env = Environment::new(0, Vec::default(), Ext::default());
	let mut extension = Extension::<Config>::default();
	assert!(matches!(
		extension.call(&mut env),
		Err(error) if error == pallet_contracts::Error::<Test>::DecodingFailed.into()
	));
}

#[test]
fn extension_call_charges_weight() {
	// specify invalid function
	let mut env = Environment::new(0, [0u8; 42].to_vec(), Ext::default());
	let mut extension = Extension::<Config>::default();
	assert!(extension.call(&mut env).is_err());
	assert_eq!(env.charged(), ContractWeights::<Test>::seal_debug_message(42))
}

#[test]
fn extension_call_read_state_works() {
	let mut env = Environment::new(ReadStateFuncId::get(), [0u8, 1].to_vec(), Ext::default());
	let mut extension = Extension::<Config>::default();
	assert!(matches!(extension.call(&mut env), Ok(Converging(0))));
}

#[test]
fn extension_call_read_state_invalid() {
	let mut env = Environment::new(ReadStateFuncId::get(), [0u8, 99].to_vec(), Ext::default());
	let mut extension = Extension::<Config>::default();
	// Failed due to the invalid read index.
	assert!(extension.call(&mut env).is_err());
}

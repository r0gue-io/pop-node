use crate::{
	build_extension_method,
	constants::{DISPATCH, READ_STATE},
	primitives::Error,
	StatusCode,
};
use ink::env::chain_extension::ChainExtensionMethod;

/// APIs for fungible tokens.
#[cfg(feature = "fungibles")]
pub mod fungibles;

pub(crate) const V0: u8 = 0;

impl From<StatusCode> for Error {
	fn from(value: StatusCode) -> Self {
		value.0.into()
	}
}

// Helper method to build a dispatch call.
//
// Parameters:
// - 'module': The index of the runtime module.
// - 'dispatchable': The index of the module dispatchable functions.
fn build_dispatch(module: u8, dispatchable: u8) -> ChainExtensionMethod<(), (), (), false> {
	build_extension_method(V0, DISPATCH, module, dispatchable)
}

// Helper method to build a call to read state.
//
// Parameters:
// - 'module': The index of the runtime module.
// - 'state_query': The index of the runtime state query.
fn build_read_state(module: u8, state_query: u8) -> ChainExtensionMethod<(), (), (), false> {
	build_extension_method(V0, READ_STATE, module, state_query)
}

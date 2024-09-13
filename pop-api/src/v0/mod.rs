use ink::env::chain_extension::ChainExtensionMethod;

use crate::{
	build_extension_method,
	constants::{DISPATCH, READ_STATE},
	primitives::Error,
	ChainExtensionMethodApi, StatusCode,
};

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
fn build_dispatch(module: u8, dispatchable: u8) -> ChainExtensionMethodApi {
	build_extension_method(DISPATCH, V0, module, dispatchable)
}

// Helper method to build a call to read state.
//
// Parameters:
// - 'module': The index of the runtime module.
// - 'state_query': The index of the runtime state query.
fn build_read_state(module: u8, state_query: u8) -> ChainExtensionMethodApi {
	build_extension_method(READ_STATE, V0, module, state_query)
}

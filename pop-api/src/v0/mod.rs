use crate::{
	build_extension_method,
	constants::{DISPATCH, READ_STATE},
	primitives::error::Error,
	StatusCode,
};
use ink::env::chain_extension::ChainExtensionMethod;

#[cfg(feature = "assets")]
pub mod assets;

pub(crate) const V0: u8 = 0;

impl From<StatusCode> for Error {
	fn from(value: StatusCode) -> Self {
		value.0.into()
	}
}

/// Helper method to build `ChainExtensionMethod` for version `v0`
///
/// Parameters:
/// - 'function': The ID of the function
/// - 'module': The index of the runtime module
/// - 'dispatchable': The index of the module dispatchable functions
fn build_extension_method_v0(
	function: u8,
	module: u8,
	dispatchable: u8,
) -> ChainExtensionMethod<(), (), (), false> {
	build_extension_method(V0, function, module, dispatchable)
}

/// Helper method to build a dispatch call `ChainExtensionMethod`
///
/// Parameters:
/// - 'module': The index of the runtime module
/// - 'dispatchable': The index of the module dispatchable functions
fn build_dispatch(module: u8, dispatchable: u8) -> ChainExtensionMethod<(), (), (), false> {
	build_extension_method_v0(DISPATCH, module, dispatchable)
}

/// Helper method to build a dispatch call `ChainExtensionMethod`
///
/// Parameters:
/// - 'module': The index of the runtime module
/// - 'state_query': The index of the runtime state query
fn build_read_state(module: u8, state_query: u8) -> ChainExtensionMethod<(), (), (), false> {
	build_extension_method_v0(READ_STATE, module, state_query)
}

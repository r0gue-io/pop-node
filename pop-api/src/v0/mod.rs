use crate::{
	build_extension_method,
	constants::{DISPATCH, READ_STATE},
	primitives::{AccountId, Balance, Error},
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

pub mod xc {
	use super::*;
	#[inline]
	pub fn asset_hub_transfer(to: AccountId, value: Balance, fee: Balance) -> crate::Result<()> {
		build_dispatch(151, 0)
			.input::<(AccountId, Balance, Balance)>()
			.output::<crate::Result<()>, true>()
			.handle_error_code::<StatusCode>()
			.call(&(to, value, fee))
	}
}

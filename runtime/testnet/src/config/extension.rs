use crate::{Runtime, RuntimeCall};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Contains;

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum RuntimeRead {}

impl pop_runtime_extension::ReadStateHandler<Runtime> for RuntimeRead {
	fn handle_read(_: RuntimeRead) -> Vec<u8> {
		sp_std::vec::Vec::default()
	}
}

/// A type to identify allowed calls to the Runtime from the API.
pub struct AllowedApiCalls;

impl Contains<RuntimeCall> for AllowedApiCalls {
	/// Allowed runtime calls from the API.
	fn contains(_: &RuntimeCall) -> bool {
		false
	}
}

impl Contains<RuntimeRead> for AllowedApiCalls {
	/// Allowed state queries from the API.
	fn contains(_: &RuntimeRead) -> bool {
		false
	}
}

impl pop_runtime_extension::Config for Runtime {
	type RuntimeRead = RuntimeRead;
	type AllowedApiCalls = AllowedApiCalls;
}

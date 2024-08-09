use crate::{config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Contains;
use pop_chain_extension::ReadStateHandler;

/// A query of runtime state.
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
#[repr(u8)]
pub enum RuntimeRead {
	/// Fungible token queries.
	#[codec(index = 150)]
	Fungibles(fungibles::Read<Runtime>),
}

impl ReadStateHandler<Runtime> for RuntimeRead {
	fn handle_read(read: RuntimeRead) -> sp_std::vec::Vec<u8> {
		match read {
			RuntimeRead::Fungibles(key) => fungibles::Pallet::read_state(key),
		}
	}
}

/// A type to identify allowed calls to the Runtime from the API.
pub struct AllowedApiCalls;

impl Contains<RuntimeCall> for AllowedApiCalls {
	/// Allowed runtime calls from the API.
	fn contains(c: &RuntimeCall) -> bool {
		use fungibles::Call::*;
		matches!(
			c,
			RuntimeCall::Fungibles(
				transfer { .. }
					| transfer_from { .. }
					| approve { .. } | increase_allowance { .. }
					| decrease_allowance { .. }
					| create { .. } | set_metadata { .. }
					| start_destroy { .. }
					| clear_metadata { .. }
					| mint { .. } | burn { .. }
			)
		)
	}
}

impl Contains<RuntimeRead> for AllowedApiCalls {
	/// Allowed state queries from the API.
	fn contains(c: &RuntimeRead) -> bool {
		use fungibles::Read::*;
		matches!(
			c,
			RuntimeRead::Fungibles(
				TotalSupply(..)
					| BalanceOf { .. } | Allowance { .. }
					| TokenName(..) | TokenSymbol(..)
					| TokenDecimals(..) | AssetExists(..)
			)
		)
	}
}

impl fungibles::Config for Runtime {
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}

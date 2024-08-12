use crate::{config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall};
use codec::{Decode, Encode, MaxEncodedLen};
use pop_chain_extension::*;
use sp_std::vec::Vec;

impl fungibles::Config for Runtime {
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}

/// A query of runtime state.
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
#[repr(u8)]
pub enum RuntimeRead {
	/// Fungible token queries.
	#[codec(index = 150)]
	Fungibles(fungibles::Read<Runtime>),
}

/// Runtime-specific implementation of the generic extension requirements.
#[derive(Default)]
pub struct Extension;

impl ReadState for Extension {
	type StateQuery = RuntimeRead;

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

	fn read(read: Self::StateQuery) -> Vec<u8> {
		match read {
			RuntimeRead::Fungibles(key) => fungibles::Pallet::read_state(key),
		}
	}
}

impl CallFilter for Extension {
	type Call = RuntimeCall;

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

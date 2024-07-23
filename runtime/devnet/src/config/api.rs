use crate::{config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Contains;

/// A query of runtime state.
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
#[repr(u8)]
pub enum RuntimeRead<T: fungibles::Config> {
	/// Fungible token queries.
	#[codec(index = 150)]
	Fungibles(fungibles::Read<T>),
}

impl fungibles::Config for Runtime {
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}

/// A type to identify allowed calls to the Runtime from the API.
pub struct AllowedApiCalls;

impl Contains<RuntimeCall> for AllowedApiCalls {
	/// Allowed runtime calls from the API.
	fn contains(c: &RuntimeCall) -> bool {
		use fungibles::Call::*;
		matches!(
			c,
			RuntimeCall::Fungibles(transfer { .. } | approve { .. } | increase_allowance { .. })
		)
	}
}

/// State queries that can be made in the API.
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
#[repr(u8)]
pub enum RuntimeStateKeys<T: fungibles::Config> {
	#[codec(index = 150)]
	Fungibles(fungibles::FungiblesKey<T>),
}

impl<T: fungibles::Config> Contains<RuntimeStateKeys<T>> for AllowedApiCalls {
	/// Allowed state queries from the API.
	fn contains(c: &RuntimeStateKeys<T>) -> bool {
		use fungibles::FungiblesKey::*;
		matches!(
			c,
			RuntimeStateKeys::Fungibles(
				TotalSupply(..)
					| BalanceOf(..) | Allowance(..)
					| TokenName(..) | TokenSymbol(..)
					| TokenDecimals(..)
			)
		)
	}
}

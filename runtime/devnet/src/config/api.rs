use crate::{
	config::assets::TrustBackedAssetsInstance, fungibles, AccountId, Assets, Balances, Runtime,
	RuntimeCall,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{fungible::NativeFromLeft, tokens::fungible::NativeOrWithId, Contains};
use pallet_api::fungibles::union_of::FungibleUnionOf;

/// A query of runtime state.
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
#[repr(u8)]
pub enum RuntimeRead<T: fungibles::Config> {
	/// Fungible token queries.
	#[codec(index = 150)]
	Fungibles(fungibles::Read<T>),
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
			)
		)
	}
}

impl<T: fungibles::Config> Contains<RuntimeRead<T>> for AllowedApiCalls {
	/// Allowed state queries from the API.
	fn contains(c: &RuntimeRead<T>) -> bool {
		use fungibles::Read::*;
		matches!(
			c,
			RuntimeRead::Fungibles(
				TotalSupply(..)
					| BalanceOf { .. } | Allowance { .. }
					| TokenName(..) | TokenSymbol(..)
					| TokenDecimals(..)
			)
		)
	}
}

pub type NativeAndTrustBackedAssets<AssetId> =
	FungibleUnionOf<Balances, Assets, NativeFromLeft, NativeOrWithId<AssetId>, AccountId>;

impl fungibles::Config for Runtime {
	type Fungibles = NativeAndTrustBackedAssets<Self::AssetId>;
	type Fungible = NativeOrWithId<Self::AssetId>;
	type FungibleCriterion = NativeFromLeft;
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

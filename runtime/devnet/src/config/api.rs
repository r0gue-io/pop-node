use crate::{
	config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall, RuntimeEvent,
};
use codec::Decode;
use cumulus_primitives_core::Weight;
use filtering::*;
use frame_support::traits::Contains;
use pallet_api::extension::*;
pub(crate) use pallet_api::Extension;
use sp_core::ConstU8;
use versioning::*;

/// A query of runtime state.
#[derive(Decode, Debug)]
#[repr(u8)]
pub enum RuntimeRead {
	/// Fungible token queries.
	#[codec(index = 150)]
	Fungibles(fungibles::Read<Runtime>),
}

impl Readable for RuntimeRead {
	/// Determines the weight of the read, used to charge the appropriate weight before the read is performed.
	fn weight(&self) -> Weight {
		// TODO: defer to relevant pallet - e.g. RuntimeRead::Fungibles(key) => fungibles::Pallet::read_weight(key),
		<Runtime as frame_system::Config>::DbWeight::get().reads(1_u64)
	}

	/// Performs the read and returns the result.
	fn read(self) -> Vec<u8> {
		match self {
			RuntimeRead::Fungibles(key) => fungibles::Pallet::read_state(key),
		}
	}
}

impl fungibles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}

#[derive(Default)]
pub struct Config;
impl pallet_api::extension::Config for Config {
	/// Functions used by the Pop API.
	///
	/// Each function corresponds to specific functionality provided by the API, facilitating the
	/// interaction between smart contracts and the runtime.
	type Functions = (
		// Dispatching calls
		DispatchCall<
			Runtime,
			DecodesAs<VersionedRuntimeCall>,
			IdentifiedByFirstByteOfFunctionId<ConstU8<0>>,
			Filter,
		>,
		// Reading state
		ReadState<
			Runtime,
			RuntimeRead,
			DecodesAs<VersionedRuntimeRead>,
			IdentifiedByFirstByteOfFunctionId<ConstU8<1>>,
			Filter,
		>,
	);

	const LOG_TARGET: &'static str = "pop-api::extension";
}

mod filtering {
	use super::*;

	pub struct Filter;

	impl Contains<RuntimeCall> for Filter {
		fn contains(c: &RuntimeCall) -> bool {
			use fungibles::Call::*;
			matches!(
				c,
				RuntimeCall::Fungibles(
					transfer { .. }
						| transfer_from { .. } | approve { .. }
						| increase_allowance { .. }
						| decrease_allowance { .. }
						| create { .. } | set_metadata { .. }
						| start_destroy { .. } | clear_metadata { .. }
						| mint { .. } | burn { .. }
				)
			)
		}
	}

	impl Contains<RuntimeRead> for Filter {
		fn contains(r: &RuntimeRead) -> bool {
			use fungibles::Read::*;
			matches!(
				r,
				RuntimeRead::Fungibles(
					TotalSupply(..)
						| BalanceOf { .. } | Allowance { .. }
						| TokenName(..) | TokenSymbol(..)
						| TokenDecimals(..) | AssetExists(..)
				)
			)
		}
	}
}

mod versioning {
	use super::*;

	/// Versioned runtime calls.
	#[derive(Decode, Debug)]
	pub enum VersionedRuntimeCall {
		/// Version zero of runtime calls.
		#[codec(index = 0)]
		V0(RuntimeCall),
	}

	impl From<VersionedRuntimeCall> for RuntimeCall {
		fn from(value: VersionedRuntimeCall) -> Self {
			// Allows mapping from some previous runtime call shape to a current valid runtime call
			match value {
				VersionedRuntimeCall::V0(call) => call,
			}
		}
	}

	/// Versioned runtime state reads.
	#[derive(Decode, Debug)]
	pub enum VersionedRuntimeRead {
		/// Version zero of runtime state reads.
		#[codec(index = 0)]
		V0(RuntimeRead),
	}

	impl From<VersionedRuntimeRead> for RuntimeRead {
		fn from(value: VersionedRuntimeRead) -> Self {
			// Allows mapping from some previous runtime call shape to a current valid runtime read
			match value {
				VersionedRuntimeRead::V0(read) => read,
			}
		}
	}
}

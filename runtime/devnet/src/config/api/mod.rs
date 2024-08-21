use crate::{
	config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall, RuntimeEvent,
};
use codec::Decode;
use cumulus_primitives_core::Weight;
use frame_support::traits::Contains;
use pallet_api::extension::*;
pub(crate) use pallet_api::Extension;
use sp_core::ConstU8;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;
use versioning::*;

mod versioning;

type DecodingFailedError = DecodingFailed<Runtime>;
type DecodesAs<Output, Logger = ()> =
	pallet_api::extension::DecodesAs<Output, DecodingFailedError, Logger>;

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
			DecodesAs<VersionedRuntimeCall, DispatchCallLogTarget>,
			IdentifiedByFirstByteOfFunctionId<ConstU8<0>>,
			Filter,
			DispatchCallLogTarget,
		>,
		// Reading state
		ReadState<
			Runtime,
			RuntimeRead,
			DecodesAs<VersionedRuntimeRead, ReadStateLogTarget>,
			IdentifiedByFirstByteOfFunctionId<ConstU8<1>>,
			Filter,
			ReadStateLogTarget,
		>,
	);
	/// Ensure errors are versioned.
	type Error = ErrorConverter<VersionedError>;

	const LOG_TARGET: &'static str = LOG_TARGET;
}

/// Filters used by the chain extension.
pub struct Filter;

impl Contains<RuntimeCall> for Filter {
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

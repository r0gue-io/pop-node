use crate::{
	config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall, RuntimeEvent,
};
use codec::{Decode, Encode, MaxEncodedLen};
use pop_chain_extension::{CallFilter, ReadState};
use sp_std::vec::Vec;

/// A query of runtime state.
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
#[repr(u8)]
pub enum RuntimeRead {
	/// Fungible token queries.
	#[codec(index = 150)]
	Fungibles(fungibles::Read<Runtime>),
}

/// A struct that implement requirements for the Pop API chain extension.
#[derive(Default)]
pub struct Extension;
impl ReadState for Extension {
	type StateQuery = RuntimeRead;

	fn contains(c: &Self::StateQuery) -> bool {
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

	fn read(read: RuntimeRead) -> Vec<u8> {
		match read {
			RuntimeRead::Fungibles(key) => fungibles::Pallet::read_state(key),
		}
	}
}

impl CallFilter for Extension {
	type Call = RuntimeCall;

	fn contains(c: &Self::Call) -> bool {
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

impl fungibles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}

pub(crate) mod reboot {
	use crate::{config::api::RuntimeRead, fungibles, Runtime, RuntimeCall};
	use codec::Decode;
	use cumulus_primitives_core::Weight;
	use frame_support::traits::Contains;
	use pallet_api::extension::*;
	use sp_core::ConstU8;

	#[derive(Default)]
	pub struct Config;
	impl pallet_api::extension::Config for Config {
		/// Functions used by the Pop API
		/// .
		/// Each function corresponds to specific functionality provided by the API, facilitating the
		// interaction between smart contracts and the runtime.
		type Functions = (
			// Dispatching calls
			DispatchCall<
				Runtime,
				DecodesAs<VersionedRuntimeCall>,
				FirstByteOfFunctionId<ConstU8<0>>,
				Filter,
			>,
			// Reading state
			ReadState<
				Runtime,
				RuntimeRead,
				DecodesAs<VersionedRuntimeRead>,
				FirstByteOfFunctionId<ConstU8<1>>,
				Filter,
			>,
		);

		const LOG_TARGET: &'static str = "pop-api::extension";
	}

	/// Versioned runtime calls.
	#[derive(Decode, Debug)]
	pub enum VersionedRuntimeCall {
		/// Version zero of runtime calls.
		#[codec(index = 0)]
		V0(RuntimeCall),
	}

	impl Into<RuntimeCall> for VersionedRuntimeCall {
		fn into(self) -> RuntimeCall {
			// Allows mapping from some previous runtime call shape to a current valid runtime call
			match self {
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

	impl Into<RuntimeRead> for VersionedRuntimeRead {
		fn into(self) -> RuntimeRead {
			// Allows mapping from some previous runtime call shape to a current valid runtime read
			match self {
				VersionedRuntimeRead::V0(read) => read,
			}
		}
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
			use crate::fungibles::Read::*;
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

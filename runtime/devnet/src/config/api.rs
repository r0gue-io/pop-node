use crate::{config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall};
use codec::{Decode, Encode, MaxEncodedLen};
use pop_chain_extension::{CallFilter, ReadState};
use sp_std::vec::Vec;

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

impl CallFilter for Extension {
	type Call = RuntimeCall;

	fn contains(c: &Self::Call) -> bool {
		use fungibles::Call::*;
		matches!(
			c,
			RuntimeCall::Fungibles(transfer { .. } | approve { .. } | increase_allowance { .. })
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

impl fungibles::Config for Runtime {
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}

pub(crate) mod reboot {
	use crate::{config::api::RuntimeRead, fungibles, Runtime, RuntimeCall};
	use codec::Decode;
	use cumulus_primitives_core::Weight;
	use frame_support::traits::Contains;
	use pop_chain_extension::reboot::{RuntimeRead as Read, *};
	use sp_core::ConstU8;

	#[derive(Decode, Debug)]
	pub enum VersionedRuntimeCall {
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

	#[derive(Decode, Debug)]
	pub enum VersionedRuntimeRead {
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

	impl Read for RuntimeRead {
		fn weight(&self) -> Weight {
			// TODO: defer to relevant pallet - e.g. RuntimeRead::Fungibles(key) => fungibles::Pallet::read_weight(key),
			<Runtime as frame_system::Config>::DbWeight::get().reads(1_u64)
		}

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

	#[derive(Default)]
	pub struct Functions;
	impl pop_chain_extension::reboot::Functions for Functions {
		// Configure the functions available
		type Function = (
			// Dispatching calls
			DispatchCall<
				Runtime,
				// Use bytes from func_id() + ext_id() to prefix the encoded input bytes to determine the versioned dispatch
				PrefixBuilder<Runtime, VersionedRuntimeCall>,
				// Type for versioning runtime calls
				VersionedRuntimeCall,
				// Use first byte of func_id to match to this function, with value of zero.
				FirstByte<ConstU8<0>>,
				// Filtering of allowed calls
				Filter,
			>,
			// Reading state
			ReadState<
				Runtime,
				// Use bytes from func_id() + ext_id() to prefix the encoded input bytes to determine the versioned read
				PrefixBuilder<Runtime, VersionedRuntimeRead>,
				// Type for versioning runtime reads
				VersionedRuntimeRead,
				// The current runtime reads available
				RuntimeRead,
				// Use first byte of func_id to match to this function, with value of one.
				FirstByte<ConstU8<1>>,
				// Filtering of allowed reads
				Filter,
			>,
		);
	}
}

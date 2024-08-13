use crate::{
	config::assets::TrustBackedAssetsInstance,
	fungibles::{self},
	Runtime, RuntimeCall,
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
					| TokenDecimals(..) | AssetExists(..)
			)
		)
	}
}

/// Wrapper to enable versioning of runtime state reads.
#[derive(Decode, Debug)]
enum VersionedStateRead<T: fungibles::Config> {
	/// Version zero of state reads.
	#[codec(index = 0)]
	V0(RuntimeRead<T>),
}

/// Wrapper to enable versioning of runtime calls.
#[derive(Decode, Debug)]
enum VersionedDispatch<T: PopApiExtensionConfig> {
	/// Version zero of dispatch calls.
	#[codec(index = 0)]
	V0(T::RuntimeCall),
}

pub struct DevnetParamsHandler;

impl DispatchCallParamsHandler for DevnetParamsHandler {
	fn handle_params<T, E>(
		env: &mut Environment<E, BufInBufOutState>,
		params: Vec<u8>,
	) -> Result<(), DispatchError>
	where
		E: Ext<T = T>,
		T: PopApiExtensionConfig,
	{
		const LOG_PREFIX: &str = " dispatch |";

		let call =
			<VersionedDispatch<T>>::decode(&mut &params[..]).map_err(|_| DECODING_FAILED_ERROR)?;

		// Contract is the origin by default.
		let origin: T::RuntimeOrigin = RawOrigin::Signed(env.ext().address().clone()).into();
		match call {
			VersionedDispatch::V0(call) => dispatch_call::<T, E>(env, call, origin, LOG_PREFIX),
		}
	}
}

impl ReadStateParamsHandler for DevnetParamsHandler {
	fn handle_params<T, E>(
		env: &mut Environment<E, BufInBufOutState>,
		params: Vec<u8>,
	) -> Result<(), DispatchError>
	where
		E: Ext<T = T>,
		T: PopApiExtensionConfig,
	{
		const LOG_PREFIX: &str = " read_state |";

		let read =
			<VersionedStateRead<T>>::decode(&mut &params[..]).map_err(|_| DECODING_FAILED_ERROR)?;

		// Charge weight for doing one storage read.
		env.charge_weight(T::DbWeight::get().reads(1_u64))?;
		let result = match read {
			VersionedStateRead::V0(read) => {
				ensure!(AllowedApiCalls::contains(&read), UNKNOWN_CALL_ERROR);
				match read {
					RuntimeRead::Fungibles(key) => fungibles::Pallet::<T>::read_state(key),
				}
			},
		};
		log::trace!(
			target:LOG_TARGET,
			"{} result: {:?}.", LOG_PREFIX, result
		);
		env.write(&result, false, None)
	}
}

impl PopApiExtensionConfig for Runtime {
	type AssetInstance = TrustBackedAssetsInstance;
	type ReadStateParamsHandler = DevnetParamsHandler;
	type DispatchCallParamsHandler = DevnetParamsHandler;
	type RuntimeRead = RuntimeRead<Self>;
	type AllowedDispatchCalls = AllowedApiCalls;
	type AllowedReadCalls = AllowedApiCalls;
}

impl fungibles::Config for Runtime {
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}

use crate::{
	config::assets::TrustBackedAssetsInstance,
	fungibles::{self},
	Runtime, RuntimeCall,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::OriginTrait;
use frame_support::{ensure, traits::Contains};
use frame_system::RawOrigin;
use pallet_contracts::chain_extension::{BufInBufOutState, Environment, Ext};
use pop_runtime_extensions::{
	constants::{DECODING_FAILED_ERROR, LOG_TARGET, UNKNOWN_CALL_ERROR},
	dispatch_call, DispatchCallParamsHandler, PopApiExtensionConfig, ReadStateParamsHandler,
};
use sp_core::Get;
use sp_runtime::DispatchError;

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

pub struct ChainExtensionEnvironment;

impl DispatchCallParamsHandler for ChainExtensionEnvironment {
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
		let mut origin: T::RuntimeOrigin = RawOrigin::Signed(env.ext().address().clone()).into();
		match call {
			VersionedDispatch::V0(call) => {
				origin.add_filter(T::AllowedDispatchCalls::contains);
				dispatch_call::<T, E>(env, call, origin, LOG_PREFIX)
			},
		}
	}
}

impl ReadStateParamsHandler for ChainExtensionEnvironment {
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
	type ReadStateParamsHandler = ChainExtensionEnvironment;
	type DispatchCallParamsHandler = ChainExtensionEnvironment;
	type AllowedDispatchCalls = AllowedApiCalls;
}

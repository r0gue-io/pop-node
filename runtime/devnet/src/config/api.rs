use crate::{
	config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall, RuntimeEvent,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::ensure;
use pallet_contracts::chain_extension::{BufInBufOutState, Environment, Ext, State};
use pop_chain_extension::{
	ExtensionTrait, ExtractEnv, HandleDispatch, ReadState, DECODING_FAILED_ERROR,
	UNKNOWN_CALL_ERROR,
};
use sp_runtime::DispatchError;
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

impl ExtensionTrait for Extension {
	type StateReader = StateReader;
	type DispatchHandler = DispatchHandler;
	type EnvExtractor = EnvExtractor;
}

#[derive(Default)]
pub struct StateReader;

impl ReadState for StateReader {
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

	fn versioned_read_state_handler(
		mut params: Vec<u8>,
		version: u8,
		pallet_index: u8,
		call_index: u8,
	) -> Result<Self::StateQuery, DispatchError> {
		match version.try_into()? {
			VersionedStateRead::V0 => {
				// Prefix params with pallet, index to simplify decoding.
				params.insert(0, pallet_index);
				params.insert(1, call_index);
				decode_checked::<Self::StateQuery>(&mut &params[..])
			},
		}
	}
}

// Wrapper to enable versioning of runtime state reads.
enum VersionedStateRead {
	// Version zero of state reads.
	V0,
}

impl TryFrom<u8> for VersionedStateRead {
	type Error = DispatchError;

	// Attempts to convert a `u8` value to its corresponding `VersionedStateRead` variant.
	//
	// If the `u8` value does not match any known function identifier, it returns a
	// `DispatchError::Other` indicating an unknown versioned state read.
	fn try_from(index: u8) -> Result<Self, Self::Error> {
		match index {
			0 => Ok(VersionedStateRead::V0),
			_ => Err(UNKNOWN_CALL_ERROR),
		}
	}
}

#[derive(Default)]
pub struct DispatchHandler;

impl HandleDispatch for DispatchHandler {
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

	fn versioned_dispatch_handler(
		mut params: Vec<u8>,
		version: u8,
		pallet_index: u8,
		call_index: u8,
	) -> Result<RuntimeCall, DispatchError> {
		match version.try_into()? {
			VersionedDispatch::V0 => {
				// Prefix params with version, pallet, index to simplify decoding.
				params.insert(1, pallet_index);
				params.insert(2, call_index);
				decode_checked::<RuntimeCall>(&mut &params[..])
			},
		}
	}
}

// Wrapper to enable versioning of runtime calls.
enum VersionedDispatch {
	// Version zero of dispatch calls.
	V0,
}

impl TryFrom<u8> for VersionedDispatch {
	type Error = DispatchError;

	// Attempts to convert a `u8` value to its corresponding `VersionedStateRead` variant.
	//
	// If the `u8` value does not match any known function identifier, it returns a
	// `DispatchError::Other` indicating an unknown versioned state read.
	fn try_from(index: u8) -> Result<Self, Self::Error> {
		match index {
			0 => Ok(VersionedDispatch::V0),
			_ => Err(UNKNOWN_CALL_ERROR),
		}
	}
}

// Helper method to decode the byte data to a provided type and throws error if failed.
fn decode_checked<T: Decode>(params: &mut &[u8]) -> Result<T, DispatchError> {
	T::decode(params).map_err(|_| DECODING_FAILED_ERROR)
}

#[derive(Default)]
pub struct EnvExtractor;

impl ExtractEnv for EnvExtractor {
	// Extract (version, function_id, pallet_index, call_index) from the payload bytes.
	fn extract_env<T, E: Ext<T = T>>(env: &Environment<E, BufInBufOutState>) -> (u8, u8, u8, u8) {
		// Extract version and function_id from first two bytes.
		let (version, function_id) = {
			let bytes = env.func_id().to_le_bytes();
			(bytes[0], bytes[1])
		};
		// Extract pallet index and call / key index from last two bytes.
		let (pallet_index, call_index) = {
			let bytes = env.ext_id().to_le_bytes();
			(bytes[0], bytes[1])
		};

		(version, function_id, pallet_index, call_index)
	}
}

impl fungibles::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}

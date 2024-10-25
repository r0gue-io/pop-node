use core::marker::PhantomData;

use codec::Decode;
use cumulus_primitives_core::Weight;
use frame_support::traits::Contains;
pub(crate) use pallet_api::Extension;
use pallet_api::{extension::*, Read};
use sp_core::ConstU8;
use sp_runtime::DispatchError;
use sp_std::vec::Vec;
use versioning::*;

use crate::{
	config::assets::TrustBackedAssetsInstance, fungibles, Runtime, RuntimeCall, RuntimeEvent,
};

mod versioning;

type DecodingFailedError = DecodingFailed<Runtime>;
type DecodesAs<Output, Logger = ()> = pallet_api::extension::DecodesAs<
	Output,
	ContractWeightsOf<Runtime>,
	DecodingFailedError,
	Logger,
>;

/// A query of runtime state.
#[derive(Decode, Debug)]
#[cfg_attr(test, derive(PartialEq, Clone))]
#[repr(u8)]
pub enum RuntimeRead {
	/// Fungible token queries.
	#[codec(index = 150)]
	Fungibles(fungibles::Read<Runtime>),
}

impl Readable for RuntimeRead {
	/// The corresponding type carrying the result of the query for runtime state.
	type Result = RuntimeResult;

	/// Determines the weight of the read, used to charge the appropriate weight before the read is
	/// performed.
	fn weight(&self) -> Weight {
		match self {
			RuntimeRead::Fungibles(key) => fungibles::Pallet::weight(key),
		}
	}

	/// Performs the read and returns the result.
	fn read(self) -> Self::Result {
		match self {
			RuntimeRead::Fungibles(key) => RuntimeResult::Fungibles(fungibles::Pallet::read(key)),
		}
	}
}

/// The result of a runtime state read.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Clone))]
pub enum RuntimeResult {
	/// Fungible token read results.
	Fungibles(fungibles::ReadResult<Runtime>),
}

impl RuntimeResult {
	/// Encodes the result.
	fn encode(&self) -> Vec<u8> {
		match self {
			RuntimeResult::Fungibles(result) => result.encode(),
		}
	}
}

impl fungibles::Config for Runtime {
	type AssetsInstance = TrustBackedAssetsInstance;
	type RuntimeEvent = RuntimeEvent;
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
			// Function ID: 0.
			IdentifiedByFirstByteOfFunctionId<ConstU8<0>>,
			// The runtime configuration.
			Runtime,
			// Decode as a versioned runtime call.
			DecodesAs<VersionedRuntimeCall, DispatchCallLogTarget>,
			// Apply any filtering.
			Filter<Runtime>,
			// Ensure errors are versioned.
			VersionedErrorConverter<VersionedError>,
			// Logging with a specific target.
			DispatchCallLogTarget,
		>,
		// Reading state
		ReadState<
			// Function ID: 1.
			IdentifiedByFirstByteOfFunctionId<ConstU8<1>>,
			// The runtime configuration.
			Runtime,
			// The runtime state reads available.
			RuntimeRead,
			// Decode as a versioned runtime read.
			DecodesAs<VersionedRuntimeRead, ReadStateLogTarget>,
			// Apply any filtering.
			Filter<Runtime>,
			// Convert the result of a read into the expected versioned result
			VersionedResultConverter<RuntimeResult, VersionedRuntimeResult>,
			// Ensure errors are versioned.
			VersionedErrorConverter<VersionedError>,
			// Logging with a specific target.
			ReadStateLogTarget,
		>,
	);

	/// The log target.
	const LOG_TARGET: &'static str = LOG_TARGET;
}

/// Filters used by the chain extension.
pub struct Filter<T>(PhantomData<T>);

impl<T: frame_system::Config<RuntimeCall = RuntimeCall>> Contains<RuntimeCall> for Filter<T> {
	fn contains(c: &RuntimeCall) -> bool {
		use fungibles::Call::*;
		use pallet_sponsorships::Call::*;
		T::BaseCallFilter::contains(c) &&
			matches!(
				c,
				RuntimeCall::Fungibles(
					transfer { .. } |
						transfer_from { .. } |
						approve { .. } | increase_allowance { .. } |
						decrease_allowance { .. } |
						create { .. } | set_metadata { .. } |
						start_destroy { .. } |
						clear_metadata { .. } |
						mint { .. } | burn { .. }
				) | RuntimeCall::Sponsorships(
					sponsor_account { .. } | remove_sponsorship_for { .. }
				)
			)
	}
}

impl<T: frame_system::Config> Contains<RuntimeRead> for Filter<T> {
	fn contains(r: &RuntimeRead) -> bool {
		use fungibles::Read::*;
		matches!(
			r,
			RuntimeRead::Fungibles(
				TotalSupply(..) |
					BalanceOf { .. } |
					Allowance { .. } |
					TokenName(..) | TokenSymbol(..) |
					TokenDecimals(..) |
					TokenExists(..)
			)
		)
	}
}

#[cfg(test)]
mod tests {
	use codec::Encode;
	use pallet_api::fungibles::Call::*;
	use sp_core::crypto::AccountId32;
	use RuntimeCall::{Balances, Fungibles};

	use super::*;

	const ACCOUNT: AccountId32 = AccountId32::new([0u8; 32]);

	#[test]
	fn runtime_result_encode_works() {
		let value = 1_000;
		let result = fungibles::ReadResult::<Runtime>::TotalSupply(value);
		assert_eq!(RuntimeResult::Fungibles(result).encode(), value.encode());
	}

	#[test]
	fn filter_prevents_runtime_filtered_calls() {
		use pallet_balances::{AdjustmentDirection, Call::*};
		use sp_runtime::MultiAddress;

		const CALLS: [RuntimeCall; 4] = [
			Balances(force_adjust_total_issuance {
				direction: AdjustmentDirection::Increase,
				delta: 0,
			}),
			Balances(force_set_balance { who: MultiAddress::Address32([0u8; 32]), new_free: 0 }),
			Balances(force_transfer {
				source: MultiAddress::Address32([0u8; 32]),
				dest: MultiAddress::Address32([0u8; 32]),
				value: 0,
			}),
			Balances(force_unreserve { who: MultiAddress::Address32([0u8; 32]), amount: 0 }),
		];

		for call in CALLS {
			assert!(!Filter::<Runtime>::contains(&call))
		}
	}

	#[test]
	fn filter_allows_fungibles_calls() {
		const CALLS: [RuntimeCall; 11] = [
			Fungibles(transfer { token: 0, to: ACCOUNT, value: 0 }),
			Fungibles(transfer_from { token: 0, from: ACCOUNT, to: ACCOUNT, value: 0 }),
			Fungibles(approve { token: 0, spender: ACCOUNT, value: 0 }),
			Fungibles(increase_allowance { token: 0, spender: ACCOUNT, value: 0 }),
			Fungibles(decrease_allowance { token: 0, spender: ACCOUNT, value: 0 }),
			Fungibles(create { id: 0, admin: ACCOUNT, min_balance: 0 }),
			Fungibles(set_metadata { token: 0, name: vec![], symbol: vec![], decimals: 0 }),
			Fungibles(start_destroy { token: 0 }),
			Fungibles(clear_metadata { token: 0 }),
			Fungibles(mint { token: 0, account: ACCOUNT, value: 0 }),
			Fungibles(burn { token: 0, account: ACCOUNT, value: 0 }),
		];

		for call in CALLS {
			assert!(Filter::<Runtime>::contains(&call))
		}
	}

	#[test]
	fn filter_allows_fungibles_reads() {
		use super::{fungibles::Read::*, RuntimeRead::*};
		const READS: [RuntimeRead; 7] = [
			Fungibles(TotalSupply(1)),
			Fungibles(BalanceOf { token: 1, owner: ACCOUNT }),
			Fungibles(Allowance { token: 1, owner: ACCOUNT, spender: ACCOUNT }),
			Fungibles(TokenName(1)),
			Fungibles(TokenSymbol(1)),
			Fungibles(TokenDecimals(10)),
			Fungibles(TokenExists(1)),
		];

		for read in READS {
			assert!(Filter::<Runtime>::contains(&read))
		}
	}
}

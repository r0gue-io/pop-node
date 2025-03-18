use alloc::vec::Vec;
use core::marker::PhantomData;

use codec::Decode;
use cumulus_primitives_core::Weight;
use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo, PostDispatchInfo},
	traits::{Contains, EnsureOrigin},
};
pub(crate) use pallet_api::Extension;
use pallet_api::{extension::*, messaging, messaging::NotifyQueryHandler, Read};
use pallet_contracts::{CollectEvents, DebugInfo, Determinism};
use pallet_xcm::Origin;
use sp_core::{ConstU32, ConstU8};
use sp_runtime::DispatchError;
use versioning::*;
use xcm::prelude::Location;


use crate::{
	config::{
		assets::TrustBackedAssetsInstance, monetary::TransactionByteFee, xcm::LocalOriginToLocation,
	},
	fungibles, AccountId, Balances, BlockNumber, Contracts, Ismp, PolkadotXcm, Runtime,
	RuntimeCall, RuntimeEvent, RuntimeHoldReason, messaging::ReadResult::*, parameter_types
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
	/// Messaging state queries.
	#[codec(index = 152)]
	Messaging(messaging::Read<Runtime>),
}

impl Readable for RuntimeRead {
	/// The corresponding type carrying the result of the query for runtime state.
	type Result = RuntimeResult;

	/// Determines the weight of the read, used to charge the appropriate weight before the read is
	/// performed.
	fn weight(&self) -> Weight {
		match self {
			RuntimeRead::Fungibles(key) => fungibles::Pallet::weight(key),
			RuntimeRead::Messaging(key) => messaging::Pallet::weight(key),
		}
	}

	/// Performs the read and returns the result.
	fn read(self) -> Self::Result {
		match self {
			RuntimeRead::Fungibles(key) => RuntimeResult::Fungibles(fungibles::Pallet::read(key)),
			RuntimeRead::Messaging(key) => RuntimeResult::Messaging(messaging::Pallet::read(key)),
		}
	}
}

/// The result of a runtime state read.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Clone))]
pub enum RuntimeResult {
	/// Fungible token read results.
	Fungibles(fungibles::ReadResult<Runtime>),
	/// Messaging state read results.
	Messaging(messaging::ReadResult),
}

impl RuntimeResult {
	/// Encodes the result.
	fn encode(&self) -> Vec<u8> {
		match self {
			RuntimeResult::Fungibles(result) => result.encode(),
			RuntimeResult::Messaging(result) => result.encode(),
		}
	}
}

parameter_types! {
	pub const MaxXcmQueryTimeoutsPerBlock: u32 = 100;

}

impl messaging::Config for Runtime {
	type OffChainByteFee = TransactionByteFee;
	type OnChainByteFee = TransactionByteFee;
	type CallbackExecutor = CallbackExecutor;
	type Deposit = Balances;
	type IsmpDispatcher = Ismp;
	type MaxContextLen = ConstU32<64>;
	type MaxDataLen = ConstU32<1024>;
	type MaxKeyLen = ConstU32<32>;
	type MaxKeys = ConstU32<10>;
	// TODO: size appropriately
	type MaxRemovals = ConstU32<1024>;
	// TODO: ensure within the contract buffer bounds
	type MaxResponseLen = ConstU32<1024>;
	type OriginConverter = LocalOriginToLocation;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Xcm = QueryHandler;
	type XcmResponseOrigin = EnsureResponse;
	type MaxXcmQueryTimeoutsPerBlock = MaxXcmQueryTimeoutsPerBlock;
}

pub struct EnsureResponse;
impl<O: Into<Result<Origin, O>> + From<Origin>> EnsureOrigin<O> for EnsureResponse {
	type Success = Location;

	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| match o {
			Origin::Response(location) => Ok(location),
			r => Err(O::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<O, ()> {
		todo!()
	}
}

pub struct CallbackExecutor;
impl messaging::CallbackExecutor<Runtime> for CallbackExecutor {
	fn execute(account: AccountId, data: Vec<u8>, weight: Weight) -> DispatchResultWithPostInfo {
		// Default
		#[cfg(not(feature = "std"))]
		let debug = DebugInfo::Skip;
		#[cfg(not(feature = "std"))]
		let collect_events = CollectEvents::Skip;
		// Testing
		#[cfg(feature = "std")]
		let debug = DebugInfo::UnsafeDebug;
		#[cfg(feature = "std")]
		let collect_events = CollectEvents::UnsafeCollect;

		let mut output = Contracts::bare_call(
			account.clone(),
			account,
			Default::default(),
			weight,
			Default::default(),
			data,
			debug,
			collect_events,
			Determinism::Enforced,
		);
		if let Ok(return_value) = &output.result {
			if return_value.did_revert() {
				output.result = Err(pallet_revive::Error::<Runtime>::ContractReverted.into());
			}
		}

		let post_info = PostDispatchInfo {
			actual_weight: Some(output.gas_consumed.saturating_add(Self::execution_weight())),
			pays_fee: Default::default(),
		};

		output
			.result
			.map(|_| post_info)
			.map_err(|e| DispatchErrorWithPostInfo { post_info, error: e })
	}

	fn execution_weight() -> Weight {
		use pallet_revive::WeightInfo;
		<Runtime as pallet_revive::Config>::WeightInfo::call()
	}
}

pub struct QueryHandler;
impl NotifyQueryHandler<Runtime> for QueryHandler {
	type WeightInfo = pallet_xcm::Pallet<Runtime>;
	fn new_notify_query(
		responder: impl Into<Location>,
		notify: messaging::Call<Runtime>,
		timeout: BlockNumber,
		match_querier: impl Into<Location>,
	) -> u64 {
		PolkadotXcm::new_notify_query(responder, notify, timeout, match_querier)
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
		let contain_fungibles: bool = {
			use fungibles::Call::*;
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
						mint { .. } | burn { .. },
				)
			)
		};

		let contain_messaging: bool = {
			use messaging::Call::*;
			matches!(
				c,
				RuntimeCall::Messaging(
						ismp_get { .. } | ismp_post { .. } |
						xcm_new_query { .. } |
						remove { .. },
				)
			)
		};

		T::BaseCallFilter::contains(c) && contain_fungibles | contain_messaging
	}
}

impl<T: frame_system::Config> Contains<RuntimeRead> for Filter<T> {
	fn contains(r: &RuntimeRead) -> bool {
		let contain_fungibles: bool = {
			use fungibles::Read::*;
			matches!(
				r,
				RuntimeRead::Fungibles(
					TotalSupply(..) |
						BalanceOf { .. } | Allowance { .. } |
						TokenName(..) | TokenSymbol(..) |
						TokenDecimals(..) | TokenExists(..),
				)
			)
		};

		let contain_messaging: bool = {
			use messaging::Read::*;
			matches!(r, RuntimeRead::Messaging(PollStatus(..) | GetResponse(..) | QueryId(..)))
		};

		contain_fungibles | contain_messaging
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

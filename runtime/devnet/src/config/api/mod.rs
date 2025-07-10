use alloc::vec::Vec;
use core::marker::PhantomData;

use codec::Decode;
use cumulus_primitives_core::Weight;
use frame_support::{
	dispatch::PostDispatchInfo,
	pallet_prelude::{DispatchResultWithPostInfo, EnsureOrigin, Pays},
	parameter_types,
	sp_runtime::DispatchError,
	traits::Contains,
};
pub(crate) use pallet_api::Extension;
use pallet_api::{extension::*, Read};
use sp_core::{ConstU32, ConstU8};
use versioning::*;

use crate::{
	config::assets::{TrustBackedAssetsInstance, TrustBackedNftsInstance},
	fungibles, nonfungibles, AccountId, Balances, BlockNumber, DealWithFees, Ismp, Runtime,
	RuntimeCall, RuntimeHoldReason, TransactionByteFee, H160,
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
	/// Non-fungible token queries.
	#[codec(index = 151)]
	NonFungibles(nonfungibles::Read<Runtime>),
}

impl Readable for RuntimeRead {
	/// The corresponding type carrying the result of the query for runtime state.
	type Result = RuntimeResult;

	/// Determines the weight of the read, used to charge the appropriate weight before the read is
	/// performed.
	fn weight(&self) -> Weight {
		match self {
			RuntimeRead::Fungibles(key) => fungibles::Pallet::weight(key),
			RuntimeRead::NonFungibles(key) => nonfungibles::Pallet::weight(key),
		}
	}

	/// Performs the read and returns the result.
	fn read(self) -> Self::Result {
		match self {
			RuntimeRead::Fungibles(key) => RuntimeResult::Fungibles(fungibles::Pallet::read(key)),
			RuntimeRead::NonFungibles(key) =>
				RuntimeResult::NonFungibles(nonfungibles::Pallet::read(key)),
		}
	}
}

/// The result of a runtime state read.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Clone))]
pub enum RuntimeResult {
	/// Fungible token read results.
	Fungibles(fungibles::ReadResult<Runtime>),
	/// Non-fungible token read results.
	NonFungibles(nonfungibles::ReadResult<Runtime>),
}

impl RuntimeResult {
	/// Encodes the result.
	fn encode(&self) -> Vec<u8> {
		match self {
			RuntimeResult::Fungibles(result) => result.encode(),
			RuntimeResult::NonFungibles(result) => result.encode(),
		}
	}
}

impl fungibles::Config for Runtime {
	type AssetsInstance = TrustBackedAssetsInstance;
	type WeightInfo = fungibles::weights::SubstrateWeight<Runtime>;
}

impl nonfungibles::Config for Runtime {
	type NftsInstance = TrustBackedNftsInstance;
	type WeightInfo = ();
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
						mint { .. } | burn { .. }
				)
			)
		};

		let contain_nonfungibles: bool =
			{
				use nonfungibles::Call::*;
				matches!(
					c,
					RuntimeCall::NonFungibles(
						approve { .. } |
							transfer { .. } | create { .. } |
							destroy { .. } | set_metadata { .. } |
							clear_metadata { .. } | set_attribute { .. } |
							clear_attribute { .. } |
							set_max_supply { .. } | approve_item_attributes { .. } |
							cancel_item_attributes_approval { .. } |
							clear_all_transfer_approvals { .. } |
							clear_collection_approvals { .. } |
							mint { .. } | burn { .. },
					)
				)
			};

		T::BaseCallFilter::contains(c) && (contain_fungibles | contain_nonfungibles)
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
						TokenDecimals(..) | TokenExists(..)
				)
			)
		};

		let contain_nonfungibles: bool = {
			use nonfungibles::Read::*;
			matches!(
				r,
				RuntimeRead::NonFungibles(
					BalanceOf { .. } |
						OwnerOf { .. } | Allowance { .. } |
						TotalSupply(..) | GetAttribute { .. } |
						ItemMetadata { .. } |
						NextCollectionId,
				)
			)
		};

		contain_fungibles | contain_nonfungibles
	}
}

impl pallet_api_vnext::fungibles::Config<TrustBackedAssetsInstance> for Runtime {
	type WeightInfo = ();
}

mod messaging {
	use pallet_api_vnext::messaging;
	use pallet_xcm::Origin;
	use xcm::latest::Location;

	use super::*;
	use crate::config::xcm::LocalOriginToLocation;

	parameter_types! {
			pub const MaxXcmQueryTimeoutsPerBlock: u32 = 100;
	}

	impl messaging::Config for Runtime {
		type CallbackExecutor = CallbackExecutor;
		type FeeHandler = DealWithFees;
		type Fungibles = Balances;
		type IsmpDispatcher = Ismp;
		type Keccak256 = Ismp;
		type MaxContextLen = ConstU32<64>;
		type MaxDataLen = ConstU32<512>;
		type MaxKeyLen = ConstU32<8>;
		type MaxKeys = ConstU32<10>;
		// TODO: size appropriately
		type MaxRemovals = ConstU32<100>;
		// TODO: ensure within the contract buffer bounds
		type MaxResponseLen = ConstU32<512>;
		type MaxXcmQueryTimeoutsPerBlock = MaxXcmQueryTimeoutsPerBlock;
		type OffChainByteFee = TransactionByteFee;
		type OnChainByteFee = TransactionByteFee;
		type OriginConverter = LocalOriginToLocation;
		type RuntimeHoldReason = RuntimeHoldReason;
		type WeightInfo = ();
		type WeightToFee = <Runtime as pallet_transaction_payment::Config>::WeightToFee;
		type Xcm = QueryHandler;
		type XcmResponseOrigin = EnsureResponse;
	}

	pub struct CallbackExecutor;
	#[cfg(not(feature = "runtime-benchmarks"))]
	impl messaging::CallbackExecutor<Runtime> for CallbackExecutor {
		fn execute(
			account: &AccountId,
			contract: H160,
			data: Vec<u8>,
			weight: Weight,
		) -> DispatchResultWithPostInfo {
			use frame_support::dispatch::DispatchErrorWithPostInfo;
			use pallet_revive::DepositLimit;

			use crate::{Revive, RuntimeOrigin};

			let mut output = Revive::bare_call(
				RuntimeOrigin::signed(account.clone()),
				contract,
				Default::default(),
				weight,
				// TODO: expose?
				DepositLimit::Balance(Default::default()),
				data,
			);

			log::debug!(target: "pop-api", "callback weight consumed={:?}, weight required={:?}", output.gas_consumed, output.gas_required);
			if let Ok(return_value) = &output.result {
				let pallet_revive::ExecReturnValue { flags: _, data } = return_value;
				log::debug!(target: "pop-api::extension", "return data={:?}", data);
				if return_value.did_revert() {
					output.result = Err(pallet_revive::Error::<Runtime>::ContractReverted.into());
				}
			}

			let post_info =
				PostDispatchInfo { actual_weight: Some(output.gas_consumed), pays_fee: Pays::No };

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
	#[cfg(feature = "runtime-benchmarks")]
	impl messaging::CallbackExecutor<Runtime> for CallbackExecutor {
		/// A successfull callback is the most expensive case.
		fn execute(
			_account: &AccountId,
			_contract: H160,
			_data: Vec<u8>,
			weight: sp_runtime::Weight,
		) -> frame_support::dispatch::DispatchResultWithPostInfo {
			DispatchResultWithPostInfo::Ok(PostDispatchInfo {
				actual_weight: Some(weight / 2),
				pays_fee: Pays::Yes,
			})
		}

		fn execution_weight() -> sp_runtime::Weight {
			Default::default()
		}
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
			Ok(Origin::Response(Location { parents: 1, interior: xcm::latest::Junctions::Here })
				.into())
		}
	}

	pub struct QueryHandler;
	impl messaging::transports::xcm::NotifyQueryHandler<Runtime> for QueryHandler {
		type WeightInfo = pallet_xcm::Pallet<Runtime>;

		fn new_notify_query(
			responder: impl Into<Location>,
			notify: messaging::Call<Runtime>,
			timeout: BlockNumber,
			match_querier: impl Into<Location>,
		) -> u64 {
			crate::PolkadotXcm::new_notify_query(responder, notify, timeout, match_querier)
		}
	}
}

#[cfg(test)]
mod tests {
	use codec::Encode;
	use pallet_api::fungibles::Call::*;
	use sp_core::{bounded_vec, crypto::AccountId32};
	use RuntimeCall::{Balances, Fungibles, NonFungibles};

	use super::*;

	const ACCOUNT: AccountId32 = AccountId32::new([0u8; 32]);

	#[test]
	fn runtime_result_encode_works() {
		let value = 1_000;
		let result = fungibles::ReadResult::<Runtime>::TotalSupply(value);
		assert_eq!(RuntimeResult::Fungibles(result).encode(), value.encode());

		let value = 1_000;
		let result = nonfungibles::ReadResult::<Runtime>::TotalSupply(value);
		assert_eq!(RuntimeResult::NonFungibles(result).encode(), value.encode());
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

	#[test]
	fn filter_allows_nonfungibles_calls() {
		use pallet_api::nonfungibles::{
			Call::*, CancelAttributesApprovalWitness, CollectionConfig, CollectionSettings,
			DestroyWitness, MintSettings,
		};

		for call in vec![
			NonFungibles(approve {
				collection: 0,
				item: Some(0),
				operator: ACCOUNT,
				approved: false,
				deadline: None,
			}),
			NonFungibles(transfer { collection: 0, item: 0, to: ACCOUNT }),
			NonFungibles(create {
				admin: ACCOUNT,
				config: CollectionConfig {
					max_supply: Some(0),
					mint_settings: MintSettings::default(),
					settings: CollectionSettings::all_enabled(),
				},
			}),
			NonFungibles(destroy {
				collection: 0,
				witness: DestroyWitness { attributes: 0, item_configs: 0, item_metadatas: 0 },
			}),
			NonFungibles(set_attribute {
				collection: 0,
				item: Some(0),
				namespace: pallet_nfts::AttributeNamespace::Pallet,
				key: bounded_vec![],
				value: bounded_vec![],
			}),
			NonFungibles(clear_attribute {
				collection: 0,
				item: Some(0),
				namespace: pallet_nfts::AttributeNamespace::Pallet,
				key: bounded_vec![],
			}),
			NonFungibles(set_metadata { collection: 0, item: 0, data: bounded_vec![] }),
			NonFungibles(clear_metadata { collection: 0, item: 0 }),
			NonFungibles(set_max_supply { collection: 0, max_supply: 0 }),
			NonFungibles(approve_item_attributes { collection: 0, item: 0, delegate: ACCOUNT }),
			NonFungibles(cancel_item_attributes_approval {
				collection: 0,
				item: 0,
				delegate: ACCOUNT,
				witness: CancelAttributesApprovalWitness { account_attributes: 0 },
			}),
			NonFungibles(clear_all_transfer_approvals { collection: 0, item: 0 }),
			NonFungibles(clear_collection_approvals { collection: 0, limit: 0 }),
			NonFungibles(mint { to: ACCOUNT, collection: 0, item: 0, witness: None }),
			NonFungibles(burn { collection: 0, item: 0 }),
		]
		.iter()
		{
			assert!(Filter::<Runtime>::contains(call))
		}
	}

	#[test]
	fn filter_allows_nonfungibles_reads() {
		use super::{nonfungibles::Read::*, RuntimeRead::*};

		for read in vec![
			NonFungibles(BalanceOf { collection: 1, owner: ACCOUNT }),
			NonFungibles(OwnerOf { collection: 1, item: 1 }),
			NonFungibles(Allowance {
				collection: 1,
				owner: ACCOUNT,
				operator: ACCOUNT,
				item: None,
			}),
			NonFungibles(TotalSupply(1)),
			NonFungibles(GetAttribute {
				collection: 1,
				item: Some(1),
				namespace: pallet_nfts::AttributeNamespace::CollectionOwner,
				key: bounded_vec![],
			}),
			NonFungibles(NextCollectionId),
			NonFungibles(ItemMetadata { collection: 1, item: 1 }),
		]
		.iter()
		{
			assert!(Filter::<Runtime>::contains(read))
		}
	}
}

use frame_support::{derive_impl, parameter_types, traits::Everything};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

use xcm_builder::FixedWeightBounds;

// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

pub use codec::{Decode, Encode};
use frame_support::traits::ContainsPair;
pub use frame_support::{
	dispatch::{DispatchInfo, DispatchResultWithPostInfo, GetDispatchInfo, PostDispatchInfo},
	ensure,
	sp_runtime::{traits::Dispatchable, DispatchError, DispatchErrorWithPostInfo},
	traits::{Contains, Get, IsInVec},
};
pub use sp_std::{
	cell::{Cell, RefCell},
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	fmt::Debug,
};
pub use xcm::latest::{prelude::*, QueryId, Weight};
use xcm_executor::traits::{Properties, QueryHandler, QueryResponseStatus};
pub use xcm_executor::{
	traits::{
		AssetExchange, AssetLock, CheckSuspension, ConvertOrigin, Enact, ExportXcm, FeeManager,
		FeeReason, LockError, OnResponse, TransactAsset,
	},
	AssetsInHolding, Config,
};

#[derive(Debug)]
pub enum ResponseSlot {
	Expecting(Location),
	Received(Response),
}
thread_local! {
	pub static QUERIES: RefCell<BTreeMap<u64, ResponseSlot>> = RefCell::new(BTreeMap::new());
}
pub struct TestResponseHandler;
impl OnResponse for TestResponseHandler {
	fn expecting_response(origin: &Location, query_id: u64, _querier: Option<&Location>) -> bool {
		QUERIES.with(|q| match q.borrow().get(&query_id) {
			Some(ResponseSlot::Expecting(ref l)) => l == origin,
			_ => false,
		})
	}
	fn on_response(
		_origin: &Location,
		query_id: u64,
		_querier: Option<&Location>,
		response: xcm::latest::Response,
		_max_weight: Weight,
		_context: &XcmContext,
	) -> Weight {
		QUERIES.with(|q| {
			q.borrow_mut().entry(query_id).and_modify(|v| {
				if matches!(*v, ResponseSlot::Expecting(..)) {
					*v = ResponseSlot::Received(response);
				}
			});
		});
		Weight::from_parts(10, 10)
	}
}
pub fn expect_response(query_id: u64, from: Location) {
	QUERIES.with(|q| q.borrow_mut().insert(query_id, ResponseSlot::Expecting(from)));
}
pub fn response(query_id: u64) -> Option<Response> {
	QUERIES.with(|q| {
		q.borrow().get(&query_id).and_then(|v| match v {
			ResponseSlot::Received(r) => Some(r.clone()),
			_ => None,
		})
	})
}

/// Mock implementation of the [`QueryHandler`] trait for creating XCM success queries and expecting
/// responses.
#[derive(Debug)]
pub struct TestQueryHandler<T, BlockNumber>(core::marker::PhantomData<(T, BlockNumber)>);
impl<T: Config, BlockNumber: sp_runtime::traits::Zero + Encode> QueryHandler
	for TestQueryHandler<T, BlockNumber>
{
	type QueryId = xcm::latest::QueryId;
	type BlockNumber = BlockNumber;
	type Error = XcmError;
	type UniversalLocation = T::UniversalLocation;

	fn new_query(
		responder: impl Into<Location>,
		_timeout: Self::BlockNumber,
		_match_querier: impl Into<Location>,
	) -> QueryId {
		let query_id = 1;
		expect_response(query_id, responder.into());
		query_id
	}

	fn report_outcome(
		message: &mut Xcm<()>,
		responder: impl Into<Location>,
		timeout: Self::BlockNumber,
	) -> Result<QueryId, Self::Error> {
		let responder = responder.into();
		let destination = Self::UniversalLocation::get()
			.invert_target(&responder)
			.map_err(|()| XcmError::LocationNotInvertible)?;
		let query_id = Self::new_query(responder, timeout, Here);
		let response_info = QueryResponseInfo { destination, query_id, max_weight: Weight::zero() };
		let report_error = Xcm(vec![ReportError(response_info)]);
		message.0.insert(0, SetAppendix(report_error));
		Ok(query_id)
	}

	fn take_response(query_id: QueryId) -> QueryResponseStatus<Self::BlockNumber> {
		// TODO: need to remove entry if Ready.
		QUERIES
			.with(|q| {
				let mut queries = q.borrow_mut();
				queries.get(&query_id).and_then(|v| match v {
					ResponseSlot::Received(r) => Some(QueryResponseStatus::Ready {
						response: r.clone(),
						at: Self::BlockNumber::zero(),
					}),
					_ => Some(QueryResponseStatus::NotFound),
				})
			})
			.unwrap_or(QueryResponseStatus::NotFound)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn expect_response(_id: QueryId, _response: xcm::latest::Response) {
		// Unnecessary since it's only a test implementation
	}
}
/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = ();

pub type XcmRouter = ();

parameter_types! {
	pub UniversalLocation: InteriorLocation = [Parachain(1u32)].into();
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000, 1024);
	pub const MaxInstructions: u32 = 100;
}
// pub struct XcmConfig;
impl xcm_executor::Config for Test {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = ();
	type IsReserve = ();
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;
	type Barrier = ();
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader = ();
	type ResponseHandler = ();
	type AssetTrap = ();
	type AssetClaims = ();
	type SubscriptionService = ();
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = ();
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = ();
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
	type Aliasers = ();
	type TransactionalProcessor = ();
}

type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Async: crate::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl crate::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type XcmQueryHandler = TestQueryHandler<Test, u64>;
	type WeightInfo = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

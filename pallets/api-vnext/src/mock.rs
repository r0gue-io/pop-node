use codec::Compact;
use frame_support::{
	assert_ok, derive_impl,
	pallet_prelude::ConstU32,
	parameter_types,
	sp_runtime::{traits::AccountIdLookup, AccountId32, BuildStorage},
	traits::{AsEnsureOriginWithArg, Get, OnInitialize},
	PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_assets::AutoIncAssetId;
pub(crate) use pallet_revive::test_utils::{ALICE, ALICE_ADDR, BOB, BOB_ADDR, CHARLIE};

use super::fungibles;

pub(crate) type AccountId = AccountId32;
pub(crate) type Balance = u128;
type Block = frame_system::mocking::MockBlockU32<Test>;
pub(crate) type ExistentialDeposit = <Test as pallet_balances::Config>::ExistentialDeposit;
// For terminology in tests.
pub(crate) type TokenId = u32;

pub(crate) const ERC20: u16 = 2;
pub(crate) const FUNGIBLES: u16 = 1;
#[cfg(feature = "messaging")]
pub(crate) const ISMP: u16 = 4;
#[cfg(feature = "messaging")]
pub(crate) const MESSAGING: u16 = 3;
pub(crate) const UNIT: Balance = 10_000_000_000;
#[cfg(feature = "messaging")]
pub(crate) const XCM: u16 = 5;

#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask,
		RuntimeViewFunction
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type Assets = pallet_assets::Pallet<Runtime>;
	#[runtime::pallet_index(1)]
	pub type Balances = pallet_balances::Pallet<Runtime>;
	#[runtime::pallet_index(2)]
	pub type Contracts = pallet_revive::Pallet<Runtime>;
	#[runtime::pallet_index(3)]
	pub type Fungibles = fungibles::Pallet<Runtime>;
	#[runtime::pallet_index(4)]
	#[runtime::disable_unsigned]
	#[cfg(feature = "messaging")]
	pub type Ismp = pallet_ismp::Pallet<Runtime>;
	#[runtime::pallet_index(5)]
	#[cfg(feature = "messaging")]
	pub type Messaging = crate::messaging::Pallet<Runtime>;
	#[runtime::pallet_index(6)]
	pub type ParachainInfo = parachain_info::Pallet<Runtime>;
	#[runtime::pallet_index(7)]
	pub type System = frame_system::Pallet<Runtime>;
	#[runtime::pallet_index(8)]
	pub type Timestamp = pallet_timestamp::Pallet<Runtime>;
	#[runtime::pallet_index(9)]
	#[cfg(feature = "messaging")]
	pub type Xcm = pallet_xcm::Pallet<Runtime>;
}

#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig)]
impl pallet_assets::Config for Test {
	type AssetIdParameter = Compact<TokenId>;
	type Balance = Balance;
	type CallbackHandle = AutoIncAssetId<Test>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<AccountId>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = Balance;
}

#[derive_impl(pallet_revive::config_preludes::TestDefaultConfig)]
impl pallet_revive::Config for Test {
	type AddressMapper = pallet_revive::AccountId32Mapper<Self>;
	type Currency = Balances;
	type InstantiateOrigin = EnsureSigned<Self::AccountId>;
	type Precompiles = (
		fungibles::precompiles::v0::Fungibles<FUNGIBLES, Test>,
		fungibles::precompiles::erc20::v0::Erc20<ERC20, Test>,
		messaging::precompiles::v0::Messaging<MESSAGING, Test>,
		messaging::precompiles::ismp::v0::Ismp<ISMP, Test>,
		messaging::precompiles::xcm::v0::Xcm<XCM, Test>,
	);
	type Time = Timestamp;
	type UploadOrigin = EnsureSigned<Self::AccountId>;
}

impl fungibles::Config for Test {
	type WeightInfo = ();
}

#[cfg(feature = "messaging")]
pub(super) mod messaging {
	use ::ismp::{host::StateMachine, module::IsmpModule, router::IsmpRouter};
	use ::xcm::latest::{
		InteriorLocation, Junction, Junction::Parachain, Junctions, Location, NetworkId,
	};
	use frame_support::{
		dispatch::PostDispatchInfo,
		pallet_prelude::{DispatchResultWithPostInfo, EnsureOrigin, Pays, Weight},
		traits::{tokens::imbalance::ResolveTo, Everything, NeverEnsureOrigin, OriginTrait},
		weights::WeightToFee,
	};
	use frame_system::EnsureRoot;
	use sp_runtime::traits::{AccountIdConversion, TryConvert};
	use xcm_builder::{EnsureXcmOrigin, FixedWeightBounds, SignedToAccountId32};

	use super::*;
	pub(super) use crate::messaging::*;
	use crate::{messaging::transports::xcm::NotifyQueryHandler, H160};

	pub(crate) const RESPONSE_LOCATION: Location =
		Location { parents: 1, interior: Junctions::Here };

	parameter_types! {
		pub const MaxInstructions: u32 = 100;
		pub const MaxXcmQueryTimeoutsPerBlock: u32 = 10;
		pub const OnChainByteFee: Balance = 10;
		pub const OffChainByteFee: Balance = 5;
		pub const RelayNetwork: Option<NetworkId> = Some(NetworkId::Polkadot);
		pub Treasury: AccountId = PalletId(*b"py/trsry").into_account_truncating();
		pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
		pub UniversalLocation: InteriorLocation = Parachain(2_000).into();
	}

	impl pallet_ismp::Config for Test {
		type AdminOrigin = EnsureRoot<AccountId>;
		type Balance = Balance;
		type ConsensusClients = ();
		type Coprocessor = Coprocessor;
		type Currency = Balances;
		type HostStateMachine = HostStateMachine;
		type OffchainDB = ();
		type Router = AlwaysErrorRouter;
		type RuntimeEvent = RuntimeEvent;
		type TimestampProvider = Timestamp;
		type WeightProvider = ();
	}

	#[derive(Default)]
	pub struct AlwaysErrorRouter;
	impl IsmpRouter for AlwaysErrorRouter {
		fn module_for_id(&self, _bytes: Vec<u8>) -> Result<Box<dyn IsmpModule>, anyhow::Error> {
			Err(anyhow::anyhow!("not implemented"))
		}
	}

	pub struct Coprocessor;
	impl Get<Option<StateMachine>> for Coprocessor {
		fn get() -> Option<StateMachine> {
			Some(HostStateMachine::get())
		}
	}

	pub struct HostStateMachine;
	impl Get<StateMachine> for HostStateMachine {
		fn get() -> StateMachine {
			StateMachine::Polkadot(2000)
		}
	}

	impl crate::messaging::Config for Test {
		type CallbackExecutor = AlwaysSuccessfullCallbackExecutor<Test>;
		type FeeHandler = ResolveTo<Treasury, Balances>;
		type Fungibles = Balances;
		type IsmpDispatcher = pallet_ismp::Pallet<Test>;
		type Keccak256 = Ismp;
		type MaxContextLen = ConstU32<64>;
		type MaxDataLen = ConstU32<1024>;
		type MaxKeyLen = ConstU32<32>;
		type MaxKeys = ConstU32<10>;
		type MaxRemovals = ConstU32<1024>;
		type MaxResponseLen = ConstU32<1024>;
		type MaxXcmQueryTimeoutsPerBlock = MaxXcmQueryTimeoutsPerBlock;
		type OffChainByteFee = OffChainByteFee;
		type OnChainByteFee = OnChainByteFee;
		type OriginConverter = AccountToLocation;
		type RuntimeHoldReason = RuntimeHoldReason;
		type WeightInfo = ();
		type WeightToFee = RefTimePlusProofTime;
		type Xcm = QueryHandler;
		type XcmResponseOrigin = EnsureRootWithResponseSuccess;
	}

	pub struct AccountToLocation;
	impl TryConvert<RuntimeOrigin, Location> for AccountToLocation {
		fn try_convert(origin: RuntimeOrigin) -> Result<Location, RuntimeOrigin> {
			let signer = origin.into_signer();
			let l = Junctions::from(Junction::AccountId32 {
				network: None,
				id: signer.expect("No account id, required.").into(),
			})
			.into_location();
			Ok(l)
		}
	}

	/// Will return half of the weight in the post info.
	/// Mocking a successfull execution, with refund.
	pub struct AlwaysSuccessfullCallbackExecutor<T>(T);
	impl<T: crate::messaging::Config> CallbackExecutor<T> for AlwaysSuccessfullCallbackExecutor<T> {
		fn execute(
			_account: &<T as frame_system::Config>::AccountId,
			_contract: H160,
			_data: Vec<u8>,
			gas_limit: sp_runtime::Weight,
			_storage_deposit_limit: BalanceOf<T>,
		) -> frame_support::dispatch::DispatchResultWithPostInfo {
			DispatchResultWithPostInfo::Ok(PostDispatchInfo {
				actual_weight: Some(gas_limit / 2),
				pays_fee: Pays::Yes,
			})
		}

		// Will be used for prepayment of response fees.
		fn execution_weight() -> Weight {
			Weight::from_parts(100_000u64, 100_000u64)
		}
	}

	pub struct EnsureRootWithResponseSuccess;
	impl EnsureOrigin<RuntimeOrigin> for EnsureRootWithResponseSuccess {
		type Success = Location;

		fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
			if EnsureRoot::<AccountId32>::ensure_origin(o.clone()).is_ok() {
				Ok(RESPONSE_LOCATION)
			} else {
				Err(o)
			}
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
			Ok(RuntimeOrigin::root())
		}
	}

	pub struct QueryHandler;
	impl NotifyQueryHandler<Test> for QueryHandler {
		type WeightInfo = pallet_xcm::Pallet<Test>;

		fn new_notify_query(
			responder: impl Into<Location>,
			notify: messaging::Call<Test>,
			timeout: u32,
			match_querier: impl Into<Location>,
		) -> u64 {
			Xcm::new_notify_query(responder, notify, timeout, match_querier)
		}
	}

	pub struct RefTimePlusProofTime;
	impl WeightToFee for RefTimePlusProofTime {
		type Balance = Balance;

		fn weight_to_fee(weight: &Weight) -> Self::Balance {
			(weight.ref_time() + weight.proof_size()) as u128
		}
	}

	type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

	impl pallet_xcm::Config for Test {
		type AdminOrigin = NeverEnsureOrigin<()>;
		type AdvertisedXcmVersion = ();
		type AuthorizedAliasConsideration = ();
		type Currency = Balances;
		type CurrencyMatcher = ();
		type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
		type MaxLockers = ();
		type MaxRemoteLockConsumers = ();
		type RemoteLockConsumerIdentifier = ();
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
		type SovereignAccountOf = ();
		type TrustedLockers = ();
		type UniversalLocation = UniversalLocation;
		type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
		type WeightInfo = pallet_xcm::TestWeightInfo;
		type XcmExecuteFilter = Everything;
		type XcmExecutor = ();
		type XcmReserveTransferFilter = ();
		type XcmRouter = ();
		type XcmTeleportFilter = ();

		const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 0;
	}
}

impl parachain_info::Config for Test {}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type Block = Block;
	type Lookup = AccountIdLookup<Self::AccountId, ()>;
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig)]
impl pallet_timestamp::Config for Test {}

pub(crate) struct ExtBuilder {
	assets: Option<Vec<(TokenId, AccountId, bool, Balance)>>,
	asset_accounts: Option<Vec<(TokenId, AccountId, Balance)>>,
	asset_metadata: Option<Vec<(TokenId, Vec<u8>, Vec<u8>, u8)>>,
	balances: Vec<(AccountId, Balance)>,
	#[cfg(feature = "messaging")]
	messages: Option<Vec<(AccountId, messaging::MessageId, messaging::Message<Test>, Balance)>>,
	#[cfg(feature = "messaging")]
	next_message_id: Option<messaging::MessageId>,
	#[cfg(feature = "messaging")]
	query_id: Option<xcm::latest::QueryId>,
}
impl ExtBuilder {
	pub(crate) fn new() -> Self {
		Self {
			assets: None,
			asset_accounts: None,
			asset_metadata: None,
			balances: vec![(ALICE, ExistentialDeposit::get())],
			#[cfg(feature = "messaging")]
			messages: None,
			#[cfg(feature = "messaging")]
			next_message_id: None,
			#[cfg(feature = "messaging")]
			query_id: None,
		}
	}

	pub(crate) fn with_assets(mut self, assets: Vec<(TokenId, AccountId, bool, Balance)>) -> Self {
		self.assets = Some(assets);
		self
	}

	pub(crate) fn with_asset_balances(
		mut self,
		accounts: Vec<(TokenId, AccountId, Balance)>,
	) -> Self {
		self.asset_accounts = Some(accounts);
		self
	}

	pub(crate) fn with_asset_metadata(
		mut self,
		metadata: Vec<(TokenId, Vec<u8>, Vec<u8>, u8)>,
	) -> Self {
		self.asset_metadata = Some(metadata);
		self
	}

	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	#[cfg(feature = "messaging")]
	pub(crate) fn with_messages(
		mut self,
		messages: Vec<(AccountId, messaging::MessageId, messaging::Message<Test>, Balance)>,
	) -> Self {
		self.messages = Some(messages);
		self
	}

	#[cfg(feature = "messaging")]
	pub(crate) fn with_message_id(mut self, id: messaging::MessageId) -> Self {
		self.next_message_id = Some(id);
		self
	}

	#[cfg(feature = "messaging")]
	pub(crate) fn with_query_id(mut self, query_id: ::xcm::latest::QueryId) -> Self {
		self.query_id = Some(query_id);
		self
	}

	pub(crate) fn build(mut self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: self.balances, ..Default::default() }
			.assimilate_storage(&mut t)
			.unwrap();

		pallet_assets::GenesisConfig::<Test> {
			assets: self.assets.take().unwrap_or_default(),
			metadata: self.asset_metadata.take().unwrap_or_default(),
			accounts: self.asset_accounts.take().unwrap_or_default(),
			next_asset_id: Some(0),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);

			#[cfg(feature = "messaging")]
			{
				if let Some(messages) = self.messages.take() {
					for (account, id, message, deposit) in messages {
						messaging::Messages::<Test>::insert(id, message);

						use frame_support::traits::fungible::MutateHold;
						use messaging::HoldReason;
						assert_ok!(<Test as messaging::Config>::Fungibles::hold(
							&HoldReason::Messaging.into(),
							&account,
							deposit
						));
					}
				}

				if let Some(id) = self.next_message_id.take() {
					crate::messaging::NextMessageId::<Test>::set(id);
				}

				if let Some(query_id) = self.query_id.take() {
					use frame_support::pallet_prelude::ValueQuery;
					use xcm::latest::QueryId;

					#[frame_support::storage_alias]
					type QueryCounter = StorageValue<Xcm, QueryId, ValueQuery>;

					QueryCounter::set(query_id);
				}
			}
		});
		ext
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub(crate) fn build_with_env<R>(
		self,
		execute: impl FnOnce(pallet_revive::precompiles::run::CallSetup<Test>) -> R,
	) -> R {
		self.build().execute_with(|| {
			let call_setup = pallet_revive::precompiles::run::CallSetup::<Test>::default();
			// let (ext, _) = call_setup.ext();
			execute(call_setup)
		})
	}
}

pub(crate) fn next_block() {
	let next_block: u32 = System::block_number() + 1;
	System::set_block_number(next_block);
	Messaging::on_initialize(next_block);
}

pub(crate) fn none() -> RuntimeOrigin {
	RuntimeOrigin::none()
}

pub(crate) fn run_to(block_number: u32) {
	while System::block_number() < block_number {
		next_block();
	}
}
pub(crate) fn root() -> RuntimeOrigin {
	RuntimeOrigin::root()
}
pub(crate) fn signed(account: AccountId) -> RuntimeOrigin {
	RuntimeOrigin::signed(account)
}

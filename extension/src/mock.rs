use crate::decoding::IdentityProcessor;
use crate::{
	environment, matching::WithFuncId, Decodes, DecodingFailed, DefaultConverter, DispatchCall,
	Extension, Function, Matches, Processor, ReadState, Readable,
};
use codec::{Decode, Encode};
use frame_support::traits::fungible::Inspect;
use frame_support::{
	derive_impl,
	pallet_prelude::Weight,
	parameter_types,
	traits::{ConstU32, Everything, Nothing},
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_contracts::{chain_extension::RetVal, DefaultAddressGenerator, Frame, Schedule};
use sp_runtime::BuildStorage;
use sp_runtime::Perbill;
use std::marker::PhantomData;

pub(crate) type AccountId = <Test as frame_system::Config>::AccountId;
pub(crate) type Balance = <<Test as pallet_contracts::Config>::Currency as Inspect<
	<Test as frame_system::Config>::AccountId,
>>::Balance;
pub(crate) type EventRecord = frame_system::EventRecord<
	<Test as frame_system::Config>::RuntimeEvent,
	<Test as frame_system::Config>::Hash,
>;

pub(crate) const ALICE: u64 = 1;
pub(crate) const DEBUG_OUTPUT: pallet_contracts::DebugInfo =
	pallet_contracts::DebugInfo::UnsafeDebug;
pub(crate) const GAS_LIMIT: Weight = Weight::from_parts(500_000_000_000, 3 * 1024 * 1024);
pub(crate) const INIT_AMOUNT: <Test as pallet_balances::Config>::Balance = 100_000_000;
pub(crate) const INVALID_FUNC_ID: u32 = 0;

type DispatchCallWith<Id, Filter, Processor> = DispatchCall<
	// Registered with func id 1
	WithFuncId<Id>,
	// Runtime config
	Test,
	// Decode inputs to the function as runtime calls
	Decodes<RuntimeCall, DecodingFailed<Test>, Processor>,
	// Accept any filterting
	Filter,
>;

type ReadStateWith<Id, Filter, Processor> = ReadState<
	// Registered with func id 1
	WithFuncId<Id>,
	// Runtime config
	Test,
	// The runtime state reads available.
	RuntimeRead,
	// Decode inputs to the function as runtime calls
	Decodes<RuntimeRead, DecodingFailed<Test>, Processor>,
	// Accept any filtering
	Filter,
	// Convert the result of a read into the expected result
	DefaultConverter<RuntimeResult>,
>;

frame_support::construct_runtime!(
	pub enum Test {
		System: frame_system,
		Balances: pallet_balances,
		Timestamp: pallet_timestamp,
		Contracts: pallet_contracts,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = u64;
	type AccountData = pallet_balances::AccountData<u64>;
	type Block = frame_system::mocking::MockBlock<Test>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig as pallet_balances::DefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type ReserveIdentifier = [u8; 8];
}

#[derive_impl(pallet_timestamp::config_preludes::TestDefaultConfig as pallet_timestamp::DefaultConfig)]
impl pallet_timestamp::Config for Test {}

impl pallet_contracts::Config for Test {
	type Time = Timestamp;
	type Randomness = Test;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type CallFilter = (); //TestFilter;
	type CallStack = [Frame<Self>; 5];
	type WeightPrice = (); //Self;
	type WeightInfo = ();
	type ChainExtension = Extension<Config>;
	type Schedule = MySchedule;
	type DepositPerByte = DepositPerByte;
	type DepositPerItem = DepositPerItem;
	type DefaultDepositLimit = DefaultDepositLimit;
	type AddressGenerator = DefaultAddressGenerator;
	type MaxCodeLen = ConstU32<{ 100 * 1024 }>;
	type MaxStorageKeyLen = ConstU32<128>;
	type UnsafeUnstableInterface = (); //UnstableInterface;
	type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Migrations = (); //crate::migration::codegen::BenchMigrations;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type MaxDelegateDependencies = MaxDelegateDependencies;
	type Debug = (); //TestDebug;
	type Environment = ();
	type Xcm = ();
}

parameter_types! {
	pub MySchedule: Schedule<Test> = {
		let schedule = <Schedule<Test>>::default();
		schedule
	};
	pub static DepositPerByte: <Test as pallet_balances::Config>::Balance = 1;
	pub const DepositPerItem: <Test as pallet_balances::Config>::Balance = 2;
	pub static MaxDelegateDependencies: u32 = 32;
	pub static MaxTransientStorageSize: u32 = 4 * 1024;
	pub static CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
	pub static DefaultDepositLimit: <Test as pallet_balances::Config>::Balance = 10_000_000;
}

impl frame_support::traits::Randomness<<Test as frame_system::Config>::Hash, BlockNumberFor<Test>>
	for Test
{
	fn random(_subject: &[u8]) -> (<Test as frame_system::Config>::Hash, BlockNumberFor<Test>) {
		(Default::default(), Default::default())
	}
}

parameter_types! {
	// IDs for functions for extension tests.
	pub const DispatchExtFuncId : u32 = 1;
	pub const ReadExtFuncId : u32 = 2;
	// IDs for function for extension tests but do nothing.
	pub const DispatchExtNoopFuncId : u32 = 3;
	pub const ReadExtNoopFuncId : u32 = 4;
	// IDs for functions for contract tests.
	pub const DispatchContractFuncId : u32 = 5;
	pub const ReadContractFuncId : u32 = 6;
	// IDs for function for contract tests but do nothing.
	pub const DispatchContractNoopFuncId : u32 = 7;
	pub const ReadContractNoopFuncId : u32 = 8;
	// ID for function that does nothing
	pub const NoopFuncId : u32 = u32::MAX;
}

/// A query of mock runtime state.
#[derive(Encode, Decode, Debug)]
#[repr(u8)]
pub enum RuntimeRead {
	#[codec(index = 1)]
	Ping,
}
impl Readable for RuntimeRead {
	/// The corresponding type carrying the result of the query for mock runtime state.
	type Result = RuntimeResult;

	/// Determines the weight of the read, used to charge the appropriate weight before the read is performed.
	fn weight(&self) -> Weight {
		match self {
			RuntimeRead::Ping => Weight::from_parts(1_000u64, 1u64),
		}
	}

	/// Performs the read and returns the result.
	fn read(self) -> Self::Result {
		match self {
			RuntimeRead::Ping => RuntimeResult::Pong("pop".to_string()),
		}
	}
}

/// The result of a mock runtime state read.
#[derive(Debug, Decode, Encode)]
pub enum RuntimeResult {
	#[codec(index = 1)]
	Pong(String),
}

impl Into<Vec<u8>> for RuntimeResult {
	fn into(self) -> Vec<u8> {
		match self {
			RuntimeResult::Pong(value) => value.encode(),
		}
	}
}

pub(crate) type Functions = (
	// Functions that allow everything for extension testing.
	DispatchCallWith<DispatchExtFuncId, Everything, IdentityProcessor>,
	ReadStateWith<ReadExtFuncId, Everything, IdentityProcessor>,
	// Functions that allow nothing for extension testing.
	DispatchCallWith<DispatchExtNoopFuncId, Nothing, IdentityProcessor>,
	ReadStateWith<ReadExtNoopFuncId, Nothing, IdentityProcessor>,
	// Functions that allow everything for contract testing.
	DispatchCallWith<DispatchContractFuncId, Everything, RemoveFirstByte>,
	ReadStateWith<ReadContractFuncId, Everything, RemoveFirstByte>,
	// Functions that allow nothing for contract testing.
	DispatchCallWith<DispatchContractNoopFuncId, Nothing, RemoveFirstByte>,
	ReadStateWith<ReadContractNoopFuncId, Nothing, RemoveFirstByte>,
	// Function that does nothing.
	Noop<WithFuncId<NoopFuncId>, Test>,
);

#[derive(Default)]
pub struct Config;
impl super::Config for Config {
	type Functions = Functions;
	const LOG_TARGET: &'static str = "pop-chain-extension";
}

// Removes first bytes of the encoded call, added by the chain extension call within the proxy contract.
pub struct RemoveFirstByte;
impl Processor for RemoveFirstByte {
	type Value = Vec<u8>;
	const LOG_TARGET: &'static str = "";

	fn process(mut value: Self::Value, _env: &impl crate::Environment) -> Self::Value {
		if !value.is_empty() {
			value.remove(0);
		}
		value
	}
}

// A function that does nothing.
pub struct Noop<M, C>(PhantomData<(M, C)>);
impl<Matcher: Matches, Config: pallet_contracts::Config> Function for Noop<Matcher, Config> {
	type Config = Config;
	type Error = ();

	fn execute(
		_env: &mut (impl environment::Environment<Config = Config> + crate::BufIn),
	) -> pallet_contracts::chain_extension::Result<RetVal> {
		Ok(RetVal::Converging(0))
	}
}
impl<M: Matches, C> Matches for Noop<M, C> {
	fn matches(env: &impl crate::Environment) -> bool {
		M::matches(env)
	}
}

/// Helper method to construct the mock environment.
pub(crate) fn mock_environment(id: u32, buffer: Vec<u8>) -> MockEnvironment<MockExt> {
	MockEnvironment::new(id, buffer, MockExt::default())
}

/// A mocked chain extension environment.
pub(crate) struct MockEnvironment<E> {
	func_id: u16,
	ext_id: u16,
	charged: Vec<Weight>,
	pub(crate) buffer: Vec<u8>,
	ext: E,
}

impl Default for MockEnvironment<MockExt> {
	fn default() -> Self {
		mock_environment(0, [].to_vec())
	}
}

impl<E> MockEnvironment<E> {
	pub(crate) fn new(id: u32, buffer: Vec<u8>, ext: E) -> Self {
		Self {
			func_id: (id & 0x0000FFFF) as u16,
			ext_id: (id >> 16) as u16,
			charged: Vec::new(),
			buffer,
			ext,
		}
	}

	pub(crate) fn charged(&self) -> Weight {
		self.charged.iter().fold(Weight::zero(), |acc, b| acc.saturating_add(*b))
	}
}

impl<E: environment::Ext<Config = Test> + Clone> environment::Environment for MockEnvironment<E> {
	type Config = Test;
	type ChargedAmount = Weight;

	fn func_id(&self) -> u16 {
		self.func_id
	}

	fn ext_id(&self) -> u16 {
		self.ext_id
	}

	fn charge_weight(
		&mut self,
		amount: Weight,
	) -> pallet_contracts::chain_extension::Result<Self::ChargedAmount> {
		self.charged.push(amount);
		Ok(amount)
	}

	fn adjust_weight(&mut self, charged: Self::ChargedAmount, actual_weight: Weight) {
		let last = self
			.charged
			.iter()
			.enumerate()
			.filter_map(|(i, c)| (c == &charged).then_some(i))
			.last()
			.unwrap();
		self.charged.remove(last);
		self.charged.insert(last, actual_weight)
	}

	fn ext(&mut self) -> impl environment::Ext<Config = Self::Config> {
		self.ext.clone()
	}
}

impl<E> environment::BufIn for MockEnvironment<E> {
	fn in_len(&self) -> u32 {
		self.buffer.len() as u32
	}

	fn read(&self, _max_len: u32) -> pallet_contracts::chain_extension::Result<Vec<u8>> {
		// TODO: handle max_len
		Ok(self.buffer.clone())
	}
}

impl<E> environment::BufOut for MockEnvironment<E> {
	fn write(
		&mut self,
		buffer: &[u8],
		_allow_skip: bool,
		_weight_per_byte: Option<Weight>,
	) -> pallet_contracts::chain_extension::Result<()> {
		// TODO handle write logic
		self.buffer = buffer.to_vec();
		Ok(())
	}
}

/// A mocked smart contract environment.
#[derive(Clone, Default)]
pub(crate) struct MockExt {
	pub(crate) address: <Test as frame_system::Config>::AccountId,
}
impl environment::Ext for MockExt {
	type Config = Test;

	fn address(&self) -> &<Self::Config as frame_system::Config>::AccountId {
		&self.address
	}
}

/// Test externalities.
pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
	let _ = env_logger::try_init();

	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Test> { balances: vec![(ALICE, INIT_AMOUNT)] }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

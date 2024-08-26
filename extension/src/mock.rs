use crate::environment::*;
use crate::matching::WithFuncId;
use crate::{Decodes, DecodingFailed, DispatchCall, Extension, Function, Matches, Processor};
use frame_support::weights::Weight;
use frame_support::{derive_impl, parameter_types, traits::ConstU32, traits::Everything};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_contracts::chain_extension::RetVal;
use pallet_contracts::{DefaultAddressGenerator, Frame, Schedule};
use sp_runtime::Perbill;
use std::marker::PhantomData;

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
	pub const DispatchCallFuncId : u32 = 1;
	pub const ReadStateFuncId : u32 = 2;
	pub const NoopFuncId : u32 = u32::MAX;
}

#[derive(Default)]
pub struct Config;
impl super::Config for Config {
	type Functions = (
		DispatchCall<
			// Registered with func id 1
			WithFuncId<DispatchCallFuncId>,
			// Runtime config
			Test,
			// Decode inputs to the function as runtime calls
			Decodes<RuntimeCall, DecodingFailed<Test>, RemoveFirstByte>,
			// Allow everything
			Everything,
		>,
		Noop<WithFuncId<NoopFuncId>, Test>,
	);
	type Error = ();
	const LOG_TARGET: &'static str = "pop-chain-extension";
}

// Removes first bytes of the encoded call, added by the chain extension call within the proxy contract.
pub struct RemoveFirstByte;
impl Processor for RemoveFirstByte {
	type Value = Vec<u8>;
	const LOG_TARGET: &'static str = "";

	fn process(mut value: Self::Value, _env: &impl crate::Environment) -> Self::Value {
		value.remove(0);
		value
	}
}

// A function that does nothing.
pub struct Noop<M, C>(PhantomData<(M, C)>);
impl<Matcher: Matches, Config: pallet_contracts::Config> Function for Noop<Matcher, Config> {
	type Config = Config;

	fn execute(
		_env: &mut (impl Environment<Config = Config> + crate::BufIn),
	) -> pallet_contracts::chain_extension::Result<RetVal> {
		Ok(RetVal::Converging(0))
	}
}
impl<M: Matches, C> Matches for Noop<M, C> {
	fn matches(env: &impl crate::Environment) -> bool {
		M::matches(env)
	}
}

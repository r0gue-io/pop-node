use crate::{
	AccountId, Balance, Balances, Ismp, IsmpParachain, ParachainInfo, Runtime, RuntimeEvent,
	Timestamp,
};
use frame_support::traits::Get;
use frame_system::EnsureRoot;
use ismp::{error::Error, host::StateMachine, module::IsmpModule, router::IsmpRouter};
use ismp_parachain::ParachainConsensusClient;
use pallet_ismp::ModuleId;
use sp_std::prelude::*;

impl pallet_ismp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AdminOrigin = EnsureRoot<AccountId>;
	type TimestampProvider = Timestamp;
	type Balance = Balance;
	type Currency = Balances;
	type HostStateMachine = HostStateMachine;
	type Coprocessor = Coprocessor;
	type Router = Router;
	type ConsensusClients = (ParachainConsensusClient<Runtime, IsmpParachain>,);
	type WeightProvider = ();
	type Mmr = pallet_ismp::NoOpMmrTree<Self>;
}

impl pallet_ismp_demo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type NativeCurrency = Balances;
	type IsmpDispatcher = Ismp;
}

impl ismp_parachain::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IsmpHost = Ismp;
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
		StateMachine::Polkadot(ParachainInfo::get().into())
	}
}

#[derive(Default)]
pub struct Router;
impl IsmpRouter for Router {
	fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, Error> {
		Ok(Box::new(pallet_ismp_demo::IsmpModuleCallback::<Runtime>::default()))
	}
}

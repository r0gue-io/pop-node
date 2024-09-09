use frame_support::traits::Get;
use frame_system::EnsureRoot;
use ismp::{error::Error, host::StateMachine, module::IsmpModule, router::IsmpRouter};
use ismp_parachain::ParachainConsensusClient;
use sp_std::prelude::*;

use crate::{
	AccountId, Balance, Balances, Ismp, IsmpParachain, ParachainInfo, Runtime, RuntimeEvent,
	Timestamp,
};

impl pallet_ismp::Config for Runtime {
	type AdminOrigin = EnsureRoot<AccountId>;
	type Balance = Balance;
	type ConsensusClients = (ParachainConsensusClient<Runtime, IsmpParachain>,);
	type Coprocessor = Coprocessor;
	type Currency = Balances;
	type HostStateMachine = HostStateMachine;
	type Mmr = pallet_ismp::NoOpMmrTree<Self>;
	type Router = Router;
	type RuntimeEvent = RuntimeEvent;
	type TimestampProvider = Timestamp;
	type WeightProvider = ();
}

impl pallet_ismp_demo::Config for Runtime {
	type Balance = Balance;
	type IsmpDispatcher = Ismp;
	type NativeCurrency = Balances;
	type RuntimeEvent = RuntimeEvent;
}

impl ismp_parachain::Config for Runtime {
	type IsmpHost = Ismp;
	type RuntimeEvent = RuntimeEvent;
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
	fn module_for_id(&self, _id: Vec<u8>) -> Result<Box<dyn IsmpModule>, Error> {
		Ok(Box::new(pallet_ismp_demo::IsmpModuleCallback::<Runtime>::default()))
	}
}

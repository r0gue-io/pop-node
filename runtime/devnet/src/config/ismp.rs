use alloc::{boxed::Box, vec::Vec};

use frame_support::traits::Get;
use frame_system::EnsureRoot;
use ismp::{error::Error, host::StateMachine, module::IsmpModule, router::IsmpRouter};
use ismp_parachain::ParachainConsensusClient;

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
	type OffchainDB = ();
	type Router = Router;
	type RuntimeEvent = RuntimeEvent;
	type TimestampProvider = Timestamp;
	type WeightProvider = WeightProvider;
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
	fn module_for_id(&self, id: Vec<u8>) -> Result<Box<dyn IsmpModule>, anyhow::Error> {
		use pallet_api_vnext::messaging::transports::ismp as messaging;
		match id {
			id if id == messaging::ID => Ok(Box::new(messaging::Module::<Runtime>::new())),
			_ => Err(Error::ModuleNotFound(id).into()),
		}
	}
}

pub struct WeightProvider;
impl pallet_ismp::weights::WeightProvider for WeightProvider {
	fn module_callback(
		dest_module: pallet_ismp::ModuleId,
	) -> Option<Box<dyn pallet_ismp::weights::IsmpModuleWeight>> {
		use pallet_api_vnext::messaging::transports::ismp as messaging;
		match dest_module.to_bytes() {
			dest_module if dest_module == messaging::ID =>
				Some(Box::new(messaging::Module::<Runtime>::new())),
			_ => None,
		}
	}
}

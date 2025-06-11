use alloc::{boxed::Box, vec::Vec};

use frame_support::traits::Get;
use frame_system::EnsureRoot;
use ismp::{error::Error, host::StateMachine, module::IsmpModule, router::IsmpRouter};
use ismp_parachain::ParachainConsensusClient;
use sp_runtime::{
	traits::ValidateUnsigned,
	transaction_validity::{
		InvalidTransaction, TransactionSource, TransactionValidity, TransactionValidityError,
	},
};

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
	type WeightProvider = ();
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
		use pallet_api::messaging::transports::ismp::*;
		if id == ID {
			return Ok(Box::new(Handler::<Runtime>::new()));
		}
		Err(Error::ModuleNotFound(id))?
	}
}

// impl ValidateUnsigned for Runtime {
// 	type Call = ();
// 	Dont allow unsigned calls.
// 	fn pre_dispatch(_call: &Self::Call) -> Result<(), TransactionValidityError> {
// 		Err(InvalidTransaction::BadSigner.into())
// 	}
// 	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
// 		Err(InvalidTransaction::BadSigner.into())
// 	}
// }

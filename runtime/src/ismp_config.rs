use crate::{AccountId, Ismp, IsmpParachain, ParachainInfo, Runtime, RuntimeEvent, Timestamp};
use frame_support::traits::Get;
use frame_system::EnsureRoot;
use ismp::{
	error::Error,
	host::StateMachine,
	module::IsmpModule,
	router::{IsmpRouter, Post, Request, Response, Timeout},
};
use ismp_parachain::ParachainConsensusClient;
use pallet_ismp::{dispatcher::FeeMetadata, primitives::ModuleId};
use sp_std::prelude::*;

impl pallet_ismp::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	const INDEXING_PREFIX: &'static [u8] = b"ISMP";
	type AdminOrigin = EnsureRoot<AccountId>;
	type HostStateMachine = HostStateMachine;
	type Coprocessor = Coprocessor;
	type TimeProvider = Timestamp;
	type Router = Router;
	type ConsensusClients = (ParachainConsensusClient<Runtime, IsmpParachain>,);
	type WeightInfo = ();
	type WeightProvider = ();
}

impl ismp_parachain::Config for Runtime {
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
pub struct ProxyModule;
impl IsmpModule for ProxyModule {
	fn on_accept(&self, request: Post) -> Result<(), Error> {
		if request.dest != HostStateMachine::get() {
			let meta = FeeMetadata { origin: [0u8; 32].into(), fee: Default::default() };
			return Ismp::dispatch_request(Request::Post(request), meta);
		}

		let pallet_id = ModuleId::from_bytes(&request.to)
			.map_err(|err| Error::ImplementationSpecific(err.to_string()))?;
		match pallet_id {
			_ => Err(Error::ImplementationSpecific("Destination module not found".to_string())),
		}
	}

	fn on_response(&self, response: Response) -> Result<(), Error> {
		if response.dest_chain() != HostStateMachine::get() {
			let meta = FeeMetadata { origin: [0u8; 32].into(), fee: Default::default() };
			return Ismp::dispatch_response(response, meta);
		}

		let request = &response.request();
		let from = match &request {
			Request::Post(post) => &post.from,
			Request::Get(get) => &get.from,
		};

		let pallet_id = ModuleId::from_bytes(from)
			.map_err(|err| Error::ImplementationSpecific(err.to_string()))?;
		match pallet_id {
			_ => Err(Error::ImplementationSpecific("Destination module not found".to_string())),
		}
	}

	fn on_timeout(&self, timeout: Timeout) -> Result<(), Error> {
		let from = match &timeout {
			Timeout::Request(Request::Post(post)) => &post.from,
			Timeout::Request(Request::Get(get)) => &get.from,
			Timeout::Response(res) => &res.post.to,
		};

		let pallet_id = ModuleId::from_bytes(from)
			.map_err(|err| Error::ImplementationSpecific(err.to_string()))?;
		match pallet_id {
			// instead of returning an error, do nothing. The timeout is for a connected chain.
			_ => Ok(()),
		}
	}
}

#[derive(Default)]
pub struct Router;

impl IsmpRouter for Router {
	fn module_for_id(&self, _bytes: Vec<u8>) -> Result<Box<dyn IsmpModule>, Error> {
		Ok(Box::new(ProxyModule::default()))
	}
}

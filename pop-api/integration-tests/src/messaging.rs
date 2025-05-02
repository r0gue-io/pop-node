use ::ismp::router::{GetResponse, IsmpRouter, PostResponse, Response, StorageValue};
use ismp::host::StateMachine;
use pallet_api::messaging::Event::*;
use pallet_ismp::offchain::Leaf;
use pop_api::{
	messaging::{
		ismp::{Get, Post},
		xcm::{self, Junction, Location, MaybeErrorCode, NetworkId, QueryId},
		MessageId, MessageStatus,
	},
	primitives::{BlockNumber, Error},
};

use pop_runtime_testnet::{config::ismp::Router, Messaging, RuntimeEvent};
use sp_io::{hashing::keccak_256, TestExternalities};
use sp_runtime::{offchain::OffchainOverlayedChange, testing::H256};
use xcm_executor::traits::OnResponse;

use super::*;

const CONTRACT: &str = "contracts/messaging/target/ink/messaging.wasm";
const ASSET_HUB: u32 = 1_000;
const HYPERBRIDGE: u32 = 4_009;
const ISMP_MODULE_ID: [u8; 3] = *b"pop";

#[test]
fn ismp_get_request_works() {
	let id = [0u8; 32];
	let key = "some_key".as_bytes().to_vec();
	let timeout = 100_000u64;
	let height = 0u32;
	let request = Get::new(ASSET_HUB, height, timeout, "some_context".as_bytes().to_vec(), vec![key.clone()]);
	let response = vec![StorageValue { key, value: Some(b"some_value".to_vec()) }];

	// Create a get request.
	let mut ext = new_test_ext();
	let contract = ext.execute_with(|| {
        let contract = Contract::new();

        assert_ok!(contract.ismp_get(id, request, 0, false));
        assert_eq!(contract.poll(id).unwrap(), Some(MessageStatus::Pending));
		assert!(System::events().iter().any(|e| {
			matches!(&e.event,
			RuntimeEvent::Messaging(IsmpGetDispatched { origin, id: message_id, ..})
				if origin == &contract.id && *message_id == id
			)
		}));
        assert!(System::events().iter().any(|e| {
            matches!(&e.event,
			RuntimeEvent::Messaging(IsmpGetDispatched { origin, id: message_id, ..})
				if origin == &contract.id && *message_id == id
			)
        }));
        assert!(System::events().iter().any(|e| {
            matches!(e.event,
			RuntimeEvent::Ismp(pallet_ismp::Event::Request { dest_chain, source_chain , ..})
				if dest_chain == StateMachine::Polkadot(ASSET_HUB) && source_chain == StateMachine::Polkadot(100)
			)
        }));

        contract
    });

	// Look up the request within offchain state in order to provide a response.
	let ismp::router::Request::Get(get) = get_ismp_request(&mut ext) else { panic!() };

	// Provide a response.
	ext.execute_with(|| {
		let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
		let commitment = H256::from(keccak_256(&ismp::router::Request::Get(get.clone()).encode()));
		module
			.on_response(Response::Get(GetResponse { get, values: response.clone() }))
			.unwrap();
		System::assert_has_event(
			IsmpGetResponseReceived { dest: contract.id.clone(), id, commitment }.into(),
		);

		assert_eq!(contract.poll(id).unwrap(), Some(MessageStatus::Complete));
		assert_eq!(contract.get(id).unwrap(), Some(response.encode()));
		assert_ok!(contract.remove(id));
	});
}

#[test]
fn ismp_get_request_with_callback_works() {
	let id = [1u8; 32];
	let key = "some_key".as_bytes().to_vec();
	let timeout = 100_000u64;
	let height = 0u32;
	let request = Get::new(ASSET_HUB, height, timeout, "some_context".as_bytes().to_vec(), vec![key.clone()]);
	let response = vec![StorageValue { key, value: Some("some_value".as_bytes().to_vec()) }];

	// Create a get request with callback.
	let mut ext = new_test_ext();
	let contract = ext.execute_with(|| {
        let contract = Contract::new();

        assert_ok!(contract.ismp_get(id, request, 0, true));
        assert_eq!(contract.poll(id).unwrap(), Some(MessageStatus::Pending));
        assert!(System::events().iter().any(|e| {
            matches!(&e.event,
			RuntimeEvent::Messaging(IsmpGetDispatched { origin, id: message_id, ..})
				if origin == &contract.id && *message_id == id
			)
        }));
        assert!(System::events().iter().any(|e| {
            matches!(e.event,
			RuntimeEvent::Ismp(pallet_ismp::Event::Request { dest_chain, source_chain , ..})
				if dest_chain == StateMachine::Polkadot(ASSET_HUB) && source_chain == StateMachine::Polkadot(100)
			)
        }));

        contract
    });

	// Look up the request within offchain state in order to provide a response.
	let ismp::router::Request::Get(get) = get_ismp_request(&mut ext) else { panic!() };

	// Provide a response.
	ext.execute_with(|| {
		let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
		let commitment = H256::from(keccak_256(&ismp::router::Request::Get(get.clone()).encode()));
		module
			.on_response(Response::Get(GetResponse { get, values: response.clone() }))
			.unwrap();
		System::assert_has_event(
			IsmpGetResponseReceived { dest: contract.id.clone(), id, commitment }.into(),
		);

		assert_eq!(contract.last_event(), Some(IsmpGetCompleted { id, values: response }.encode()));
		assert_eq!(contract.poll(id).unwrap(), None);
		assert!(System::events().iter().any(|e| {
			matches!(&e.event,
			RuntimeEvent::Messaging(CallbackExecuted { origin, id: message_id, ..})
				if origin == &contract.id && *message_id == id
			)
		}));
		
	});
}

#[test]
fn ismp_post_request_works() {
	let id = [3u8; 32];
	let timeout = 100_000u64;
	let request = Post::new(HYPERBRIDGE, timeout, "some_data".as_bytes().to_vec());
	let response = b"some_value".to_vec();

	// Create a post request.
	let mut ext = new_test_ext();
	let contract = ext.execute_with(|| {
        let contract = Contract::new();

        assert_ok!(contract.ismp_post(id, request, 0, false));
        assert_eq!(contract.poll(id).unwrap(), Some(MessageStatus::Pending));
        assert!(System::events().iter().any(|e| {
            matches!(&e.event,
			RuntimeEvent::Messaging(IsmpPostDispatched { origin, id: message_id, ..})
				if origin == &contract.id && *message_id == id
			)
        }));
        assert!(System::events().iter().any(|e| {
            matches!(e.event,
			RuntimeEvent::Ismp(pallet_ismp::Event::Request { dest_chain, source_chain , ..})
				if dest_chain == StateMachine::Polkadot(HYPERBRIDGE) && source_chain == StateMachine::Polkadot(100)
			)
        }));

        contract
    });

	// Look up the request within offchain state in order to provide a response.
	let ismp::router::Request::Post(post) = get_ismp_request(&mut ext) else { panic!() };

	// Provide a response.
	ext.execute_with(|| {
		let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
		let commitment =
			H256::from(keccak_256(&ismp::router::Request::Post(post.clone()).encode()));
		module
			.on_response(Response::Post(PostResponse {
				post,
				response: response.clone(),
				timeout_timestamp: timeout,
			}))
			.unwrap();
		System::assert_has_event(
			IsmpPostResponseReceived { dest: contract.id.clone(), id, commitment }.into(),
		);

		assert_eq!(contract.poll(id).unwrap(), Some(MessageStatus::Complete));
		assert_eq!(contract.get(id).unwrap(), Some(response.encode()));
		assert_ok!(contract.remove(id));
	});
}

#[test]
fn ismp_post_request_with_callback_works() {
	let id = [4u8; 32];
	let timeout = 100_000u64;
	let request = Post::new(HYPERBRIDGE, timeout, "some_data".as_bytes().to_vec());
	let response = "some_value".as_bytes().to_vec();

	// Create a post request with callback.
	let mut ext = new_test_ext();
	let contract = ext.execute_with(|| {
        let contract = Contract::new();

        assert_ok!(contract.ismp_post(id, request, 0, true));
        assert_eq!(contract.poll(id).unwrap(), Some(MessageStatus::Pending));
        assert!(System::events().iter().any(|e| {
            matches!(&e.event,
			RuntimeEvent::Messaging(IsmpPostDispatched { origin, id: message_id, ..})
				if origin == &contract.id && *message_id == id
			)
        }));
        assert!(System::events().iter().any(|e| {
            matches!(e.event,
			RuntimeEvent::Ismp(pallet_ismp::Event::Request { dest_chain, source_chain , ..})
				if dest_chain == StateMachine::Polkadot(HYPERBRIDGE) && source_chain == StateMachine::Polkadot(100)
			)
        }));

        contract
    });

	// Look up the request within offchain state in order to provide a response.
	let ismp::router::Request::Post(post) = get_ismp_request(&mut ext) else { panic!() };

	// Provide a response.
	ext.execute_with(|| {
		let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
		let commitment =
			H256::from(keccak_256(&ismp::router::Request::Post(post.clone()).encode()));
		module
			.on_response(Response::Post(PostResponse {
				post,
				response: response.clone(),
				timeout_timestamp: timeout,
			}))
			.unwrap();
		System::assert_has_event(
			IsmpPostResponseReceived { dest: contract.id.clone(), id, commitment }.into(),
		);

		assert_eq!(contract.last_event(), Some(IsmpPostCompleted { id, response }.encode()));
		assert_eq!(contract.poll(id).unwrap(), None);
		assert!(System::events().iter().any(|e| {
			matches!(&e.event,
			RuntimeEvent::Messaging(CallbackExecuted { origin, id: message_id, ..})
				if origin == &contract.id && *message_id == id
			)
		}));
	});
}

#[test]
fn xcm_query_works() {
	let id: [u8; 32] = [5u8; 32];
	let origin = Location::new(1, [Junction::Parachain(ASSET_HUB)]);
	let responder = origin.clone();
	let timeout = 100;
	let response = xcm::Response::DispatchResult(MaybeErrorCode::Success);
	new_test_ext().execute_with(|| {
		let contract = Contract::new();

		// Create a new query and check its status
		let query_id = contract.xcm_new_query(id, responder, timeout, false).unwrap().unwrap();
		assert_eq!(query_id, 0);
		assert_eq!(contract.poll(id).unwrap(), Some(MessageStatus::Pending));
		assert!(System::events().iter().any(|e| {
			matches!(&e.event,
			RuntimeEvent::Messaging(XcmQueryCreated { origin, id: message_id, query_id, ..})
				if origin == &contract.id && *message_id == id && *query_id == 0
			)
		}));

		// Provide a response.
		let origin = translate(&origin);
		let querier: Location = Junction::AccountId32 {
			network: Some(NetworkId::Polkadot),
			id: contract.id.clone().into(),
		}
		.into();
		assert!(pop_runtime_testnet::PolkadotXcm::expecting_response(
			&origin,
			query_id,
			Some(&translate(&querier))
		));
		assert_ok!(Messaging::xcm_response(
			pallet_xcm::Origin::Response(origin).into(),
			query_id,
			translate(&response)
		));

		assert_eq!(contract.poll(id).unwrap(), Some(MessageStatus::Complete));
		assert_eq!(contract.get(id).unwrap(), Some(response.encode()));
		assert_ok!(contract.remove(id));
	});
}

#[test]
fn xcm_query_with_callback_works() {
	let id = [6u8; 32];
	let origin = Location::new(1, [Junction::Parachain(ASSET_HUB)]);
	let responder = origin.clone();
	let timeout = 100;
	let response = xcm::Response::DispatchResult(MaybeErrorCode::Success);
	new_test_ext().execute_with(|| {
		let contract = Contract::new();

		// Create a new query and check its status
		let query_id = contract.xcm_new_query(id, responder, timeout, true).unwrap().unwrap();
		assert_eq!(query_id, 0);
		assert_eq!(contract.poll(id).unwrap(), Some(MessageStatus::Pending));
		assert!(System::events().iter().any(|e| {
			matches!(&e.event,
			RuntimeEvent::Messaging(XcmQueryCreated { origin, id: message_id, query_id, ..})
				if origin == &contract.id && *message_id == id && *query_id == 0
			)
		}));

		// Provide a response.
		let origin = translate(&origin);
		let querier: Location = Junction::AccountId32 {
			network: Some(NetworkId::Polkadot),
			id: contract.id.clone().into(),
		}
		.into();
		assert!(pop_runtime_testnet::PolkadotXcm::expecting_response(
			&origin,
			query_id,
			Some(&translate(&querier))
		));
		assert_ok!(Messaging::xcm_response(
			pallet_xcm::Origin::Response(origin).into(),
			query_id,
			translate(&response)
		));

		assert_eq!(contract.last_event(), Some(XcmCompleted { id, result: response }.encode()));
		assert_eq!(contract.poll(id).unwrap(), None);
		let events = System::events();
		println!("{events:?}");
		assert!(System::events().iter().any(|e| {
			matches!(&e.event,
			RuntimeEvent::Messaging(CallbackExecuted { origin, id: message_id, ..})
				if origin == &contract.id && *message_id == id
			)
		}));
	});
}

// Get the last ismp request.
fn get_ismp_request(ext: &mut TestExternalities) -> ismp::router::Request {
	// Get commitment from last ismp request event.
	let commitment = ext.execute_with(|| {
		System::read_events_for_pallet::<pallet_ismp::Event<Runtime>>()
			.iter()
			.filter_map(|e| match e {
				pallet_ismp::Event::<Runtime>::Request { commitment, .. } =>
					Some(commitment.clone()),
				_ => None,
			})
			.last()
			.unwrap()
	});
	// Read value from offchain storage overlay, stored via `NoOpMmrTree`.
	let key = ("storage".as_bytes().to_vec(), (b"no_op", commitment).encode());
	let request = ext
		.overlayed_changes()
		.offchain()
		.overlay()
		.changes()
		.filter_map(|c| {
			(c.0 == &key).then(|| match c.1.value_ref() {
				OffchainOverlayedChange::SetValue(value) => {
					match Leaf::decode(&mut &value[..]).unwrap() {
						Leaf::Request(req) => Some(req),
						Leaf::Response(_) => None,
					}
				},
				_ => None,
			})
		})
		.last()
		.flatten()
		.unwrap();
	// Ensure the request matches the commitment.
	assert_eq!(commitment.0, keccak_256(&request.encode()));
	request
}

// Translate a source type into a target type via encoding/decoding.
fn translate<S: Encode, T: Decode>(source: &S) -> T {
	T::decode(&mut &source.encode()[..]).unwrap()
}

// A simple, strongly typed wrapper for the contract.
struct Contract {
	address: AccountId32,
	id: AccountId32,
}
impl Contract {
	fn new() -> Self {
		let address = instantiate(CONTRACT, INIT_VALUE, vec![]);
		Self { address: address.clone(), id: address }
	}

	fn ismp_get(
		&self,
		id: MessageId,
		request: Get,
		fee: Balance,
		callback: bool,
	) -> Result<(), Error> {
		let result = self.call("ismp_get", (id, request, fee, callback).encode(), 0);
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn ismp_post(
		&self,
		id: MessageId,
		request: Post,
		fee: Balance,
		callback: bool,
	) -> Result<(), Error> {
		let result = self.call("ismp_post", (id, request, fee, callback).encode(), 0);
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn xcm_new_query(
		&self,
		id: MessageId,
		responder: Location,
		timeout: BlockNumber,
		callback: bool,
	) -> Result<Option<QueryId>, Error> {
		let result = self.call("xcm_new_query", (id, responder, timeout, callback).encode(), 0);
		<Result<Option<QueryId>, Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn poll(&self, request: MessageId) -> Result<Option<MessageStatus>, Error> {
		let result = self.call("poll", request.encode(), 0);
		Result::<Option<MessageStatus>, Error>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn get(&self, request: MessageId) -> Result<Option<Vec<u8>>, Error> {
		let result = self.call("get", request.encode(), 0);
		Result::<Option<Vec<u8>>, Error>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn remove(&self, request: MessageId) -> Result<(), Error> {
		let result = self.call("remove", request.encode(), 0);
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn call(&self, function: &str, params: Vec<u8>, value: u128) -> ExecReturnValue {
		let function = function_selector(function);
		let params = [function, params].concat();
 		bare_call(self.address.clone(), params, value).expect("should work")
	}

	fn last_event(&self) -> Option<Vec<u8>> {
		let events = System::read_events_for_pallet::<pallet_contracts::Event<Runtime>>();
		let contract_events = events
			.iter()
			.filter_map(|event| match event {
				pallet_contracts::Event::<Runtime>::ContractEmitted { contract, data, .. }
					if contract == &self.address =>
					Some(data.as_slice()),
				_ => None,
			})
			.collect::<Vec<&[u8]>>();
		contract_events.last().map(|e|e.to_vec())
	}
}

#[derive(Decode, Encode)]
pub struct IsmpGetCompleted {
	pub id: MessageId,
	pub values: Vec<StorageValue>,
}

#[derive(Decode, Encode)]
pub struct IsmpPostCompleted {
	pub id: MessageId,
	pub response: Vec<u8>,
}

#[derive(Decode, Encode)]
pub struct XcmCompleted {
	pub id: MessageId,
	pub result: xcm::Response,
}

use ::ismp::router::{GetResponse, IsmpRouter, PostResponse, Response, StorageValue};
use pallet_ismp::mmr::Leaf;
use pop_api::{
	messaging::{
		ismp::{Get, Post},
		xcm::{
			Junction, Location, MaybeErrorCode, QueryId, VersionedLocation, VersionedResponse,
			XcmHash,
		},
		RequestId, Status,
	},
	primitives::{BlockNumber, Error},
};
use sp_io::{hashing::keccak_256, TestExternalities};
use sp_runtime::offchain::OffchainOverlayedChange;
use xcm_executor::traits::OnResponse;

use super::*;

const CONTRACT: &str = "contracts/messaging/target/ink/messaging.wasm";
const ASSET_HUB: u32 = 1_000;
const HYPERBRIDGE: u32 = 4_009;
const ISMP_MODULE_ID: [u8; 3] = *b"pop";

#[test]
fn ismp_get_request_works() {
	let id = 42;
	let key = "some_key".as_bytes().to_vec();
	let request = Get::new(ASSET_HUB, 0, 0, "some_context".as_bytes().to_vec(), vec![key.clone()]);
	let response = vec![StorageValue { key, value: Some("some_value".as_bytes().to_vec()) }];

	// Create a get request.
	let mut ext = new_test_ext();
	let contract = ext.execute_with(|| {
		let contract = Contract::new();
		assert_ok!(contract.ismp_get(id, request, 0));
		assert_eq!(contract.poll(id).unwrap(), Some(Status::Pending));

		// TODO: assert events from messaging and ismp pallets emitted
		println!("{:#?}", System::events());

		contract
	});

	// Look up the request within offchain state in order to provide a response.
	let ::ismp::router::Request::Get(get) = get_ismp_request(&mut ext) else { panic!() };

	// Provide a response.
	ext.execute_with(|| {
		let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
		module
			.on_response(Response::Get(GetResponse { get, values: response.clone() }))
			.unwrap();

		assert_eq!(contract.poll(id).unwrap(), Some(Status::Complete));
		assert_eq!(contract.get(id).unwrap(), Some(response.encode()));
		assert_ok!(contract.remove(id));
	});
}

#[test]
fn ismp_post_request_works() {
	let id = 42;
	let request = Post::new(HYPERBRIDGE, 0, "some_data".as_bytes().to_vec());
	let response = "some_value".as_bytes().to_vec();

	// Create a post request.
	let mut ext = new_test_ext();
	let contract = ext.execute_with(|| {
		let contract = Contract::new();
		assert_ok!(contract.ismp_post(id, request, 0));
		assert_eq!(contract.poll(id).unwrap(), Some(Status::Pending));

		// TODO: assert events from messaging and ismp pallets emitted
		println!("{:#?}", System::events());
		contract
	});

	// Look up the request within offchain state in order to provide a response.
	let ::ismp::router::Request::Post(post) = get_ismp_request(&mut ext) else { panic!() };

	// Provide a response.
	ext.execute_with(|| {
		let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
		module
			.on_response(Response::Post(PostResponse {
				post,
				response: response.clone(),
				timeout_timestamp: 0,
			}))
			.unwrap();

		assert_eq!(contract.poll(id).unwrap(), Some(Status::Complete));
		assert_eq!(contract.get(id).unwrap(), Some(response));
		assert_ok!(contract.remove(id));
	});
}

#[test]
fn xcm_request_works() {
	let id = 42u64;
	let origin = Location::new(1, [Junction::Parachain(ASSET_HUB)]);
	let responder = origin.clone().into_versioned();
	let timeout = 100;
	let response = pop_api::messaging::xcm::Response::DispatchResult(MaybeErrorCode::Success);
	let context = staging_xcm::prelude::XcmContext {
		origin: None,
		message_id: XcmHash::default(),
		topic: None,
	};
	new_test_ext().execute_with(|| {
		let contract = Contract::new();
		let query_id = contract.xcm_new_query(id, responder, timeout).unwrap().unwrap();
		assert_eq!(query_id, 0);
		assert_eq!(contract.poll(id).unwrap(), Some(Status::Pending));

		// TODO: assert events from messaging pallet emitted
		println!("{:#?}", System::events());

		// Provide a response.
		let querier = staging_xcm::prelude::Location::new(
			0,
			[staging_xcm::prelude::Junction::AccountId32 {
				network: None,
				id: contract.0.clone().into(),
			}],
		);
		let origin = staging_xcm::prelude::Location::decode(&mut &origin.encode()[..]).unwrap();
		assert!(Messaging::expecting_response(&origin, query_id, Some(&querier)));
		// TODO: update weight
		assert_eq!(
			Messaging::on_response(
				&origin,
				query_id,
				Some(&querier),
				staging_xcm::prelude::Response::decode(&mut &response.encode()[..]).unwrap(),
				Weight::MAX,
				&context
			),
			Weight::zero()
		);

		assert_eq!(contract.poll(id).unwrap(), Some(Status::Complete));
		assert_eq!(contract.get(id).unwrap(), Some(VersionedResponse::from(response).encode()));
		assert_ok!(contract.remove(id));
	});
}

// Get the last ismp request.
fn get_ismp_request(ext: &mut TestExternalities) -> ::ismp::router::Request {
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

// A simple, strongly typed wrapper for the contract.
struct Contract(AccountId32);
impl Contract {
	fn new() -> Self {
		Self(instantiate(CONTRACT, INIT_VALUE, vec![]))
	}

	fn ismp_get(&self, id: RequestId, request: Get, fee: Balance) -> Result<(), Error> {
		let result = self.call("ismp_get", (id, request, fee).encode(), 0);
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn ismp_post(&self, id: RequestId, request: Post, fee: Balance) -> Result<(), Error> {
		let result = self.call("ismp_post", (id, request, fee).encode(), 0);
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn xcm_new_query(
		&self,
		id: RequestId,
		responder: VersionedLocation,
		timeout: BlockNumber,
	) -> Result<Option<QueryId>, Error> {
		let result = self.call("xcm_new_query", (id, responder, timeout).encode(), 0);
		<Result<Option<QueryId>, Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn poll(&self, request: RequestId) -> Result<Option<Status>, Error> {
		let result = self.call("poll", request.encode(), 0);
		Result::<Option<Status>, Error>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn get(&self, request: RequestId) -> Result<Option<Vec<u8>>, Error> {
		let result = self.call("get", request.encode(), 0);
		Result::<Option<Vec<u8>>, Error>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn remove(&self, request: RequestId) -> Result<(), Error> {
		let result = self.call("remove", request.encode(), 0);
		<Result<(), Error>>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn call(&self, function: &str, params: Vec<u8>, value: u128) -> ExecReturnValue {
		let function = function_selector(function);
		let params = [function, params].concat();
		bare_call(self.0.clone(), params, value).expect("should work")
	}
}

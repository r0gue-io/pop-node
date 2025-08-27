
use core::clone::Clone;
use ::ismp::router::{IsmpRouter, StorageValue, GetResponse, Response};

use frame_support::{assert_ok, pallet_prelude::TypeInfo};
use ismp::host::StateMachine;
use pallet_contracts::{test_utils::AccountId32, ExecReturnValue};
use codec::Encode;
use sp_runtime::{testing::H256, app_crypto::sp_core::H160};
use pop_runtime_testnet::{config::ismp::Router, Messaging, RuntimeEvent};
use pallet_api::messaging::Event::*;
use sp_io::{hashing::keccak_256};

use super::*;
use crate::{nonfungibles::NftsInstance, messaging::{IsmpGetCompleted, ISMP_MODULE_ID, get_ismp_request}, bare_call, function_selector, instantiate, INIT_VALUE};

const DAO: &str = "contracts/dao/target/ink/nft_verifier.wasm";
const ASSET_HUB: u32 = 1_000;

#[test]
fn instantiate_dao() {
	new_test_ext().execute_with(|| {
		let expected_collection =
			pallet_nfts::NextCollectionId::<Runtime, NftsInstance>::get().unwrap_or_default();
		let contract = Dao::new();
		let collection = contract.collection_id();
		assert_eq!(collection, expected_collection);
		assert_eq!(
			pallet_nfts::Collection::<Runtime, NftsInstance>::get(collection),
			Some(pallet_nfts::CollectionDetails {
				owner: contract.id,
				owner_deposit: 100000000000,
				items: 0,
				item_metadatas: 0,
				item_configs: 0,
				attributes: 0,
			})
		);
		assert!(pallet_nfts::CollectionConfigOf::<Runtime, NftsInstance>::get(collection).is_some());
	});
}

#[test]
fn register() {
	let id = 1;
	let key = "some_key".as_bytes().to_vec();
	let response = vec![StorageValue { key, value: Some(().encode()) }];

	let mut ext = new_test_ext();
	let (contract, collection) = ext.execute_with(|| {
		let contract = Dao::new();
		let collection = contract.collection_id();
		// Item account holds in required nft collection.
		let item = 0;
		assert_ok!(contract.register(0, item));
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
		assert_eq!(contract.last_event(), RegistrationRequested { account: ALICE, item: 0 }.encode());
		(contract, collection)
	});

	// Look up the request within offchain state in order to provide a response.
	let ismp::router::Request::Get(get) = get_ismp_request(&mut ext) else { panic!() };

	ext.execute_with(|| {
		let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
		let commitment = H256::from(keccak_256(&ismp::router::Request::Get(get.clone()).encode()));
		module
			.on_response(Response::Get(GetResponse { get, values: response.clone() }))
			.unwrap();
		System::assert_has_event(
			IsmpGetResponseReceived { dest: contract.id.clone(), id, commitment }.into(),
		);

		assert_eq!(contract.last_event(), RegistrationCompleted { account: ALICE, verified_item: 0, membership: Some(1) }.encode());
		assert!(System::events().iter().any(|e| {
			matches!(&e.event,
			RuntimeEvent::Messaging(CallbackExecuted { origin, id: message_id, ..})
				if origin == &contract.id && *message_id == id
			)
		}));
		System::assert_has_event(
			Removed { origin: contract.id.clone(), messages: vec![id] }.into(),
		);

		assert_eq!(pallet_nfts::Account::<Runtime, NftsInstance>::get((ALICE, collection, 1)), Some(()));
		assert_eq!(ALICE, pallet_nfts::Pallet::<Runtime, NftsInstance>::owner(collection, 1).unwrap());
	});
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
pub enum Error {
	StatusCode(u32),
	NotReady,
	Unknown,
	DecodingFailed,
	Rejected,
}

#[derive(Decode, Encode)]
pub struct RegistrationCompleted {
	pub account: AccountId32,
	pub verified_item: u32,
	pub membership: Option<u32>,
}

#[derive(Decode, Encode)]
pub struct RegistrationRequested {
	pub account: AccountId32,
	pub item: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct Dao {
	address: AccountId32,
	id: AccountId32,
}
impl Dao {
	fn new() -> Self {
		let address = instantiate(DAO, INIT_VALUE, vec![]);
		Self { address: address.clone(), id: address }
	}

	fn register(&self, height: u32, item: u32) -> Result<(), Error> {
		let result = self.call("register", (height, item).encode(), 0);
		Result::<(), Error>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn complete(&self, item: u32, outcome: bool) -> Result<u32, Error> {
		let result = self.call("complete", (item, outcome).encode(), 0);
		Result::<u32, Error>::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn collection_id(&self) -> u32 {
		let result = self.call("collection_id", vec![], 0);
		u32::decode(&mut &result.data[1..])
			.unwrap_or_else(|_| panic!("Contract reverted: {:?}", result))
	}

	fn call(&self, function: &str, params: Vec<u8>, value: u128) -> ExecReturnValue {
		let function = function_selector(function);
		let params = [function, params].concat();
		bare_call(self.address.clone(), params, value).expect("should work")
	}

	fn last_event(&self) -> Vec<u8> {
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
		contract_events.last().unwrap().to_vec()
	}
}

use codec::{Decode, Encode};
use pallet_api_vnext::messaging::{
	precompiles::v0::IMessaging::{
		getResponseCall, idCall, pollStatusCall, remove_0Call, MessageStatus,
	},
	Event::CallbackExecuted,
	Message, MessageId,
};
use pallet_revive::precompiles::alloy::sol_types::SolEvent;
use pop_api::{
	messaging::{Bytes, Error},
	sol::SolBytes,
	SolErrorDecode,
};
use sp_io::{
	hashing::{keccak_256, twox_256},
	TestExternalities,
};

use super::*;

const ASSET_HUB: u32 = 1_000;
const CONTRACT: &str = "contracts/messaging/target/ink/messaging.polkavm";
const GAS_LIMIT: Weight = Weight::from_parts(7_500_000_000, 200_000);
const HYPERBRIDGE: u32 = 4_009;
const INIT_VALUE: Balance = 1 * UNIT / 2;
const STORAGE_DEPOSIT_LIMIT: DepositLimit<Balance> = DepositLimit::Balance(1 * UNIT);

mod ismp {
	use ::ismp::{
		host::StateMachine,
		router::{GetResponse, IsmpRouter, PostResponse, Request, Response, StorageValue},
	};
	use pallet_api_vnext::messaging::{
		precompiles::ismp::v0::{
			Callback, Encoding, Weight,
			IISMP::{
				get_0Call, get_1Call, post_0Call, post_1Call, Get, GetDispatched_0,
				GetDispatched_1, Post, PostDispatched_0, PostDispatched_1,
			},
		},
		transports::ismp::ID as ISMP_MODULE_ID,
		Event::{IsmpGetResponseReceived, IsmpPostResponseReceived},
	};
	use pallet_ismp::offchain::Leaf;
	use pop_api::messaging::ismp::{IsmpGetCompleted, IsmpPostCompleted, PRECOMPILE_ADDRESS};
	#[cfg(feature = "devnet")]
	use pop_runtime_devnet::config::ismp::Router;
	use sp_runtime::offchain::OffchainOverlayedChange;

	use super::*;

	#[test]
	fn get_works() {
		let origin = ALICE;
		let key = b"some_key".to_vec();
		let timeout = 100_000u64;
		let height = 0u64;
		let request = Get {
			destination: ASSET_HUB,
			height,
			timeout,
			context: b"some_context".to_vec().into(),
			keys: vec![key.clone().into()].into(),
		};
		let response = vec![StorageValue { key, value: Some(b"some_value".to_vec()) }];

		// Create a get request.
		let mut ext = ExtBuilder::new().build();
		let (contract, id, commitment) = ext.execute_with(|| {
            let contract = Contract::new(&origin, INIT_VALUE);

            let id = contract.get(request, U256::zero(), None).unwrap();
            assert_eq!(contract.poll_status(id), MessageStatus::Pending);
            let Some(Message::Ismp{ commitment, .. }) = Messaging::get(id) else { panic!() };
            let expected = GetDispatched_0 { origin: contract.address.0.into(), id, commitment: commitment.0.into() }.encode_data();
            assert_eq!(last_contract_event(&PRECOMPILE_ADDRESS), expected);
            assert!(System::events().iter().any(|e| {
                matches!(e.event,
    			RuntimeEvent::Ismp(pallet_ismp::Event::Request { dest_chain, source_chain , ..})
    				if dest_chain == StateMachine::Polkadot(ASSET_HUB) && source_chain == StateMachine::Polkadot(100)
			)}));

            (contract, id, commitment)
        });

		// Look up the request within offchain state in order to provide a response.
		let Request::Get(get) = get_ismp_request(&mut ext) else { panic!() };

		// Provide a response.
		ext.execute_with(|| {
			let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
			assert_ok!(
				module.on_response(Response::Get(GetResponse { get, values: response.clone() }))
			);
			System::assert_has_event(
				IsmpGetResponseReceived { dest: contract.address, id, commitment }.into(),
			);

			assert_eq!(contract.poll_status(id), MessageStatus::Complete);
			assert_eq!(contract.get_response(id).0, response.encode());
			assert_ok!(contract.remove(id));
		});
	}

	#[test]
	fn get_with_callback_works() {
		let origin = ALICE;
		let key = b"some_key".to_vec();
		let timeout = 100_000u64;
		let height = 0u64;
		let request = Get {
			destination: ASSET_HUB,
			height,
			timeout,
			context: b"some_context".to_vec().into(),
			keys: vec![key.clone().into()].into(),
		};
		let response = vec![StorageValue { key, value: Some(b"some_value".to_vec()) }];

		// Create a get request with callback.
		let mut ext = ExtBuilder::new().build();
		let (contract, id, commitment) = ext.execute_with(|| {
			let contract = Contract::new(&origin, INIT_VALUE);

			let callback = Callback {
				destination: contract.address.0.into(),
				encoding: Encoding::SolidityAbi,
				selector: 0x9bf78ffbu32.into(),
				gasLimit: Weight { refTime: 1_100_000_000, proofSize: 110_000 },
				storageDepositLimit: alloy::U256::from(1 * UNIT / 5)
			};
			let id = contract.get(request, U256::zero(), Some(callback.clone())).unwrap();
			assert_eq!(contract.poll_status(id), MessageStatus::Pending);
			let Some(Message::Ismp { commitment, .. }) = Messaging::get(id) else { panic!() };
			let expected = GetDispatched_1 {
				origin: contract.address.0.into(),
				id,
				commitment: commitment.0.into(),
				callback,
			}
			.encode_data();
			assert_eq!(last_contract_event(&PRECOMPILE_ADDRESS), expected);
			assert!(System::events().iter().any(|e| {
				matches!(e.event,
	   				RuntimeEvent::Ismp(pallet_ismp::Event::Request { dest_chain, source_chain , ..})
	   					if dest_chain == StateMachine::Polkadot(ASSET_HUB) && source_chain == StateMachine::Polkadot(100)    				)
			}));

			(contract, id, commitment)
		});

		// Look up the request within offchain state in order to provide a response.
		let Request::Get(get) = get_ismp_request(&mut ext) else { panic!() };

		// Provide a response.
		ext.execute_with(|| {
			let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
			assert_ok!(
				module.on_response(Response::Get(GetResponse { get, values: response.clone() }))
			);
			System::assert_has_event(
				IsmpGetResponseReceived { dest: contract.address, id, commitment }.into(),
			);
			assert!(System::events().iter().any(|e| {
				matches!(&e.event,
				RuntimeEvent::Messaging(CallbackExecuted { origin, id: message_id, ..})
					if origin == &contract.account_id() && *message_id == id
				)
			}));

			assert_eq!(
				contract.last_event(),
				IsmpGetCompleted {
					id,
					response: response
						.into_iter()
						.map(|v| pop_api::messaging::ismp::StorageValue {
							key: SolBytes(v.key),
							value: v.value.map(|v| SolBytes(v))
						})
						.collect()
				}
				.encode()
			);
			assert_eq!(contract.poll_status(id), MessageStatus::NotFound);
		});
	}

	#[test]
	fn post_works() {
		let origin = ALICE;
		let timeout = 100_000u64;
		let request = Post {
			destination: HYPERBRIDGE,
			to: vec![255u8; 20].into(),
			timeout,
			data: b"some_data".to_vec().into(),
		};
		let response = b"some_value".to_vec();

		// Create a post request.
		let mut ext = ExtBuilder::new().build();
		let (contract, id, commitment) = ext.execute_with(|| {
		    let contract = Contract::new(&origin, INIT_VALUE);

            let id = contract.post(request,  U256::zero(), None).unwrap();
            assert_eq!(contract.poll_status(id), MessageStatus::Pending);
            let Some(Message::Ismp{ commitment, .. }) = Messaging::get(id) else { panic!() };
            let expected = PostDispatched_0 { origin: contract.address.0.into(), id, commitment: commitment.0.into() }.encode_data();
            assert_eq!(last_contract_event(&PRECOMPILE_ADDRESS), expected);
            assert!(System::events().iter().any(|e| {
                matches!(e.event,
    				RuntimeEvent::Ismp(pallet_ismp::Event::Request { dest_chain, source_chain , ..})
    					if dest_chain == StateMachine::Polkadot(HYPERBRIDGE) && source_chain == StateMachine::Polkadot(100)
    				)
            }));

            (contract,id, commitment)
        });

		// Look up the request within offchain state in order to provide a response.
		let Request::Post(post) = get_ismp_request(&mut ext) else { panic!() };

		// Provide a response.
		ext.execute_with(|| {
			let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
			assert_ok!(module.on_response(Response::Post(PostResponse {
				post,
				response: response.clone(),
				timeout_timestamp: timeout,
			})));
			System::assert_has_event(
				IsmpPostResponseReceived { dest: contract.address, id, commitment }.into(),
			);

			assert_eq!(contract.poll_status(id), MessageStatus::Complete);
			assert_eq!(contract.get_response(id).0, response.encode());
			assert_ok!(contract.remove(id));
		});
	}

	#[test]
	fn post_with_callback_works() {
		let origin = ALICE;
		let timeout = 100_000u64;
		let request = Post {
			destination: HYPERBRIDGE,
			to: vec![255u8; 20].into(),
			timeout,
			data: b"some_data".to_vec().into(),
		};
		let response = b"some_value".to_vec();

		// Create a post request with callback.
		let mut ext = ExtBuilder::new().build();
		let (contract, id, commitment) = ext.execute_with(|| {
            let contract = Contract::new(&origin, INIT_VALUE);

            let callback = Callback {
				destination: contract.address.0.into(),
				encoding: Encoding::SolidityAbi,
				selector: 0xbe910d67u32.into(),
				gasLimit: Weight { refTime: 850_000_000, proofSize: 110_000 },
				storageDepositLimit: alloy::U256::from(1 * UNIT / 5)
			};
            let id = contract.post(request,  U256::zero(), Some(callback.clone())).unwrap();
            assert_eq!(contract.poll_status(id), MessageStatus::Pending);
            let Some(Message::Ismp{ commitment, .. }) = Messaging::get(id) else { panic!() };
            let expected = PostDispatched_1 { origin: contract.address.0.into(), id, commitment: commitment.0.into(), callback }.encode_data();
            assert_eq!(last_contract_event(&PRECOMPILE_ADDRESS), expected);
            assert!(System::events().iter().any(|e| {
                matches!(e.event,
    				RuntimeEvent::Ismp(pallet_ismp::Event::Request { dest_chain, source_chain , ..})
    					if dest_chain == StateMachine::Polkadot(HYPERBRIDGE) && source_chain == StateMachine::Polkadot(100)
    				)
            }));

            (contract, id, commitment)
        });

		// Look up the request within offchain state in order to provide a response.
		let Request::Post(post) = get_ismp_request(&mut ext) else { panic!() };

		// Provide a response.
		ext.execute_with(|| {
			let module = Router::default().module_for_id(ISMP_MODULE_ID.to_vec()).unwrap();
			assert_ok!(module.on_response(Response::Post(PostResponse {
				post,
				response: response.clone(),
				timeout_timestamp: timeout,
			})));
			System::assert_has_event(
				IsmpPostResponseReceived { dest: contract.address, id, commitment }.into(),
			);
			assert!(System::events().iter().any(|e| {
				matches!(&e.event,
				RuntimeEvent::Messaging(CallbackExecuted { origin, id: message_id, ..})
					if origin == &contract.account_id() && *message_id == id
				)
			}));

			assert_eq!(
				contract.last_event(),
				IsmpPostCompleted { id, response: SolBytes(response) }.encode()
			);
			assert_eq!(contract.poll_status(id), MessageStatus::NotFound);
		});
	}

	impl Contract {
		fn get(
			&self,
			request: Get,
			fee: U256,
			callback: Option<Callback>,
		) -> Result<MessageId, ismp::Error> {
			let fee = alloy::U256::from_be_bytes(fee.to_big_endian());
			match callback {
				None => {
					let call = get_0Call { request, fee };
					self.call(&self.creator, call, 0)
				},
				Some(callback) => {
					let call = get_1Call { request, fee, callback };
					self.call(&self.creator, call, 0)
				},
			}
		}

		fn post(
			&self,
			request: Post,
			fee: U256,
			callback: Option<Callback>,
		) -> Result<MessageId, ismp::Error> {
			let fee = alloy::U256::from_be_bytes(fee.to_big_endian());
			match callback {
				None => {
					let call = post_0Call { request, fee };
					self.call(&self.creator, call, 0)
				},
				Some(callback) => {
					let call = post_1Call { request, fee, callback };
					self.call(&self.creator, call, 0)
				},
			}
		}
	}

	// Get the last ismp request.
	fn get_ismp_request(ext: &mut TestExternalities) -> Request {
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
}

mod xcm {
	use ::xcm::prelude::{Junction, Location, MaybeErrorCode::Success, NetworkId, Response};
	use pallet_api_vnext::messaging::precompiles::xcm::v0::{
		Callback, Encoding, Weight,
		IXCM::{newQuery_0Call, newQuery_1Call, QueryCreated_0, QueryCreated_1},
	};
	use pop_api::messaging::xcm::{QueryId, XcmCompleted, PRECOMPILE_ADDRESS};
	use xcm_executor::traits::OnResponse;

	use super::*;

	#[test]
	fn id_works() {
		let origin = ALICE;
		ExtBuilder::new().build().execute_with(|| {
			let contract = Contract::new(&origin, INIT_VALUE);
			assert_eq!(contract.id(), u32::from(ParachainInfo::parachain_id()));
		});
	}

	#[test]
	fn query_works() {
		let origin = ALICE;
		let responder = Location::new(1, [Junction::Parachain(ASSET_HUB)]);
		let timeout = 100;
		let response = Response::DispatchResult(Success);
		ExtBuilder::new().build().execute_with(|| {
			let contract = Contract::new(&origin, INIT_VALUE);

			// Create a new query and check its status
			let (id, query_id) = contract.new_query(responder.encode(), timeout, None).unwrap();
			assert_eq!(query_id, 0);
			assert_eq!(contract.poll_status(id), MessageStatus::Pending);
			let expected =
				QueryCreated_0 { account: contract.address.0.into(), id, queryId: query_id }
					.encode_data();
			assert_eq!(last_contract_event(&PRECOMPILE_ADDRESS), expected);

			// Provide a response.
			let querier: Location = Junction::AccountId32 {
				network: Some(NetworkId::Polkadot),
				id: contract.account_id().into(),
			}
			.into();
			assert!(PolkadotXcm::expecting_response(&responder, query_id, Some(&querier)));
			assert_ok!(Messaging::xcm_response(
				pallet_xcm::Origin::Response(responder).into(),
				query_id,
				response.clone()
			));

			assert_eq!(contract.poll_status(id), MessageStatus::Complete);
			assert_eq!(contract.get_response(id).0, response.encode());
			assert_ok!(contract.remove(id));
		});
	}

	#[test]
	fn query_with_callback_works() {
		let origin = ALICE;
		let responder = Location::new(1, [Junction::Parachain(ASSET_HUB)]);
		let timeout = 100;
		let response = Response::DispatchResult(Success);
		ExtBuilder::new().build().execute_with(|| {
			let contract = Contract::new(&origin, INIT_VALUE);

			// Create a new query and check its status
			let callback = Callback {
				destination: contract.address.0.into(),
				encoding: Encoding::SolidityAbi,
				selector: 0x97dbf9fbu32.into(),
				gasLimit: Weight { refTime: 850_000_000, proofSize: 110_000 },
				storageDepositLimit: alloy::U256::from(1 * UNIT / 5),
			};
			let (id, query_id) =
				contract.new_query(responder.encode(), timeout, Some(callback.clone())).unwrap();
			assert_eq!(query_id, 0);
			assert_eq!(contract.poll_status(id), MessageStatus::Pending);
			let expected = QueryCreated_1 {
				account: contract.address.0.into(),
				id,
				queryId: query_id,
				callback,
			}
			.encode_data();
			assert_eq!(last_contract_event(&PRECOMPILE_ADDRESS), expected);

			// Provide a response.
			let querier: Location = Junction::AccountId32 {
				network: Some(NetworkId::Polkadot),
				id: contract.account_id().into(),
			}
			.into();
			assert!(PolkadotXcm::expecting_response(&responder, query_id, Some(&querier)));
			assert_ok!(Messaging::xcm_response(
				pallet_xcm::Origin::Response(responder).into(),
				query_id,
				response.clone()
			));
			assert!(System::events().iter().any(|e| {
				matches!(&e.event,
				RuntimeEvent::Messaging(CallbackExecuted { origin, id: message_id, ..})
					if origin == &contract.account_id() && *message_id == id
				)
			}));

			assert_eq!(
				contract.last_event(),
				XcmCompleted { id, result: SolBytes(response.encode()) }.encode()
			);
			assert_eq!(contract.poll_status(id), MessageStatus::NotFound);
		});
	}

	impl Contract {
		fn id(&self) -> u32 {
			self.call::<_, Error>(&self.creator, idCall {}, 0).unwrap()
		}

		fn new_query(
			&self,
			responder: Vec<u8>,
			timeout: BlockNumber,
			callback: Option<Callback>,
		) -> Result<(MessageId, QueryId), xcm::Error> {
			match callback {
				None => {
					let call = newQuery_0Call { responder: responder.into(), timeout };
					let result = self.call(&self.creator, call, 0)?;
					Ok((result.id, result.queryId))
				},
				Some(callback) => {
					let call = newQuery_1Call { responder: responder.into(), timeout, callback };
					let result = self.call(&self.creator, call, 0)?;
					Ok((result.id, result.queryId))
				},
			}
		}
	}
}

// A simple, strongly typed wrapper for the contract.
struct Contract {
	address: H160,
	creator: AccountId,
}
impl Contract {
	// Create a new instance of the contract through on-chain instantiation.
	fn new(origin: &AccountId, value: Balance) -> Self {
		let data = vec![]; // Default solidity constructor
		let salt = twox_256(&value.to_le_bytes());
		let address = instantiate(
			RuntimeOrigin::signed(origin.clone()),
			CONTRACT,
			value,
			GAS_LIMIT,
			STORAGE_DEPOSIT_LIMIT,
			data,
			Some(salt),
		);
		Self { address, creator: origin.clone() }
	}

	fn get_response(&self, message: MessageId) -> Bytes {
		let call = getResponseCall { message };
		SolBytes(self.call::<_, Error>(&self.creator, call, 0).unwrap().0.into())
	}

	fn poll_status(&self, message: MessageId) -> MessageStatus {
		let call = pollStatusCall { message };
		self.call::<_, Error>(&self.creator, call, 0).unwrap()
	}

	fn remove(&self, message: MessageId) -> Result<(), Error> {
		let call = remove_0Call { message };
		self.call(&self.creator, call, 0)?;
		Ok(())
	}

	fn account_id(&self) -> AccountId {
		to_account_id(&self.address)
	}

	fn call<T: SolCall, E: SolErrorDecode>(
		&self,
		origin: &AccountId,
		call: T,
		value: Balance,
	) -> Result<T::Return, E> {
		let origin = RuntimeOrigin::signed(origin.clone());
		let dest = self.address.clone();
		let data = call.abi_encode();
		let result = bare_call(origin, dest, value, GAS_LIMIT, STORAGE_DEPOSIT_LIMIT, data)
			.expect("should work");
		match result.did_revert() {
			true => Err(E::decode(&result.data).expect(&format!(
				"unable to decode error value from '{:?}'",
				String::from_utf8_lossy(&result.data)
			))),
			false =>
				Ok(T::abi_decode_returns(&result.data).expect("unable to decode success value")),
		}
	}

	fn last_event(&self) -> Vec<u8> {
		last_contract_event(&self.address)
	}
}

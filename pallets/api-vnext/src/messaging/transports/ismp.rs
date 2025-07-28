pub(crate) use ::ismp::dispatcher::{FeeMetadata, IsmpDispatcher};
use ::ismp::{
	dispatcher::{
		DispatchGet, DispatchPost,
		DispatchRequest::{self},
	},
	messaging::hash_request,
	module::IsmpModule,
	router::{GetResponse, PostRequest, PostResponse, Request, Response, Timeout},
};
use frame_support::{
	ensure,
	pallet_prelude::Weight,
	traits::{fungible::MutateHold, tokens::Precision::Exact, Get as _},
};
use pallet_ismp::weights::IsmpModuleWeight;

use super::{
	super::{Message, Pallet},
	*,
};

type DbWeightOf<T> = <T as frame_system::Config>::DbWeight;
type GetState<T> = (
	StateMachine,
	[u8; 4],
	BoundedVec<BoundedVec<u8, <T as Config>::MaxKeyLen>, <T as Config>::MaxKeys>,
	u64,
	BoundedVec<u8, <T as Config>::MaxContextLen>,
	u64,
);
type PostState<T> =
	(StateMachine, [u8; 4], [u8; 4], u64, BoundedVec<u8, <T as Config>::MaxDataLen>);
type StateMachine = (u8, u32);

pub const ID: [u8; 3] = *b"pop";

/// Submit a new ISMP `Get` request.
///
/// This sends a `Get` request through ISMP, optionally with a callback to handle the
/// response.
///
/// # Parameters
/// - `origin`: The account submitting the request.
/// - `message`: The ISMP `Get` message containing query details.
/// - `fee`: The fee to be paid to relayers.
/// - `callback`: Optional callback to execute upon receiving a response.
///
/// # Returns
/// A unique identifier for the message.
pub(crate) fn get<T: Config>(
	origin: Origin<T::AccountId>,
	message: DispatchGet,
	fee: BalanceOf<T>,
	callback: Option<Callback>,
) -> Result<(MessageId, H256), DispatchError> {
	// Take deposits and fees.
	let message_deposit =
		calculate_protocol_deposit::<T, T::OnChainByteFee>(ProtocolStorageDeposit::IsmpRequests)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>())
			.saturating_add(calculate_deposit_of::<T, T::OffChainByteFee, GetState<T>>());

	T::Fungibles::hold(&HoldReason::Messaging.into(), &origin.account, message_deposit)?;

	if let Some(cb) = callback.as_ref() {
		T::Fungibles::hold(
			&HoldReason::CallbackGas.into(),
			&origin.account,
			T::WeightToFee::weight_to_fee(&cb.weight),
		)?;
	}

	// Process message by dispatching request via ISMP.
	let commitment = match T::IsmpDispatcher::default().dispatch_request(
		DispatchRequest::Get(message),
		FeeMetadata { payer: origin.account.clone(), fee },
	) {
		Ok(commitment) => Ok::<H256, DispatchError>(commitment),
		Err(e) => {
			if let Ok(err) = e.downcast::<::ismp::Error>() {
				log::error!("ISMP Dispatch failed!! {:?}", err);
			}
			return Err(Error::<T>::IsmpDispatchFailed.into());
		},
	}?;
	// Store commitment for lookup on response, message for querying,
	// response/timeout handling.
	let id = next_message_id::<T>()?;
	IsmpRequests::<T>::insert(commitment, id);
	Messages::<T>::insert(id, Message::Ismp { origin, commitment, callback, message_deposit });
	Ok((id, commitment))
}

/// Submit a new ISMP `Post` request.
///
/// Sends a `Post` message through ISMP with arbitrary data and an optional callback.
///
/// # Parameters
/// - `origin`: The account submitting the request.
/// - `message`: The ISMP `Post` message containing the payload.
/// - `fee`: The fee to be paid to relayers.
/// - `callback`: Optional callback to execute upon receiving a response.
///
/// # Returns
/// A unique identifier for the message.
pub(crate) fn post<T: Config>(
	origin: Origin<T::AccountId>,
	message: DispatchPost,
	fee: BalanceOf<T>,
	callback: Option<Callback>,
) -> Result<(MessageId, H256), DispatchError> {
	// Take deposits and fees.
	let message_deposit =
		calculate_protocol_deposit::<T, T::OnChainByteFee>(ProtocolStorageDeposit::IsmpRequests)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>())
			.saturating_add(calculate_deposit_of::<T, T::OffChainByteFee, PostState<T>>());

	T::Fungibles::hold(&HoldReason::Messaging.into(), &origin.account, message_deposit)?;

	if let Some(cb) = callback.as_ref() {
		T::Fungibles::hold(
			&HoldReason::CallbackGas.into(),
			&origin.account,
			T::WeightToFee::weight_to_fee(&cb.weight),
		)?;
	}

	// Process message by dispatching request via ISMP.
	let commitment = T::IsmpDispatcher::default()
		.dispatch_request(
			DispatchRequest::Post(message),
			FeeMetadata { payer: origin.account.clone(), fee },
		)
		.map_err(|_| Error::<T>::IsmpDispatchFailed)?;

	// Store commitment for lookup on response, message for querying,
	// response/timeout handling.
	let id = next_message_id::<T>()?;
	IsmpRequests::<T>::insert(commitment, id);
	Messages::<T>::insert(id, Message::Ismp { origin, commitment, callback, message_deposit });
	Ok((id, commitment))
}

pub(crate) fn process_response<T: Config>(
	commitment: &H256,
	response_data: impl Encode + EncodeCallback,
	event: impl Fn(H160, MessageId) -> Event<T>,
) -> Result<(), anyhow::Error> {
	// TODO: handle Solidity encoding size
	ensure!(
		response_data.encoded_size() <= T::MaxResponseLen::get() as usize,
		::ismp::Error::Custom("Response length exceeds maximum allowed length.".into())
	);

	let id = IsmpRequests::<T>::get(commitment)
		.ok_or(::ismp::Error::Custom("Request not found.".into()))?;

	let Some(Message::Ismp { origin, commitment, callback, message_deposit }) =
		Messages::<T>::get(id)
	else {
		return Err(::ismp::Error::Custom("Message must be an ismp request.".into()).into());
	};

	// Deposit that the message has been recieved before a potential callback execution.
	Pallet::<T>::deposit_event(event(origin.address, id));

	// Attempt callback with result if specified.
	if let Some(callback) = callback {
		if call::<T>(&origin.account, callback, &id, &response_data).is_ok() {
			// Clean storage, return deposit
			Messages::<T>::remove(id);
			IsmpRequests::<T>::remove(commitment);
			T::Fungibles::release(
				&HoldReason::Messaging.into(),
				&origin.account,
				message_deposit,
				Exact,
			)
			.map_err(|_| ::ismp::Error::Custom("failed to release message deposit.".into()))?;

			return Ok(());
		}
	}

	// No callback or callback error: store response for manual retrieval and removal.
	let response: BoundedVec<u8, T::MaxResponseLen> = codec::Encode::encode(&response_data)
		.try_into()
		.map_err(|_| ::ismp::Error::Custom("response exceeds max".into()))?;
	Messages::<T>::insert(
		id,
		Message::IsmpResponse { origin: origin.address, commitment, message_deposit, response },
	);
	Ok(())
}

pub(crate) fn timeout_commitment<T: Config>(commitment: &H256) -> Result<(), anyhow::Error> {
	let key = IsmpRequests::<T>::get(commitment).ok_or(::ismp::Error::Custom(
		"Request commitment not found while processing timeout.".into(),
	))?;
	Messages::<T>::try_mutate(key, |message| {
		let Some(Message::Ismp { origin, commitment, message_deposit, callback }) = message else {
			return Err(::ismp::Error::Custom("Invalid message".into()));
		};
		let callback_deposit = callback.map(|cb| T::WeightToFee::weight_to_fee(&cb.weight));
		*message = Some(Message::IsmpTimeout {
			origin: origin.address,
			message_deposit: *message_deposit,
			commitment: *commitment,
			callback_deposit,
		});
		Ok(())
	})?;

	Pallet::<T>::deposit_event(Event::<T>::IsmpTimedOut { commitment: *commitment });
	Ok(())
}

pub struct Handler<T>(PhantomData<T>);
impl<T> Default for Handler<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Handler<T> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<T: Config> IsmpModule for Handler<T> {
	fn on_accept(&self, _request: PostRequest) -> Result<(), anyhow::Error> {
		Ok(())
	}

	fn on_response(&self, response: Response) -> Result<(), anyhow::Error> {
		// Hash request to determine key for message lookup.
		match response {
			Response::Get(GetResponse { get, values }) => {
				log::debug!(target: "pop-api::messaging::ismp", "StorageValue={:?}", values);
				let commitment = hash_request::<T::Keccak256>(&Request::Get(get));
				process_response(&commitment, values, |dest, id| {
					Event::<T>::IsmpGetResponseReceived { dest, id, commitment }
				})
			},
			Response::Post(PostResponse { post, response, .. }) => {
				let commitment = hash_request::<T::Keccak256>(&Request::Post(post));
				process_response(&commitment, response, |dest, id| {
					Event::<T>::IsmpPostResponseReceived { dest, id, commitment }
				})
			},
		}
	}

	fn on_timeout(&self, timeout: Timeout) -> Result<(), anyhow::Error> {
		match timeout {
			Timeout::Request(request) => {
				// hash request to determine key for original request id lookup
				let commitment = hash_request::<T::Keccak256>(&request);
				timeout_commitment::<T>(&commitment)
			},
			Timeout::Response(PostResponse { post, .. }) => {
				let commitment = hash_request::<T::Keccak256>(&Request::Post(post));
				timeout_commitment::<T>(&commitment)
			},
		}
	}
}

impl<T: Config> IsmpModuleWeight for Pallet<T> {
	// Static as not in use.
	fn on_accept(&self, _request: &PostRequest) -> Weight {
		DbWeightOf::<T>::get().reads_writes(1, 1)
	}

	fn on_timeout(&self, timeout: &Timeout) -> Weight {
		let x = match timeout {
			Timeout::Request(Request::Get(_)) => 0u32,
			Timeout::Request(Request::Post(_)) => 1u32,
			Timeout::Response(_) => 2u32,
		};
		T::WeightInfo::ismp_on_timeout(x)
	}

	// todo: test
	fn on_response(&self, response: &Response) -> Weight {
		let x = match response {
			Response::Get(_) => 0,
			Response::Post(_) => 1,
		};

		// Also add actual weight consumed by contract env.
		T::WeightInfo::ismp_on_response(x).saturating_add(T::CallbackExecutor::execution_weight())
	}
}

#[cfg(test)]
mod tests {
	use ::ismp::{host::StateMachine, module::IsmpModule, Error as IsmpError};
	use frame_support::{assert_ok, traits::fungible::InspectHold, weights::WeightToFee as _};

	use super::{super::tests::events, messaging::HoldReason::*, mock::*, *};

	type Fungibles = <Test as Config>::Fungibles;
	type GetState = super::GetState<Test>;
	type IsmpRequests = super::IsmpRequests<Test>;
	type MaxContextLen = <Test as Config>::MaxContextLen;
	type MaxDataLen = <Test as Config>::MaxDataLen;
	type MaxKeyLen = <Test as Config>::MaxKeyLen;
	type MaxKeys = <Test as Config>::MaxKeys;
	type MaxResponseLen = <Test as Config>::MaxResponseLen;
	type Messages = super::Messages<Test>;
	type OffChainByteFee = <Test as Config>::OffChainByteFee;
	type OnChainByteFee = <Test as Config>::OnChainByteFee;
	type PostState = super::PostState<Test>;
	type WeightToFee = <Test as Config>::WeightToFee;

	mod get {
		use super::*;

		#[test]
		fn takes_deposit() {
			let origin = Origin::from((ALICE_ADDR, ALICE));
			let weight = Weight::from_parts(100_000_000, 100_000_000);
			let callback = Callback::new(H160::zero(), Encoding::Scale, [1; 4], weight);
			let callback_deposit = WeightToFee::weight_to_fee(&weight);
			let fee: Balance = u32::MAX.into();
			let expected_deposit = calculate_protocol_deposit::<Test, OnChainByteFee>(
				ProtocolStorageDeposit::IsmpRequests,
			) + calculate_message_deposit::<Test, OnChainByteFee>() +
				calculate_deposit_of::<Test, OffChainByteFee, GetState>() +
				callback_deposit;
			let endowment = existential_deposit() + expected_deposit + fee;
			ExtBuilder::new()
				.with_balances(vec![(origin.account.clone(), endowment)])
				.build()
				.execute_with(|| {
					let held_balance_pre_hold = Balances::total_balance_on_hold(&origin.account);
					assert_eq!(held_balance_pre_hold, 0);
					assert!(expected_deposit != 0);

					assert_ok!(get::<Test>(origin.clone(), message(), fee, Some(callback)));

					let held_balance_post_hold = Balances::total_balance_on_hold(&origin.account);
					assert_eq!(held_balance_post_hold, expected_deposit);
				})
		}

		#[test]
		fn assert_state() {
			let origin = Origin::from((ALICE_ADDR, ALICE));
			let id = 1;
			let fee: Balance = u32::MAX.into();
			let callback = None;
			let deposit = calculate_protocol_deposit::<Test, OnChainByteFee>(
				ProtocolStorageDeposit::IsmpRequests,
			) + calculate_message_deposit::<Test, OnChainByteFee>() +
				calculate_deposit_of::<Test, OffChainByteFee, GetState>();
			let endowment = existential_deposit() + deposit + fee;
			ExtBuilder::new()
				.with_balances(vec![(origin.account.clone(), endowment)])
				.with_message_id(id)
				.build()
				.execute_with(|| {
					let (id, commitment) =
						get::<Test>(origin.clone(), message(), fee, callback).unwrap();

					assert_eq!(IsmpRequests::get(commitment), Some(id));
					let Some(Message::Ismp {
						origin: o,
						commitment: c,
						callback: cb,
						message_deposit: d,
					}) = Messages::get(id)
					else {
						panic!("wrong message type");
					};
					assert_eq!((o, c, cb, d), (origin, commitment, callback, deposit))
				})
		}

		#[test]
		fn max_len_works() {
			let get = message();
			assert_eq!(
				GetState::max_encoded_len(),
				(get.dest, get.from, get.keys, get.height, get.context, get.timeout)
					.encode()
					.len()
			)
		}

		fn message() -> DispatchGet {
			DispatchGet {
				dest: StateMachine::Polkadot(u32::MAX),
				from: ID.into(),
				keys: vec![
					vec![255; <MaxKeyLen as Get<u32>>::get() as usize];
					<MaxKeys as Get<u32>>::get() as usize
				],
				height: u64::MAX,
				context: vec![255; <MaxContextLen as Get<u32>>::get() as usize],
				timeout: u64::MAX,
			}
		}
	}

	mod post {
		use super::*;

		#[test]
		fn takes_deposit() {
			let origin = Origin::from((ALICE_ADDR, ALICE));
			let weight = Weight::from_parts(100_000_000, 100_000_000);
			let callback = Callback::new(H160::zero(), Encoding::Scale, [1; 4], weight);
			let callback_deposit = <Test as Config>::WeightToFee::weight_to_fee(&weight);
			let fee: Balance = u32::MAX.into();
			let expected_deposit =
				calculate_protocol_deposit::<Test, <Test as Config>::OnChainByteFee>(
					ProtocolStorageDeposit::IsmpRequests,
				) + calculate_message_deposit::<Test, <Test as Config>::OnChainByteFee>() +
					calculate_deposit_of::<Test, OffChainByteFee, PostState>() +
					callback_deposit;
			let endowment = existential_deposit() + expected_deposit + fee;
			ExtBuilder::new()
				.with_balances(vec![(origin.account.clone(), endowment)])
				.build()
				.execute_with(|| {
					let held_balance_pre_hold = Balances::total_balance_on_hold(&origin.account);
					assert_eq!(held_balance_pre_hold, 0);
					assert_ne!(callback_deposit, 0);
					assert_ne!(expected_deposit, 0);

					assert_ok!(post::<Test>(origin.clone(), message(), fee, Some(callback)));

					let held_balance_post_hold = Balances::total_balance_on_hold(&origin.account);
					assert_eq!(held_balance_post_hold, expected_deposit);
				})
		}

		#[test]
		fn assert_state() {
			let origin = Origin::from((ALICE_ADDR, ALICE));
			let id = 1;
			let fee: Balance = u32::MAX.into();
			let callback = None;
			let deposit = calculate_protocol_deposit::<Test, OnChainByteFee>(
				ProtocolStorageDeposit::IsmpRequests,
			) + calculate_message_deposit::<Test, OnChainByteFee>() +
				calculate_deposit_of::<Test, OffChainByteFee, PostState>();
			let endowment = existential_deposit() + deposit + fee;
			ExtBuilder::new()
				.with_balances(vec![(origin.account.clone(), endowment)])
				.with_message_id(id)
				.build()
				.execute_with(|| {
					let (id, commitment) =
						post::<Test>(origin.clone(), message(), fee, callback).unwrap();

					assert_eq!(IsmpRequests::get(commitment), Some(id));
					let Some(Message::Ismp {
						origin: o,
						commitment: c,
						callback: cb,
						message_deposit: d,
					}) = Messages::get(id)
					else {
						panic!("wrong message type");
					};
					assert_eq!((o, c, cb, d), (origin, commitment, callback, deposit))
				})
		}

		#[test]
		fn max_len_works() {
			let post = message();
			assert_eq!(
				PostState::max_encoded_len(),
				(post.dest, post.from, post.to, post.timeout, post.body).encode().len()
			)
		}

		fn message() -> DispatchPost {
			DispatchPost {
				dest: StateMachine::Polkadot(u32::MAX),
				from: ID.to_vec(),
				to: ID.to_vec(),
				timeout: u64::MAX,
				body: vec![255; <MaxDataLen as Get<u32>>::get() as usize],
			}
		}
	}

	mod ismp_hooks {
		use super::*;

		mod on_accept {
			use super::*;

			/// The on_accept must return Ok even when not in use.
			/// If an error is returned the receipt is not removed and a replay attack is possible.
			#[test]
			fn is_ok() {
				let handler = handler();
				ExtBuilder::new()
					.build()
					.execute_with(|| assert!(handler.on_accept(post_request(100usize)).is_ok()))
			}
		}

		mod timeout_commitment {
			use super::*;

			#[test]
			fn request_not_found() {
				ExtBuilder::new().build().execute_with(|| {
					let err = timeout_commitment::<Test>(&Default::default()).unwrap_err();
					assert_eq!(
						err.downcast::<IsmpError>().unwrap(),
						IsmpError::Custom(
							"Request commitment not found while processing timeout.".into()
						)
					)
				})
			}

			#[test]
			fn invalid_request() {
				let origin = Origin::from((ALICE_ADDR, ALICE));
				let id = 1;
				let commitment: H256 = [1u8; 32].into();
				let message = Message::xcm_query(origin, 0, None, 100);
				ExtBuilder::new().build().execute_with(|| {
					IsmpRequests::insert(commitment, id);
					Messages::insert(id, &message);

					let err = timeout_commitment::<Test>(&commitment).unwrap_err();
					assert_eq!(
						err.downcast::<IsmpError>().unwrap(),
						IsmpError::Custom("Invalid message".into())
					)
				})
			}

			#[test]
			fn actually_timesout_assert_event() {
				let origin = Origin::from((ALICE_ADDR, ALICE));
				let id = 1;
				let commitment: H256 = [1u8; 32].into();
				let message_deposit = 100;
				let message = Message::ismp(origin, commitment, None, message_deposit);
				ExtBuilder::new().build().execute_with(|| {
					IsmpRequests::insert(commitment, id);
					Messages::insert(id, &message);

					let res = timeout_commitment::<Test>(&commitment);

					assert!(res.is_ok(), "{:?}", res.unwrap_err().downcast::<IsmpError>().unwrap());

					if let Some(Message::IsmpTimeout { commitment, .. }) = Messages::get(id) {
						assert!(events().contains(&Event::IsmpTimedOut { commitment }))
					} else {
						panic!("Message not timed out.")
					}
				})
			}

			#[test]
			fn success_callback_releases_deposit() {
				let origin = Origin::from((ALICE_ADDR, ALICE));
				let response = vec![1u8];
				let commitment = H256::default();
				let id = 1;
				let callback = Callback::new(H160::zero(), Encoding::Scale, [1; 4], 100.into());
				let message_deposit = 100;
				let message =
					Message::ismp(origin.clone(), commitment, Some(callback), message_deposit);
				ExtBuilder::new()
					.with_balances(vec![(
						origin.account.clone(),
						existential_deposit() + message_deposit,
					)])
					.build()
					.execute_with(|| {
						assert_ok!(Fungibles::hold(
							&Messaging.into(),
							&origin.account,
							message_deposit
						));
						let post_hold = Balances::free_balance(&origin.account);

						IsmpRequests::insert(commitment, id);
						Messages::insert(id, message);

						let res = process_response::<Test>(&commitment, response, |dest, id| {
							Event::IsmpGetResponseReceived { dest, id, commitment }
						});

						assert!(res.is_ok(), "process_response failed");

						let post_process = Balances::free_balance(&origin.account);
						assert_eq!(post_process - message_deposit, post_hold);
					})
			}
		}

		mod process_response {
			use sp_runtime::bounded_vec;

			use super::*;

			#[test]
			fn response_exceeds_max_encoded_len_limit() {
				let commitment = H256::zero();
				let response = vec![1u8; <MaxResponseLen as Get<u32>>::get() as usize + 1usize];
				ExtBuilder::new().build().execute_with(|| {
					let err = process_response::<Test>(&commitment, response, |dest, id| {
						Event::IsmpGetResponseReceived { dest, id, commitment }
					})
					.unwrap_err();

					assert_eq!(
						err.downcast::<IsmpError>().unwrap(),
						IsmpError::Custom(
							"Response length exceeds maximum allowed length.".to_string()
						)
					);
				})
			}

			#[test]
			fn request_not_found() {
				let commitment = H256::zero();
				let response = vec![1u8];
				ExtBuilder::new().build().execute_with(|| {
					let err = process_response::<Test>(&commitment, response, |dest, id| {
						Event::IsmpGetResponseReceived { dest, id, commitment }
					})
					.unwrap_err();

					assert_eq!(
						err.downcast::<IsmpError>().unwrap(),
						IsmpError::Custom("Request not found.".to_string())
					);
				})
			}

			#[test]
			fn message_must_be_ismp_request() {
				let origin = (ALICE_ADDR, ALICE);
				let response = bounded_vec![1u8];
				let commitment = H256::default();
				let id = 1;
				let message = Message::ismp_response(origin.0, commitment, 100, response);
				ExtBuilder::new().build().execute_with(|| {
					IsmpRequests::insert(commitment, id);
					Messages::insert(id, message);

					let err = process_response::<Test>(&commitment, vec![1u8], |dest, id| {
						Event::IsmpGetResponseReceived { dest, id, commitment }
					})
					.unwrap_err();
					assert_eq!(
						err.downcast::<IsmpError>().unwrap(),
						IsmpError::Custom("Message must be an ismp request.".to_string())
					);
				})
			}

			#[test]
			fn no_callback_saves_response() {
				let origin = Origin::from((ALICE_ADDR, ALICE));
				let response = vec![1u8];
				let commitment = H256::default();
				let id = 1;
				let message = Message::ismp(origin, commitment, None, 100);
				ExtBuilder::new().build().execute_with(|| {
					IsmpRequests::insert(commitment, id);
					Messages::insert(id, message);

					let res = process_response::<Test>(&commitment, response, |dest, id| {
						Event::IsmpGetResponseReceived { dest, id, commitment }
					});

					assert!(res.is_ok(), "process_response failed");

					let Some(Message::IsmpResponse { .. }) = Messages::get(id) else {
						panic!("wrong message type.")
					};
				})
			}
		}

		fn handler() -> ismp::Handler<Test> {
			ismp::Handler::<Test>::new()
		}

		fn post_request(body_len: usize) -> PostRequest {
			PostRequest {
				source: StateMachine::Polkadot(2000),
				dest: StateMachine::Polkadot(2001),
				nonce: 100u64,
				from: [1u8; 32].to_vec(),
				to: [1u8; 32].to_vec(),
				timeout_timestamp: 100_000,
				body: vec![1u8; body_len],
			}
		}
	}

	fn existential_deposit() -> Balance {
		<ExistentialDeposit as Get<Balance>>::get()
	}
}

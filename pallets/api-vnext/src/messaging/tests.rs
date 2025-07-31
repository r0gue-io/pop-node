use frame_support::{
	assert_noop, assert_ok,
	dispatch::PostDispatchInfo,
	storage::{with_transaction, TransactionOutcome},
	traits::fungible::InspectHold,
	weights::WeightToFee as _,
};
use sp_runtime::TokenError::FundsUnavailable;
use HoldReason::*;

use super::{CallbackExecutor as _, WeightInfo as _, *};
use crate::mock::*;

type CallbackExecutor = <Test as Config>::CallbackExecutor;
type Error = super::Error<Test>;
type Fungibles = <Test as Config>::Fungibles;
type IsmpRequests = super::IsmpRequests<Test>;
type Message = super::Message<Test>;
type Messages = super::Messages<Test>;
type Origin = super::Origin<Test>;
type WeightInfo = <Test as Config>::WeightInfo;
type WeightToFee = <Test as Config>::WeightToFee;
type XcmQueries = super::XcmQueries<Test>;

mod remove {
	use super::*;

	#[test]
	fn message_not_found() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = 0;
		ExtBuilder::new().build().execute_with(|| {
			assert_noop!(remove(origin, &[message]), Error::MessageNotFound);
		})
	}

	#[test]
	fn only_originator_can_remove() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let messages = [
			Message::ismp_response(origin.address, H256::zero(), 0, BoundedVec::default()),
			Message::ismp_timeout(origin.address, H256::zero(), 0, None),
			Message::xcm_response(origin.address, 0, 0, Response::Null),
			Message::xcm_timeout(origin.address, 0, 0, None),
		];
		let messages_len = messages.len();
		let caller = Origin::from((BOB_ADDR, BOB));
		ExtBuilder::new()
			.with_messages(
				messages
					.into_iter()
					.enumerate()
					.map(|(i, m)| (origin.account.clone(), i as MessageId, m, 0))
					.collect(),
			)
			.build()
			.execute_with(|| {
				for message in 0..messages_len {
					assert_noop!(
						remove(caller.clone(), &[message as MessageId]),
						DispatchError::BadOrigin
					);
				}
			})
	}

	#[test]
	fn multiple_messages_remove_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let deposit: Balance = 100;
		// An ismp response can always be removed.
		let message =
			Message::ismp_response(origin.address, H256::default(), deposit, BoundedVec::default());
		let messages = 3;
		let endowment = existential_deposit() + deposit * messages as Balance;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.with_messages(
				(0..messages)
					.map(|i| (origin.account.clone(), i, message.clone(), deposit))
					.collect(),
			)
			.build()
			.execute_with(|| {
				let messages = (0..messages).collect::<Vec<_>>();
				assert_ok!(remove(origin, &messages));

				for id in messages {
					assert!(Messages::get(id).is_none(), "message should have been removed.");
				}
			});
	}

	#[test]
	fn deposit_is_returned_if_try_remove_is_ok() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let deposit: Balance = 100;
		// An ismp response can always be removed.
		let message =
			Message::ismp_response(origin.address, H256::default(), deposit, BoundedVec::default());
		let id = 1;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), existential_deposit() + deposit)])
			.with_messages(vec![(origin.account.clone(), id, message, deposit)])
			.build()
			.execute_with(|| {
				let free_balance = Balances::free_balance(&origin.account);

				assert_ok!(remove(origin.clone(), &[id]));

				assert_eq!(Balances::free_balance(&origin.account), free_balance + deposit);
			});
	}

	#[test]
	fn deposit_is_not_returned_if_try_remove_is_noop() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let deposit: Balance = 100;
		let message = Message::ismp(origin.clone(), H256::default(), None, deposit);
		let id = 1;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), existential_deposit() + deposit)])
			.with_messages(vec![(origin.account.clone(), id, message, deposit)])
			.build()
			.execute_with(|| {
				let free_balance = Balances::free_balance(&origin.account);

				assert_noop!(remove(origin.clone(), &[id]), Error::RequestPending);

				assert_eq!(Balances::free_balance(&origin.account), free_balance);
			});
	}

	#[test]
	fn multiple_messages_rolls_back_if_one_fails() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let deposit: Balance = 100;
		let good_message =
			Message::ismp_response(origin.address, H256::default(), deposit, BoundedVec::default());
		let erroneous_message = Message::ismp(origin.clone(), H256::default(), None, deposit);
		let messages = 5;
		let endowment = existential_deposit() + deposit * messages as Balance;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.with_messages(
				(0..messages - 1)
					.map(|i| (origin.account.clone(), i, good_message.clone(), deposit))
					.chain([(origin.account.clone(), messages - 1, erroneous_message, deposit)])
					.collect(),
			)
			.build()
			.execute_with(|| {
				let messages = (0..messages).collect::<Vec<_>>();
				let free_balance = Balances::free_balance(&origin.account);

				assert_noop!(remove(origin.clone(), &messages), Error::RequestPending);

				for message in messages {
					assert!(Messages::get(message).is_some());
				}
				assert_eq!(Balances::free_balance(&origin.account), free_balance);
			});
	}

	// Basic remove tests to ensure storage is cleaned.
	#[test]
	fn remove_ismp_message() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let commitment = H256::default();
		let id = 1;
		let deposit = 100;
		let message = Message::ismp(origin.clone(), commitment, None, deposit);
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), existential_deposit() + deposit)])
			.build()
			.execute_with(|| {
				Messages::insert(id, &message);
				IsmpRequests::insert(commitment, &id);
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin.account, deposit));

				assert_noop!(remove(origin.clone(), &[id]), Error::RequestPending);

				assert!(
					Messages::get(id).is_some(),
					"Message should not have been removed but has."
				);
				assert!(
					IsmpRequests::get(commitment).is_some(),
					"Message should not have been removed but has."
				);
			})
	}

	#[test]
	fn remove_ismp_response() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let commitment = H256::default();
		let id = 1;
		let deposit = 100;
		let message =
			Message::ismp_response(origin.address, commitment, deposit, BoundedVec::default());
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), existential_deposit() + deposit)])
			.build()
			.execute_with(|| {
				Messages::insert(id, &message);
				IsmpRequests::insert(commitment, &id);
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin.account, deposit));

				assert_ok!(remove(origin, &[id]));

				assert!(Messages::get(id).is_none(), "Message should have been removed but hasnt.");
				assert!(
					IsmpRequests::get(commitment).is_none(),
					"Request should have been removed but hasnt."
				);
			})
	}

	#[test]
	fn remove_ismp_timeout() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let commitment = H256::default();
		let deposit = 100;
		let callback_deposit = 100_000;
		let id = 1;
		let message =
			Message::ismp_timeout(origin.address, commitment, deposit, Some(callback_deposit));
		let endowment = existential_deposit() + deposit + callback_deposit;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.build()
			.execute_with(|| {
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin.account, deposit));
				assert_ok!(Fungibles::hold(&CallbackGas.into(), &origin.account, callback_deposit));

				Messages::insert(id, &message);
				IsmpRequests::insert(commitment, id);

				assert_ok!(remove(origin.clone(), &[id]));

				assert!(Messages::get(id).is_none(), "Message should have been removed but hasnt.");
				assert!(
					IsmpRequests::get(commitment).is_none(),
					"Request should have been removed but hasnt."
				);
				assert_eq!(Balances::total_balance_on_hold(&origin.account), 0);
			})
	}

	#[test]
	fn remove_xcm_query() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let query_id = 42;
		let id = 1;
		let deposit = 100;
		let message = Message::xcm_query(origin.clone(), query_id, None, deposit);
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), existential_deposit() + deposit)])
			.build()
			.execute_with(|| {
				Messages::insert(id, &message);
				XcmQueries::insert(query_id, &id);
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin.account, deposit));

				assert_noop!(remove(origin, &[id]), Error::RequestPending);
				assert!(
					Messages::get(id).is_some(),
					"Message should not have been removed but has"
				);
				assert!(
					XcmQueries::get(query_id).is_some(),
					"Message should not have been removed but has."
				);
			})
	}

	#[test]
	fn remove_xcm_response() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let query_id = 42;
		let id = 1;
		let message_deposit = 100;
		let message =
			Message::xcm_response(origin.address, query_id, message_deposit, Response::default());
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), existential_deposit() + message_deposit)])
			.build()
			.execute_with(|| {
				Messages::insert(id, &message);
				XcmQueries::insert(query_id, &id);
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin.account, message_deposit));

				assert_ok!(remove(origin, &[id]));

				assert!(Messages::get(id).is_none(), "Message should have been removed but hasnt");
				assert!(
					XcmQueries::get(query_id).is_none(),
					"Message should have been removed but hasnt."
				);
			})
	}

	#[test]
	fn remove_xcm_timeout() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let query_id = 42;
		let id = 1;
		let message_deposit = 100;
		let callback_deposit = 100_000;
		let message =
			Message::xcm_timeout(origin.address, query_id, message_deposit, Some(callback_deposit));
		let endowment = existential_deposit() + message_deposit + callback_deposit;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.build()
			.execute_with(|| {
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin.account, message_deposit));
				assert_ok!(Fungibles::hold(&CallbackGas.into(), &origin.account, callback_deposit));

				Messages::insert(id, &message);
				XcmQueries::insert(query_id, id);

				assert_ok!(remove(origin.clone(), &[id]));

				assert!(Messages::get(id).is_none(), "Message should have been removed but hasnt");
				assert!(
					XcmQueries::get(query_id).is_none(),
					"Message should have been removed but hasnt."
				);

				// Assert that all holds specified have been released
				assert_eq!(Balances::total_balance_on_hold(&origin.account), 0);
			})
	}

	// `remove` is no longer a dispatchable and only callable via a precompile, hence we simply
	// wrap calls to it in a transaction to simulate. See additional precompiles tests for
	// further assurances.
	fn remove(origin: Origin, messages: &[MessageId]) -> DispatchResult {
		with_transaction(|| -> TransactionOutcome<DispatchResult> {
			let result = super::remove::<Test>(origin, messages);
			match &result {
				Ok(_) => TransactionOutcome::Commit(result),
				Err(_) => TransactionOutcome::Rollback(result),
			}
		})
	}
}

mod xcm_response {
	use transports::xcm::tests::{deposit, new_query, xcm_response_fee};

	use super::{
		mock::{messaging::*, Messaging},
		*,
	};

	type BlockWeight = frame_system::BlockWeight<Test>;
	type Pallet = Messaging;

	#[test]
	fn message_not_found() {
		ExtBuilder::new().build().execute_with(|| {
			assert_noop!(
				Pallet::xcm_response(root(), 0, Default::default()),
				Error::MessageNotFound
			);
		})
	}

	#[test]
	fn timeout_messages_are_noop() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message_id = 1;
		let query_id = 42;
		let endowment = existential_deposit() + deposit() + xcm_response_fee();
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.with_message_id(message_id)
			.with_query_id(query_id)
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;

				assert_ok!(new_query(origin, RESPONSE_LOCATION, timeout, None));

				// Update the message to XcmTimedOut
				Messages::mutate(message_id, |message| {
					let Some(Message::XcmQuery { origin, query_id, message_deposit, .. }): &mut Option<
						Message,
					> = message
					else {
						panic!("No message!");
					};
					*message = Some(Message::xcm_timeout(
						origin.address,
						*query_id,
						*message_deposit,
						None,
					));
				});

				assert_noop!(
					Pallet::xcm_response(root(), query_id, Default::default()),
					Error::RequestTimedOut
				);
			})
	}

	#[test]
	fn assert_event_no_callback() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let id = 1;
		let query_id = 42;
		let response = Response::Null;
		let endowment = existential_deposit() + deposit() + xcm_response_fee();
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.with_message_id(id)
			.with_query_id(query_id)
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;
				assert_ok!(new_query(origin.clone(), RESPONSE_LOCATION, timeout, None));

				assert_ok!(Pallet::xcm_response(root(), query_id, response.clone()));

				assert!(events().contains(&Event::XcmResponseReceived {
					dest: origin.address,
					id,
					query_id,
					response
				}));
			})
	}

	#[test]
	fn assert_message_is_stored_for_polling_no_callback() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let id = 1;
		let query_id = 42;
		let response = Response::ExecutionResult(None);
		let endowment = existential_deposit() + deposit() + xcm_response_fee();
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.with_message_id(id)
			.with_query_id(query_id)
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;
				assert_ok!(new_query(origin, RESPONSE_LOCATION, timeout, None));

				assert_ok!(Pallet::xcm_response(root(), query_id, response.clone()));

				let Some(Message::XcmResponse { query_id: q, response: r, .. }): Option<Message> =
					Messages::get(id)
				else {
					panic!("wrong message type");
				};

				assert_eq!(q, query_id);
				assert_eq!(r, response);
			})
	}

	#[test]
	fn message_is_removed_after_successfull_callback_execution() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let id = 1;
		let query_id = 42;
		let response = Response::ExecutionResult(None);
		let callback =
			Callback::new(H160::zero(), Encoding::Scale, [1; 4], 100.into(), 100u8.into());
		let callback_fee = WeightToFee::weight_to_fee(&callback.gas_limit);
		let endowment = existential_deposit() + deposit() + xcm_response_fee() + callback_fee;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.with_message_id(id)
			.with_query_id(query_id)
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;

				assert_ok!(new_query(origin, RESPONSE_LOCATION, timeout, Some(callback)));

				assert_ok!(Pallet::xcm_response(root(), query_id, response.clone()));

				assert!(Messages::get(id).is_none());
				assert!(XcmQueries::get(query_id).is_none());
			})
	}

	#[test]
	fn message_deposit_returned_after_successfull_callback_execution() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let id = 1;
		let query_id = 42;
		let response = Response::ExecutionResult(None);
		let callback = Callback::new(H160::zero(), Encoding::Scale, [1; 4], Zero::zero(), 0);
		let endowment = existential_deposit() + deposit() + xcm_response_fee();
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.with_message_id(id)
			.with_query_id(query_id)
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;

				assert_ok!(new_query(origin.clone(), RESPONSE_LOCATION, timeout, Some(callback)));

				let held_balance_pre_release = Balances::total_balance_on_hold(&origin.account);
				assert_ne!(held_balance_pre_release, 0);

				assert_ok!(Pallet::xcm_response(root(), query_id, response.clone()));

				let held_balance_post_release = Balances::total_balance_on_hold(&origin.account);
				assert_eq!(held_balance_post_release, 0);
			})
	}

	// Dont include any callback weight so we can test the xcm_response blockweight mutation.
	#[test]
	fn assert_blockweight_mutation_no_callback() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let id = 1;
		let query_id = 42;
		let xcm_response = Response::ExecutionResult(None);
		let endowment = existential_deposit() + deposit() + xcm_response_fee();
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.with_message_id(id)
			.with_query_id(query_id)
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;

				let block_weight_pre_call =
					BlockWeight::get().get(DispatchClass::Normal).to_owned();

				assert_ne!(
					CallbackExecutor::execution_weight(),
					Zero::zero(),
					"Please set a callback executor execution_weight to run this test."
				);
				assert_ne!(
					WeightInfo::xcm_response(),
					Zero::zero(),
					"Please set an T::WeightInfo::xcm_response() to run this test."
				);
				assert_ok!(new_query(origin, RESPONSE_LOCATION, timeout, None));

				assert_ok!(Pallet::xcm_response(root(), query_id, xcm_response.clone()));

				let block_weight_post_call =
					BlockWeight::get().get(DispatchClass::Normal).to_owned();

				assert_eq!(
					block_weight_post_call - block_weight_pre_call,
					WeightInfo::xcm_response() + CallbackExecutor::execution_weight()
				)
			})
	}
}

mod call {
	use super::*;

	type BlockWeight = frame_system::pallet::BlockWeight<Test>;

	#[test]
	fn assert_error_event() {
		let origin = ALICE;
		let weight = Weight::from_parts(100_000, 100_000);
		let callback = Callback::new(H160::zero(), Encoding::Scale, [0u8; 4], weight, 100_000);
		let message_id = 1;
		let data = vec![100u8; 5];
		ExtBuilder::new().build().execute_with(|| {
			assert_ok!(call::<Test>(&origin, callback, &message_id, &data));

			System::assert_last_event(
				Event::WeightRefundErrored {
					message_id,
					error: DispatchError::Token(FundsUnavailable),
				}
				.into(),
			);
		})
	}

	// AlwaysSuccessfullCallbackExecutor should return half the weight of the callback.weight
	// TODO: there may be a better way of handling this case.
	#[test]
	fn block_weight_mutation_happens() {
		let origin = ALICE;
		let weight = Weight::from_parts(10_000_000, 10_000_000);
		let callback = Callback::new(H160::zero(), Encoding::Scale, [0u8; 4], weight, 10_000_000);
		let id = 1;
		let data = vec![100u8; 5];
		let callback_fee = <Test as Config>::WeightToFee::weight_to_fee(&weight);
		let endowment = existential_deposit() + callback_fee;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), endowment)])
			.build()
			.execute_with(|| {
				let block_weight_pre_call =
					BlockWeight::get().get(DispatchClass::Normal).to_owned();
				assert_ok!(Fungibles::hold(&CallbackGas.into(), &origin, callback_fee));

				assert_ok!(call::<Test>(&origin, callback, &id, &data));

				let block_weight_post_call =
					BlockWeight::get().get(DispatchClass::Normal).to_owned();
				assert_ne!(block_weight_post_call, Zero::zero());
				// callback weight used in tests is total / 2.
				assert_eq!(block_weight_post_call - block_weight_pre_call, weight / 2);
			})
	}
}

mod process_callback_weight {
	use super::*;

	#[test]
	fn ok_with_weight_returns_weight() {
		let weight = Weight::from_parts(100_000, 100_000);
		let result = DispatchResultWithPostInfo::Ok(PostDispatchInfo {
			actual_weight: Some(weight),
			pays_fee: Pays::Yes,
		});
		let max_weight = Weight::zero();
		assert_eq!(process_callback_weight(&result, max_weight), weight);
	}

	#[test]
	fn ok_without_weight_returns_max_weight() {
		let result = DispatchResultWithPostInfo::Ok(PostDispatchInfo {
			actual_weight: None,
			pays_fee: Pays::Yes,
		});
		let max_weight = Weight::from_parts(200_000, 200_000);
		assert_eq!(process_callback_weight(&result, max_weight), max_weight);
	}

	#[test]
	fn err_returns_max_weight() {
		let result = DispatchResultWithPostInfo::Err(DispatchErrorWithPostInfo {
			post_info: Default::default(),
			error: Error::InvalidMessage.into(),
		});
		let max_weight = Weight::from_parts(200_000, 200_000);
		assert_eq!(process_callback_weight(&result, max_weight), max_weight);
	}
}

mod deposit_callback_event {
	use super::*;

	#[test]
	fn emits_callback_executed_event_on_success() {
		let origin = ALICE;
		let message_id = 1;
		let weight = Weight::from_parts(100_000, 100_000);
		let callback = Callback::new(H160::zero(), Encoding::Scale, [0; 4], weight, 100_000);
		let result: DispatchResultWithPostInfo = Ok(PostDispatchInfo {
			actual_weight: Some(Weight::from_parts(1_000, 0)),
			pays_fee: Default::default(),
		});
		ExtBuilder::new().build().execute_with(|| {
			deposit_callback_event::<Test>(origin.clone(), message_id, &callback, &result);
			System::assert_last_event(
				Event::<Test>::CallbackExecuted { origin, id: message_id, callback }.into(),
			);
		});
	}

	#[test]
	fn emits_callback_failed_event_on_error() {
		let origin = BOB;
		let message_id = 2;
		let weight = Weight::from_parts(100_000, 100_000);
		let callback = Callback::new(H160::zero(), Encoding::Scale, [0; 4], weight, 100_000);
		let result = DispatchResultWithPostInfo::Err(DispatchErrorWithPostInfo {
			post_info: Default::default(),
			error: Error::InvalidMessage.into(),
		});
		ExtBuilder::new().build().execute_with(|| {
			deposit_callback_event::<Test>(origin.clone(), message_id, &callback, &result);

			System::assert_last_event(
				Event::CallbackFailed {
					origin,
					id: message_id,
					callback,
					error: result.unwrap_err(),
				}
				.into(),
			);
		});
	}
}

mod manage_fees {
	use mock::messaging::Treasury;

	use super::*;

	#[test]
	fn assert_payback_when_execution_weight_is_less_than_deposit_held() {
		let origin = ALICE;
		let actual_weight_executed = Weight::from_parts(50_000_000, 70_000_000);
		let callback_weight_reserved = Weight::from_parts(100_000_000, 100_000_000);
		let deposit = WeightToFee::weight_to_fee(&callback_weight_reserved);
		assert!(deposit != 0, "Please set an appropriate weight to fee implementation.");
		let endowment = existential_deposit() + deposit;
		let fee_account = Treasury::get();
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), endowment)])
			.build()
			.execute_with(|| {
				// Artificially take the deposit
				assert_ok!(Fungibles::hold(&CallbackGas.into(), &origin, deposit));

				let expected_refund = deposit - WeightToFee::weight_to_fee(&actual_weight_executed);
				assert!(expected_refund != 0);

				let fee_pot_payment = deposit - expected_refund;

				let fee_account_pre_handle = Balances::free_balance(&fee_account);
				let origin_balance_pre_handle = Balances::free_balance(&origin);

				assert_ok!(manage_fees::<Test>(
					&origin,
					actual_weight_executed,
					callback_weight_reserved
				));

				// origin should have been refunded by the tune of expected refund.
				// the fee pot should have been increased by fee_pot_payment.
				let fee_account_post_handle = Balances::free_balance(&fee_account);
				let origin_balance_post_handle = Balances::free_balance(&origin);

				assert_eq!(origin_balance_post_handle - origin_balance_pre_handle, expected_refund);
				assert_eq!(fee_account_post_handle, fee_account_pre_handle + fee_pot_payment);
			})
	}
}

pub fn events() -> Vec<Event<Test>> {
	let result = System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let RuntimeEvent::Messaging(inner) = e { Some(inner) } else { None })
		.collect();

	System::reset_events();
	result
}

fn existential_deposit() -> Balance {
	<ExistentialDeposit as Get<Balance>>::get()
}

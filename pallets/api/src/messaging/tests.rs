#![cfg(test)]
use codec::Encode;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::WithPostDispatchInfo,
	sp_runtime::{traits::Zero, DispatchError::BadOrigin},
	testing_prelude::bounded_vec,
	weights::Weight,
};
use pallet_nfts::{CollectionSetting, MintWitness, WeightInfo as NftsWeightInfoTrait};
use sp_core::H256;

use crate::{messaging::*, mock::*, Read};

fn events() -> Vec<Event<Test>> {
	let result = System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let crate::mock::RuntimeEvent::Messaging(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>();

	System::reset_events();

	result
}

// Tests for the remove extrinsic.
mod remove {
	use super::*;

	#[test]
	fn success_event() {
		new_test_ext().execute_with(|| {
			let deposit: Balance = 100;
			let m = Message::IsmpResponse {
				commitment: Default::default(),
				deposit,
				response: Default::default(),
			};
			let m_id = [0u8; 32];
			let m2_id = [1u8; 32];

			Messages::<Test>::insert(&ALICE, m_id, &m);
			Messages::<Test>::insert(&ALICE, m2_id, &m);

			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();
			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();

			assert_ok!(Messaging::remove(signed(ALICE), bounded_vec!(m_id, m2_id)));

			assert!(events()
				.contains(&Event::<Test>::Removed { origin: ALICE, messages: vec![m_id, m2_id] }));
		})
	}

	#[test]
	fn message_not_found() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Messaging::remove(signed(ALICE), bounded_vec!(Default::default())),
				Error::<Test>::MessageNotFound
			);
		})
	}

	#[test]
	fn multiple_messages_remove_works() {
		new_test_ext().execute_with(|| {
			let deposit: Balance = 100;
			// An ismp response can always be removed.
			let m = Message::IsmpResponse {
				commitment: Default::default(),
				deposit,
				response: Default::default(),
			};
			let m_id = [0; 32];
			let m2_id = [1; 32];
			let m3_id = [2; 32];

			Messages::<Test>::insert(&ALICE, m_id, &m);
			Messages::<Test>::insert(&ALICE, m2_id, &m);
			Messages::<Test>::insert(&ALICE, m3_id, &m);

			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();
			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();
			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();

			assert_ok!(Messaging::remove(signed(ALICE), bounded_vec!(m_id, m2_id, m3_id)));

			assert!(
				Messages::<Test>::get(&ALICE, m_id).is_none(),
				"Message should have been removed."
			);
			assert!(
				Messages::<Test>::get(&ALICE, m2_id).is_none(),
				"Message should have been removed."
			);
			assert!(
				Messages::<Test>::get(&ALICE, m3_id).is_none(),
				"Message should have been removed."
			);
		});
	}

	#[test]
	fn deposit_is_returned_if_try_remove_is_ok() {
		new_test_ext().execute_with(|| {
			let alice_initial_balance = Balances::free_balance(&ALICE);
			let deposit: Balance = 100;
			// An ismp response can always be removed.
			let m = Message::IsmpResponse {
				commitment: Default::default(),
				deposit,
				response: Default::default(),
			};
			let m_id = [0; 32];

			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();
			Messages::<Test>::insert(&ALICE, m_id, &m);

			let alice_balance_post_hold = Balances::free_balance(&ALICE);

			assert_ok!(Messaging::remove(signed(ALICE), bounded_vec!(m_id)));

			let alice_balance_post_remove = Balances::free_balance(&ALICE);

			assert_eq!(alice_initial_balance, alice_balance_post_remove);
			assert_eq!(alice_balance_post_remove, alice_balance_post_hold + deposit);
		});
	}

	#[test]
	fn deposit_is_not_returned_if_try_remove_is_noop() {
		new_test_ext().execute_with(|| {
			let alice_initial_balance = Balances::free_balance(&ALICE);
			let deposit: Balance = 100;

			// Ismp message with status of Ok is considered pending.
			let m = Message::Ismp { commitment: H256::default(), callback: None, deposit };
			let m_id = [0; 32];

			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();
			Messages::<Test>::insert(&ALICE, m_id, &m);

			let alice_balance_post_hold = Balances::free_balance(&ALICE);

			assert_noop!(
				Messaging::remove(signed(ALICE), bounded_vec!(m_id)),
				Error::<Test>::RequestPending
			);

			let alice_balance_post_remove = Balances::free_balance(&ALICE);

			assert_eq!(alice_initial_balance, alice_balance_post_remove + deposit);
			assert_eq!(alice_balance_post_remove, alice_balance_post_hold);
		});
	}

	#[test]
	fn multiple_messages_rolls_back_if_one_fails() {
		new_test_ext().execute_with(|| {
			let deposit: Balance = 100;
			let alice_initial_balance = Balances::free_balance(&ALICE);
			let good_message = Message::IsmpResponse {
				commitment: Default::default(),
				deposit: 0,
				response: Default::default(),
			};

			let erroneous_message =
				Message::Ismp { commitment: H256::default(), callback: None, deposit: 100 };

			let good_id_1 = [0; 32];
			let good_id_2 = [1; 32];
			let good_id_3 = [2; 32];
			let good_id_4 = [3; 32];
			let erroneous_id_1 = [4; 32];

			Messages::<Test>::insert(&ALICE, good_id_1, &good_message);
			Messages::<Test>::insert(&ALICE, good_id_2, &good_message);
			Messages::<Test>::insert(&ALICE, good_id_3, &good_message);
			Messages::<Test>::insert(&ALICE, good_id_4, &good_message);
			Messages::<Test>::insert(&ALICE, erroneous_id_1, &erroneous_message);

			// gonna do 5 messages.
			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();
			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();
			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();
			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();
			<Test as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&ALICE,
				deposit,
			)
			.unwrap();

			let alice_balance_post_hold = Balances::free_balance(&ALICE);

			assert_noop!(
				Messaging::remove(
					signed(ALICE),
					bounded_vec!(good_id_1, good_id_2, good_id_3, good_id_4, erroneous_id_1)
				),
				Error::<Test>::RequestPending
			);

			assert!(Messages::<Test>::get(&ALICE, good_id_1).is_some());
			assert!(Messages::<Test>::get(&ALICE, good_id_2).is_some());
			assert!(Messages::<Test>::get(&ALICE, good_id_3).is_some());
			assert!(Messages::<Test>::get(&ALICE, good_id_4).is_some());
			assert!(Messages::<Test>::get(&ALICE, erroneous_id_1).is_some());

			let alice_balance_post_remove = Balances::free_balance(&ALICE);
			assert_eq!(alice_initial_balance, alice_balance_post_hold + deposit * 5);
			assert_eq!(alice_balance_post_remove, alice_balance_post_hold);
		});
	}

	// Basic remove tests to ensure storage is cleaned.
	#[test]
	fn remove_ismp_message() {
		new_test_ext().execute_with(|| {
			let commitment = H256::default();
			let message_id = [0u8; 32];
			let deposit = 100;
			let m = Message::Ismp { commitment, callback: None, deposit };
			Messages::<Test>::insert(&ALICE, &message_id, &m);
			IsmpRequests::<Test>::insert(&commitment, (&ALICE, &message_id));
			<Test as Config>::Deposit::hold(&HoldReason::Messaging.into(), &ALICE, deposit)
				.unwrap();

			assert_noop!(
				Messaging::remove(signed(ALICE), bounded_vec!(message_id)),
				Error::<Test>::RequestPending
			);

			assert!(
				Messages::<Test>::get(&ALICE, &message_id).is_some(),
				"Message should not have been removed but has."
			);
			assert!(
				IsmpRequests::<Test>::get(&commitment).is_some(),
				"Message should not have been removed but has."
			);
		})
	}

	#[test]
	fn remove_ismp_response() {
		new_test_ext().execute_with(|| {
			let commitment = H256::default();
			let message_id = [0u8; 32];
			let deposit = 100;

			let m = Message::IsmpResponse { commitment, response: bounded_vec!(), deposit };
			Messages::<Test>::insert(&ALICE, &message_id, &m);
			IsmpRequests::<Test>::insert(&commitment, (&ALICE, &message_id));
			<Test as Config>::Deposit::hold(&HoldReason::Messaging.into(), &ALICE, deposit)
				.unwrap();

			assert_ok!(Messaging::remove(signed(ALICE), bounded_vec!(message_id)));

			assert!(
				Messages::<Test>::get(&ALICE, &message_id).is_none(),
				"Message should have been removed but hasnt."
			);
			assert!(
				IsmpRequests::<Test>::get(&commitment).is_none(),
				"Request should have been removed but hasnt."
			);
		})
	}

	#[test]
	fn remove_ismp_timeout() {
		new_test_ext().execute_with(|| {
			let commitment = H256::default();
			let message_id = [0u8; 32];
			let deposit = 100;

			let m = Message::IsmpTimeout { commitment, deposit };
			Messages::<Test>::insert(&ALICE, &message_id, &m);
			IsmpRequests::<Test>::insert(&commitment, (&ALICE, &message_id));
			<Test as Config>::Deposit::hold(&HoldReason::Messaging.into(), &ALICE, deposit)
				.unwrap();

			assert_ok!(Messaging::remove(signed(ALICE), bounded_vec!(message_id)));

			assert!(
				Messages::<Test>::get(&ALICE, &message_id).is_none(),
				"Message should have been removed but hasnt."
			);
			assert!(
				IsmpRequests::<Test>::get(&commitment).is_none(),
				"Request should have been removed but hasnt."
			);
		})
	}

	#[test]
	fn remove_xcm_query() {
		new_test_ext().execute_with(|| {
			let query_id = 0;
			let message_id = [0u8; 32];
			let deposit = 100;

			let m = Message::XcmQuery { query_id, callback: None, deposit };
			Messages::<Test>::insert(&ALICE, &message_id, &m);
			XcmQueries::<Test>::insert(query_id, (&ALICE, &message_id));
			<Test as Config>::Deposit::hold(&HoldReason::Messaging.into(), &ALICE, deposit)
				.unwrap();

			assert_noop!(
				Messaging::remove(signed(ALICE), bounded_vec!(message_id)),
				Error::<Test>::RequestPending
			);
			assert!(
				Messages::<Test>::get(&ALICE, &message_id).is_some(),
				"Message should not have been removed but has"
			);
			assert!(
				XcmQueries::<Test>::get(query_id).is_some(),
				"Message should not have been removed but has."
			);
		})
	}

	#[test]
	fn remove_xcm_response() {
		new_test_ext().execute_with(|| {
			let query_id = 0;
			let message_id = [0u8; 32];
			let deposit = 100;
			let m = Message::XcmResponse { query_id, deposit, response: Default::default() };
			Messages::<Test>::insert(&ALICE, &message_id, &m);
			XcmQueries::<Test>::insert(query_id, (&ALICE, &message_id));
			<Test as Config>::Deposit::hold(&HoldReason::Messaging.into(), &ALICE, deposit)
				.unwrap();

			assert_ok!(Messaging::remove(signed(ALICE), bounded_vec!(message_id)));

			assert!(
				Messages::<Test>::get(&ALICE, &message_id).is_none(),
				"Message should have been removed but hasnt"
			);
			assert!(
				XcmQueries::<Test>::get(query_id).is_none(),
				"Message should have been removed but hasnt."
			);
		})
	}

	#[test]
	fn remove_xcm_timeout() {
		new_test_ext().execute_with(|| {
			let query_id = 0;
			let message_id = [0u8; 32];
			let deposit = 100;
			let m = Message::XcmTimeout { query_id, deposit };

			<Test as Config>::Deposit::hold(&HoldReason::Messaging.into(), &ALICE, deposit)
				.unwrap();

			Messages::<Test>::insert(&ALICE, &message_id, &m);
			XcmQueries::<Test>::insert(query_id, (&ALICE, &message_id));

			assert_ok!(Messaging::remove(signed(ALICE), bounded_vec!(message_id)));

			assert!(
				Messages::<Test>::get(&ALICE, &message_id).is_none(),
				"Message should have been removed but hasnt"
			);
			assert!(
				XcmQueries::<Test>::get(query_id).is_none(),
				"Message should have been removed but hasnt."
			);
		})
	}
}

mod xcm_new_query {

	use super::*;

	#[test]
	fn success_assert_last_event() {
		new_test_ext().execute_with(|| {
			let timeout = System::block_number() + 1;
			let message_id = [0; 32];
			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				None,
			));
			assert!(events().contains(&Event::<Test>::XcmQueryCreated {
				origin: ALICE,
				id: message_id,
				query_id: 0,
				callback: None
			}));
		})
	}

	#[test]
	fn message_id_already_exists() {
		new_test_ext().execute_with(|| {
			let message_id = [0; 32];
			let timeout = System::block_number() + 1;

			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				None,
			));

			assert_noop!(
				Messaging::xcm_new_query(
					signed(ALICE),
					message_id,
					RESPONSE_LOCATION,
					timeout,
					None,
				),
				Error::<Test>::MessageExists
			);
		})
	}

	#[test]
	fn takes_deposit() {
		new_test_ext().execute_with(|| {
			let timeout = System::block_number() + 1;
			let expected_deposit = calculate_protocol_deposit::<
				Test,
				<Test as Config>::OnChainByteFee,
			>(ProtocolStorageDeposit::XcmQueries)
			.saturating_add(calculate_message_deposit::<Test, <Test as Config>::OnChainByteFee>());

			assert!(
				expected_deposit > 0,
				"set an onchain byte fee with T::OnChainByteFee to run this test."
			);

			let alices_balance_pre_hold = Balances::free_balance(&ALICE);

			let message_id = [0; 32];
			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				None,
			));

			let alices_balance_post_hold = Balances::free_balance(&ALICE);

			assert_eq!(alices_balance_pre_hold - alices_balance_post_hold, expected_deposit);
		});
	}

	#[test]
	fn assert_state() {
		new_test_ext().execute_with(|| {
			// Looking for an item in Messages and XcmQueries.
			let message_id = [0; 32];
			let expected_callback =
				Callback { selector: [1; 4], weight: 100.into(), spare_weight_creditor: BOB };
			let timeout = System::block_number() + 1;
			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				Some(expected_callback.clone()),
			));
			let m = Messages::<Test>::get(ALICE, message_id)
				.expect("should exist after xcm_new_query.");
			if let Message::XcmQuery { query_id, callback, deposit } = m {
				assert_eq!(query_id, 0);
				assert_eq!(callback, Some(expected_callback));
			} else {
				panic!("Wrong message type.")
			}

			assert_eq!(XcmQueries::<Test>::get(0), Some((ALICE, message_id)));
		})
	}

	#[test]
	fn xcm_timeouts_must_be_in_the_future() {
		new_test_ext().execute_with(|| {
			let message_id = [0; 32];
			let timeout = System::block_number();
			assert_noop!(
				Messaging::xcm_new_query(
					signed(ALICE),
					message_id,
					RESPONSE_LOCATION,
					timeout,
					None
				),
				Error::<Test>::FutureTimeoutMandatory
			);
		})
	}
}

mod xcm_response {
	use super::*;

	#[test]
	fn message_not_found() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Messaging::xcm_response(root(), 0, Default::default()),
				Error::<Test>::MessageNotFound
			);
		})
	}

	#[test]
	fn timeout_messages_are_noop() {
		new_test_ext().execute_with(|| {
			let message_id = [0; 32];
			let timeout = System::block_number() + 1;
			let mut generated_query_id = 0;

			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				None,
			));

			// Update the message to XcmTimedOut
			Messages::<Test>::mutate(&ALICE, &message_id, |message| {
				let Some(Message::XcmQuery { query_id, deposit, .. }): &mut Option<Message<Test>> =
					message
				else {
					panic!("No message!");
				};
				generated_query_id = *query_id;
				*message = Some(Message::XcmTimeout { query_id: *query_id, deposit: *deposit });
			});

			assert_noop!(
				Messaging::xcm_response(root(), generated_query_id, Default::default()),
				Error::<Test>::RequestTimedOut
			);
		})
	}

	#[test]
	fn assert_event_no_callback() {
		new_test_ext().execute_with(|| {
			let message_id = [0; 32];
			let timeout = System::block_number() + 1;
			let mut generated_query_id = 0;
			let xcm_response = Response::Null;

			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				None,
			));

			assert_ok!(Messaging::xcm_response(root(), generated_query_id, xcm_response));

			assert!(events().contains(&Event::<Test>::XcmResponseReceived {
				dest: ALICE,
				id: message_id,
				query_id: generated_query_id,
				response: Response::Null
			}));
		})
	}

	#[test]
	fn assert_message_is_stored_for_polling_no_callback() {
		new_test_ext().execute_with(|| {
			let message_id = [0; 32];
			let timeout = System::block_number() + 1;
			let mut expected_query_id = 0;
			let xcm_response = Response::ExecutionResult(None);

			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				None,
			));

			assert_ok!(Messaging::xcm_response(root(), expected_query_id, xcm_response.clone()));
			let Some(Message::XcmResponse { query_id, deposit, response }): Option<Message<Test>> =
				Messages::get(&ALICE, message_id)
			else {
				panic!("wrong message type");
			};

			assert_eq!(query_id, expected_query_id);
			assert_eq!(xcm_response, response);
		})
	}

	#[test]
	fn message_is_removed_after_successfull_callback_execution() {
		new_test_ext().execute_with(|| {
			let message_id = [0; 32];
			let timeout = System::block_number() + 1;
			let mut expected_query_id = 0;
			let xcm_response = Response::ExecutionResult(None);
			let callback =
				Callback { selector: [1; 4], weight: 100.into(), spare_weight_creditor: BOB };

			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				Some(callback),
			));

			assert_ok!(Messaging::xcm_response(root(), expected_query_id, xcm_response.clone()));
			assert!(Messages::<Test>::get(&ALICE, &message_id).is_none());
			assert!(XcmQueries::<Test>::get(expected_query_id).is_none());
		})
	}

	#[test]
	fn deposit_returned_after_successfull_callback_execution() {
		new_test_ext().execute_with(|| {
			let message_id = [0; 32];
			let timeout = System::block_number() + 1;
			let mut expected_query_id = 0;
			let xcm_response = Response::ExecutionResult(None);
			let callback =
				Callback { selector: [1; 4], weight: 100.into(), spare_weight_creditor: BOB };
			let expected_deposit = calculate_protocol_deposit::<
				Test,
				<Test as crate::messaging::Config>::OnChainByteFee,
			>(ProtocolStorageDeposit::XcmQueries) +
				calculate_message_deposit::<
					Test,
					<Test as crate::messaging::Config>::OnChainByteFee,
				>();

			let alice_balance_pre_hold = Balances::free_balance(&ALICE);

			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				Some(callback),
			));

			let alice_balance_post_hold = Balances::free_balance(&ALICE);

			assert_ok!(Messaging::xcm_response(root(), expected_query_id, xcm_response.clone()));

			let alice_balance_post_release = Balances::free_balance(&ALICE);

			assert_eq!(alice_balance_pre_hold - alice_balance_post_hold, expected_deposit);
			assert_eq!(alice_balance_post_release, alice_balance_pre_hold);
		})
	}
}

mod hooks {
	use super::*;

	#[test]
	fn xcm_queries_expire_on_expiry_block() {
		new_test_ext().execute_with(|| {
			let message_id = [0; 32];
			let timeout = System::block_number() + 10;
			let xcm_response = Response::ExecutionResult(None);

			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				None,
			));

			run_to(timeout + 1);

			let Some(Message::XcmTimeout { .. }): Option<Message<Test>> =
				Messages::get(&ALICE, message_id)
			else {
				panic!("Message should be timedout!")
			};
		})
	}
}

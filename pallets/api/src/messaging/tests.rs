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

// All the tests for the Message enum impls.
mod message {
	use super::*;

	// Basic remove tests to ensure storage is cleaned.
	#[test]
	fn remove_ismp_message() {
		new_test_ext().execute_with(|| {
			let commitment = H256::default();
			let message_id = [0u8; 32];
			let m = Message::Ismp {
				commitment,
				callback: None,
				deposit: 100,
				status: QueryStatus::Pending,
			};
			Messages::<Test>::insert(&ALICE, &message_id, &m);
			IsmpRequests::<Test>::insert(&commitment, (&ALICE, &message_id));
			m.remove(&ALICE, &message_id);
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
	fn remove_ismp_response() {
		new_test_ext().execute_with(|| {
			let commitment = H256::default();
			let message_id = [0u8; 32];
			let m = Message::IsmpResponse {
				commitment,
				response: bounded_vec!(),
				deposit: 100,
				status: ResponseStatus::Received,
			};
			Messages::<Test>::insert(&ALICE, &message_id, &m);
			IsmpRequests::<Test>::insert(&commitment, (&ALICE, &message_id));
			m.remove(&ALICE, &message_id);
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
			let m = Message::XcmQuery {
				query_id,
				callback: None,
				deposit: 0,
				status: QueryStatus::Pending,
			};
			Messages::<Test>::insert(&ALICE, &message_id, &m);
			XcmQueries::<Test>::insert(query_id, (&ALICE, &message_id));
			m.remove(&ALICE, &message_id);
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
	fn remove_xcm_response() {
		new_test_ext().execute_with(|| {
			let query_id = 0;
			let message_id = [0u8; 32];
			let m = Message::XcmResponse {
				query_id,
				deposit: 0,
				response: Default::default(),
				status: ResponseStatus::Received,
			};
			Messages::<Test>::insert(&ALICE, &message_id, &m);
			XcmQueries::<Test>::insert(query_id, (&ALICE, &message_id));
			m.remove(&ALICE, &message_id);
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

	// Basic try_remove tests to ensure validation of message status is correct.
	#[test]
	fn try_remove_ismp_message_is_noop_when_status_is_ok() {
		new_test_ext().execute_with(|| {
			let message_id: MessageId = [0u8; 32];
			let m = Message::Ismp {
				commitment: H256::default(),
				callback: None,
				deposit: 100,
				status: QueryStatus::Pending,
			};
			Messages::<Test>::insert(&ALICE, &message_id, &m);
			assert_noop!(m.try_remove(&ALICE, &message_id), Error::<Test>::RequestPending);
			assert!(
				Messages::<Test>::get(ALICE, &message_id).is_some(),
				"Message has been deleted when it should exist."
			);
		});
	}

	#[test]
	fn try_remove_ismp_message_is_ok_when_status_is_timeout() {
		new_test_ext().execute_with(|| {
			let message_id: MessageId = [0u8; 32];
			let m = Message::Ismp {
				commitment: H256::default(),
				callback: None,
				deposit: 0,
				status: QueryStatus::Timeout,
			};
			Messages::<Test>::insert(&ALICE, message_id, &m);
			assert_ok!(m.try_remove(&ALICE, &message_id));
			assert!(Messages::<Test>::get(&ALICE, message_id).is_none());
		});
	}

	#[test]
	fn try_remove_ismp_message_is_ok_when_status_is_err() {
		new_test_ext().execute_with(|| {
			let message_id: MessageId = [0u8; 32];
			let m = Message::Ismp {
				commitment: H256::default(),
				callback: None,
				deposit: 0,
				status: QueryStatus::Err(Error::<Test>::IsmpDispatchFailed.into()),
			};
			Messages::<Test>::insert(&ALICE, message_id, &m);
			assert_ok!(m.try_remove(&ALICE, &message_id));
			assert!(Messages::<Test>::get(&ALICE, message_id).is_none());
		});
	}

	#[test]
	fn try_remove_ismp_response_is_ok_any_status() {
		new_test_ext().execute_with(|| {
			let m_id = [0; 32];
			let m2_id = [1; 32];
			let m3_id = [2; 32];

			let m = Message::IsmpResponse {
				commitment: Default::default(),
				deposit: 0,
				response: Default::default(),
				status: ResponseStatus::Received,
			};
			let m2 = Message::IsmpResponse {
				commitment: Default::default(),
				deposit: 0,
				response: Default::default(),
				status: ResponseStatus::Received,
			};

			let m3 = Message::IsmpResponse {
				commitment: Default::default(),
				deposit: 0,
				response: Default::default(),
				status: ResponseStatus::Received,
			};

			Messages::<Test>::insert(&ALICE, m_id, &m);
			Messages::<Test>::insert(&ALICE, m2_id, &m2);
			Messages::<Test>::insert(&ALICE, m3_id, &m3);

			assert_ok!(m.try_remove(&ALICE, &m_id));
			assert_ok!(m.try_remove(&ALICE, &m2_id));
			assert_ok!(m.try_remove(&ALICE, &m3_id));

			assert!(Messages::<Test>::get(&ALICE, m_id).is_none());
			assert!(Messages::<Test>::get(&ALICE, m2_id).is_none());
			assert!(Messages::<Test>::get(&ALICE, m3_id).is_none());
		});
	}

	#[test]
	fn try_remove_xcm_query_is_noop_when_status_is_ok() {
		new_test_ext().execute_with(|| {
			let message_id: MessageId = [0u8; 32];
			let m = Message::XcmQuery {
				query_id: 0,
				callback: None,
				deposit: 0,
				status: QueryStatus::Pending,
			};
			Messages::<Test>::insert(&ALICE, message_id, &m);
			assert_noop!(m.try_remove(&ALICE, &message_id), Error::<Test>::RequestPending);
			assert!(Messages::<Test>::get(&ALICE, message_id).is_some());
		});
	}

	#[test]
	fn try_remove_xcm_query_is_ok_when_status_is_timeout() {
		new_test_ext().execute_with(|| {
			let message_id: MessageId = [0u8; 32];
			let m = Message::XcmQuery {
				query_id: 0,
				callback: None,
				deposit: 0,
				status: QueryStatus::Timeout,
			};
			Messages::<Test>::insert(&ALICE, message_id, &m);
			assert_ok!(m.try_remove(&ALICE, &message_id));
			assert!(Messages::<Test>::get(&ALICE, message_id).is_none());
		});
	}

	#[test]
	fn try_remove_xcm_query_is_ok_when_status_is_err() {
		new_test_ext().execute_with(|| {
			let message_id: MessageId = [0u8; 32];
			let m = Message::XcmQuery {
				query_id: 0,
				callback: None,
				deposit: 0,
				status: QueryStatus::Err(Error::<Test>::IsmpDispatchFailed.into()),
			};
			Messages::<Test>::insert(&ALICE, message_id, &m);
			assert_ok!(m.try_remove(&ALICE, &message_id));
			assert!(Messages::<Test>::get(&ALICE, message_id).is_none());
		});
	}

	#[test]
	fn try_remove_xcm_response_is_ok_any_status() {
		new_test_ext().execute_with(|| {
			let m_id = [0; 32];
			let m2_id = [1; 32];
			let m3_id = [2; 32];

			let m = Message::XcmResponse {
				query_id: 0,
				deposit: 0,
				response: Default::default(),
				status: ResponseStatus::Received,
			};

			let m2 = Message::XcmResponse {
				query_id: 0,
				deposit: 0,
				response: Default::default(),
				status: ResponseStatus::Received,
			};

			let m3 = Message::XcmResponse {
				query_id: 0,
				deposit: 0,
				response: Default::default(),
				status: ResponseStatus::Err(Error::<Test>::InvalidMessage.into()),
			};

			Messages::<Test>::insert(&ALICE, m_id, &m);
			Messages::<Test>::insert(&ALICE, m2_id, &m2);
			Messages::<Test>::insert(&ALICE, m3_id, &m3);

			assert_ok!(m.try_remove(&ALICE, &m_id));
			assert_ok!(m.try_remove(&ALICE, &m2_id));
			assert_ok!(m.try_remove(&ALICE, &m3_id));

			assert!(Messages::<Test>::get(&ALICE, m_id).is_none());
			assert!(Messages::<Test>::get(&ALICE, m2_id).is_none());
			assert!(Messages::<Test>::get(&ALICE, m3_id).is_none());
		});
	}

	// Basic release deposit to ensure quantites are correct.
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
				status: ResponseStatus::Received,
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
				status: ResponseStatus::Received,
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
				status: ResponseStatus::Received,
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
			let m = Message::Ismp {
				commitment: H256::default(),
				callback: None,
				deposit,
				status: QueryStatus::Pending,
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
				status: ResponseStatus::Received,
			};

			let erroneous_message = Message::Ismp {
				commitment: H256::default(),
				callback: None,
				deposit: 100,
				status: QueryStatus::Pending,
			};

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
}

mod xcm_new_query {

	use super::*;

	#[test]
	fn success_assert_last_event() {
		new_test_ext().execute_with(|| {
			let timeout = 0;
			let message_id = [0; 32];
			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				Default::default(),
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
			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				Default::default(),
				None,
			));

			assert_noop!(
				Messaging::xcm_new_query(
					signed(ALICE),
					message_id,
					RESPONSE_LOCATION,
					Default::default(),
					None,
				),
				Error::<Test>::MessageExists
			);
		})
	}

	#[test]
	fn takes_deposit() {
		new_test_ext().execute_with(|| {
			let timeout = 0;
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
				Callback { selector: [0; 4], weight: 100.into(), spare_weight_creditor: BOB };
			let timeout = 0;
			assert_ok!(Messaging::xcm_new_query(
				signed(ALICE),
				message_id,
				RESPONSE_LOCATION,
				timeout,
				Some(expected_callback.clone()),
			));
			let m = Messages::<Test>::get(ALICE, message_id)
				.expect("should exist after xcm_new_query.");
			if let Message::XcmQuery { query_id, callback, deposit, status } = m {
				assert_eq!(query_id, 0);
				assert_eq!(callback, Some(expected_callback));
				assert_eq!(status, QueryStatus::Pending);
			} else {
				panic!("Wrong message type.")
			}

			assert_eq!(XcmQueries::<Test>::get(0), Some((ALICE, message_id)));
		})
	}

	#[test]
	fn xcm_timeouts_must_be_in_the_future() {
		new_test_ext().execute_with(|| {
			
		})
	}

}


mod xcm_response {


}

mod hooks {
	use super::*;

	fn xcm_queries_expire_on_expiry_block() {

	}
}
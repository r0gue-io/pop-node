use codec::Encode;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::WithPostDispatchInfo,
	sp_runtime::{traits::Zero, BoundedVec, DispatchError::BadOrigin},
	weights::Weight,
};
use pallet_nfts::{CollectionSetting, MintWitness, WeightInfo as NftsWeightInfoTrait};

use crate::{
	mock::*,
	Read,
    messaging::*,
};
use sp_core::H256;
use frame_support::testing_prelude::bounded_vec;

    mod remove {
        use super::*;
        #[test]
        fn ismp_message_is_noop_when_status_is_ok() {
            new_test_ext().execute_with(|| {
                let m = Message::Ismp { commitment: H256::default(), callback: None, deposit: 100, status: MessageStatus::Ok };
                let message_id: MessageId =  [0u8; 32];
                Messages::<Test>::insert(&ALICE, message_id, &m);
                assert_noop!(Messaging::remove(signed(ALICE), bounded_vec!(message_id)), Error::<Test>::RequestPending);
                assert!(Messages::<Test>::get(ALICE, message_id).is_some(), "Message has been deleted when it should exist.");
            });
        }

        #[test]
        fn ismp_message_is_removed_and_deposit_returned_when_status_is_timeout() {
            new_test_ext().execute_with(|| {
                let deposit: Balance = 100;
                let message_id: MessageId =  [0u8; 32];
                let m = Message::Ismp { commitment: H256::default(), callback: None, deposit, status: MessageStatus::Timeout };
                let alice_balance_pre_hold = Balances::free_balance(&ALICE);

                <Test as crate::messaging::Config>::Deposit::hold(&HoldReason::Messaging.into(), &ALICE, deposit).unwrap();

                let alice_balance_post_hold = Balances::free_balance(&ALICE);

                Messages::<Test>::insert(&ALICE, message_id, &m);
                assert_ok!(Messaging::remove(signed(ALICE), bounded_vec!(message_id)));

                let alice_balance_post_remove = Balances::free_balance(&ALICE);

                assert_eq!(alice_balance_post_hold + deposit, alice_balance_pre_hold, "deposit amount is incorrect");
                assert_eq!(alice_balance_post_remove, alice_balance_pre_hold, "alice balance has been mutated when it shouldnt have.");
                assert!(Messages::<Test>::get(&ALICE, message_id).is_none());
            });
        }

        #[test]
        fn ismp_message_is_removed_and_deposit_returned_when_status_is_err() {
            new_test_ext().execute_with(|| {
                let deposit: Balance = 100;
                let message_id: MessageId =  [0u8; 32];
                let m = Message::Ismp { commitment: H256::default(), callback: None, deposit, status: MessageStatus::Err(Error::<Test>::IsmpDispatchFailed.into()) };
                let alice_balance_pre_hold = Balances::free_balance(&ALICE);

                <Test as crate::messaging::Config>::Deposit::hold(&HoldReason::Messaging.into(), &ALICE, deposit).unwrap();

                let alice_balance_post_hold = Balances::free_balance(&ALICE);

                Messages::<Test>::insert(&ALICE, message_id, &m);
                assert_ok!(Messaging::remove(signed(ALICE), bounded_vec!(message_id)));

                let alice_balance_post_remove = Balances::free_balance(&ALICE);

                assert_eq!(alice_balance_post_hold + deposit, alice_balance_pre_hold, "deposit amount is incorrect");
                assert_eq!(alice_balance_post_remove, alice_balance_pre_hold, "alice balance has been mutated when it shouldnt have.");
                assert!(Messages::<Test>::get(&ALICE, message_id).is_none());
            });
        }

        #[test]
        fn ismp_response_message_can_always_be_removed() {
            new_test_ext().execute_with(|| {
                
            });
        }
        #[test]
        fn xcm_queries_cannot_be_removed () {
            new_test_ext().execute_with(|| {
                let m_1_id = [0;32];
                let m_1 = Message::XcmQuery { query_id: 0, callback: None, deposit: 0, status: MessageStatus::Ok };
                let m_2_id = [1;32];
                let m_2 = Message::XcmQuery { query_id: 1, callback: None, deposit: 0, status: MessageStatus::Timeout };
                let m_3_id = [2;32];
                let m_3 = Message::XcmQuery { query_id: 2, callback: None, deposit: 0, status: MessageStatus::Err(Error::<Test>::InvalidQuery.into())};
                Messages::<Test>::insert(&ALICE, m_1_id, &m_1);
                Messages::<Test>::insert(&ALICE, m_2_id, &m_2);
                Messages::<Test>::insert(&ALICE, m_3_id, &m_3);
                assert_noop!(Messaging::remove(signed(ALICE), bounded_vec!(m_1_id)), Error::<Test>::RequestPending);
                assert_noop!(Messaging::remove(signed(ALICE), bounded_vec!(m_2_id)), Error::<Test>::RequestPending);
                assert_noop!(Messaging::remove(signed(ALICE), bounded_vec!(m_3_id)), Error::<Test>::RequestPending);
                assert!(Messages::<Test>::get(ALICE, &m_1_id).is_some(), "Message has been deleted when it should still exist.");
                assert!(Messages::<Test>::get(ALICE, &m_2_id).is_some(), "Message has been deleted when it should still exist.");
                assert!(Messages::<Test>::get(ALICE, &m_3_id).is_some(), "Message has been deleted when it should still exist.");
            });
        }

        #[test]
        fn xcm_response_messages_can_always_be_removed() {
            new_test_ext().execute_with(|| {
    
            });
        }

        #[test]
        fn multiple_messages_remove_works() {
            new_test_ext().execute_with(|| {
    
            });
        }

        #[test]
        fn multiple_messages_remove_ignores_erroneous_removes_and_continues() {
            new_test_ext().execute_with(|| {
    
            });
        }

        #[test]
        fn origin_can_only_remove_messages_related_to_itself() {
            new_test_ext().execute_with(|| {
    
            });
        }
    }
    
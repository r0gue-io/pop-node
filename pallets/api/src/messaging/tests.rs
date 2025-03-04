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
            });
        }

        #[test]
        fn ismp_message_is_removed_and_deposit_returned_when_status_is_timeout() {
            new_test_ext().execute_with(|| {
    
            });
        }

        #[test]
        fn ismp_message_is_removed_and_deposit_returned_when_status_is_err() {
            new_test_ext().execute_with(|| {
    
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
    }
    
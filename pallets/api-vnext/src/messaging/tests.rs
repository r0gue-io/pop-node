use frame_support::{
	assert_noop, assert_ok,
	dispatch::PostDispatchInfo,
	storage::{with_transaction, TransactionOutcome},
	traits::fungible::InspectHold,
	weights::WeightToFee as _,
};
use sp_runtime::TokenError::FundsUnavailable;
use HoldReason::*;

use super::*;
use crate::mock::*;

type Error = super::Error<Test>;
type Fungibles = <Test as Config>::Fungibles;
type IsmpRequests = super::IsmpRequests<Test>;
type Messages = super::Messages<Test>;
type XcmQueries = super::XcmQueries<Test>;

mod remove {
	use super::*;

	#[test]
	fn message_not_found() {
		let origin = ALICE;
		let message = 0;
		ExtBuilder::new().build().execute_with(|| {
			assert_noop!(remove(&origin, &[message]), Error::MessageNotFound);
		})
	}

	#[test]
	fn multiple_messages_remove_works() {
		let origin = ALICE;
		let deposit: Balance = 100;
		// An ismp response can always be removed.
		let message = Message::ismp_response(H256::default(), deposit, BoundedVec::default());
		let messages = 3;
		let endowment = existential_deposit() + deposit * messages as Balance;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), endowment)])
			.with_messages(
				(0..messages).map(|i| (origin.clone(), i, message.clone(), deposit)).collect(),
			)
			.build()
			.execute_with(|| {
				let messages = (0..messages).collect::<Vec<_>>();
				assert_ok!(remove(&origin, &messages));

				for id in messages {
					assert!(
						Messages::get(&origin, id).is_none(),
						"message should have been removed."
					);
				}
			});
	}

	#[test]
	fn deposit_is_returned_if_try_remove_is_ok() {
		let origin = ALICE;
		let deposit: Balance = 100;
		// An ismp response can always be removed.
		let message = Message::ismp_response(H256::default(), deposit, BoundedVec::default());
		let id = 1;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), existential_deposit() + deposit)])
			.with_messages(vec![(origin.clone(), id, message, deposit)])
			.build()
			.execute_with(|| {
				let free_balance = Balances::free_balance(&origin);

				assert_ok!(remove(&origin, &[id]));

				assert_eq!(Balances::free_balance(&origin), free_balance + deposit);
			});
	}

	#[test]
	fn deposit_is_not_returned_if_try_remove_is_noop() {
		let origin = ALICE;
		let deposit: Balance = 100;
		let message =
			Message::Ismp { commitment: H256::default(), callback: None, message_deposit: deposit };
		let id = 1;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), existential_deposit() + deposit)])
			.with_messages(vec![(origin.clone(), id, message, deposit)])
			.build()
			.execute_with(|| {
				let free_balance = Balances::free_balance(&origin);

				assert_noop!(remove(&origin, &[id]), Error::RequestPending);

				assert_eq!(Balances::free_balance(&origin), free_balance);
			});
	}

	#[test]
	fn multiple_messages_rolls_back_if_one_fails() {
		let origin = ALICE;
		let deposit: Balance = 100;
		let good_message = Message::ismp_response(H256::default(), deposit, BoundedVec::default());
		let erroneous_message =
			Message::Ismp { commitment: H256::default(), callback: None, message_deposit: deposit };
		let messages = 5;
		let endowment = existential_deposit() + deposit * messages as Balance;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), endowment)])
			.with_messages(
				(0..messages - 1)
					.map(|i| (origin.clone(), i, good_message.clone(), deposit))
					.chain([(origin.clone(), messages - 1, erroneous_message, deposit)])
					.collect(),
			)
			.build()
			.execute_with(|| {
				let messages = (0..messages).collect::<Vec<_>>();
				let free_balance = Balances::free_balance(&origin);

				assert_noop!(remove(&origin, &messages), Error::RequestPending);

				for message in messages {
					assert!(Messages::get(&origin, message).is_some());
				}
				assert_eq!(Balances::free_balance(&origin), free_balance);
			});
	}

	// Basic remove tests to ensure storage is cleaned.
	#[test]
	fn remove_ismp_message() {
		let origin = ALICE;
		let commitment = H256::default();
		let id = 1;
		let deposit = 100;
		let message = Message::Ismp { commitment, callback: None, message_deposit: deposit };
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), existential_deposit() + deposit)])
			.build()
			.execute_with(|| {
				Messages::insert(&origin, id, &message);
				IsmpRequests::insert(commitment, (&origin, &id));
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin, deposit));

				assert_noop!(remove(&origin, &[id]), Error::RequestPending);

				assert!(
					Messages::get(origin, id).is_some(),
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
		let origin = ALICE;
		let commitment = H256::default();
		let id = 1;
		let deposit = 100;
		let message = Message::ismp_response(commitment, deposit, BoundedVec::default());
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), existential_deposit() + deposit)])
			.build()
			.execute_with(|| {
				Messages::insert(&origin, id, &message);
				IsmpRequests::insert(commitment, (&origin, &id));
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin, deposit));

				assert_ok!(remove(&origin, &[id]));

				assert!(
					Messages::get(&origin, id).is_none(),
					"Message should have been removed but hasnt."
				);
				assert!(
					IsmpRequests::get(commitment).is_none(),
					"Request should have been removed but hasnt."
				);
			})
	}

	#[test]
	fn remove_ismp_timeout() {
		let origin = ALICE;
		let commitment = H256::default();
		let deposit = 100;
		let callback_deposit = 100_000;
		let id = 1;
		let message = Message::IsmpTimeout {
			commitment,
			message_deposit: deposit,
			callback_deposit: Some(callback_deposit),
		};
		let endowment = existential_deposit() + deposit + callback_deposit;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), endowment)])
			.build()
			.execute_with(|| {
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin, deposit));
				assert_ok!(Fungibles::hold(&CallbackGas.into(), &origin, callback_deposit));

				Messages::insert(&origin, id, &message);
				IsmpRequests::insert(commitment, (&origin, &id));

				assert_ok!(remove(&origin, &[id]));

				assert!(
					Messages::get(&origin, id).is_none(),
					"Message should have been removed but hasnt."
				);
				assert!(
					IsmpRequests::get(commitment).is_none(),
					"Request should have been removed but hasnt."
				);
				assert_eq!(Balances::total_balance_on_hold(&origin), 0);
			})
	}

	#[test]
	fn remove_xcm_query() {
		let origin = ALICE;
		let query_id = 42;
		let id = 1;
		let deposit = 100;
		let message = Message::XcmQuery { query_id, callback: None, message_deposit: deposit };
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), existential_deposit() + deposit)])
			.build()
			.execute_with(|| {
				Messages::insert(&origin, id, &message);
				XcmQueries::insert(query_id, (&origin, &id));
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin, deposit));

				assert_noop!(remove(&origin, &[id]), Error::RequestPending);
				assert!(
					Messages::get(&origin, id).is_some(),
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
		let origin = ALICE;
		let query_id = 42;
		let id = 1;
		let message_deposit = 100;
		let message =
			Message::XcmResponse { query_id, message_deposit, response: Response::default() };
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), existential_deposit() + message_deposit)])
			.build()
			.execute_with(|| {
				Messages::insert(&origin, id, &message);
				XcmQueries::insert(query_id, (&origin, &id));
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin, message_deposit));

				assert_ok!(remove(&origin, &[id]));

				assert!(
					Messages::get(ALICE, id).is_none(),
					"Message should have been removed but hasnt"
				);
				assert!(
					XcmQueries::get(query_id).is_none(),
					"Message should have been removed but hasnt."
				);
			})
	}

	#[test]
	fn remove_xcm_timeout() {
		let origin = ALICE;
		let query_id = 42;
		let id = 1;
		let message_deposit = 100;
		let callback_deposit = 100_000;
		let message = Message::XcmTimeout {
			query_id,
			message_deposit,
			callback_deposit: Some(callback_deposit),
		};
		let endowment = existential_deposit() + message_deposit + callback_deposit;
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), endowment)])
			.build()
			.execute_with(|| {
				assert_ok!(Fungibles::hold(&Messaging.into(), &origin, message_deposit));
				assert_ok!(Fungibles::hold(&CallbackGas.into(), &origin, callback_deposit));

				Messages::insert(&origin, id, &message);
				XcmQueries::insert(query_id, (&origin, id));

				assert_ok!(remove(&origin, &[id]));

				assert!(
					Messages::get(ALICE, id).is_none(),
					"Message should have been removed but hasnt"
				);
				assert!(
					XcmQueries::get(query_id).is_none(),
					"Message should have been removed but hasnt."
				);

				// Assert that all holds specified have been released
				assert_eq!(Balances::total_balance_on_hold(&ALICE), 0);
			})
	}

	// `remove` is no longer a dispatchable and only callable via a precompile, hence we simply
	// wrap calls to it in a transaction to simulate. See additional precompiles tests for
	// further assurances.
	fn remove(origin: &AccountId, messages: &[MessageId]) -> DispatchResult {
		with_transaction(|| -> TransactionOutcome<DispatchResult> {
			let result = super::remove::<Test>(origin, messages);
			match &result {
				Ok(_) => TransactionOutcome::Commit(result),
				Err(_) => TransactionOutcome::Rollback(result),
			}
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
		let callback = Callback::new(H160::zero(), Encoding::Scale, [0u8; 4], weight);
		let message_id = 1;
		let data = [100u8; 5];
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
		let callback = Callback::new(H160::zero(), Encoding::Scale, [0u8; 4], weight);
		let id = 1;
		let data = [100u8; 5];
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

fn existential_deposit() -> Balance {
	<ExistentialDeposit as Get<Balance>>::get()
}

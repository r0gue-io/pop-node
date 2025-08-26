pub(crate) use ::xcm::latest::{Location, QueryId, Response};
use xcm_builder::QueryControllerWeightInfo;

use super::*;
use crate::messaging::{pallet::Call, BlockNumberOf, Config};

/// Initiate a new XCM query.
///
/// Starts a query using the XCM interface, specifying a responder and timeout block.
///
/// # Parameters
/// - `origin`: The account initiating the query.
/// - `id`: A unique message ID.
/// - `responder`: Location of the XCM responder.
/// - `timeout`: Block number after which the query should timeout.
/// - `callback`: Optional callback for handling the response.
///
/// # Returns
/// A unique identifier for the message.
pub(crate) fn new_query<T: Config>(
	origin: Origin<T>,
	responder: Location,
	timeout: BlockNumberOf<T>,
	callback: Option<Callback<BalanceOf<T>>>,
) -> Result<(MessageId, QueryId), DispatchError> {
	let querier_location =
		T::OriginConverter::try_convert(T::RuntimeOrigin::signed(origin.account.clone()))
			.map_err(|_| Error::<T>::OriginConversionFailed)?;

	let current_block = frame_system::Pallet::<T>::block_number();
	ensure!(current_block < timeout, Error::<T>::FutureTimeoutMandatory);

	let id = next_message_id::<T>()?;
	XcmQueryTimeouts::<T>::try_mutate(current_block.saturating_add(timeout), |bounded_vec| {
		bounded_vec
			.try_push(id)
			.map_err(|_| Error::<T>::MaxMessageTimeoutPerBlockReached)
	})?;

	// Take deposits and fees.
	let message_deposit =
		calculate_protocol_deposit::<T, T::OnChainByteFee>(ProtocolStorageDeposit::XcmQueries)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>());
	T::Fungibles::hold(&HoldReason::Messaging.into(), &origin.account, message_deposit)?;

	let mut callback_execution_weight = Weight::zero();

	if let Some(cb) = callback.as_ref() {
		T::Fungibles::hold(
			&HoldReason::CallbackGas.into(),
			&origin.account,
			T::WeightToFee::weight_to_fee(&cb.gas_limit),
		)?;

		callback_execution_weight = T::CallbackExecutor::execution_weight();
	}

	let response_prepayment_amount = T::WeightToFee::weight_to_fee(
		&T::WeightInfo::xcm_response().saturating_add(callback_execution_weight),
	);

	let credit = T::Fungibles::withdraw(
		&origin.account,
		response_prepayment_amount,
		Precision::Exact,
		Preservation::Preserve,
		Fortitude::Polite,
	)?;

	T::FeeHandler::on_unbalanced(credit);

	// Process message by creating new query via XCM.
	// Xcm only uses/stores pallet, index - i.e. (u8,u8), hence the fields in xcm_response
	// are ignored.
	let notify = Call::<T>::xcm_response { query_id: 0, xcm_response: Default::default() };
	let query_id = T::Xcm::new_notify_query(responder, notify, timeout, querier_location);

	// Store query id for later lookup on response, message for querying status,
	// response/timeout handling.
	XcmQueries::<T>::insert(query_id, id);
	Messages::<T>::insert(id, Message::XcmQuery { origin, query_id, callback, message_deposit });
	Ok((id, query_id))
}

/// A handler for the creation of a XCM query notification.
pub trait NotifyQueryHandler<T: Config> {
	type WeightInfo: QueryControllerWeightInfo;
	/// Attempt to create a new query ID and register it as a query that is yet to respond, and
	/// which will call a dispatchable when a response happens.
	fn new_notify_query(
		responder: impl Into<Location>,
		notify: Call<T>,
		timeout: BlockNumberOf<T>,
		match_querier: impl Into<Location>,
	) -> QueryId;
}

#[cfg(test)]
pub(crate) mod tests {
	use frame_support::{
		assert_noop, assert_ok,
		storage::{with_transaction, TransactionOutcome},
		traits::fungible::InspectHold,
		weights::WeightToFee as _,
	};

	use super::{
		super::{Callback, Encoding, HoldReason::*},
		mock::{messaging::*, *},
		CallbackExecutor as _, WeightInfo as _, *,
	};

	type CallbackExecutor = <Test as Config>::CallbackExecutor;
	type Error = super::Error<Test>;
	type Event = super::Event<Test>;
	type Fungibles = <Test as Config>::Fungibles;
	type Messages = super::Messages<Test>;
	type OnChainByteFee = <Test as Config>::OnChainByteFee;
	type Origin = super::Origin<Test>;
	type WeightInfo = <Test as Config>::WeightInfo;
	type WeightToFee = <Test as Config>::WeightToFee;
	type XcmQueries = super::XcmQueries<Test>;

	#[test]
	fn ensure_xcm_response_has_weight() {
		assert_ne!(
			WeightInfo::xcm_response(),
			Weight::zero(),
			"Please set a weight for xcm_response to run these tests."
		);
	}

	#[test]
	fn ensure_xcm_response_fee() {
		assert_ne!(WeightToFee::weight_to_fee(&(WeightInfo::xcm_response())), 0);
		assert_ne!(WeightToFee::weight_to_fee(&(CallbackExecutor::execution_weight())), 0);
	}

	#[test]
	fn ensure_callback_executor_has_weight() {
		assert_ne!(
			CallbackExecutor::execution_weight(),
			Weight::zero(),
			"Please set a weight for CallbackExecutor::execution_weight to run these tests."
		);
	}

	#[test]
	fn takes_response_fee_no_callback() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let response_fee = WeightToFee::weight_to_fee(&(WeightInfo::xcm_response()));
		let callback = None;
		let endowment = existential_deposit() + deposit() + response_fee;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;
				let balance_pre_transfer = Balances::free_balance(&origin.account);

				assert_ok!(new_query(origin.clone(), RESPONSE_LOCATION, timeout, callback));

				let balance_post_transfer = Balances::free_balance(&origin.account);
				let total_balance_on_hold = Balances::total_balance_on_hold(&origin.account);
				assert_eq!(
					balance_pre_transfer - balance_post_transfer - total_balance_on_hold,
					response_fee
				);
			})
	}

	#[test]
	fn takes_response_fee_with_callback() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let response_fee = xcm_response_fee();
		let callback = Callback::new(H160::zero(), Encoding::Scale, [1; 4], Weight::zero(), 0);
		let endowment = existential_deposit() + deposit() + response_fee;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;
				assert_ne!(response_fee, 0);
				let balance_pre_query = Balances::free_balance(&origin.account);

				assert_ok!(new_query(origin.clone(), RESPONSE_LOCATION, timeout, Some(callback)));

				let balance_post_query = Balances::free_balance(&origin.account);
				let total_balance_on_hold = Balances::total_balance_on_hold(&origin.account);

				assert_eq!(
					balance_pre_query - balance_post_query - total_balance_on_hold,
					response_fee
				);
			})
	}

	#[test]
	fn takes_callback_hold() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let weight = Weight::from_parts(100_000_000, 100_000_000);
		let callback = Callback::new(H160::zero(), Encoding::Scale, [1; 4], weight, 100_000_000);
		let callback_deposit = WeightToFee::weight_to_fee(&weight);
		let endowment = existential_deposit() + deposit() + xcm_response_fee() + callback_deposit;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;
				let held_balance_pre_query =
					Fungibles::balance_on_hold(&CallbackGas.into(), &origin.account);
				assert_eq!(held_balance_pre_query, 0);
				assert_ne!(callback_deposit, 0);

				assert_ok!(new_query(origin.clone(), RESPONSE_LOCATION, timeout, Some(callback)));

				let held_balance_post_query =
					Fungibles::balance_on_hold(&CallbackGas.into(), &origin.account);
				assert_eq!(held_balance_post_query, callback_deposit);
			})
	}

	#[test]
	fn takes_messaging_hold() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let expected_deposit = deposit();
		let endowment = existential_deposit() + deposit() + xcm_response_fee();
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;
				let held_balance_pre_hold =
					Fungibles::balance_on_hold(&Messaging.into(), &origin.account);
				assert_ne!(
					expected_deposit, 0,
					"set an onchain byte fee with T::OnChainByteFee to run this test."
				);
				assert_eq!(held_balance_pre_hold, 0);

				assert_ok!(new_query(origin.clone(), RESPONSE_LOCATION, timeout, None));

				let held_balance_post_hold =
					Fungibles::balance_on_hold(&Messaging.into(), &origin.account);
				assert_eq!(held_balance_post_hold, expected_deposit);
			});
	}

	#[test]
	fn assert_state() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let weight = Weight::from_parts(100_000_000, 100_000_000);
		let callback = Callback::new(H160::zero(), Encoding::Scale, [1; 4], weight, 100_000_000);
		let callback_deposit = WeightToFee::weight_to_fee(&weight);
		let endowment = existential_deposit() + deposit() + xcm_response_fee() + callback_deposit;
		let id = 1;
		let query_id = 42;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.with_message_id(id)
			.with_query_id(query_id)
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 1;

				assert_ok!(new_query(origin.clone(), RESPONSE_LOCATION, timeout, Some(callback)));

				let message = Messages::get(id).expect("should exist after xcm_new_query.");
				let Message::XcmQuery { query_id: qid, callback: c, .. } = message else {
					panic!("Wrong message type.")
				};
				assert_eq!(qid, query_id);
				assert_eq!(c, Some(callback));
				assert_eq!(XcmQueries::get(query_id), Some(id));
			})
	}

	#[test]
	fn xcm_timeouts_must_be_in_the_future() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let endowment = existential_deposit() + deposit() + xcm_response_fee();
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.build()
			.execute_with(|| {
				let timeout = System::block_number();

				assert_noop!(
					new_query(origin, RESPONSE_LOCATION, timeout, None),
					Error::FutureTimeoutMandatory
				);
			})
	}

	#[test]
	fn xcm_queries_expire_on_expiry_block() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let messages = 2;
		let query_id = 42;
		let endowment =
			existential_deposit() + (deposit() + xcm_response_fee()) * messages as Balance;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), endowment)])
			.with_query_id(query_id)
			.build()
			.execute_with(|| {
				let timeout = System::block_number() + 10;
				for _ in 0..messages {
					assert_ok!(new_query(origin.clone(), RESPONSE_LOCATION, timeout, None));
				}

				run_to(timeout + 1);

				for id in 0..messages {
					let Some(Message::XcmTimeout { .. }) = Messages::get(id) else {
						panic!("Message should be timed out!")
					};
				}
				let query_ids = (0..messages).map(|id| query_id + id).collect();
				System::assert_has_event(Event::XcmQueriesTimedOut { query_ids }.into());
			})
	}

	pub(crate) fn deposit() -> Balance {
		calculate_protocol_deposit::<Test, OnChainByteFee>(ProtocolStorageDeposit::XcmQueries) +
			calculate_message_deposit::<Test, OnChainByteFee>()
	}

	fn existential_deposit() -> Balance {
		<ExistentialDeposit as Get<Balance>>::get()
	}

	// `new_query` is no longer a dispatchable and only callable via a precompile, hence we simply
	// wrap calls to it in a transaction to simulate. See additional precompiles tests for
	// further assurances.
	pub(crate) fn new_query(
		origin: Origin,
		responder: Location,
		timeout: u32,
		callback: Option<Callback<Balance>>,
	) -> Result<(MessageId, QueryId), DispatchError> {
		with_transaction(|| {
			let result = super::new_query::<Test>(origin, responder, timeout, callback);
			match &result {
				Ok(_) => TransactionOutcome::Commit(result),
				Err(_) => TransactionOutcome::Rollback(result),
			}
		})
	}

	pub(crate) fn xcm_response_fee() -> Balance {
		WeightToFee::weight_to_fee(
			&(WeightInfo::xcm_response() + CallbackExecutor::execution_weight()),
		)
	}
}

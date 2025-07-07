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
	origin: &T::AccountId,
	responder: Location,
	timeout: BlockNumberOf<T>,
	callback: Option<Callback>,
) -> Result<(MessageId, QueryId), DispatchError> {
	let querier_location =
		T::OriginConverter::try_convert(T::RuntimeOrigin::signed(origin.clone()))
			.map_err(|_| Error::<T>::OriginConversionFailed)?;

	let current_block = frame_system::Pallet::<T>::block_number();
	ensure!(current_block < timeout, Error::<T>::FutureTimeoutMandatory);

	let id = next_message_id::<T>(origin)?;
	XcmQueryTimeouts::<T>::try_mutate(current_block.saturating_add(timeout), |bounded_vec| {
		bounded_vec
			.try_push((origin.clone(), id))
			.map_err(|_| Error::<T>::MaxMessageTimeoutPerBlockReached)
	})?;

	// Take deposits and fees.
	let message_deposit =
		calculate_protocol_deposit::<T, T::OnChainByteFee>(ProtocolStorageDeposit::XcmQueries)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>());
	T::Fungibles::hold(&HoldReason::Messaging.into(), &origin, message_deposit)?;

	let mut callback_execution_weight = Weight::zero();

	if let Some(cb) = callback.as_ref() {
		T::Fungibles::hold(
			&HoldReason::CallbackGas.into(),
			&origin,
			T::WeightToFee::weight_to_fee(&cb.weight),
		)?;

		callback_execution_weight = T::CallbackExecutor::execution_weight();
	}

	let response_prepayment_amount = T::WeightToFee::weight_to_fee(
		&T::WeightInfo::xcm_response().saturating_add(callback_execution_weight),
	);

	let credit = T::Fungibles::withdraw(
		&origin,
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
	XcmQueries::<T>::insert(query_id, (&origin, id));
	Messages::<T>::insert(&origin, id, Message::XcmQuery { query_id, callback, message_deposit });
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

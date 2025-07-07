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

use super::*;

type DbWeightOf<T> = <T as frame_system::Config>::DbWeight;

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
	origin: &T::AccountId,
	message: DispatchGet,
	fee: BalanceOf<T>,
	callback: Option<Callback>,
) -> Result<(MessageId, H256), DispatchError> {
	// Take deposits and fees.
	let message_deposit =
		calculate_protocol_deposit::<T, T::OnChainByteFee>(ProtocolStorageDeposit::IsmpRequests)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>())
		// TODO: include length of `DispatchGet` as own `Get` no longer required
		// 	.saturating_add(calculate_deposit_of::<T, T::OffChainByteFee, Get<T>>())
		;

	T::Fungibles::hold(&HoldReason::Messaging.into(), &origin, message_deposit)?;

	if let Some(cb) = callback.as_ref() {
		T::Fungibles::hold(
			&HoldReason::CallbackGas.into(),
			&origin,
			T::WeightToFee::weight_to_fee(&cb.weight),
		)?;
	}

	// Process message by dispatching request via ISMP.
	let commitment = match T::IsmpDispatcher::default()
		.dispatch_request(DispatchRequest::Get(message), FeeMetadata { payer: origin.clone(), fee })
	{
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
	let id = next_message_id::<T>(origin)?;
	IsmpRequests::<T>::insert(commitment, (&origin, id));
	Messages::<T>::insert(&origin, id, Message::Ismp { commitment, callback, message_deposit });
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
	origin: &T::AccountId,
	message: DispatchPost,
	fee: BalanceOf<T>,
	callback: Option<Callback>,
) -> Result<(MessageId, H256), DispatchError> {
	// Take deposits and fees.
	let message_deposit =
		calculate_protocol_deposit::<T, T::OnChainByteFee>(ProtocolStorageDeposit::IsmpRequests)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>())
			// TODO: include length of `DispatchGet` as own `Get` no longer required
		//	.saturating_add(calculate_deposit_of::<T, T::OffChainByteFee, ismp::Post<T>>())
		;

	T::Fungibles::hold(&HoldReason::Messaging.into(), origin, message_deposit)?;

	if let Some(cb) = callback.as_ref() {
		T::Fungibles::hold(
			&HoldReason::CallbackGas.into(),
			origin,
			T::WeightToFee::weight_to_fee(&cb.weight),
		)?;
	}

	// Process message by dispatching request via ISMP.
	let commitment = T::IsmpDispatcher::default()
		.dispatch_request(
			DispatchRequest::Post(message),
			FeeMetadata { payer: origin.clone(), fee },
		)
		.map_err(|_| Error::<T>::IsmpDispatchFailed)?;

	// Store commitment for lookup on response, message for querying,
	// response/timeout handling.
	let id = next_message_id::<T>(origin)?;
	IsmpRequests::<T>::insert(commitment, (origin, id));
	Messages::<T>::insert(origin, id, Message::Ismp { commitment, callback, message_deposit });
	Ok((id, commitment))
}

pub(crate) fn process_response<T: Config>(
	commitment: &H256,
	response_data: &impl Encode,
	event: impl Fn(AccountIdOf<T>, MessageId) -> Event<T>,
) -> Result<(), anyhow::Error> {
	ensure!(
		response_data.encoded_size() <= T::MaxResponseLen::get() as usize,
		::ismp::Error::Custom("Response length exceeds maximum allowed length.".into())
	);

	let (initiating_origin, id) = IsmpRequests::<T>::get(commitment)
		.ok_or(::ismp::Error::Custom("Request not found.".into()))?;

	let Some(super::super::Message::Ismp { commitment, callback, message_deposit }) =
		Messages::<T>::get(&initiating_origin, id)
	else {
		return Err(::ismp::Error::Custom("Message must be an ismp request.".into()).into());
	};

	// Deposit that the message has been recieved before a potential callback execution.
	Pallet::<T>::deposit_event(event(initiating_origin.clone(), id));

	// Attempt callback with result if specified.
	if let Some(callback) = callback {
		if call::<T>(&initiating_origin, callback, &id, response_data).is_ok() {
			// Clean storage, return deposit
			Messages::<T>::remove(&initiating_origin, id);
			IsmpRequests::<T>::remove(commitment);
			T::Fungibles::release(
				&HoldReason::Messaging.into(),
				&initiating_origin,
				message_deposit,
				Exact,
			)
			.map_err(|_| ::ismp::Error::Custom("failed to release message deposit.".into()))?;

			return Ok(());
		}
	}

	// No callback or callback error: store response for manual retrieval and removal.
	let encoded_response: BoundedVec<u8, T::MaxResponseLen> = response_data
		.encode()
		.try_into()
		.map_err(|_| ::ismp::Error::Custom("response exceeds max".into()))?;
	Messages::<T>::insert(
		&initiating_origin,
		id,
		super::super::Message::IsmpResponse {
			commitment,
			message_deposit,
			response: encoded_response,
		},
	);
	Ok(())
}

pub(crate) fn timeout_commitment<T: Config>(commitment: &H256) -> Result<(), anyhow::Error> {
	let key = IsmpRequests::<T>::get(commitment).ok_or(::ismp::Error::Custom(
		"Request commitment not found while processing timeout.".into(),
	))?;
	Messages::<T>::try_mutate(key.0, key.1, |message| {
		let Some(super::super::Message::Ismp { commitment, message_deposit, callback }) = message
		else {
			return Err(::ismp::Error::Custom("Invalid message".into()));
		};
		let callback_deposit = callback.map(|cb| T::WeightToFee::weight_to_fee(&cb.weight));
		*message = Some(super::super::Message::IsmpTimeout {
			message_deposit: *message_deposit,
			commitment: *commitment,
			callback_deposit,
		});
		Ok(())
	})?;

	crate::messaging::Pallet::<T>::deposit_event(Event::<T>::IsmpTimedOut {
		commitment: *commitment,
	});
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
				log::debug!(target: "pop-api::extension", "StorageValue={:?}", values);
				let commitment = hash_request::<T::Keccak256>(&Request::Get(get));
				process_response(&commitment, &values, |dest, id| {
					Event::<T>::IsmpGetResponseReceived { dest, id, commitment }
				})
			},
			Response::Post(PostResponse { post, response, .. }) => {
				let commitment = hash_request::<T::Keccak256>(&Request::Post(post));
				process_response(&commitment, &response, |dest, id| {
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
			Timeout::Request(Request::Post(_)) => 0u32,
			Timeout::Request(Request::Get(_)) => 1u32,
			Timeout::Response(_) => 2u32,
		};
		T::WeightInfo::ismp_on_timeout(x)
	}

	// todo: test
	fn on_response(&self, response: &Response) -> Weight {
		let x = match response {
			Response::Get(_) => 1,
			Response::Post(_) => 0,
		};

		T::WeightInfo::ismp_on_response(x).saturating_add(T::CallbackExecutor::execution_weight())
		// Also add actual weight consumed by contract env.
	}
}

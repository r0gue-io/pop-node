extern crate alloc;

pub use alloc::borrow::ToOwned;

use ::ismp::Error as IsmpError;
use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo},
	pallet_prelude::*,
	storage::KeyLenOf,
	traits::{
		tokens::{
			fungible::{hold::Mutate as HoldMutate, Balanced, Credit, Inspect, Mutate},
			Fortitude, Precision, Preservation,
		},
		Get, OnUnbalanced,
	},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	traits::{Saturating, TryConvert},
	BoundedVec, DispatchError,
};
use sp_std::vec::Vec;
use sp_weights::WeightToFee;
use transports::{
	ismp::{self as ismp, FeeMetadata, IsmpDispatcher},
	xcm::{self as xcm, Location, QueryId},
};
pub use xcm::NotifyQueryHandler;
use xcm::Response;

use super::Weight;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// Messaging transports.
pub mod transports;

/// Message storage deposit calculations.
mod deposits;
use deposits::*;

mod weights;
pub use weights::WeightInfo;

#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "runtime-benchmarks"))]
pub(crate) mod test_utils;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BlockNumberOf<T> = BlockNumberFor<T>;
type BalanceOf<T> = <<T as Config>::Fungibles as Inspect<AccountIdOf<T>>>::Balance;
type DbWeightOf<T> = <T as frame_system::Config>::DbWeight;

pub type MessageId = [u8; 32];

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	/// Configuration of the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type OriginConverter: TryConvert<Self::RuntimeOrigin, Location>;

		/// The base byte fee for data stored onchain.
		#[pallet::constant]
		type OnChainByteFee: Get<BalanceOf<Self>>;

		/// The base byte fee for data stored offchain.
		#[pallet::constant]
		type OffChainByteFee: Get<BalanceOf<Self>>;

		/// The type responsible for executing callbacks.
		type CallbackExecutor: CallbackExecutor<Self>;

		/// The deposit + fee mechanism.
		type Fungibles: HoldMutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ Mutate<Self::AccountId>
			+ Balanced<Self::AccountId>;

		/// The ISMP message dispatcher.
		type IsmpDispatcher: IsmpDispatcher<Account = Self::AccountId, Balance = BalanceOf<Self>>;

		/// The maximum length of any additional application-specific metadata relating to a
		/// request.
		#[pallet::constant]
		type MaxContextLen: Get<u32>;

		/// The maximum length of outbound (posted) data.
		#[pallet::constant]
		type MaxDataLen: Get<u32>;

		/// The maximum amount of key for an outbound request.
		#[pallet::constant]
		type MaxKeys: Get<u32>;

		/// The maximum byte length for a single key of an ismp request.
		#[pallet::constant]
		type MaxKeyLen: Get<u32>;

		/// The maximum length for a response.
		#[pallet::constant]
		type MaxResponseLen: Get<u32>;

		/// The maximum amount of removals in a single call to remove.
		#[pallet::constant]
		type MaxRemovals: Get<u32>;

		/// Overarching hold reason.
		type RuntimeHoldReason: From<HoldReason>;

		/// Wrapper type for creating a query with a notify
		type Xcm: NotifyQueryHandler<Self>;

		/// The origin of the response for xcm.
		type XcmResponseOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Location>;

		/// SAFETY: Recommended this is small as is used to updated a message status in the hooks.
		/// The maximum number of xcm timeout updates that can be processed per block.
		#[pallet::constant]
		type MaxXcmQueryTimeoutsPerBlock: Get<u32>;

		/// Where the callback fees or response fees are charged to.
		type FeeHandler: OnUnbalanced<Credit<Self::AccountId, Self::Fungibles>>;

		/// The type responsible for converting between weight and balance, commonly transaction
		/// payment.
		type WeightToFee: WeightToFee<Balance = BalanceOf<Self>>;

		/// The implementation of Keccak used for commitment hashes.
		type Keccak256: ::ismp::messaging::Keccak256;

		/// Pallet weights.
		type WeightInfo: super::WeightInfo;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(crate) type Messages<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		MessageId,
		Message<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub(super) type IsmpRequests<T: Config> =
		StorageMap<_, Identity, H256, (T::AccountId, MessageId), OptionQuery>;

	#[pallet::storage]
	pub(super) type XcmQueries<T: Config> =
		StorageMap<_, Identity, QueryId, (T::AccountId, MessageId), OptionQuery>;

	#[pallet::storage]
	pub(super) type XcmQueryTimeouts<T: Config> = StorageMap<
		_,
		Identity,
		BlockNumberOf<T>,
		BoundedVec<(T::AccountId, MessageId), T::MaxXcmQueryTimeoutsPerBlock>,
		ValueQuery,
	>;

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A GET has been dispatched via ISMP.
		IsmpGetDispatched {
			/// The origin of the request.
			origin: T::AccountId,
			/// The identifier specified for the request.
			id: MessageId,
			/// The ISMP request commitment.
			commitment: H256,
			/// An optional callback to be used to return the response.
			callback: Option<Callback>,
		},
		/// A response to a GET has been received via ISMP.
		IsmpGetResponseReceived {
			/// The destination of the response.
			dest: T::AccountId,
			/// The identifier specified for the request.
			id: MessageId,
			/// The ISMP request commitment.
			commitment: H256,
		},
		/// A POST has been dispatched via ISMP.
		IsmpPostDispatched {
			/// The origin of the request.
			origin: T::AccountId,
			/// The identifier specified for the request.
			id: MessageId,
			/// The ISMP request commitment.
			commitment: H256,
			/// An optional callback to be used to return the response.
			callback: Option<Callback>,
		},
		/// A response to a POST has been received via ISMP.
		IsmpPostResponseReceived {
			/// The destination of the response.
			dest: T::AccountId,
			/// The identifier specified for the request.
			id: MessageId,
			/// The ISMP request commitment.
			commitment: H256,
		},
		/// A XCM query has been created.
		XcmQueryCreated {
			/// The origin of the request.
			origin: T::AccountId,
			/// The identifier specified for the request.
			id: MessageId,
			/// The identifier of the created XCM query.
			query_id: QueryId,
			/// An optional callback to be used to return the response.
			callback: Option<Callback>,
		},
		/// A response to a XCM query has been received.
		XcmResponseReceived {
			/// The destination of the response.
			dest: T::AccountId,
			/// The identifier specified for the request.
			id: MessageId,
			/// The identifier of the XCM query.
			query_id: QueryId,
			/// The query response.
			response: Response,
		},
		/// A callback has been executed successfully.
		CallbackExecuted {
			/// The origin of the callback.
			origin: T::AccountId,
			/// The identifier specified for the request.
			id: MessageId,
			/// The successful callback.
			callback: Callback,
		},
		/// A callback has failed.
		CallbackFailed {
			/// The origin of the callback.
			origin: T::AccountId,
			/// The identifier specified for the request.
			id: MessageId,
			/// The callback which failed.
			callback: Callback,
			/// The error which occurred.
			error: DispatchErrorWithPostInfo,
		},
		/// One or more messages have been removed for the origin.
		Removed {
			/// The origin of the messages.
			origin: T::AccountId,
			/// The messages which were removed.
			messages: Vec<MessageId>,
		},
		/// An ISMP message has timed out.
		IsmpTimedOut { commitment: H256 },
		/// A collection of xcm queries have timed out.
		XcmQueriesTimedOut { query_ids: Vec<QueryId> },
		/// An error has occured while attempting to refund weight.
		WeightRefundErrored { message_id: MessageId, error: DispatchError },
		/// Callback gas has been topped up.
		CallbackGasIncreased { message_id: MessageId, total_weight: Weight },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The message is invalid.
		InvalidMessage,
		/// The query is invalid.
		InvalidQuery,
		/// Failed to convert origin.
		OriginConversionFailed,
		/// The message already exists.
		MessageExists,
		/// The request is pending.
		RequestPending,
		/// dispatching a call via ISMP falied
		IsmpDispatchFailed,
		/// The message was not found
		MessageNotFound,
		/// The request has timed out
		RequestTimedOut,
		/// Timeouts must be in the future.
		FutureTimeoutMandatory,
		/// Message block limit has been reached for this expiry block. Try a different timeout.
		MaxMessageTimeoutPerBlockReached,
		/// This callback cannot be processed due to lack of blockspace. Please poll the response.
		BlockspaceAllowanceReached,
		/// This is not possible as the message has completed.
		MessageCompleted,
		/// No callback has been found for this query.
		NoCallbackFound,
		/// Weight cannot be zero.
		ZeroWeight,
	}

	/// A reason for the pallet placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// Held for the duration of a message's lifespan.
		#[codec(index = 0)]
		Messaging,
		#[codec(index = 1)]
		CallbackGas,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberOf<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberOf<T>) -> Weight {
			// As of polkadot-2412 XCM timeouts are not handled by the implementation of OnResponse
			// in pallet-xcm. As a result, we must handle timeouts in the pallet.
			// Iterate through the queries that have expired and update their status.
			let mut weight: Weight = Zero::zero();
			let mut query_ids = Vec::new();
			for (origin, message_id) in XcmQueryTimeouts::<T>::get(n) {
				weight = weight.saturating_add(DbWeightOf::<T>::get().reads_writes(2, 1));
				Messages::<T>::mutate(origin, message_id, |maybe_message| {
					if let Some(Message::XcmQuery { query_id, message_deposit, callback }) =
						maybe_message.as_mut()
					{
						let callback_deposit =
							callback.map(|cb| T::WeightToFee::weight_to_fee(&cb.weight));
						query_ids.push(*query_id);
						*maybe_message = Some(Message::XcmTimeout {
							query_id: *query_id,
							message_deposit: *message_deposit,
							callback_deposit,
						});
					}
				})
			}

			if !query_ids.is_empty() {
				Self::deposit_event(Event::<T>::XcmQueriesTimedOut { query_ids })
			}
			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submit a new ISMP `Get` request.
		///
		/// This sends a `Get` request through ISMP, optionally with a callback to handle the
		/// response.
		///
		/// # Parameters
		/// - `origin`: The account submitting the request.
		/// - `id`: A unique identifier for the message.
		/// - `message`: The ISMP `Get` message containing query details.
		/// - `callback`: Optional callback to execute upon receiving a response.
		#[pallet::call_index(1)]
		#[pallet::weight(
			{
				let keys_len: u32 = message.keys.len().try_into().unwrap_or(T::MaxKeys::get());
				let context_len: u32 = message.context.len().try_into().unwrap_or(T::MaxContextLen::get());
				let has_callback = callback.is_some() as u32;
				T::WeightInfo::ismp_get(context_len, keys_len, has_callback)
			}
		)]
		pub fn ismp_get(
			origin: OriginFor<T>,
			id: MessageId,
			message: ismp::Get<T>,
			ismp_relayer_fee: BalanceOf<T>,
			callback: Option<Callback>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(!Messages::<T>::contains_key(&origin, id), Error::<T>::MessageExists);

			// Take deposits and fees.
			let message_deposit = calculate_protocol_deposit::<T, T::OnChainByteFee>(
				ProtocolStorageDeposit::IsmpRequests,
			)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>())
			.saturating_add(calculate_deposit_of::<T, T::OffChainByteFee, ismp::Get<T>>());

			T::Fungibles::hold(&HoldReason::Messaging.into(), &origin, message_deposit)?;

			if let Some(cb) = callback.as_ref() {
				T::Fungibles::hold(
					&HoldReason::CallbackGas.into(),
					&origin,
					T::WeightToFee::weight_to_fee(&cb.weight),
				)?;
			}

			// Process message by dispatching request via ISMP.
			let commitment = match T::IsmpDispatcher::default().dispatch_request(
				message.into(),
				FeeMetadata { payer: origin.clone(), fee: ismp_relayer_fee },
			) {
				Ok(commitment) => Ok::<H256, DispatchError>(commitment),
				Err(e) => {
					if let Ok(err) = e.downcast::<IsmpError>() {
						log::error!("ISMP Dispatch failed!! {:?}", err);
					}
					return Err(Error::<T>::IsmpDispatchFailed.into());
				},
			}?;
			// Store commitment for lookup on response, message for querying,
			// response/timeout handling.
			IsmpRequests::<T>::insert(commitment, (&origin, id));
			Messages::<T>::insert(
				&origin,
				id,
				Message::Ismp { commitment, callback, message_deposit },
			);
			Pallet::<T>::deposit_event(Event::<T>::IsmpGetDispatched {
				origin,
				id,
				commitment,
				callback,
			});
			Ok(())
		}

		/// Submit a new ISMP `Post` request.
		///
		/// Sends a `Post` message through ISMP with arbitrary data and an optional callback.
		///
		/// # Parameters
		/// - `origin`: The account submitting the request.
		/// - `id`: A unique identifier for the message.
		/// - `message`: The ISMP `Post` message containing the payload.
		/// - `callback`: Optional callback to execute upon receiving a response.
		#[pallet::call_index(2)]
		#[pallet::weight({
			let data_len: u32 = message.data.len().try_into().unwrap_or(T::MaxDataLen::get());
			let has_callback = callback.is_some() as u32;
			T::WeightInfo::ismp_post(data_len, has_callback)
		})]
		pub fn ismp_post(
			origin: OriginFor<T>,
			id: MessageId,
			message: ismp::Post<T>,
			ismp_relayer_fee: BalanceOf<T>,
			callback: Option<Callback>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(!Messages::<T>::contains_key(&origin, id), Error::<T>::MessageExists);

			// Take deposits and fees.
			let message_deposit = calculate_protocol_deposit::<T, T::OnChainByteFee>(
				ProtocolStorageDeposit::IsmpRequests,
			)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>())
			.saturating_add(calculate_deposit_of::<T, T::OffChainByteFee, ismp::Post<T>>());

			T::Fungibles::hold(&HoldReason::Messaging.into(), &origin, message_deposit)?;

			if let Some(cb) = callback.as_ref() {
				T::Fungibles::hold(
					&HoldReason::CallbackGas.into(),
					&origin,
					T::WeightToFee::weight_to_fee(&cb.weight),
				)?;
			}

			// Process message by dispatching request via ISMP.
			let commitment = T::IsmpDispatcher::default()
				.dispatch_request(
					message.into(),
					FeeMetadata { payer: origin.clone(), fee: ismp_relayer_fee },
				)
				.map_err(|_| Error::<T>::IsmpDispatchFailed)?;

			// Store commitment for lookup on response, message for querying,
			// response/timeout handling.
			IsmpRequests::<T>::insert(commitment, (&origin, id));
			Messages::<T>::insert(
				&origin,
				id,
				Message::Ismp { commitment, callback, message_deposit },
			);
			Pallet::<T>::deposit_event(Event::<T>::IsmpPostDispatched {
				origin,
				id,
				commitment,
				callback,
			});
			Ok(())
		}

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
		#[pallet::call_index(3)]
		#[pallet::weight(
			{
				T::WeightInfo::xcm_new_query(callback.is_some() as u32)
			}
		)]
		pub fn xcm_new_query(
			origin: OriginFor<T>,
			id: MessageId,
			responder: Location,
			timeout: BlockNumberOf<T>,
			callback: Option<Callback>,
		) -> DispatchResult {
			let querier_location = T::OriginConverter::try_convert(origin.clone())
				.map_err(|_| Error::<T>::OriginConversionFailed)?;
			let origin = ensure_signed(origin)?;
			ensure!(!Messages::<T>::contains_key(&origin, id), Error::<T>::MessageExists);

			let current_block = frame_system::Pallet::<T>::block_number();
			ensure!(current_block < timeout, Error::<T>::FutureTimeoutMandatory);

			XcmQueryTimeouts::<T>::try_mutate(
				current_block.saturating_add(timeout),
				|bounded_vec| {
					bounded_vec
						.try_push((origin.clone(), id))
						.map_err(|_| Error::<T>::MaxMessageTimeoutPerBlockReached)
				},
			)?;

			// Take deposits and fees.
			let message_deposit = calculate_protocol_deposit::<T, T::OnChainByteFee>(
				ProtocolStorageDeposit::XcmQueries,
			)
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
			Messages::<T>::insert(
				&origin,
				id,
				Message::XcmQuery { query_id, callback, message_deposit },
			);
			Pallet::<T>::deposit_event(Event::<T>::XcmQueryCreated {
				origin,
				id,
				query_id,
				callback,
			});
			Ok(())
		}

		/// Handle a response to a previous XCM query.
		///
		/// Executes a stored callback or updates the state with the received response.
		///
		/// # Parameters
		/// - `origin`: The XCM responder origin.
		/// - `query_id`: The ID of the XCM query being responded to.
		/// - `xcm_response`: The response data.
		#[pallet::call_index(4)]
		#[pallet::weight({
			// This is only used to check against max_weight field in the OnResponse implementation in pallet-xcm.
			T::WeightInfo::xcm_response()
		})]
		pub fn xcm_response(
			origin: OriginFor<T>,
			query_id: QueryId,
			xcm_response: Response,
		) -> DispatchResult {
			T::XcmResponseOrigin::ensure_origin(origin)?;

			let extrinsic_weight = T::WeightInfo::xcm_response()
				.saturating_add(T::CallbackExecutor::execution_weight());

			ensure!(
				frame_system::BlockWeight::<T>::get()
					.checked_accrue(extrinsic_weight, DispatchClass::Normal)
					.is_ok(),
				Error::<T>::BlockspaceAllowanceReached
			);

			// Manually adjust weight ahead of fallible execution.
			// The fees of which should have been paid.
			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				T::WeightInfo::xcm_response()
					.saturating_add(T::CallbackExecutor::execution_weight()),
				DispatchClass::Normal,
			);

			let (initiating_origin, id) =
				XcmQueries::<T>::get(query_id).ok_or(Error::<T>::MessageNotFound)?;
			let xcm_query_message =
				Messages::<T>::get(&initiating_origin, id).ok_or(Error::<T>::MessageNotFound)?;

			let (query_id, callback, message_deposit) = match &xcm_query_message {
				Message::XcmQuery { query_id, callback, message_deposit } =>
					(query_id, callback, message_deposit),
				Message::XcmTimeout { .. } => return Err(Error::<T>::RequestTimedOut.into()),
				_ => return Err(Error::<T>::InvalidMessage.into()),
			};

			// Emit event before possible callback execution.
			Self::deposit_event(Event::<T>::XcmResponseReceived {
				dest: initiating_origin.clone(),
				id,
				query_id: *query_id,
				response: xcm_response.clone(),
			});

			if let Some(callback) = callback {
				// Attempt callback with response if specified.
				log::debug!(target: "pop-api::extension", "xcm callback={:?}, response={:?}", callback, xcm_response);
				// Never roll back state if call fails.
				// Ensure that the response can be polled.
				if Self::call(&initiating_origin, callback.to_owned(), &id, &xcm_response).is_ok() {
					Messages::<T>::remove(&initiating_origin, id);
					XcmQueries::<T>::remove(query_id);
					T::Fungibles::release(
						&HoldReason::Messaging.into(),
						&initiating_origin,
						*message_deposit,
						Precision::Exact,
					)?;

					return Ok(())
				}
			}
			// No callback is executed,
			Messages::<T>::insert(
				&initiating_origin,
				id,
				Message::XcmResponse {
					query_id: *query_id,
					message_deposit: *message_deposit,
					response: xcm_response,
				},
			);
			Ok(().into())
		}

		/// Remove a batch of completed or timed-out messages.
		///
		/// Allows users to clean up storage and reclaim deposits for messages that have concluded.
		///
		/// # Parameters
		/// - `origin`: The account removing the messages.
		/// - `messages`: List of message IDs to remove (bounded by `MaxRemovals`).
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::remove(messages.len() as u32))]
		pub fn remove(
			origin: OriginFor<T>,
			messages: BoundedVec<MessageId, T::MaxRemovals>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			for id in &messages {
				let Some(message) = Messages::<T>::get(&origin, id) else {
					return Err(Error::<T>::MessageNotFound.into());
				};

				let (message_deposit, maybe_callback_deposit) = match message {
					Message::Ismp { .. } => Err(Error::<T>::RequestPending),
					Message::XcmQuery { .. } => Err(Error::<T>::RequestPending),
					Message::IsmpResponse { message_deposit, commitment, .. } => {
						Messages::<T>::remove(&origin, id);
						IsmpRequests::<T>::remove(commitment);
						Ok((message_deposit, None))
					},
					Message::XcmResponse { message_deposit, query_id, .. } => {
						Messages::<T>::remove(&origin, id);
						XcmQueries::<T>::remove(query_id);
						Ok((message_deposit, None))
					},
					Message::IsmpTimeout {
						message_deposit, commitment, callback_deposit, ..
					} => {
						Messages::<T>::remove(&origin, id);
						IsmpRequests::<T>::remove(commitment);
						Ok((message_deposit, callback_deposit))
					},
					Message::XcmTimeout { query_id, message_deposit, callback_deposit, .. } => {
						Messages::<T>::remove(&origin, id);
						XcmQueries::<T>::remove(query_id);
						Ok((message_deposit, callback_deposit))
					},
				}?;

				T::Fungibles::release(
					&HoldReason::Messaging.into(),
					&origin,
					message_deposit,
					Precision::Exact,
				)?;
				if let Some(callback_deposit) = maybe_callback_deposit {
					T::Fungibles::release(
						&HoldReason::CallbackGas.into(),
						&origin,
						callback_deposit,
						Precision::Exact,
					)?;
				}
			}

			Self::deposit_event(Event::<T>::Removed { origin, messages: messages.into_inner() });

			Ok(())
		}

		/// Top up the callback weight for a pending message.
		///
		/// This extrinsic allows an origin to increase the gas (weight) budget allocated for a
		/// callback associated with an in-flight message. This is useful when the initially
		/// allocated weight is insufficient to complete the callback.
		///
		/// The additional fee for the new weight is held from the origin using the
		/// `HoldReason::CallbackGas`.
		///
		/// Only pending requests can have their weight increased.
		///
		/// # Parameters
		///
		/// - `origin`: Must be a signed account.
		/// - `message_id`: The identifier of the message to be topped up.
		/// - `additional_weight`: The additional weight to be appended to the message's existing
		///   callback weight.
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::top_up_callback_weight())]
		pub fn top_up_callback_weight(
			origin: OriginFor<T>,
			message_id: MessageId,
			additional_weight: Weight,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			if additional_weight.any_eq(<Weight as Zero>::zero()) {
				return Err(Error::<T>::ZeroWeight.into());
			}

			T::Fungibles::hold(
				&HoldReason::CallbackGas.into(),
				&who,
				T::WeightToFee::weight_to_fee(&additional_weight),
			)?;

			Messages::<T>::try_mutate(&who, message_id, |maybe_message| {
				if let Some(message) = maybe_message {
					// Mutate to accrue new weight.
					let total_weight = match message {
						Message::Ismp { callback, .. } => callback.as_mut().map_or_else(
							|| Err(Error::<T>::NoCallbackFound),
							|cb| Ok(cb.increase_callback_weight(additional_weight)),
						),
						Message::XcmQuery { callback, .. } => callback.as_mut().map_or_else(
							|| Err(Error::<T>::NoCallbackFound),
							|cb| Ok(cb.increase_callback_weight(additional_weight)),
						),
						Message::IsmpResponse { .. } => Err(Error::<T>::MessageCompleted),
						Message::XcmResponse { .. } => Err(Error::<T>::MessageCompleted),
						Message::IsmpTimeout { .. } => Err(Error::<T>::RequestTimedOut),
						Message::XcmTimeout { .. } => Err(Error::<T>::RequestTimedOut),
					}?;

					Self::deposit_event(Event::<T>::CallbackGasIncreased {
						message_id,
						total_weight,
					})
				} else {
					return Err(Error::<T>::MessageNotFound);
				}

				Ok(())
			})?;

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Executes a registered callback with the given input data and manually charges block
		/// weight.
		///
		/// This function is responsible for handling the full lifecycle of a callback invocation:
		/// - Calculating the total weight cost of the callback.
		/// - Ensuring that sufficient blockspace is available before execution.
		/// - Executing the callback via the configured `CallbackExecutor`.
		/// - Registering the actual weight used with the runtime.
		/// - Managing any associated weight fee refund logic.
		///
		/// # Parameters
		///
		/// - `initiating_origin`: The account that triggered the callback. This will be passed to
		///   the executor and used for fee management and event attribution.
		/// - `callback`: The callback definition.
		/// - `id`: The message ID associated with this callback's message.
		/// - `data`: The encoded payload to send to the callback.
		///
		/// # Weight Handling
		///
		/// - Before executing the callback, this function checks whether the total expected weight
		///   (`callback.weight`) can be accommodated in the current block.
		/// - If the block is saturated, the function returns early with an error and does not
		///   mutate state.
		/// - After execution, the actual weight used by the callback is determined using
		///   [`Self::process_callback_weight`] and registered via
		///   [`frame_system::Pallet::<T>::register_extra_weight_unchecked`].
		pub(crate) fn call(
			initiating_origin: &AccountIdOf<T>,
			callback: Callback,
			id: &MessageId,
			data: &impl Encode,
		) -> DispatchResult {
			// This is the total weight that should be deducted from the blockspace for callback
			// execution.
			let max_weight = callback.weight;

			// Dont mutate state if blockspace will be saturated.
			ensure!(
				frame_system::BlockWeight::<T>::get()
					.checked_accrue(max_weight, DispatchClass::Normal)
					.is_ok(),
				Error::<T>::BlockspaceAllowanceReached
			);

			// Execute callback.
			// Its important to note that we must still ensure that the weight used is accounted for
			// in frame_system. Hence all calls after this must not return an err and state
			// should not be rolled back.
			let result = T::CallbackExecutor::execute(
				initiating_origin,
				match callback.abi {
					Abi::Scale => [callback.selector.to_vec(), (id, data).encode()].concat(),
				},
				callback.weight,
			);

			log::debug!(target: "pop-api::extension", "callback weight={:?}, result={result:?}", callback.weight);
			Self::deposit_callback_event(initiating_origin, *id, &callback, &result);
			let callback_weight_used = Self::process_callback_weight(&result, callback.weight);

			// Manually adjust callback weight.
			frame_system::Pallet::<T>::register_extra_weight_unchecked(
				callback_weight_used,
				DispatchClass::Normal,
			);

			match Self::manage_fees(&initiating_origin, callback_weight_used, callback.weight) {
				Ok(_) => (),
				// Dont return early, we must register the weight used by the callback.
				Err(error) =>
					Self::deposit_event(Event::WeightRefundErrored { message_id: *id, error }),
			}
			Ok(())
		}

		/// Deposits an event indicating the outcome of a callback execution.
		///
		/// This function is intended to be called after attempting to dispatch a callback.
		/// It emits either a `CallbackExecuted` or `CallbackFailed` event based on the result.
		///
		/// # Parameters
		///
		/// - `initiating_origin`: The account that originally initiated the message.
		/// - `message_id`: The unique identifier associated with the message that triggered the
		///   callback.
		/// - `callback`: The callback object that was attempted to be executed.
		/// - `result`: The outcome of the callback execution, containing either success or failure.
		pub(crate) fn deposit_callback_event(
			initiating_origin: &T::AccountId,
			message_id: MessageId,
			callback: &Callback,
			result: &DispatchResultWithPostInfo,
		) {
			match result {
				Ok(_) => {
					Self::deposit_event(Event::<T>::CallbackExecuted {
						origin: initiating_origin.clone(),
						id: message_id,
						callback: callback.clone(),
					});
				},
				Err(error) => {
					Self::deposit_event(Event::<T>::CallbackFailed {
						origin: initiating_origin.clone(),
						id: message_id,
						callback: callback.clone(),
						error: error.clone(),
					});
				},
			}
		}

		/// Determines the actual weight consumed by a callback execution, falling back to the
		/// maximum if unknown.
		///
		/// This function is used to calculate the weight to be accounted for after attempting to
		/// dispatch a callback. It ensures that even if the callback execution fails or does not
		/// report actual weight, the worst-case (`max_weight`) is used to avoid under-accounting.
		///
		/// # Parameters
		///
		/// - `result`: The result of the callback dispatch, including any `PostDispatchInfo` if
		///   successful.
		/// - `max_weight`: The maximum weight budgeted for the callback execution.
		///
		/// # Rationale
		///
		/// - Protects against underestimating weight in cases where `actual_weight` is missing or
		///   the dispatch fails.
		/// - Ensures conservative accounting to avoid exceeding block or message limits.
		pub(crate) fn process_callback_weight(
			result: &DispatchResultWithPostInfo,
			max_weight: Weight,
		) -> Weight {
			match result {
				// callback has succeded.
				Ok(post_info) => {
					match post_info.actual_weight {
						Some(w) => w,
						// return the worst case if the callback executor does not populate the
						// actual weight.
						None => max_weight,
					}
				},
				// callback has failed.
				Err(_) => {
					// return the maximum weight.
					max_weight
				},
			}
		}

		/// Handles fee management and refund logic for callback execution.
		///
		/// This function is intended to balance the fees collected upfront for a callback
		/// against the actual weight used during execution. If the callback uses less weight
		/// than originally reserved, the surplus is refunded to the user, and the remainder
		/// is transferred as an execution reward to the fee collector account.
		///
		/// # Parameters
		///
		/// - `initiating_origin`: The account that initially paid for the callback execution.
		/// - `weight_used`: The actual weight consumed by the callback.
		/// - `max_weight`: The maximum weight that was budgeted and paid for in advance.
		pub(crate) fn manage_fees(
			initiating_origin: &AccountIdOf<T>,
			weight_used: Weight,
			max_weight: Weight,
		) -> DispatchResult {
			let weight_to_refund = max_weight.saturating_sub(weight_used);
			let total_deposit = T::WeightToFee::weight_to_fee(&max_weight);
			let reason = HoldReason::CallbackGas.into();

			let to_reward = if weight_to_refund.any_gt(Zero::zero()) {
				let returnable_deposit = T::WeightToFee::weight_to_fee(&weight_to_refund);
				let execution_reward = total_deposit.saturating_sub(returnable_deposit);

				execution_reward
			} else {
				total_deposit
			};

			// Release the deposit.
			T::Fungibles::release(&reason, initiating_origin, total_deposit, Precision::Exact)?;

			// Withdraw assets.
			let credit = T::Fungibles::withdraw(
				&initiating_origin,
				to_reward,
				Precision::Exact,
				Preservation::Preserve,
				Fortitude::Polite,
			)?;

			// Handle assets.
			T::FeeHandler::on_unbalanced(credit);
			Ok(())
		}
	}
}
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
#[repr(u8)]
#[allow(clippy::unnecessary_cast)]
pub enum Read<T: Config> {
	#[codec(index = 0)]
	PollStatus((T::AccountId, MessageId)),
	#[codec(index = 1)]
	GetResponse((T::AccountId, MessageId)),
	#[codec(index = 2)]
	QueryId((T::AccountId, MessageId)),
}

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
pub enum ReadResult {
	Poll(Option<MessageStatus>),
	Get(Option<Vec<u8>>),
	QueryId(Option<QueryId>),
}

impl ReadResult {
	pub fn encode(&self) -> Vec<u8> {
		use ReadResult::*;
		match self {
			Poll(status) => status.encode(),
			Get(response) => response.encode(),
			QueryId(query_id) => query_id.encode(),
		}
	}
}

impl<T: Config> crate::Read for Pallet<T> {
	type Read = Read<T>;
	type Result = ReadResult;

	fn weight(_read: &Self::Read) -> Weight {
		T::DbWeight::get().reads(2)
	}

	fn read(request: Self::Read) -> Self::Result {
		match request {
			Read::PollStatus(request) => ReadResult::Poll(
				Messages::<T>::get(request.0, request.1).map(|m| MessageStatus::from(&m)),
			),
			Read::GetResponse(request) =>
				ReadResult::Get(Messages::<T>::get(request.0, request.1).and_then(|m| match m {
					Message::Ismp { .. } => None,
					Message::XcmQuery { .. } => None,
					Message::IsmpResponse { response, .. } => Some(response.into_inner()),
					Message::XcmResponse { response, .. } => Some(response.encode()),
					Message::IsmpTimeout { .. } => None,
					Message::XcmTimeout { .. } => None,
				})),
			Read::QueryId(request) => ReadResult::QueryId(
				Messages::<T>::get(request.0, request.1).and_then(|m| match m {
					Message::XcmQuery { query_id, .. } => Some(query_id),
					_ => None,
				}),
			),
		}
	}
}

/// Represents a cross-chain message in the system.
///
/// Each variant of this enum captures a different state or type of message lifecycle:
/// - A request in progress.
/// - A response received.
/// - A timeout occurred.
///
/// This is used internally to track, manage, and clean up messages, along with any
/// associated deposits and optional callback metadata.
#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub(crate) enum Message<T: Config> {
	/// Represents a pending ISMP request.
	///
	/// # Fields
	/// - `commitment`: The cryptographic commitment of the request payload.
	/// - `callback`: An optional callback to invoke upon receiving a response.
	/// - `deposit`: The total deposit held to cover message and callback fees.
	Ismp { commitment: H256, callback: Option<Callback>, message_deposit: BalanceOf<T> },

	/// Represents a pending XCM query request.
	///
	/// # Fields
	/// - `query_id`: Unique identifier for the XCM query.
	/// - `callback`: An optional callback for handling the response.
	/// - `deposit`: The deposit held to cover fees for query execution and callback.
	XcmQuery { query_id: QueryId, callback: Option<Callback>, message_deposit: BalanceOf<T> },

	/// Represents a received ISMP response.
	///
	/// # Fields
	/// - `commitment`: The original commitment for the request.
	/// - `deposit`: The held deposit for the message, which may be released or burned.
	/// - `response`: The encoded response payload, size-bounded by `T::MaxResponseLen`.
	IsmpResponse {
		commitment: H256,
		message_deposit: BalanceOf<T>,
		response: BoundedVec<u8, T::MaxResponseLen>,
	},

	/// Represents a received XCM response.
	///
	/// # Fields
	/// - `query_id`: Identifier that matches a previously sent XCM query.
	/// - `deposit`: The deposit originally held for this message.
	/// - `response`: The deserialized response payload.
	XcmResponse { query_id: QueryId, message_deposit: BalanceOf<T>, response: Response },

	/// Represents an ISMP request that timed out before a response was received.
	///
	/// # Fields
	/// - `commitment`: The original commitment of the request.
	/// - `deposit`: The deposit held for the request, which may be reclaimed.
	IsmpTimeout {
		commitment: H256,
		message_deposit: BalanceOf<T>,
		callback_deposit: Option<BalanceOf<T>>,
	},

	/// Represents an XCM query that timed out before a response was received.
	///
	/// # Fields
	/// - `query_id`: The original query ID that timed out.
	/// - `deposit`: The deposit held for the query, which may be reclaimed.
	XcmTimeout {
		query_id: QueryId,
		message_deposit: BalanceOf<T>,
		callback_deposit: Option<BalanceOf<T>>,
	},
}

impl<T: Config> From<&Message<T>> for MessageStatus {
	fn from(value: &Message<T>) -> Self {
		match *value {
			Message::Ismp { .. } => MessageStatus::Pending,
			Message::XcmQuery { .. } => MessageStatus::Pending,
			Message::IsmpResponse { .. } => MessageStatus::Complete,
			Message::XcmResponse { .. } => MessageStatus::Complete,
			Message::IsmpTimeout { .. } => MessageStatus::Timeout,
			Message::XcmTimeout { .. } => MessageStatus::Timeout,
		}
	}
}

/// The related message status of a Message.
#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum MessageStatus {
	Pending,
	Complete,
	Timeout,
}

/// Message selector and pre-paid weight used as gas limit.
#[derive(Copy, Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Callback {
	pub abi: Abi,
	pub selector: [u8; 4],
	pub weight: Weight,
}

impl Callback {
	pub(crate) fn increase_callback_weight(&mut self, additional_weight: Weight) -> Weight {
		let new_callback_weight = self.weight.saturating_add(additional_weight);
		self.weight = new_callback_weight;
		new_callback_weight
	}
}

/// The encoding used for the data going to the contract.
#[derive(Copy, Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum Abi {
	Scale,
}

/// The trait responsible for executing callbacks in response to cross-chain messages.
///
/// Implementors of this trait define the mechanism by which callback data is executed
/// for a given account, along with the expected weight cost of this operation.
///
/// This trait enables customizable and extensible behavior for handling asynchronous
/// responses via optional callback logic — e.g., invoking a runtime call or a smart contract.
pub trait CallbackExecutor<T: Config> {
	/// Execute the callback logic for a specific account with the given encoded payload.
	///
	/// # Parameters
	/// - `account`: The account that initiated the original cross-chain request.
	/// - `data`: Encoded callback data, typically ABI-encoded input including selector and
	///   parameters.
	/// - `weight`: The maximum weight allowed for executing this callback.
	fn execute(account: &T::AccountId, data: Vec<u8>, weight: Weight)
		-> DispatchResultWithPostInfo;

	/// Returns the baseline weight required for a single callback execution.
	///
	/// This serves as an overhead estimate, useful for pallet-level weight calculations.
	fn execution_weight() -> Weight;
}

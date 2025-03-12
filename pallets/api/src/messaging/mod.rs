use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo, PostDispatchInfo},
	pallet_prelude::MaxEncodedLen,
	storage::KeyLenOf,
	traits::{
		fungible::Inspect,
		tokens::{fungible::hold::Mutate, Precision::Exact},
		Get, OriginTrait,
	},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{traits::Saturating, BoundedVec, DispatchError};
use sp_std::vec::Vec;
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

#[cfg(test)]
mod tests;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BlockNumberOf<T> = BlockNumberFor<T>;
type BalanceOf<T> = <<T as Config>::Deposit as Inspect<AccountIdOf<T>>>::Balance;
type DbWeightOf<T> = <T as frame_system::Config>::DbWeight;

pub type MessageId = [u8; 32];

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		dispatch::DispatchResult, pallet_prelude::*, traits::{tokens::fungible::hold::Mutate, OnInitialize},
	};
	use sp_core::H256;
	use sp_runtime::traits::TryConvert;

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

		type CallbackExecutor: CallbackExecutor<Self>;

		/// The deposit mechanism.
		type Deposit: Mutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ Inspect<Self::AccountId>;

		/// The ISMP message dispatcher.
		type IsmpDispatcher: IsmpDispatcher<Account = Self::AccountId, Balance = BalanceOf<Self>>;

		/// The maximum length of any additional application-specific metadata relating to a
		/// request.
		#[pallet::constant]
		type MaxContextLen: Get<u32>;
		/// The maximum length of outbound (posted) data.
		#[pallet::constant]
		type MaxDataLen: Get<u32>;
		#[pallet::constant]
		type MaxKeys: Get<u32>;
		#[pallet::constant]
		type MaxKeyLen: Get<u32>;

		#[pallet::constant]
		type MaxResponseLen: Get<u32>;
		#[pallet::constant]
		type MaxRemovals: Get<u32>;

		/// Overarching hold reason.
		type RuntimeHoldReason: From<HoldReason>;

		type Xcm: NotifyQueryHandler<Self>;

		type XcmResponseOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Location>;

		/// The maximum number of xcm timout updates that can be processed per block.
		#[pallet::constant]
		type MaxXcmQueryTimeoutsPerBlock: Get<u32>;
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
	pub(super) type XcmQueryTimeouts<T: Config> =
		StorageMap<_, Identity, BlockNumberOf<T>, BoundedVec<(T::AccountId, MessageId), T::MaxXcmQueryTimeoutsPerBlock>, ValueQuery>;
	

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
			callback: Option<Callback<T::AccountId>>,
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
			callback: Option<Callback<T::AccountId>>,
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
			callback: Option<Callback<T::AccountId>>,
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
			callback: Callback<T::AccountId>,
		},
		/// A callback has failed.
		CallbackFailed {
			/// The origin of the callback.
			origin: T::AccountId,
			/// The identifier specified for the request.
			id: MessageId,
			/// The callback which failed.
			callback: Callback<T::AccountId>,
			post_info: PostDispatchInfo,
			/// The error which occurred.
			error: DispatchError,
		},
		/// One or more messages have been removed for the origin.
		Removed {
			/// The origin of the messages.
			origin: T::AccountId,
			/// The messages which were removed.
			messages: Vec<MessageId>,
		},
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
		FutureTimeoutMandatory
	}

	/// A reason for the pallet placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// Held for the duration of a message's lifespan.
		#[codec(index = 0)]
		Messaging,
	}

	#[pallet::hooks]
	impl <T: Config> Hooks<BlockNumberOf<T>> for Pallet<T>{
		fn on_initialize(n: BlockNumberOf<T>) -> Weight {
			// As of polkadot-2412 XCM timeouts are not handled by the implementation of OnResponse in pallet-xcm.
			// As a result, we must handle timeouts in the pallet.
			// Iterate through the queries that have expired and update their status.
			let mut weight: Weight = Zero::zero();
			for (origin, message_id) in XcmQueryTimeouts::<T>::get(n) {
				if weight.checked_add(&DbWeightOf::<T>::get().reads_writes(2, 1)).is_some() {
					Messages::<T>::mutate(origin, message_id, |maybe_message|{
						if let Some(Message::XcmQuery { status, .. }) = maybe_message.as_mut() {
							*status = QueryStatus::Timeout;
						}
					})
				} else {
					// maxed out weight, this shouldnt happen if the limit is reasonable.
				}
			}

			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO: does ismp allow querying to ensure that specified para id is supported?
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::zero())]
		pub fn ismp_get(
			origin: OriginFor<T>,
			id: MessageId,
			message: ismp::Get<T>,
			fee: BalanceOf<T>,
			callback: Option<Callback<T::AccountId>>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(!Messages::<T>::contains_key(&origin, &id), Error::<T>::MessageExists);

			let deposit = calculate_protocol_deposit::<T, T::OnChainByteFee>(
				ProtocolStorageDeposit::IsmpRequests,
			) + calculate_message_deposit::<T, T::OnChainByteFee>() +
				calculate_deposit_of::<T, T::OffChainByteFee, ismp::Get<T>>();

			T::Deposit::hold(&HoldReason::Messaging.into(), &origin, deposit)?;

			// Process message by dispatching request via ISMP.
			let commitment = T::IsmpDispatcher::default()
				.dispatch_request(message.into(), FeeMetadata { payer: origin.clone(), fee })
				.map_err(|_| Error::<T>::IsmpDispatchFailed)?;
			// Store commitment for lookup on response, message for querying,
			// response/timeout handling.
			IsmpRequests::<T>::insert(&commitment, (&origin, id));
			Messages::<T>::insert(
				&origin,
				id,
				Message::Ismp {
					commitment,
					callback: callback.clone(),
					deposit,
					status: QueryStatus::Pending,
				},
			);
			Pallet::<T>::deposit_event(Event::<T>::IsmpGetDispatched {
				origin,
				id,
				commitment,
				callback,
			});
			Ok(())
		}

		// TODO: does ismp allow querying to ensure that specified para id is supported?
		#[pallet::call_index(2)]
		#[pallet::weight(Weight::zero())]
		pub fn ismp_post(
			origin: OriginFor<T>,
			id: MessageId,
			message: ismp::Post<T>,
			fee: BalanceOf<T>,
			callback: Option<Callback<T::AccountId>>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(!Messages::<T>::contains_key(&origin, &id), Error::<T>::MessageExists);

			let deposit = calculate_protocol_deposit::<T, T::OnChainByteFee>(
				ProtocolStorageDeposit::IsmpRequests,
			) + calculate_message_deposit::<T, T::OnChainByteFee>() +
				calculate_deposit_of::<T, T::OffChainByteFee, ismp::Post<T>>();

			T::Deposit::hold(&HoldReason::Messaging.into(), &origin, deposit)?;

			// Process message by dispatching request via ISMP.
			let commitment = T::IsmpDispatcher::default()
				.dispatch_request(message.into(), FeeMetadata { payer: origin.clone(), fee })
				.map_err(|_| Error::<T>::IsmpDispatchFailed)?;

			// Store commitment for lookup on response, message for querying,
			// response/timeout handling.
			IsmpRequests::<T>::insert(&commitment, (&origin, id));
			Messages::<T>::insert(
				&origin,
				id,
				Message::Ismp {
					commitment,
					callback: callback.clone(),
					deposit,
					status: QueryStatus::Pending,
				},
			);
			Pallet::<T>::deposit_event(Event::<T>::IsmpPostDispatched {
				origin,
				id,
				commitment,
				callback,
			});
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::zero())]
		pub fn xcm_new_query(
			origin: OriginFor<T>,
			id: MessageId,
			responder: Location,
			timeout: BlockNumberOf<T>,
			callback: Option<Callback<T::AccountId>>,
		) -> DispatchResult {
			let querier_location = T::OriginConverter::try_convert(origin.clone())
				.map_err(|_| Error::<T>::OriginConversionFailed)?;
			let origin = ensure_signed(origin)?;
			ensure!(frame_system::Pallet::<T>::block_number() < timeout, Error::<T>::FutureTimeoutMandatory);

			ensure!(!Messages::<T>::contains_key(&origin, &id), Error::<T>::MessageExists);

			let deposit = calculate_protocol_deposit::<T, T::OnChainByteFee>(
				ProtocolStorageDeposit::XcmQueries,
			)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>());

			T::Deposit::hold(&HoldReason::Messaging.into(), &origin, deposit)?;

			// Process message by creating new query via XCM.
			// Xcm only uses/stores pallet, index - i.e. (u8,u8), hence the fields in xcm_response
			// are ignored.
			let notify = Call::<T>::xcm_response { query_id: 0, response: Default::default() };
			let query_id = T::Xcm::new_notify_query(responder, notify, timeout, querier_location);

			// Store query id for later lookup on response, message for querying status,
			// response/timeout handling.
			XcmQueries::<T>::insert(&query_id, (&origin, id));
			Messages::<T>::insert(
				&origin,
				id,
				Message::XcmQuery {
					query_id,
					callback: callback.clone(),
					deposit,
					status: QueryStatus::Pending,
				},
			);
			Pallet::<T>::deposit_event(Event::<T>::XcmQueryCreated {
				origin,
				id,
				query_id,
				callback,
			});
			Ok(())
		}

		// NOTE: dispatchable should not fail, otherwise response will be lost.
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::zero())] // todo: benchmarking
		pub fn xcm_response(
			origin: OriginFor<T>,
			query_id: QueryId,
			response: Response,
		) -> DispatchResult {
			T::XcmResponseOrigin::ensure_origin(origin)?;
			let (origin, id) = XcmQueries::<T>::get(query_id).ok_or(Error::<T>::InvalidQuery)?;
			let xcm_query_message =
				Messages::<T>::get(&origin, &id).ok_or(Error::<T>::MessageNotFound)?;
			let Message::XcmQuery { query_id, callback, deposit, status } = &xcm_query_message else {
				return Err(Error::<T>::InvalidMessage.into())
			};

			ensure!(*status != QueryStatus::Timeout, Error::<T>::RequestTimedOut);

			// Emit event before possible callback execution.			
			Self::deposit_event(Event::<T>::XcmResponseReceived {
				dest: origin.clone(),
				id,
				query_id: *query_id,
				response: response.clone(),
			});

			if let Some(callback) = callback {
				// Attempt callback with response if specified.
				log::debug!(target: "pop-api::extension", "xcm callback={:?}, response={:?}", callback, response);
				match Self::call(&origin, callback.to_owned(), &id, &response) {
					Ok(_) => {
						// A successfull callback can be removed from storage and its deposit
						// returned. We can ignore the status as we know the response has been
						// received.
						xcm_query_message.remove(&origin, &id);
						T::Deposit::release(
							&HoldReason::Messaging.into(),
							&origin,
							*deposit,
							Exact,
						)?;
					},
					Err(e) => {
						// A problematic callback should still have the ability for polling the
						// response. Update the message to a response with error information.
						Messages::<T>::insert(
							&origin,
							&id,
							Message::XcmResponse {
								query_id: *query_id,
								response,
								deposit: *deposit,
								status: ResponseStatus::CallbackExecutionFailed(e),
							},
						);
					},
				}
			} else {
				// We have no callback here so just update the message to Received and store for
				// polling.
				Messages::<T>::insert(
					&origin,
					&id,
					Message::XcmResponse {
						query_id: *query_id,
						response,
						deposit: *deposit,
						status: ResponseStatus::Received,
					},
				);
			}

			Ok(())
		}

		/// Try and remove a collection of messages.
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::zero())]
		pub fn remove(
			origin: OriginFor<T>,
			messages: BoundedVec<MessageId, T::MaxRemovals>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			for id in &messages {
				let Some(message) = Messages::<T>::get(&origin, id) else {
					return Err(Error::<T>::MessageNotFound.into());
				};

				message.try_remove(&origin, id).and_then(|_| {
					let deposit = match message {
						Message::Ismp { deposit, .. } => deposit,
						Message::IsmpResponse { deposit, .. } => deposit,
						Message::XcmQuery { deposit, .. } => deposit,
						Message::XcmResponse { deposit, .. } => deposit,
					};
					T::Deposit::release(&HoldReason::Messaging.into(), &origin, deposit, Exact)?;

					Ok(())
				})?;
			}
			Self::deposit_event(Event::<T>::Removed { origin, messages: messages.into_inner() });

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	// Attempt to notify via callback.
	fn call(
		origin: &AccountIdOf<T>,
		callback: Callback<T::AccountId>,
		id: &MessageId,
		data: &impl Encode,
	) -> DispatchResult {
		let result = T::CallbackExecutor::execute(
			origin.clone(),
			[callback.selector.to_vec(), (id, data).encode()].concat(),
			callback.weight,
		);

		log::debug!(target: "pop-api::extension", "callback weight={:?}, result={result:?}", callback.weight);
		match result {
			Ok(_post_info) => {
				// todo!("return weight")
				Self::deposit_event(Event::<T>::CallbackExecuted {
					origin: origin.clone(),
					id: id.clone(),
					callback,
				});

				Ok(())
			},
			Err(sp_runtime::DispatchErrorWithPostInfo::<PostDispatchInfo> { post_info, error }) => {
				// Fallback to storing the message for polling - pre-paid weight is lost.
				Self::deposit_event(Event::<T>::CallbackFailed {
					origin: origin.clone(),
					id: id.clone(),
					callback,
					post_info,
					error,
				});
				// TODO: logging
				Err(error)
			},
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
		// TODO: implement benchmarks
		Weight::zero()
	}

	fn read(request: Self::Read) -> Self::Result {
		match request {
			Read::PollStatus(request) =>
				ReadResult::Poll(Messages::<T>::get(request.0, request.1).map(|m| match m {
					Message::Ismp { status, .. } => MessageStatus::QueryStatus(status),
					Message::IsmpResponse { status, .. } => MessageStatus::ResponseStatus(status),
					Message::XcmQuery { status, .. } => MessageStatus::QueryStatus(status),
					Message::XcmResponse { status, .. } => MessageStatus::ResponseStatus(status),
				})),
			Read::GetResponse(request) =>
				ReadResult::Get(Messages::<T>::get(request.0, request.1).and_then(|m| match m {
					Message::Ismp { .. } => None,
					Message::IsmpResponse { response, .. } => Some(response.into_inner()),
					Message::XcmQuery { .. } => None,
					Message::XcmResponse { response, .. } => Some(response.encode()),
				})),
			Read::QueryId(request) => ReadResult::QueryId(
				Messages::<T>::get(request.0, request.1).and_then(|m| match m {
					Message::XcmQuery { query_id, .. } | Message::XcmResponse { query_id, .. } =>
						Some(query_id),
					_ => None,
				}),
			),
		}
	}
}

#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum Message<T: Config> {
	Ismp {
		commitment: H256,
		callback: Option<Callback<T::AccountId>>,
		deposit: BalanceOf<T>,
		status: QueryStatus,
	},
	IsmpResponse {
		commitment: H256,
		deposit: BalanceOf<T>,
		response: BoundedVec<u8, T::MaxResponseLen>,
		status: ResponseStatus,
	},
	XcmQuery {
		query_id: QueryId,
		callback: Option<Callback<T::AccountId>>,
		deposit: BalanceOf<T>,
		status: QueryStatus,
	},
	XcmResponse {
		query_id: QueryId,
		deposit: BalanceOf<T>,
		response: Response,
		status: ResponseStatus,
	},
}

impl<T: Config> Message<T> {
	/// Try and remove self.
	/// Intended for users to remove messages they have sent.
	pub fn try_remove(
		&self,
		origin: &AccountIdOf<T>,
		id: &MessageId,
	) -> Result<(), DispatchError> {
		match self {
			// Ismp messages can only be removed if their status is erroneous.
			Message::Ismp { status, .. } => match status {
				QueryStatus::Pending => Err(Error::<T>::RequestPending.into()),
				QueryStatus::Timeout | QueryStatus::Err(_) => {
					self.remove(origin, id);
					Ok(())
				},
			},
			// Ismp responses can always be removed.
			Message::IsmpResponse { .. } => {
				self.remove(origin, id);
				Ok(())
			},
			// Xcm queries can only be removed if their status is erroneous.
			Message::XcmQuery { status, .. } => match status {
				QueryStatus::Pending => Err(Error::<T>::RequestPending.into()),
				QueryStatus::Timeout | QueryStatus::Err(_) => {
					self.remove(origin, id);
					Ok(())
				},
			},
			// XCM responses can always be removed.
			Message::XcmResponse { .. } => {
				self.remove(origin, id);
				Ok(())
			},
		}
	}

	/// Remove a message from storage.
	/// Does no check on whether a message should be removed.
	pub(crate) fn remove(&self, origin: &AccountIdOf<T>, id: &MessageId) {
		Messages::<T>::remove(&origin, &id);
		match self {
			Message::Ismp { commitment, .. } => {
				IsmpRequests::<T>::remove(commitment);
			},
			Message::IsmpResponse { commitment, .. } => {
				IsmpRequests::<T>::remove(commitment);
			},
			Message::XcmQuery { query_id, .. } => {
				XcmQueries::<T>::remove(query_id);
			},
			Message::XcmResponse { query_id, .. } => {
				XcmQueries::<T>::remove(query_id);
			},
		}
	}
}

#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum MessageStatus {
	QueryStatus(QueryStatus),
	ResponseStatus(ResponseStatus),
}

#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum QueryStatus {
	/// A query has been sent and is pending a response.
	Pending,
	/// An error has occurred with this message.
	Err(DispatchError),
	/// A timeout has occurred.
	Timeout,
}

#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum ResponseStatus {
	/// A query has been sent and is pending a response.
	Received,
	/// The specified callback has errored.
	CallbackExecutionFailed(DispatchError),
	/// An error has occurred with this message.
	Err(DispatchError),
}

#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum CallbackErrorReason {
	NotEnoughGas,
	BadExecution,
}

// Message selector and pre-paid weight used as gas limit
#[derive(Copy, Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Callback<AccountId> {
	pub selector: [u8; 4],
	pub weight: Weight,
	pub spare_weight_creditor: AccountId,
}

pub trait CallbackExecutor<T: Config> {
	fn execute(account: T::AccountId, data: Vec<u8>, weight: Weight) -> DispatchResultWithPostInfo;

	fn execution_weight() -> Weight;
}

//! TODO: pallet docs.

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
use sp_runtime::{traits::Saturating, BoundedVec, DispatchError, SaturatedConversion};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};
use transports::{
	ismp::{self as ismp, FeeMetadata, IsmpDispatcher},
	xcm::{self as xcm, Location, QueryId},
};
pub use xcm::NotifyQueryHandler;
use xcm::Response;

use super::Weight;

/// Messaging transports.
pub mod transports;

#[cfg(test)]
mod tests;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BlockNumberOf<T> = BlockNumberFor<T>;
type BalanceOf<T> = <<T as Config>::Deposit as Inspect<AccountIdOf<T>>>::Balance;
pub type MessageId = [u8; 32];

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		storage::KeyLenOf,
		traits::tokens::{fungible::hold::Mutate, Precision::Exact},
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

		#[pallet::constant]
		type ByteFee: Get<BalanceOf<Self>>;

		type CallbackExecutor: CallbackExecutor<Self>;

		/// The deposit mechanism.
		type Deposit: Mutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ Inspect<Self::AccountId>;

		#[pallet::constant]
		type IsmpByteFee: Get<BalanceOf<Self>>;

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
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Messages<T: Config> = StorageDoubleMap<
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
	}

	/// A reason for the pallet placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// Held for the duration of a message's lifespan.
		#[codec(index = 0)]
		Messaging,
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

			// Calculate deposit and place on hold.
			let deposit = Self::calculate_deposit(
				message.calculate_deposit() +
					// IsmpRequests
					(KeyLenOf::<IsmpRequests<T>>::get() as usize +
						AccountIdOf::<T>::max_encoded_len() +
						MessageId::max_encoded_len())
					.saturated_into::<BalanceOf<T>>() *
						T::ByteFee::get(),
			);
			T::Deposit::hold(&HoldReason::Messaging.into(), &origin, deposit)?;

			// Process message by dispatching request via ISMP.
			let maybe_commitment = T::IsmpDispatcher::default()
				.dispatch_request(message.into(), FeeMetadata { payer: origin.clone(), fee });

			match maybe_commitment {
				Ok(commitment) => {
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
							status: MessageStatus::Ok,
						},
					);
					Pallet::<T>::deposit_event(Event::<T>::IsmpGetDispatched {
						origin,
						id,
						commitment,
						callback,
					});
					Ok(())
				},
				Err(e) => {
					// Allow a caller to poll for the status still and retreive the message deposit.
					Messages::<T>::insert(
						&origin,
						id,
						Message::Ismp {
							commitment: Default::default(),
							callback,
							deposit,
							status: MessageStatus::Err(Error::<T>::IsmpDispatchFailed.into()),
						},
					);
					Err(Error::<T>::IsmpDispatchFailed.into())
				},
			}
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

			// Calculate deposit and place on hold.
			let deposit = Self::calculate_deposit(
				message.calculate_deposit() +
					// IsmpRequests
					(KeyLenOf::<IsmpRequests<T>>::get() as usize +
						AccountIdOf::<T>::max_encoded_len() +
						MessageId::max_encoded_len())
						.saturated_into::<BalanceOf<T>>() *
						T::ByteFee::get(),
			);
			T::Deposit::hold(&HoldReason::Messaging.into(), &origin, deposit)?;

			// Process message by dispatching request via ISMP.
			let maybe_commitment = T::IsmpDispatcher::default()
				.dispatch_request(message.into(), FeeMetadata { payer: origin.clone(), fee });

			match maybe_commitment {
				Ok(commitment) => {
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
							status: MessageStatus::Ok,
						},
					);
					Pallet::<T>::deposit_event(Event::<T>::IsmpPostDispatched {
						origin,
						id,
						commitment,
						callback,
					});
					Ok(())
				},
				Err(e) => {
					// Allow a caller to poll for the status still and retreive the message deposit.
					Messages::<T>::insert(
						&origin,
						id,
						Message::Ismp {
							commitment: Default::default(),
							callback,
							deposit,
							status: MessageStatus::Err(Error::<T>::IsmpDispatchFailed.into()),
						},
					);
					Err(Error::<T>::IsmpDispatchFailed.into())
				},
			}
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
			let origin = ensure_signed(origin)?;
			let querier = T::OriginConverter::try_convert(T::RuntimeOrigin::signed(origin.clone()))
				.map_err(|_| Error::<T>::OriginConversionFailed)?;

			ensure!(!Messages::<T>::contains_key(&origin, &id), Error::<T>::MessageExists);
			// Calculate deposit and place on hold.
			let deposit = Self::calculate_deposit(
				// XcmQueries
				(KeyLenOf::<XcmQueries<T>>::get() as usize +
					AccountIdOf::<T>::max_encoded_len() +
					MessageId::max_encoded_len() +
					Option::<Callback<T::AccountId>>::max_encoded_len())
				.saturated_into::<BalanceOf<T>>() *
					T::ByteFee::get(),
			);
			T::Deposit::hold(&HoldReason::Messaging.into(), &origin, deposit)?;

			// Process message by creating new query via XCM.
			// Xcm only uses/stores pallet, index - i.e. (u8,u8), hence the fields in xcm_response
			// are ignored.
			let notify = Call::<T>::xcm_response { query_id: 0, response: Default::default() };
			let query_id = T::Xcm::new_notify_query(responder, notify, timeout, querier);

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
					status: MessageStatus::Ok,
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

			// Lookup message from query id.
			let (origin, id) = XcmQueries::<T>::get(query_id).ok_or(Error::<T>::InvalidQuery)?;
			let Some(Message::XcmQuery { query_id, callback, deposit, status }) =
				Messages::<T>::get(&origin, &id)
			else {
				return Err(Error::<T>::InvalidMessage.into())
			};

			// Attempt callback with response if specified.
			if let Some(callback) = callback {
				log::debug!(target: "pop-api::extension", "xcm callback={:?}, response={:?}", callback, response);
				if Self::call(origin.clone(), callback, id, &response, deposit).is_ok() {
					Self::deposit_event(Event::<T>::XcmResponseReceived {
						dest: origin,
						id,
						query_id,
						response,
					});
					return Ok(());
				}
			}

			// Otherwise store response for manual retrieval and removal.
			Messages::<T>::insert(
				&origin,
				&id,
				Message::XcmResponse {
					query_id,
					response: response.clone(),
					deposit,
					status: MessageStatus::Ok,
				},
			);
			Self::deposit_event(Event::<T>::XcmResponseReceived {
				dest: origin,
				id,
				query_id,
				response,
			});
			Ok(())
		}

		// Remove a request/response, returning any deposit previously taken.
		/// Will ignore any erroneous messages and continue trying to process the remainder.
		#[frame_support::transactional]
		#[pallet::call_index(5)]
		#[pallet::weight(Weight::zero())]
		pub fn remove(
			origin: OriginFor<T>,
			mut messages: BoundedVec<MessageId, T::MaxRemovals>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			for id in &messages {
				let Some(message) = Messages::<T>::get(&origin, id) else {
					return Err(Error::<T>::MessageNotFound.into());
				};

				message.try_remove(&origin, id).and_then(|_| message.release_deposit(&origin))?;
			}

			Self::deposit_event(Event::<T>::Removed { origin, messages: messages.into_inner() });

			Ok(())
		}
	}
}
impl<T: Config> Pallet<T> {
	// Calculate the deposit required for a particular message.
	fn calculate_deposit(deposit: BalanceOf<T>) -> BalanceOf<T> {
		// Add amount for `Messages` key and value
		deposit.saturating_add(
			(KeyLenOf::<Messages<T>>::get().saturated_into::<BalanceOf<T>>() +
				Message::<T>::max_encoded_len().saturated_into::<BalanceOf<T>>()) *
				T::ByteFee::get(),
		)
	}

	// Attempt to notify via callback.
	fn call(
		origin: AccountIdOf<T>,
		callback: Callback<T::AccountId>,
		id: MessageId,
		data: &impl Encode,
		deposit: BalanceOf<T>,
	) -> DispatchResult {
		let result = T::CallbackExecutor::execute(
			origin.clone(),
			[callback.selector.to_vec(), (id, data).encode()].concat(),
			callback.weight,
		);
		log::debug!(target: "pop-api::extension", "callback weight={:?}, result={result:?}", callback.weight);
		match result {
			Ok(_post_info) => {
				// TODO: do something with post_info: e.g. refund unused weight
				// Return deposit.
				T::Deposit::release(&HoldReason::Messaging.into(), &origin, deposit, Exact)?;
				Messages::<T>::remove(&origin, &id);
				Self::deposit_event(Event::<T>::CallbackExecuted {
					origin: origin.clone(),
					id,
					callback,
				});
				Self::deposit_event(Event::<T>::Removed {
					origin: origin.clone(),
					messages: [id].to_vec(),
				});
				Ok(())
			},
			Err(sp_runtime::DispatchErrorWithPostInfo::<PostDispatchInfo> { post_info, error }) => {
				// Fallback to storing the message for polling - pre-paid weight is lost.
				Self::deposit_event(Event::<T>::CallbackFailed {
					origin: origin.clone(),
					id,
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

#[derive(Clone, Decode, Debug, Encode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum Status {
	Pending,
	TimedOut,
	Complete,
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
					Message::Ismp { status, .. } => status,
					Message::IsmpResponse { status, .. } => status,
					Message::XcmQuery { status, .. } => status,
					Message::XcmResponse { status, .. } => status,
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

trait CalculateDeposit<Deposit> {
	fn calculate_deposit(&self) -> Deposit;
}

#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
enum Message<T: Config> {
	Ismp {
		commitment: H256,
		callback: Option<Callback<T::AccountId>>,
		deposit: BalanceOf<T>,
		status: MessageStatus,
	},
	IsmpResponse {
		commitment: H256,
		deposit: BalanceOf<T>,
		response: BoundedVec<u8, T::MaxResponseLen>,
		status: MessageStatus,
	},
	XcmQuery {
		query_id: QueryId,
		callback: Option<Callback<T::AccountId>>,
		deposit: BalanceOf<T>,
		status: MessageStatus,
	},
	XcmResponse {
		query_id: QueryId,
		deposit: BalanceOf<T>,
		response: Response,
		status: MessageStatus,
	},
}

impl<T: Config> Message<T> {
	/// Try and remove self.
	pub fn try_remove(&self, origin: &AccountIdOf<T>, id: &MessageId) -> Result<(), DispatchError> {
		match self {
			/// Ismp messages can only be removed if their status is erroneous.
			Message::Ismp { status, .. } => match status {
				MessageStatus::Ok => Err(Error::<T>::RequestPending.into()),
				MessageStatus::Timeout | MessageStatus::Err(_) => {
					self.remove(origin, id);
					Ok(())
				},
			},
			/// Ismp responses can always be removed.
			Message::IsmpResponse { .. } => {
				self.remove(origin, id);
				Ok(())
			},
			/// Xcm queries can only be removed if their status is erroneous.
			Message::XcmQuery { status, .. } => match status {
				MessageStatus::Ok => Err(Error::<T>::RequestPending.into()),
				MessageStatus::Timeout | MessageStatus::Err(_) => {
					self.remove(origin, id);
					Ok(())
				},
			},
			/// XCM responses can always be removed.
			Message::XcmResponse { .. } => {
				self.remove(origin, id);
				Ok(())
			},
		}
	}

	/// Remove a message from storage.
	/// Does no check on wether a message should be removed.
	fn remove(&self, origin: &AccountIdOf<T>, id: &MessageId) {
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

	pub fn release_deposit(&self, origin: &T::AccountId) -> Result<(), DispatchError> {
		let deposit = match self {
			Message::Ismp { deposit, .. } => deposit,
			Message::IsmpResponse { deposit, .. } => deposit,
			Message::XcmQuery { deposit, .. } => deposit,
			Message::XcmResponse { deposit, .. } => deposit,
		};
		T::Deposit::release(&HoldReason::Messaging.into(), origin, *deposit, Exact)?;
		Ok(())
	}
}

#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum MessageStatus {
	/// No errors have been recorded.
	Ok,
	/// An error has occurred with this message>
	Err(DispatchError),
	/// A timeout has occurred
	Timeout,
}

#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum CallbackExecutionStatus {
	Failure(CallbackErrorReason),
	Success,
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

//! TODO: pallet docs.

use codec::{Decode, Encode};
use frame_support::{
	ensure,
	pallet_prelude::MaxEncodedLen,
	traits::{
		fungible::{Inspect, MutateHold},
		Get, OriginTrait,
	},
	CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	traits::{Saturating, TryConvert},
	BoundedVec, DispatchResult, SaturatedConversion,
};
use sp_std::vec::Vec;
use transports::{
	ismp::{self as ismp, FeeMetadata, GetOf, IsmpDispatcher, PostOf},
	xcm::{self as xcm, Location, QueryHandler, QueryId, VersionedLocation},
};

use super::Weight;

/// Messaging transports.
pub mod transports;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BlockNumberOf<T> = BlockNumberFor<T>;
type BalanceOf<T> = <<T as Config>::Deposit as Inspect<AccountIdOf<T>>>::Balance;
type MaxContextLenOf<T> = <T as Config>::MaxContextLen;
type MaxKeysOf<T> = <T as Config>::MaxKeys;
type MaxKeyLenOf<T> = <T as Config>::MaxKeyLen;
type MaxDataLenOf<T> = <T as Config>::MaxDataLen;
type MessageId = u64;
type MessageOf<T> = Message<
	BalanceOf<T>,
	BlockNumberOf<T>,
	MaxContextLenOf<T>,
	MaxKeysOf<T>,
	MaxKeyLenOf<T>,
	MaxDataLenOf<T>,
>;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
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

		type Xcm: QueryHandler<BlockNumber = BlockNumberOf<Self>>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Requests<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		MessageId,
		// TODO: improve
		(Status, BalanceOf<T>, Option<H256>, Option<QueryId>),
		OptionQuery,
	>;

	#[pallet::storage]
	pub(super) type IsmpRequests<T: Config> =
		StorageMap<_, Identity, H256, (T::AccountId, MessageId), OptionQuery>;

	#[pallet::storage]
	pub(super) type XcmRequests<T: Config> =
		StorageMap<_, Identity, QueryId, (T::AccountId, MessageId), OptionQuery>;

	#[pallet::storage]
	pub(super) type Responses<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		MessageId,
		BoundedVec<u8, T::MaxResponseLen>,
		OptionQuery,
	>;

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// TODO: split into Dispatched/NewQueryCreated
		Requested { origin: T::AccountId, id: MessageId },
		Removed { origin: T::AccountId, messages: Vec<MessageId> },
		ResponseReceived { dest: T::AccountId, id: MessageId },
	}

	#[pallet::error]
	pub enum Error<T> {
		DispatchFailed,
		InvalidRequest,
		OriginConversionFailed,
		RequestExists,
		RequestPending,
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
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())] // todo: benchmarking after consolidating storage
		pub fn send(_origin: OriginFor<T>, _id: MessageId) -> DispatchResult {
			todo!(
				"Reserved for messaging abstractions - e.g. Message::StateQuery { dest: 1000, \
				 keys: vec![] }"
			)
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::zero())] // todo: benchmarking after consolidating storage
		pub fn ismp_get(
			origin: OriginFor<T>,
			id: MessageId,
			message: GetOf<T>,
			fee: BalanceOf<T>,
		) -> DispatchResult {
			Self::do_send(
				origin,
				MessageOf::<T>::Ismp { id, message: ismp::Message::Get(message), fee },
			)
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::zero())] // todo: benchmarking after consolidating storage
		pub fn ismp_post(
			origin: OriginFor<T>,
			id: MessageId,
			message: PostOf<T>,
			fee: BalanceOf<T>,
		) -> DispatchResult {
			Self::do_send(
				origin,
				MessageOf::<T>::Ismp { id, message: ismp::Message::Post(message), fee },
			)
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::zero())] // todo: benchmarking after consolidating storage
		pub fn xcm_new_query(
			origin: OriginFor<T>,
			id: u64,
			responder: VersionedLocation,
			timeout: BlockNumberOf<T>,
		) -> DispatchResult {
			Self::do_send(origin, MessageOf::<T>::Xcm { id, responder, timeout })
		}

		// Remove a request/response, returning any deposit previously taken.
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::zero())] // todo: benchmarking after consolidating storage
		pub fn remove(
			origin: OriginFor<T>,
			messages: BoundedVec<MessageId, T::MaxRemovals>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			for id in &messages {
				// Ensure request exists and is not pending.
				let Some((status, deposit, ismp, xcm)) = Requests::<T>::take(&origin, id) else {
					return Err(Error::<T>::InvalidRequest.into());
				};
				ensure!(status != Status::Pending, Error::<T>::RequestPending);
				// Remove associated data and return deposit.
				Responses::<T>::remove(&origin, id);
				if let Some(commitment) = ismp {
					IsmpRequests::<T>::remove(commitment);
				};
				if let Some(query_id) = xcm {
					XcmRequests::<T>::remove(query_id);
				};
				T::Deposit::release(&HoldReason::Messaging.into(), &origin, deposit, Exact)?;
			}
			Pallet::<T>::deposit_event(Event::<T>::Removed {
				origin,
				messages: messages.into_inner(),
			});
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	// Calculate the deposit required for a particular request.
	fn calculate_deposit(message: &MessageOf<T>) -> BalanceOf<T> {
		let mut deposit = BalanceOf::<T>::default();

		// Add amount for `Requests` key.
		let request_key_len: BalanceOf<T> =
			(T::AccountId::max_encoded_len() + MessageId::max_encoded_len()).saturated_into();
		deposit.saturating_accrue(T::ByteFee::get() * request_key_len);

		// Add amount for `Requests` value.
		let request_value_len = Status::max_encoded_len() +
			BalanceOf::<T>::max_encoded_len() +
			// todo: could be optimised away
			Option::<H256>::max_encoded_len();
		deposit.saturating_accrue(T::ByteFee::get() * request_value_len.saturated_into());

		match &message {
			Message::Ismp { message: request, .. } => {
				// Determine length of variable data included in the request.
				// todo: use ismp::DispatchRequest types instead
				let len = match request {
					ismp::Message::Get(GetOf::<T> {
						dest: para,
						height,
						timeout,
						context,
						keys,
					}) =>
						para.encoded_size() +
							height.encoded_size() + timeout.encoded_size() +
							context.len() + keys.iter().map(|k| k.len()).sum::<usize>(),
					ismp::Message::Post(PostOf::<T> { dest: para, timeout, data }) =>
						para.encoded_size() + timeout.encoded_size() + data.len(),
				};

				// Add amount for data held by ISMP, using separate ISMP byte fee.
				deposit.saturating_accrue(T::IsmpByteFee::get() * len.saturated_into());

				// Add amount for `IsmpRequests` lookup.
				deposit.saturating_accrue(
					T::ByteFee::get() *
						(H256::max_encoded_len().saturated_into::<BalanceOf<T>>() +
							request_key_len),
				);
			},
			Message::Xcm { .. } => {},
		}

		// Add amount for storing response.
		deposit.saturating_accrue(T::ByteFee::get() * request_key_len);
		deposit.saturating_accrue(T::ByteFee::get() * T::MaxResponseLen::get().saturated_into());

		deposit
	}

	fn do_send(origin: OriginFor<T>, message: MessageOf<T>) -> DispatchResult {
		let origin = ensure_signed(origin)?;
		// Calculate deposit for request and place on hold.
		let deposit = Self::calculate_deposit(&message);
		T::Deposit::hold(&HoldReason::Messaging.into(), &origin, deposit)?;
		// Process request.
		let (id, request) = match message {
			// TODO: does ismp allow querying to ensure that specified para id is supported?
			Message::Ismp { id, message, fee } => {
				ensure!(!Requests::<T>::contains_key(&origin, &id), Error::<T>::RequestExists);
				// Dispatch request via ISMP.
				let commitment = T::IsmpDispatcher::default()
					.dispatch_request(message.into(), FeeMetadata { payer: origin.clone(), fee })
					.map_err(|_| Error::<T>::DispatchFailed)?;
				// Store commitment for later lookup on response.
				IsmpRequests::<T>::insert(&commitment, (&origin, id));
				(id, (Status::Pending, deposit, Some(commitment), None::<QueryId>))
			},
			Message::Xcm { id, responder, timeout } => {
				ensure!(!Requests::<T>::contains_key(&origin, &id), Error::<T>::RequestExists);
				let responder = Location::try_from(responder)
					.map_err(|_| Error::<T>::OriginConversionFailed)?;
				// TODO: neater way of doing this
				let querier =
					T::OriginConverter::try_convert(T::RuntimeOrigin::signed(origin.clone()))
						.map_err(|_| Error::<T>::OriginConversionFailed)?;
				let query_id = T::Xcm::new_query(responder, timeout, querier);
				// Store query id for later lookup on response.
				XcmRequests::<T>::insert(&query_id, (&origin, id));
				(id, (Status::Pending, deposit, None::<H256>, Some(query_id)))
			},
		};
		// Store request for querying, response/timeout handling
		Requests::<T>::insert(&origin, id, request);
		// TODO: event per dispatchable for determining usage
		Pallet::<T>::deposit_event(Event::<T>::Requested { origin, id });
		Ok(())
	}
}

#[derive(CloneNoBound, DebugNoBound, Encode, EqNoBound, Decode, PartialEqNoBound, TypeInfo)]
#[scale_info(skip_type_params(MaxContextLen, MaxKeys, MaxKeyLen, MaxDataLen))]
pub enum Message<
	Balance: Clone + core::fmt::Debug + PartialEq,
	BlockNumber: Clone + core::fmt::Debug + PartialEq,
	MaxContextLen: Get<u32>,
	MaxKeys: Get<u32>,
	MaxKeyLen: Get<u32>,
	MaxDataLen: Get<u32>,
> {
	Ismp {
		id: MessageId,
		message: ismp::Message<MaxContextLen, MaxKeys, MaxKeyLen, MaxDataLen>,
		fee: Balance,
	},
	Xcm {
		id: MessageId,
		responder: VersionedLocation,
		timeout: BlockNumber,
	},
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
	Poll((T::AccountId, MessageId)),
	#[codec(index = 1)]
	Get((T::AccountId, MessageId)),
	#[codec(index = 2)]
	QueryId((T::AccountId, MessageId)),
}

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
pub enum ReadResult<T: Config> {
	Poll(Option<Status>),
	Get(Option<BoundedVec<u8, T::MaxResponseLen>>),
	QueryId(Option<QueryId>),
}

impl<T: Config> ReadResult<T> {
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
	type Result = ReadResult<T>;

	fn weight(_read: &Self::Read) -> Weight {
		// TODO: implement benchmarks
		Weight::zero()
	}

	fn read(request: Self::Read) -> Self::Result {
		match request {
			Read::Poll(request) =>
				ReadResult::Poll(Requests::<T>::get(request.0, request.1).map(|r| r.0)),
			Read::Get(request) => ReadResult::Get(Responses::<T>::get(request.0, request.1)),
			Read::QueryId(request) =>
				ReadResult::QueryId(Requests::<T>::get(request.0, request.1).and_then(|r| r.3)),
		}
	}
}

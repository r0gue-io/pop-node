//! TODO: pallet docs.

use codec::{Decode, Encode};
use frame_support::{
	pallet_prelude::MaxEncodedLen,
	traits::{fungible::Inspect, Get},
	CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound,
};
pub use pallet::*;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{traits::Saturating, BoundedVec, SaturatedConversion};
use sp_std::vec::Vec;

use super::Weight;

pub mod ismp;
mod xcm;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Deposit as Inspect<AccountIdOf<T>>>::Balance;
type MaxContextLenOf<T> = <T as Config>::MaxContextLen;
type MaxKeysOf<T> = <T as Config>::MaxKeys;
type MaxKeyLenOf<T> = <T as Config>::MaxKeyLen;
type MaxDataLenOf<T> = <T as Config>::MaxDataLen;
type RequestId = u64;
type RequestOf<T> =
	Request<BalanceOf<T>, MaxContextLenOf<T>, MaxKeysOf<T>, MaxKeyLenOf<T>, MaxDataLenOf<T>>;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::tokens::{fungible::hold::Mutate, Precision::Exact},
	};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;

	use super::{
		ismp::{FeeMetadata, IsmpDispatcher},
		*,
	};

	/// Configuration of the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
		#[pallet::constant]
		type MaxKeys: Get<u32>;
		#[pallet::constant]
		type MaxKeyLen: Get<u32>;

		/// The maximum length of outbound (posted) data.
		#[pallet::constant]
		type MaxDataLen: Get<u32>;
		#[pallet::constant]
		type MaxResponseLen: Get<u32>;

		/// Overarching hold reason.
		type RuntimeHoldReason: From<HoldReason>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub(super) type Requests<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		RequestId,
		// TODO: improve
		(Status, BalanceOf<T>, Option<H256>),
		OptionQuery,
	>;

	#[pallet::storage]
	pub(super) type IsmpRequests<T: Config> =
		StorageMap<_, Identity, H256, (T::AccountId, RequestId), OptionQuery>;

	#[pallet::storage]
	pub(super) type Responses<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		RequestId,
		BoundedVec<u8, T::MaxResponseLen>,
		OptionQuery,
	>;

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Requested { id: (T::AccountId, RequestId) },
		Removed { id: (T::AccountId, RequestId) },
		ResponseReceived { id: (T::AccountId, RequestId) },
	}

	#[pallet::error]
	pub enum Error<T> {
		DispatchFailed,
		InvalidRequest,
		RequestExists,
		RequestPending,
	}

	/// A reason for the pallet placing a hold on funds.
	#[pallet::composite_enum]
	pub enum HoldReason {
		/// Held for a cross chain request.
		#[codec(index = 0)]
		CrossChainRequest,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Register/dispatch an async request for data.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn request(origin: OriginFor<T>, request: RequestOf<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// Calculate deposit for request and place on hold.
			let deposit = Self::calculate_deposit(&request);
			T::Deposit::hold(&HoldReason::CrossChainRequest.into(), &caller, deposit)?;
			// Process request.
			let (id, request) = match request {
				// TODO: does ismp allow querying to ensure that specified para id is supported?
				Request::Ismp { id, request, fee } => {
					ensure!(!Requests::<T>::contains_key(&caller, &id), Error::<T>::RequestExists);
					// Dispatch request via ISMP.
					let commitment = T::IsmpDispatcher::default()
						.dispatch_request(
							request.into(),
							FeeMetadata { payer: caller.clone(), fee },
						)
						.map_err(|_| Error::<T>::DispatchFailed)?;
					// Store commitment for later lookup on response.
					let id = (caller, id);
					IsmpRequests::<T>::insert(&commitment, &id);
					(id, (Status::Pending, deposit, Some(commitment)))
				},
				Request::Xcm { id } => {
					ensure!(!Requests::<T>::contains_key(&caller, &id), Error::<T>::RequestExists);
					((caller, id), (Status::Pending, deposit, None::<H256>))
				},
			};
			// Store request for querying, response/timeout handling
			Requests::<T>::insert(&id.0, &id.1, request);
			Pallet::<T>::deposit_event(Event::<T>::Requested { id });
			Ok(())
		}

		// Remove a request/response, returning any deposit previously taken.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::zero())]
		pub fn remove(origin: OriginFor<T>, request: RequestId) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			// Ensure request exists and is not pending.
			let id = (caller, request);
			let Some((status, deposit, ismp)) = Requests::<T>::take(&id.0, &id.1) else {
				return Err(Error::<T>::InvalidRequest.into());
			};
			ensure!(status != Status::Pending, Error::<T>::RequestPending);
			// Remove associated data and return deposit.
			Responses::<T>::remove(&id.0, &id.1);
			if let Some(commitment) = ismp {
				IsmpRequests::<T>::remove(commitment);
			};
			T::Deposit::release(&HoldReason::CrossChainRequest.into(), &id.0, deposit, Exact)?;
			Pallet::<T>::deposit_event(Event::<T>::Removed { id });
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	// Calculate the deposit required for a particular request.
	fn calculate_deposit(request: &RequestOf<T>) -> BalanceOf<T> {
		let mut deposit = BalanceOf::<T>::default();

		// Add amount for `Requests` key.
		let request_key_len: BalanceOf<T> =
			(T::AccountId::max_encoded_len() + RequestId::max_encoded_len()).saturated_into();
		deposit.saturating_accrue(T::ByteFee::get() * request_key_len);

		// Add amount for `Requests` value.
		let request_value_len = Status::max_encoded_len() +
			BalanceOf::<T>::max_encoded_len() +
			// todo: could be optimised away
			Option::<H256>::max_encoded_len();
		deposit.saturating_accrue(T::ByteFee::get() * request_value_len.saturated_into());

		match &request {
			Request::Ismp { request, .. } => {
				// Determine length of variable data included in the request.
				// todo: use ismp::DispatchRequest types instead
				let len = match request {
					ismp::Request::Get { para, height, timeout, context, keys } =>
						para.encoded_size() +
							height.encoded_size() + timeout.encoded_size() +
							context.len() + keys.iter().map(|k| k.len()).sum::<usize>(),
					ismp::Request::Post { para, timeout, data } =>
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
			Request::Xcm { .. } => {},
		}

		// Add amount for storing response.
		deposit.saturating_accrue(T::ByteFee::get() * request_key_len);
		deposit.saturating_accrue(T::ByteFee::get() * T::MaxResponseLen::get().saturated_into());

		deposit
	}
}

#[derive(CloneNoBound, DebugNoBound, Encode, EqNoBound, Decode, PartialEqNoBound, TypeInfo)]
#[scale_info(skip_type_params(MaxContextLen, MaxKeys, MaxKeyLen, MaxDataLen))]
pub enum Request<
	Balance: Clone + core::fmt::Debug + PartialEq,
	MaxContextLen: Get<u32>,
	MaxKeys: Get<u32>,
	MaxKeyLen: Get<u32>,
	MaxDataLen: Get<u32>,
> {
	Ismp {
		id: RequestId,
		request: ismp::Request<MaxContextLen, MaxKeys, MaxKeyLen, MaxDataLen>,
		fee: Balance,
	},
	Xcm {
		id: RequestId,
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
	Poll((T::AccountId, RequestId)),
	#[codec(index = 1)]
	Get((T::AccountId, RequestId)),
}

#[derive(Debug)]
#[cfg_attr(feature = "std", derive(PartialEq, Clone))]
pub enum ReadResult<T: Config> {
	Poll(Option<Status>),
	Get(Option<BoundedVec<u8, T::MaxResponseLen>>),
}

impl<T: Config> ReadResult<T> {
	pub fn encode(&self) -> Vec<u8> {
		use ReadResult::*;
		match self {
			Poll(status) => status.encode(),
			Get(response) => response.encode(),
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
		}
	}
}

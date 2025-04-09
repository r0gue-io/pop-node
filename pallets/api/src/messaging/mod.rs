extern crate alloc;

pub use alloc::borrow::ToOwned;

use ::ismp::Error as IsmpError;
use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo, PostDispatchInfo},
	pallet_prelude::{MaxEncodedLen, Zero},
	storage::KeyLenOf,
	traits::{
		fungible::Inspect,
		tokens::{
			fungible::hold::Mutate,
			Fortitude,
			Precision::{BestEffort, Exact},
			Preservation, Restriction,
		},
		Get, OriginTrait,
	},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{traits::Saturating, BoundedVec, DispatchError};
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

#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "runtime-benchmarks"))]
pub(crate) mod test_utils;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BlockNumberOf<T> = BlockNumberFor<T>;
type BalanceOf<T> = <<T as Config>::Fungibles as Inspect<AccountIdOf<T>>>::Balance;
type DbWeightOf<T> = <T as frame_system::Config>::DbWeight;

pub type MessageId = [u8; 32];

pub trait WeightInfo {
	fn remove(x: u32) -> Weight;
	fn xcm_new_query(x: u32) -> Weight;
	fn xcm_response(x: u32) -> Weight;
	fn ismp_on_response(x: u32, y: u32) -> Weight;
	fn ismp_on_timeout(x: u32, y: u32) -> Weight;
	fn ismp_get(x: u32, y: u32, z: u32, a: u32) -> Weight;
	fn ismp_post(x: u32, y: u32) -> Weight;
}

impl WeightInfo for () {
	fn remove(x: u32) -> Weight {
		Zero::zero()
	}

	fn xcm_new_query(x: u32) -> Weight {
		Zero::zero()
	}

	fn xcm_response(x: u32) -> Weight {
		Zero::zero()
	}

	fn ismp_on_response(x: u32, y: u32) -> Weight {
		Zero::zero()
	}

	fn ismp_on_timeout(x: u32, y: u32) -> Weight {
		Zero::zero()
	}

	fn ismp_get(x: u32, y: u32, z: u32, a: u32) -> Weight {
		Zero::zero()
	}

	fn ismp_post(x: u32, y: u32) -> Weight {
		Zero::zero()
	}
}

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		traits::{
			tokens::fungible::{hold::Mutate as HoldMutate, Mutate},
			OnInitialize,
		},
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

		/// The type responsible for executing callbacks.
		type CallbackExecutor: CallbackExecutor<Self>;

		/// The deposit + fee mechanism.
		type Fungibles: HoldMutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ Inspect<Self::AccountId>
			+ Mutate<Self::AccountId>;

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

		/// Where the callback fees go once any refunds have occured after cb execution.
		type FeeAccount: Get<AccountIdOf<Self>>;

		/// The type responsible for converting between weight and balance, commonly transaction
		/// payment.
		type WeightToFee: WeightToFee<Balance = BalanceOf<Self>>;

		/// The fee paid to the relayers account for relaying a message.
		type IsmpRelayerFee: Get<BalanceOf<Self>>;

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
		FutureTimeoutMandatory,
		/// Message block limit has been reached for this expiry block. Try a different timeout.
		MaxMessageTimeoutPerBlockReached,
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
			for (origin, message_id) in XcmQueryTimeouts::<T>::get(n) {
				weight = weight.saturating_add(DbWeightOf::<T>::get().reads_writes(2, 1));
				Messages::<T>::mutate(origin, message_id, |maybe_message| {
					if let Some(Message::XcmQuery { query_id, deposit, .. }) =
						maybe_message.as_mut()
					{
						*maybe_message =
							Some(Message::XcmTimeout { query_id: *query_id, deposit: *deposit });
					}
				})
			}

			weight
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO: does ismp allow querying to ensure that specified para id is supported?
		#[pallet::call_index(1)]
		#[pallet::weight(
			// {
				// let keys_len = message.keys.len() as u32;
				// let has_callback = if callback.is_some() {
				// 	1
				// } else {
				// 	0
				// };
				// T::WeightInfo::ismp_get(keys_len, has_callback)
			// }
			Weight::default()
		)]
		pub fn ismp_get(
			origin: OriginFor<T>,
			id: MessageId,
			message: ismp::Get<T>,
			callback: Option<Callback>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(!Messages::<T>::contains_key(&origin, &id), Error::<T>::MessageExists);

			// Take the storage deposit for this particular message.
			let deposit = calculate_protocol_deposit::<T, T::OnChainByteFee>(
				ProtocolStorageDeposit::IsmpRequests,
			)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>())
			.saturating_add(calculate_deposit_of::<T, T::OffChainByteFee, ismp::Get<T>>());

			T::Fungibles::hold(&HoldReason::Messaging.into(), &origin, deposit)?;

			// Response + Callback Execution.
			let response_prepayment_amount = T::WeightToFee::weight_to_fee(
				&T::WeightInfo::ismp_on_response(1, if callback.is_some() { 1 } else { 0 })
					.saturating_add(T::CallbackExecutor::execution_weight()),
			);

			T::Fungibles::transfer(
				&origin,
				&T::FeeAccount::get(),
				response_prepayment_amount,
				Preservation::Preserve,
			)?;

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
				FeeMetadata { payer: origin.clone(), fee: T::IsmpRelayerFee::get() },
			) {
				Ok(commitment) => Ok::<H256, DispatchError>(commitment),
				Err(e) => {
					let err = e.downcast::<IsmpError>().unwrap();
					log::error!("ISMP Dispatch failed!! {:?}", err);
					return Err(Error::<T>::IsmpDispatchFailed.into())
				},
			}?;
			// Store commitment for lookup on response, message for querying,
			// response/timeout handling.
			IsmpRequests::<T>::insert(&commitment, (&origin, id));
			Messages::<T>::insert(
				&origin,
				id,
				Message::Ismp { commitment, callback: callback.clone(), deposit },
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
			callback: Option<Callback>,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			ensure!(!Messages::<T>::contains_key(&origin, &id), Error::<T>::MessageExists);
			let deposit = calculate_protocol_deposit::<T, T::OnChainByteFee>(
				ProtocolStorageDeposit::IsmpRequests,
			)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>())
			.saturating_add(calculate_deposit_of::<T, T::OffChainByteFee, ismp::Post<T>>());

			T::Fungibles::hold(&HoldReason::Messaging.into(), &origin, deposit)?;

			let response_prepayment_amount = T::WeightToFee::weight_to_fee(
				&T::WeightInfo::ismp_on_response(0, if callback.is_some() { 1 } else { 0 })
					.saturating_add(T::CallbackExecutor::execution_weight()),
			);

			T::Fungibles::transfer(
				&origin,
				&T::FeeAccount::get(),
				response_prepayment_amount,
				Preservation::Preserve,
			)?;

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
					FeeMetadata { payer: origin.clone(), fee: T::IsmpRelayerFee::get() },
				)
				.map_err(|_| Error::<T>::IsmpDispatchFailed)?;

			// Store commitment for lookup on response, message for querying,
			// response/timeout handling.
			IsmpRequests::<T>::insert(&commitment, (&origin, id));
			Messages::<T>::insert(
				&origin,
				id,
				Message::Ismp { commitment, callback: callback.clone(), deposit },
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
			callback: Option<Callback>,
		) -> DispatchResult {
			let querier_location = T::OriginConverter::try_convert(origin.clone())
				.map_err(|_| Error::<T>::OriginConversionFailed)?;
			let origin = ensure_signed(origin)?;
			ensure!(!Messages::<T>::contains_key(&origin, &id), Error::<T>::MessageExists);

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

			let deposit = calculate_protocol_deposit::<T, T::OnChainByteFee>(
				ProtocolStorageDeposit::XcmQueries,
			)
			.saturating_add(calculate_message_deposit::<T, T::OnChainByteFee>());
			T::Fungibles::hold(&HoldReason::Messaging.into(), &origin, deposit)?;

			if let Some(cb) = callback.as_ref() {
				T::Fungibles::hold(
					&HoldReason::CallbackGas.into(),
					&origin,
					T::WeightToFee::weight_to_fee(&cb.weight),
				)?;

				let response_prepayment_amount = T::WeightToFee::weight_to_fee(
					&T::WeightInfo::xcm_response(1)
						.saturating_add(T::CallbackExecutor::execution_weight()),
				);
				T::Fungibles::transfer(
					&origin,
					&T::FeeAccount::get(),
					response_prepayment_amount,
					Preservation::Preserve,
				)?;
			} else {
				let response_prepayment_amount =
					T::WeightToFee::weight_to_fee(&T::WeightInfo::xcm_response(0));
				T::Fungibles::transfer(
					&origin,
					&T::FeeAccount::get(),
					response_prepayment_amount,
					Preservation::Preserve,
				)?;
			}

			// Process message by creating new query via XCM.
			// Xcm only uses/stores pallet, index - i.e. (u8,u8), hence the fields in xcm_response
			// are ignored.
			let notify = Call::<T>::xcm_response { query_id: 0, xcm_response: Default::default() };
			let query_id = T::Xcm::new_notify_query(responder, notify, timeout, querier_location);

			// Store query id for later lookup on response, message for querying status,
			// response/timeout handling.
			XcmQueries::<T>::insert(&query_id, (&origin, id));
			Messages::<T>::insert(
				&origin,
				id,
				Message::XcmQuery { query_id, callback: callback.clone(), deposit },
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
			xcm_response: Response,
		) -> DispatchResult {
			T::XcmResponseOrigin::ensure_origin(origin)?;
			let (initiating_origin, id) =
				XcmQueries::<T>::get(query_id).ok_or(Error::<T>::MessageNotFound)?;
			let xcm_query_message =
				Messages::<T>::get(&initiating_origin, &id).ok_or(Error::<T>::MessageNotFound)?;

			let (query_id, callback, deposit) = match &xcm_query_message {
				Message::XcmQuery { query_id, callback, deposit } => (query_id, callback, deposit),
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
				if Self::call(&initiating_origin, callback.to_owned(), &id, &xcm_response).is_ok() {
					Messages::<T>::remove(&initiating_origin, &id);
					XcmQueries::<T>::remove(query_id);
					T::Fungibles::release(
						&HoldReason::Messaging.into(),
						&initiating_origin,
						*deposit,
						Exact,
					)?;
					return Ok(());
				}
			}
			// No callback is executed,
			Messages::<T>::insert(
				&initiating_origin,
				&id,
				Message::XcmResponse {
					query_id: *query_id,
					deposit: *deposit,
					response: xcm_response,
				},
			);

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

				let deposit = match message {
					Message::Ismp { .. } => Err(Error::<T>::RequestPending),
					Message::XcmQuery { .. } => Err(Error::<T>::RequestPending),
					Message::IsmpResponse { deposit, commitment, .. } => {
						Messages::<T>::remove(&origin, &id);
						IsmpRequests::<T>::remove(&commitment);
						Ok(deposit)
					},
					Message::XcmResponse { deposit, query_id, .. } => {
						Messages::<T>::remove(&origin, &id);
						XcmQueries::<T>::remove(query_id);
						Ok(deposit)
					},
					Message::IsmpTimeout { deposit, commitment, .. } => {
						Messages::<T>::remove(&origin, &id);
						IsmpRequests::<T>::remove(&commitment);
						Ok(deposit)
					},
					Message::XcmTimeout { query_id, deposit, .. } => {
						Messages::<T>::remove(&origin, &id);
						XcmQueries::<T>::remove(query_id);
						Ok(deposit)
					},
				}?;

				T::Fungibles::release(&HoldReason::Messaging.into(), &origin, deposit, Exact)?;
			}

			Self::deposit_event(Event::<T>::Removed { origin, messages: messages.into_inner() });

			Ok(())
		}
	}
}
impl<T: Config> Pallet<T> {
	/// Attempt to notify via callback.
	pub(crate) fn call(
		initiating_origin: &AccountIdOf<T>,
		callback: Callback,
		id: &MessageId,
		data: &impl Encode,
	) -> DispatchResult {
		let result = T::CallbackExecutor::execute(
			initiating_origin,
			match callback.abi {
				Abi::Scale => [callback.selector.to_vec(), (id, data).encode()].concat(),
			},
			callback.weight,
		);

		log::debug!(target: "pop-api::extension", "callback weight={:?}, result={result:?}", callback.weight);
		Self::handle_callback_result(initiating_origin, id, result, callback)
	}

	/// Handle the result of a callback execution.
	pub(crate) fn handle_callback_result(
		initiating_origin: &AccountIdOf<T>,
		id: &MessageId,
		result: DispatchResultWithPostInfo,
		callback: Callback,
	) -> DispatchResult {
		match result {
			Ok(post_info) => {
				// Try and return any unused callback weight.
				if let Some(weight_used) = post_info.actual_weight {
					let weight_to_refund = callback.weight.saturating_sub(weight_used);
					if weight_to_refund.all_gt(Zero::zero()) {
						let total_deposit = T::WeightToFee::weight_to_fee(&callback.weight);
						let returnable_deposit = T::WeightToFee::weight_to_fee(&weight_to_refund);
						let execution_reward = total_deposit.saturating_sub(returnable_deposit);
						let reason = HoldReason::CallbackGas.into();

						T::Fungibles::release(
							&reason,
							&initiating_origin,
							returnable_deposit,
							BestEffort,
						)?;
						T::Fungibles::transfer_on_hold(
							&reason,
							&initiating_origin,
							&T::FeeAccount::get(),
							execution_reward,
							BestEffort,
							Restriction::Free,
							Fortitude::Polite,
						)?;
					}
				}

				Self::deposit_event(Event::<T>::CallbackExecuted {
					origin: initiating_origin.clone(),
					id: *id,
					callback,
				});

				Ok(())
			},
			Err(sp_runtime::DispatchErrorWithPostInfo::<PostDispatchInfo> { post_info, error }) => {
				let total_deposit = T::WeightToFee::weight_to_fee(&callback.weight);
				T::Fungibles::transfer_on_hold(
					&HoldReason::CallbackGas.into(),
					&initiating_origin,
					&T::FeeAccount::get(),
					total_deposit,
					BestEffort,
					Restriction::Free,
					Fortitude::Polite,
				)?;

				// Fallback to storing the message for polling - pre-paid weight is lost.
				Self::deposit_event(Event::<T>::CallbackFailed {
					origin: initiating_origin.clone(),
					id: id.clone(),
					callback,
					post_info,
					error,
				});
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

/// The main message type that describes a message and its status.
#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum Message<T: Config> {
	Ismp {
		commitment: H256,
		callback: Option<Callback>,
		deposit: BalanceOf<T>,
	},
	XcmQuery {
		query_id: QueryId,
		callback: Option<Callback>,
		deposit: BalanceOf<T>,
	},
	IsmpResponse {
		commitment: H256,
		deposit: BalanceOf<T>,
		response: BoundedVec<u8, T::MaxResponseLen>,
	},
	XcmResponse {
		query_id: QueryId,
		deposit: BalanceOf<T>,
		response: Response,
	},
	IsmpTimeout {
		commitment: H256,
		deposit: BalanceOf<T>,
	},
	XcmTimeout {
		query_id: QueryId,
		deposit: BalanceOf<T>,
	},
}

impl<T: Config> From<&Message<T>> for MessageStatus {
	fn from(value: &Message<T>) -> Self {
		match value {
			&Message::Ismp { .. } => MessageStatus::Pending,
			&Message::XcmQuery { .. } => MessageStatus::Pending,
			&Message::IsmpResponse { .. } => MessageStatus::Complete,
			&Message::XcmResponse { .. } => MessageStatus::Complete,
			&Message::IsmpTimeout { .. } => MessageStatus::Timeout,
			&Message::XcmTimeout { .. } => MessageStatus::Timeout,
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

/// The encoding used for the data going to the contract.
#[derive(Copy, Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum Abi {
	Scale,
}

/// The type responsible for calling into the contract env.
pub trait CallbackExecutor<T: Config> {
	fn execute(account: &T::AccountId, data: Vec<u8>, weight: Weight)
		-> DispatchResultWithPostInfo;
	/// The weight of calling into a contract env, seperate from the weight specified for callback
	/// execution.
	fn execution_weight() -> Weight;
}

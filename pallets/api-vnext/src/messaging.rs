use deposits::*;
use frame_support::{
	dispatch::{DispatchErrorWithPostInfo, DispatchResult, DispatchResultWithPostInfo},
	pallet_prelude::{DispatchError::BadOrigin, *},
	storage::KeyLenOf,
	traits::{
		tokens::{
			fungible::{hold::Mutate as HoldMutate, Balanced, Credit, Inspect, Mutate},
			Fortitude, Precision, Preservation,
		},
		Get, OnUnbalanced,
	},
	weights::WeightToFee,
	BoundedSlice,
};
use frame_system::pallet_prelude::*;
pub use pallet::{Error, *};
use pallet_revive::{
	sp_runtime::traits::{SaturatedConversion, Saturating, TryConvert},
	H160,
};
use sp_runtime::ArithmeticError;
use transports::{
	ismp::IsmpDispatcher,
	xcm::{Location, NotifyQueryHandler, QueryId, Response},
};
use weights::WeightInfo;

use super::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod deposits;
/// The messaging precompiles offer a streamlined interface for cross-chain messaging.
pub mod precompiles;
#[cfg(test)]
mod tests;
/// Messaging transports.
pub mod transports;
mod weights;

type BalanceOf<T> = <<T as Config>::Fungibles as Inspect<AccountIdOf<T>>>::Balance;
type BlockNumberOf<T> = BlockNumberFor<T>;
type DbWeightOf<T> = <T as frame_system::Config>::DbWeight;
pub type MessageId = u64; // TODO: determine why this was changed to [u8; 32] - U256?;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::ensure;

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The type responsible for executing callbacks.
		type CallbackExecutor: CallbackExecutor<Self>;
		/// Where the callback fees or response fees are charged to.
		type FeeHandler: OnUnbalanced<Credit<Self::AccountId, Self::Fungibles>>;
		/// The deposit + fee mechanism.
		type Fungibles: HoldMutate<Self::AccountId, Reason = Self::RuntimeHoldReason>
			+ Mutate<Self::AccountId>
			+ Balanced<Self::AccountId>;
		/// The ISMP message dispatcher.
		type IsmpDispatcher: IsmpDispatcher<Account = Self::AccountId, Balance = BalanceOf<Self>>;
		/// The implementation of Keccak used for commitment hashes.
		type Keccak256: ::ismp::messaging::Keccak256;
		/// The maximum length of any additional application-specific metadata relating to a
		/// request.
		#[pallet::constant]
		type MaxContextLen: Get<u32>;
		/// The maximum length of outbound (posted) data.
		#[pallet::constant]
		type MaxDataLen: Get<u32>;
		/// The maximum byte length for a single key of an ismp request.
		#[pallet::constant]
		type MaxKeyLen: Get<u32>;
		/// The maximum number of keys for an outbound request.
		#[pallet::constant]
		type MaxKeys: Get<u32>;
		/// The maximum number of messages which can be removed at a time.
		#[pallet::constant]
		type MaxRemovals: Get<u32>;
		/// The maximum length for a response.
		#[pallet::constant]
		type MaxResponseLen: Get<u32>;
		/// SAFETY: Recommended this is small as is used to updated a message status in the hooks.
		/// The maximum number of xcm timeout updates that can be processed per block.
		#[pallet::constant]
		type MaxXcmQueryTimeoutsPerBlock: Get<u32>;
		/// The base byte fee for data stored offchain.
		#[pallet::constant]
		type OffChainByteFee: Get<BalanceOf<Self>>;
		/// The base byte fee for data stored onchain.
		#[pallet::constant]
		type OnChainByteFee: Get<BalanceOf<Self>>;
		/// A converter for conversion of a call origin to a location.
		type OriginConverter: TryConvert<Self::RuntimeOrigin, Location>;
		/// The overarching hold reason for deposits.
		type RuntimeHoldReason: From<HoldReason>;
		/// The type responsible for converting between weight and balance, commonly transaction
		/// payment.
		type WeightToFee: WeightToFee<Balance = BalanceOf<Self>>;
		/// Pallet weights.
		type WeightInfo: WeightInfo;
		/// A handler for the creation of a XCM query notification.
		type Xcm: NotifyQueryHandler<Self>;
		/// The origin of the response for xcm.
		type XcmResponseOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Location>;
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
		/// Dispatching a call via ISMP failed.
		IsmpDispatchFailed,
		/// The message was not found.
		MessageNotFound,
		/// The request has timed out.
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
		/// The number of messages exceeds the limit.
		TooManyMessages,
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
			for message_id in XcmQueryTimeouts::<T>::get(n) {
				weight = weight.saturating_add(DbWeightOf::<T>::get().reads_writes(2, 1));
				Messages::<T>::mutate(message_id, |maybe_message| {
					if let Some(Message::XcmQuery { origin, query_id, message_deposit, callback }) =
						maybe_message.as_mut()
					{
						let callback_deposit =
							callback.map(|cb| T::WeightToFee::weight_to_fee(&cb.weight));
						query_ids.push(*query_id);
						*maybe_message = Some(Message::XcmTimeout {
							origin: origin.address,
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

	/// The message queue.
	#[pallet::storage]
	pub(crate) type Messages<T: Config> = StorageMap<_, Twox64Concat, MessageId, Message<T>>;

	/// The next message identifier.
	///
	/// Also serves as a count of the total number of messages sent.
	#[pallet::storage]
	pub(crate) type NextMessageId<T: Config> = StorageValue<_, MessageId, ValueQuery>;

	/// The active ISMP requests, mapped to the originating message identifier.
	#[pallet::storage]
	pub(super) type IsmpRequests<T: Config> = StorageMap<_, Identity, H256, MessageId>;

	/// The active XCM queries, mapped to the originating message identifier.
	#[pallet::storage]
	pub(super) type XcmQueries<T: Config> = StorageMap<_, Twox64Concat, QueryId, MessageId>;

	/// The timeouts of XCM queries, by block number.
	#[pallet::storage]
	pub(super) type XcmQueryTimeouts<T: Config> = StorageMap<
		_,
		Twox64Concat,
		BlockNumberOf<T>,
		BoundedVec<MessageId, T::MaxXcmQueryTimeoutsPerBlock>,
		ValueQuery,
	>;

	/// The events that can be emitted.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
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
		/// A response to a GET has been received via ISMP.
		IsmpGetResponseReceived {
			/// The destination of the response.
			dest: H160,
			/// The identifier specified for the request.
			id: MessageId,
			/// The ISMP request commitment.
			commitment: H256,
		},
		/// A response to a POST has been received via ISMP.
		IsmpPostResponseReceived {
			/// The destination of the response.
			dest: H160,
			/// The identifier specified for the request.
			id: MessageId,
			/// The ISMP request commitment.
			commitment: H256,
		},
		/// An ISMP message has timed out.
		IsmpTimedOut { commitment: H256 },
		/// An error has occured while attempting to refund weight.
		WeightRefundErrored { message_id: MessageId, error: DispatchError },
		/// A collection of xcm queries have timed out.
		XcmQueriesTimedOut { query_ids: Vec<QueryId> },
		/// A response to a XCM query has been received.
		XcmResponseReceived {
			/// The destination of the response.
			dest: H160,
			/// The identifier specified for the request.
			id: MessageId,
			/// The identifier of the XCM query.
			query_id: QueryId,
			/// The query response.
			response: Response,
		},
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Handle a response to a previous XCM query.
		///
		/// Executes a stored callback or updates the state with the received response.
		///
		/// # Parameters
		/// - `origin`: The XCM responder origin.
		/// - `query_id`: The ID of the XCM query being responded to.
		/// - `xcm_response`: The response data.
		#[pallet::call_index(0)]
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

			let id = XcmQueries::<T>::get(query_id).ok_or(Error::<T>::MessageNotFound)?;
			let xcm_query_message = Messages::<T>::get(id).ok_or(Error::<T>::MessageNotFound)?;

			let (origin, query_id, callback, message_deposit) = match &xcm_query_message {
				Message::XcmQuery { origin, query_id, callback, message_deposit } =>
					(origin, query_id, callback, message_deposit),
				Message::XcmTimeout { .. } => return Err(Error::<T>::RequestTimedOut.into()),
				_ => return Err(Error::<T>::InvalidMessage.into()),
			};

			// Emit event before possible callback execution.
			Self::deposit_event(Event::<T>::XcmResponseReceived {
				dest: origin.address,
				id,
				query_id: *query_id,
				response: xcm_response.clone(),
			});

			if let Some(callback) = callback {
				// Attempt callback with response if specified.
				log::debug!(target: "pop-api::extension", "xcm callback={:?}, response={:?}", callback, xcm_response);
				// Never roll back state if call fails.
				// Ensure that the response can be polled.
				if call::<T>(&origin.account, *callback, &id, &xcm_response).is_ok() {
					Messages::<T>::remove(id);
					XcmQueries::<T>::remove(query_id);
					T::Fungibles::release(
						&HoldReason::Messaging.into(),
						&origin.account,
						*message_deposit,
						Precision::Exact,
					)?;

					return Ok(());
				}
			}
			// No callback is executed,
			Messages::<T>::insert(
				id,
				Message::XcmResponse {
					origin: origin.address,
					query_id: *query_id,
					message_deposit: *message_deposit,
					response: xcm_response,
				},
			);
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Get a message by its identifier.
	///
	/// # Parameters
	/// - `id`: The message identifier.
	pub fn get(id: MessageId) -> Option<Message<T>> {
		<Messages<T>>::get(id)
	}
}

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
/// - `initiating_origin`: The account that triggered the callback. This will be passed to the
///   executor and used for fee management and event attribution.
/// - `callback`: The callback definition.
/// - `id`: The message ID associated with this callback's message.
/// - `data`: The encoded payload to send to the callback.
///
/// # Weight Handling
///
/// - Before executing the callback, this function checks whether the total expected weight
///   (`callback.weight`) can be accommodated in the current block.
/// - If the block is saturated, the function returns early with an error and does not mutate state.
/// - After execution, the actual weight used by the callback is determined using
///   [`Self::process_callback_weight`] and registered via
///   [`frame_system::Pallet::<T>::register_extra_weight_unchecked`].
pub(crate) fn call<T: Config>(
	initiating_origin: &AccountIdOf<T>,
	callback: Callback,
	id: &MessageId,
	data: &impl EncodeCallback,
) -> DispatchResult {
	// This is the total weight that should be deducted from the blockspace for callback
	// execution.
	let max_weight = callback.weight;

	// Dont mutate state if blockspace will be saturated.
	frame_support::ensure!(
		frame_system::BlockWeight::<T>::get()
			.checked_accrue(max_weight, DispatchClass::Normal)
			.is_ok(),
		Error::<T>::BlockspaceAllowanceReached
	);

	// Execute callback.
	// Its important to note that we must still ensure that the weight used is accounted for
	// in frame_system. Hence all calls after this must not return an err and state
	// should not be rolled back.
	let data = data.encode(callback.encoding, callback.selector, *id);
	let result = T::CallbackExecutor::execute(
		&initiating_origin,
		callback.destination,
		data,
		callback.weight,
	);

	log::debug!(target: "pop-api::extension", "callback weight={:?}, result={result:?}", callback.weight);
	deposit_callback_event::<T>(initiating_origin.clone(), *id, &callback, &result);
	let callback_weight_used = process_callback_weight(&result, callback.weight);

	// Manually adjust callback weight.
	frame_system::Pallet::<T>::register_extra_weight_unchecked(
		callback_weight_used,
		DispatchClass::Normal,
	);

	match manage_fees::<T>(&initiating_origin, callback_weight_used, callback.weight) {
		Ok(_) => (),
		// Dont return early, we must register the weight used by the callback.
		Err(error) =>
			<Pallet<T>>::deposit_event(Event::WeightRefundErrored { message_id: *id, error }),
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
/// - `message_id`: The unique identifier associated with the message that triggered the callback.
/// - `callback`: The callback object that was attempted to be executed.
/// - `result`: The outcome of the callback execution, containing either success or failure.
pub(crate) fn deposit_callback_event<T: Config>(
	initiating_origin: T::AccountId,
	message_id: MessageId,
	callback: &Callback,
	result: &DispatchResultWithPostInfo,
) {
	match result {
		Ok(_) => {
			<Pallet<T>>::deposit_event(Event::<T>::CallbackExecuted {
				origin: initiating_origin,
				id: message_id,
				callback: callback.clone(),
			});
		},
		Err(error) => {
			<Pallet<T>>::deposit_event(Event::<T>::CallbackFailed {
				origin: initiating_origin,
				id: message_id,
				callback: callback.clone(),
				error: error.clone(),
			});
		},
	}
}

fn get<T: Config>(id: &MessageId) -> Vec<u8> {
	use Message::*;
	Messages::<T>::get(id)
		.and_then(|m| match m {
			Ismp { .. } | IsmpTimeout { .. } | XcmQuery { .. } | XcmTimeout { .. } => None,
			IsmpResponse { response, .. } => Some(response.into_inner()),
			XcmResponse { response, .. } => Some(codec::Encode::encode(&response)),
		})
		.unwrap_or_default()
}

fn id<T: parachain_info::Config>() -> u32 {
	parachain_info::Pallet::<T>::parachain_id().into()
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
pub(crate) fn manage_fees<T: Config>(
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
		initiating_origin,
		to_reward,
		Precision::Exact,
		Preservation::Preserve,
		Fortitude::Polite,
	)?;

	// Handle assets.
	T::FeeHandler::on_unbalanced(credit);
	Ok(())
}

fn next_message_id<T: Config>() -> Result<MessageId, DispatchError> {
	NextMessageId::<T>::try_mutate(|next| {
		let id = *next;
		*next = next.checked_add(One::one()).ok_or(ArithmeticError::Overflow)?;
		Ok(id)
	})
}

fn poll_status<T: Config>(id: &MessageId) -> MessageStatus {
	<Messages<T>>::get(id).map_or(MessageStatus::NotFound, |m| MessageStatus::from(&m))
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
/// - `result`: The result of the callback dispatch, including any `PostDispatchInfo` if successful.
/// - `max_weight`: The maximum weight budgeted for the callback execution.
///
/// # Rationale
///
/// - Protects against underestimating weight in cases where `actual_weight` is missing or the
///   dispatch fails.
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

/// Remove a batch of completed or timed-out messages.
///
/// Allows users to clean up storage and reclaim deposits for messages that have concluded.
///
/// # Parameters
/// - `origin`: The account removing its messages.
/// - `messages`: List of message IDs to remove (bounded by `MaxRemovals`).
fn remove<T: Config>(origin: Origin<T::AccountId>, messages: &[MessageId]) -> DispatchResult {
	for id in <BoundedSlice<_, T::MaxRemovals>>::try_from(messages)
		.map_err(|_| <Error<T>>::TooManyMessages)?
	{
		let Some(message) = Messages::<T>::get(id) else {
			return Err(Error::<T>::MessageNotFound.into());
		};

		let (message_deposit, maybe_callback_deposit) = match message {
			Message::Ismp { .. } | Message::XcmQuery { .. } => Err(Error::<T>::RequestPending),
			Message::IsmpResponse { origin: initiator, message_deposit, commitment, .. } => {
				frame_support::ensure!(origin.address == initiator, BadOrigin);
				Messages::<T>::remove(id);
				IsmpRequests::<T>::remove(commitment);
				Ok((message_deposit, None))
			},
			Message::IsmpTimeout {
				origin: initiator,
				message_deposit,
				commitment,
				callback_deposit,
				..
			} => {
				frame_support::ensure!(origin.address == initiator, BadOrigin);
				Messages::<T>::remove(id);
				IsmpRequests::<T>::remove(commitment);
				Ok((message_deposit, callback_deposit))
			},
			Message::XcmResponse { origin: initiator, message_deposit, query_id, .. } => {
				frame_support::ensure!(origin.address == initiator, BadOrigin);
				Messages::<T>::remove(id);
				XcmQueries::<T>::remove(query_id);
				Ok((message_deposit, None))
			},
			Message::XcmTimeout {
				origin: initiator,
				query_id,
				message_deposit,
				callback_deposit,
				..
			} => {
				frame_support::ensure!(origin.address == initiator, BadOrigin);
				Messages::<T>::remove(id);
				XcmQueries::<T>::remove(query_id);
				Ok((message_deposit, callback_deposit))
			},
		}?;

		T::Fungibles::release(
			&HoldReason::Messaging.into(),
			&origin.account,
			message_deposit,
			Precision::Exact,
		)?;
		if let Some(callback_deposit) = maybe_callback_deposit {
			T::Fungibles::release(
				&HoldReason::CallbackGas.into(),
				&origin.account,
				callback_deposit,
				Precision::Exact,
			)?;
		}
	}
	Ok(())
}

/// A message callback.
#[derive(
	Copy,
	Clone,
	Debug,
	Decode,
	DecodeWithMemTracking,
	Encode,
	Eq,
	MaxEncodedLen,
	PartialEq,
	TypeInfo,
)]
#[scale_info(skip_type_params(T))]
pub struct Callback {
	/// The contract address to which the callback should be sent.
	pub destination: H160,
	/// The encoding used for the data going to the contract.
	pub encoding: Encoding,
	/// The message selector to be used for the callback.
	pub selector: [u8; 4],
	/// The pre-paid weight used as a gas limit for the callback.
	pub weight: Weight,
}

impl Callback {
	/// A new message callback.
	///
	/// # Parameters
	/// - `destination`: The contract address to which the callback should be sent.
	/// - `encoding`: The encoding used for the data going to the contract.
	/// - `selector`: The message selector to be used for the callback.
	/// - `weight`: The pre-paid weight used as a gas limit for the callback.
	pub(crate) fn new(
		destination: H160,
		encoding: Encoding,
		selector: [u8; 4],
		weight: Weight,
	) -> Self {
		Self { destination, encoding, selector, weight }
	}
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
	/// - `destination`: The contract address to which the callback should be sent.
	/// - `data`: Encoded callback data, typically ABI-encoded input including selector and
	///   parameters.
	/// - `weight`: The maximum weight allowed for executing this callback.
	fn execute(
		account: &T::AccountId,
		destination: H160,
		data: Vec<u8>,
		weight: Weight,
	) -> DispatchResultWithPostInfo;

	/// Returns the baseline weight required for a single callback execution.
	///
	/// This serves as an overhead estimate, useful for pallet-level weight calculations.
	fn execution_weight() -> Weight;
}

/// The specificiation of how data must be encoded before being sent to a contract.
#[derive(
	Copy,
	Clone,
	Debug,
	Decode,
	DecodeWithMemTracking,
	Encode,
	Eq,
	MaxEncodedLen,
	PartialEq,
	TypeInfo,
)]
pub enum Encoding {
	/// SCALE (Simple Concatenated Aggregate Little-Endian) encoding.
	Scale,
	/// Solidity ABI (Application Binary Interface) encoding,
	SolidityAbi,
}

/// Trait for encoding a response callback.
pub(crate) trait EncodeCallback {
	/// Encodes the data using the specified encoding.
	///
	/// # Parameters
	/// - `encoding`: The encoding to use.
	/// - `selector`: The message selector to be used for the callback.
	/// - `id`: The originating message identifier.
	fn encode(&self, encoding: Encoding, selector: [u8; 4], id: MessageId) -> Vec<u8>;
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
pub enum Message<T: Config> {
	/// Represents a pending ISMP request.
	///
	/// # Fields
	/// - `origin`: The origin of the request.
	/// - `commitment`: The cryptographic commitment of the request payload.
	/// - `callback`: An optional callback to invoke upon receiving a response.
	/// - `deposit`: The total deposit held to cover message and callback fees.
	Ismp {
		origin: Origin<T::AccountId>,
		commitment: H256,
		callback: Option<Callback>,
		message_deposit: BalanceOf<T>,
	},

	/// Represents a pending XCM query request.
	///
	/// # Fields
	/// - `origin`: The origin of the request.
	/// - `query_id`: Unique identifier for the XCM query.
	/// - `callback`: An optional callback for handling the response.
	/// - `deposit`: The deposit held to cover fees for query execution and callback.
	XcmQuery {
		origin: Origin<T::AccountId>,
		query_id: QueryId,
		callback: Option<Callback>,
		message_deposit: BalanceOf<T>,
	},

	/// Represents a received ISMP response.
	///
	/// # Fields
	/// - `origin`: The origin of the request.
	/// - `commitment`: The original commitment for the request.
	/// - `deposit`: The held deposit for the message, which may be released or burned.
	/// - `response`: The encoded response payload, size-bounded by `T::MaxResponseLen`.
	IsmpResponse {
		origin: H160,
		commitment: H256,
		message_deposit: BalanceOf<T>,
		response: BoundedVec<u8, T::MaxResponseLen>,
	},

	/// Represents a received XCM response.
	///
	/// # Fields
	/// - `origin`: The origin of the request.
	/// - `query_id`: Identifier that matches a previously sent XCM query.
	/// - `deposit`: The deposit originally held for this message.
	/// - `response`: The deserialized response payload.
	XcmResponse {
		origin: H160,
		query_id: QueryId,
		message_deposit: BalanceOf<T>,
		response: Response,
	},

	/// Represents an ISMP request that timed out before a response was received.
	///
	/// # Fields
	/// - `origin`: The origin of the request.
	/// - `commitment`: The original commitment of the request.
	/// - `deposit`: The deposit held for the request, which may be reclaimed.
	IsmpTimeout {
		origin: H160,
		commitment: H256,
		message_deposit: BalanceOf<T>,
		callback_deposit: Option<BalanceOf<T>>,
	},

	/// Represents an XCM query that timed out before a response was received.
	///
	/// # Fields
	/// - `origin`: The origin of the request.
	/// - `query_id`: The original query ID that timed out.
	/// - `deposit`: The deposit held for the query, which may be reclaimed.
	XcmTimeout {
		origin: H160,
		query_id: QueryId,
		message_deposit: BalanceOf<T>,
		callback_deposit: Option<BalanceOf<T>>,
	},
}

impl<T: Config> Message<T> {
	#[cfg(test)]
	fn ismp(
		origin: Origin<T::AccountId>,
		commitment: H256,
		callback: Option<Callback>,
		message_deposit: BalanceOf<T>,
	) -> Self {
		Self::Ismp { origin, commitment, callback, message_deposit }
	}

	#[cfg(any(test, feature = "runtime-benchmarks"))]
	fn ismp_response(
		origin: H160,
		commitment: H256,
		message_deposit: BalanceOf<T>,
		response: BoundedVec<u8, T::MaxResponseLen>,
	) -> Self {
		Self::IsmpResponse { origin, commitment, message_deposit, response }
	}

	#[cfg(any(test, feature = "runtime-benchmarks"))]
	fn ismp_timeout(
		origin: H160,
		commitment: H256,
		message_deposit: BalanceOf<T>,
		callback_deposit: Option<BalanceOf<T>>,
	) -> Self {
		Self::IsmpTimeout { origin, commitment, message_deposit, callback_deposit }
	}

	#[cfg(test)]
	fn xcm_query(
		origin: Origin<T::AccountId>,
		query_id: QueryId,
		callback: Option<Callback>,
		message_deposit: BalanceOf<T>,
	) -> Self {
		Self::XcmQuery { origin, query_id, callback, message_deposit }
	}

	#[cfg(test)]
	fn xcm_response(
		origin: H160,
		query_id: QueryId,
		message_deposit: BalanceOf<T>,
		response: Response,
	) -> Self {
		Self::XcmResponse { origin, query_id, message_deposit, response }
	}

	#[cfg(test)]
	fn xcm_timeout(
		origin: H160,
		query_id: QueryId,
		message_deposit: BalanceOf<T>,
		callback_deposit: Option<BalanceOf<T>>,
	) -> Self {
		Self::XcmTimeout { origin, query_id, message_deposit, callback_deposit }
	}
}

/// The related message status of a Message.
#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum MessageStatus {
	NotFound,
	Pending,
	Complete,
	Timeout,
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

/// The origin of a request.
#[derive(Clone, Debug, Encode, Eq, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(Account))]
pub struct Origin<Account> {
	address: H160,
	account: Account,
}

#[cfg(test)]
impl<Account> From<(H160, Account)> for Origin<Account> {
	fn from((address, account): (H160, Account)) -> Self {
		Self { address, account }
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl<Account> Origin<Account> {
	pub fn from_address<T: pallet_revive::Config<AccountId = Account>>(address: H160) -> Self {
		let account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&address);
		Self { address, account }
	}
}

impl<T: pallet_revive::Config> TryFrom<pallet_revive::Origin<T>> for Origin<T::AccountId> {
	type Error = pallet_revive::precompiles::Error;

	fn try_from(origin: pallet_revive::Origin<T>) -> Result<Self, Self::Error> {
		use pallet_revive::Origin::*;
		let account = match origin {
			Signed(id) => Ok(id),
			Root => Err(DispatchError::RootNotAllowed),
		}?;
		let address = <T as pallet_revive::Config>::AddressMapper::to_address(&account);
		Ok(Self { address, account })
	}
}

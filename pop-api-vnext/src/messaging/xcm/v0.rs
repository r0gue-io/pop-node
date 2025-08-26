pub use errors::{Error, Error::*};
pub use ink::xcm::prelude::{Location, VersionedLocation, VersionedResponse, VersionedXcm};
use ink::{scale::Decode, Address};

use super::{super::v0::Callback, *};

mod errors;

pub type QueryId = u64;

// Precompile index within the runtime
const PRECOMPILE: u16 = 5;
/// The address of the XCM precompile.
pub const PRECOMPILE_ADDRESS: Address = fixed_address(PRECOMPILE);

/// The XCM precompile offers a streamlined interface for messaging using Polkadot's Cross-Consensus
/// Messaging (XCM).
#[ink::trait_definition]
pub trait Xcm {
	/// Execute an XCM message from a local, signed, origin.
	///
	/// # Parameters
	/// - `message` - A SCALE-encoded versioned XCM message.
	/// - `weight` - The maximum allowed weight for execution.
	///
	/// # Returns
	/// A SCALE-encoded dispatch result.
	#[ink(message)]
	fn execute(&self, message: DynBytes, weight: Weight) -> DynBytes;

	/// Initiate a new XCM query.
	///
	/// Starts a query using the XCM interface, specifying a responder and timeout block.
	///
	/// # Parameters
	/// - `responder` - A SCALE-encoded versioned location of the XCM responder.
	/// - `timeout` - Block number after which the query should timeout.
	///
	/// # Returns
	/// A unique query identifier.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn newQuery(&self, responder: DynBytes, timeout: BlockNumber) -> (MessageId, QueryId);

	/// Send an XCM from a given origin.
	///
	/// # Parameters
	/// - `destination` - The SCALE-encoded versioned location for the destination of the message.
	/// - `message` - A SCALE-encoded versioned XCM message.
	///
	/// # Returns
	/// A SCALE-encoded dispatch result.
	#[ink(message)]
	fn send(&self, destination: DynBytes, message: DynBytes) -> DynBytes;
}

/// The XCM precompile offers a streamlined interface for messaging using Polkadot's Cross-Consensus
/// Messaging (XCM).
#[ink::trait_definition]
pub trait XcmCallback {
	/// Initiate a new XCM query.
	///
	/// Starts a query using the XCM interface, specifying a responder and timeout block.
	///
	/// # Parameters
	/// - `responder` - A SCALE-encoded versioned location of the XCM responder.
	/// - `timeout` - Block number after which the query should timeout.
	/// - `callback` - The callback to execute upon receiving a response.
	///
	/// # Returns
	/// A unique query identifier.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn newQuery(
		&self,
		responder: DynBytes,
		timeout: BlockNumber,
		callback: Callback,
	) -> (MessageId, QueryId);
}

/// The messaging interface of the XCM precompile offers a general interface for cross-chain
/// messaging operations.
///
/// This convenience trait simply provides access to general cross-chain messaging operations via
/// the XCM precompile, so that users need only use a single precompile if desired.
#[ink::trait_definition]
pub trait Messaging {
	/// Returns the response to a message.
	///
	/// A non-existent message identifier will return an empty response, which could also be a valid
	/// response depending on the source message.
	///
	/// # Parameters
	/// - `message` - The message identifier.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn getResponse(&self, message: MessageId) -> DynBytes;

	/// The identifier of this chain.
	///
	/// # Returns
	/// The identifier of this chain.
	#[ink(message)]
	fn id(&self) -> u32;

	/// Polls the status of a message.
	///
	/// # Parameters
	/// - `message` - The message identifier to poll.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn pollStatus(&self, message: MessageId) -> MessageStatus;

	/// Remove a completed or timed-out message.
	///
	/// Allows users to clean up storage and reclaim deposits for messages that have concluded.
	///
	/// # Parameters
	/// - `message` - The identifier of the message to remove.
	#[ink(message)]
	fn remove(&self, message: MessageId);

	/// Remove a batch of completed or timed-out messages.
	///
	/// Allows users to clean up storage and reclaim deposits for messages that have concluded.
	///
	/// # Parameters
	/// - `messages` - A set of identifiers of messages to remove (bounded by `MaxRemovals`).
	#[ink(message)]
	#[allow(non_snake_case)]
	fn removeMany(&self, messages: Vec<MessageId>);
}

/// Execute an XCM message from a local, signed, origin.
///
/// # Parameters
/// - `message` - A XCM message.
/// - `weight` - The maximum allowed weight for execution.
///
/// # Returns
/// A SCALE-encoded dispatch result.
#[inline]
pub fn execute<Call: Encode>(message: VersionedXcm<Call>, weight: Weight) -> Result<(), Error> {
	let precompile: contract_ref!(Xcm, Pop, Sol) = PRECOMPILE_ADDRESS.into();
	let result = precompile.execute(DynBytes(message.encode()), weight);
	Result::<(), ()>::decode(&mut result.0.as_slice())
		.map_err(|_| Error::DecodingFailed)?
		.map_err(|_| Error::ExecutionFailed(result))
}

/// Returns the response to a message.
///
/// A non-existent message identifier will return an empty response, which could also be a valid
/// response depending on the source message.
///
/// # Parameters
/// - `message` - The message identifier.
#[inline]
pub fn get_response(message: MessageId) -> DynBytes {
	let precompile: contract_ref!(Messaging, Pop, Sol) = PRECOMPILE_ADDRESS.into();
	precompile.getResponse(message)
}

/// The identifier of this chain.
///
/// NOTE: this is a precompile call and therefore has associated costs.
#[inline]
pub fn id() -> u32 {
	let precompile: contract_ref!(Messaging, Pop, Sol) = PRECOMPILE_ADDRESS.into();
	precompile.id()
}

/// Initiate a new XCM query.
///
/// Starts a query using the XCM interface, specifying a responder and timeout block.
///
/// # Parameters
/// - `responder` - The location of the XCM responder.
/// - `timeout` - Block number after which the query should timeout.
/// - `callback` - An optional callback to execute upon receiving a response.
///
/// # Returns
/// A unique query identifier.
#[inline]
pub fn new_query(
	responder: Location,
	timeout: BlockNumber,
	callback: Option<Callback>,
) -> (MessageId, QueryId) {
	let responder = DynBytes(responder.encode());
	match callback {
		None => {
			let precompile: contract_ref!(Xcm, Pop, Sol) = PRECOMPILE_ADDRESS.into();
			precompile.newQuery(responder, timeout)
		},
		Some(callback) => {
			let precompile: contract_ref!(XcmCallback, Pop, Sol) = PRECOMPILE_ADDRESS.into();
			precompile.newQuery(responder, timeout, callback)
		},
	}
}

/// Polls the status of a message.
///
/// # Parameters
/// - `message` - The message identifier to poll.
#[inline]
pub fn poll_status(message: MessageId) -> MessageStatus {
	let precompile: contract_ref!(Messaging, Pop, Sol) = PRECOMPILE_ADDRESS.into();
	precompile.pollStatus(message)
}

/// Remove a completed or timed-out message.
///
/// Allows users to clean up storage and reclaim deposits for messages that have concluded.
///
/// # Parameters
/// - `message` - The identifier of the message to remove.
#[inline]
pub fn remove(message: MessageId) {
	let precompile: contract_ref!(Messaging, Pop, Sol) = PRECOMPILE_ADDRESS.into();
	precompile.remove(message)
}

/// Remove a batch of completed or timed-out messages.
///
/// Allows users to clean up storage and reclaim deposits for messages that have concluded.
///
/// # Parameters
/// - `messages` - A set of identifiers of messages to remove (bounded by `MaxRemovals`).
#[inline]
pub fn remove_many(messages: Vec<MessageId>) {
	let precompile: contract_ref!(Messaging, Pop, Sol) = PRECOMPILE_ADDRESS.into();
	precompile.removeMany(messages)
}

/// Send an XCM from a given origin.
///
/// # Parameters
/// - `destination` - The destination of the message.
/// - `message` - A XCM message.
///
/// # Returns
/// A SCALE-encoded dispatch result.
#[inline]
pub fn send<Call: Encode>(
	destination: VersionedLocation,
	message: VersionedXcm<Call>,
) -> Result<(), Error> {
	let precompile: contract_ref!(Xcm, Pop, Sol) = PRECOMPILE_ADDRESS.into();
	let result = precompile.send(DynBytes(destination.encode()), DynBytes(message.encode()));
	Result::<(), ()>::decode(&mut result.0.as_slice())
		.map_err(|_| Error::DecodingFailed)?
		.map_err(|_| Error::SendingFailed(result))
}

/// A callback for handling responses to XCM queries.
#[ink::trait_definition]
pub trait OnQueryResponse {
	/// Handles a response to a XCM query.
	///
	/// # Parameters
	/// - `id` - The identifier of the originating message.
	/// - `response` - The response message.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn onQueryResponse(&mut self, id: MessageId, response: DynBytes);
}

/// Event emitted when a XCM query is completed.
#[ink::event]
pub struct XcmCompleted {
	/// The identifier of the originating message.
	#[ink(topic)]
	pub id: MessageId,
	/// The response message.
	pub result: DynBytes,
}

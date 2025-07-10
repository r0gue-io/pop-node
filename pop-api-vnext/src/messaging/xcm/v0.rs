pub use ink::xcm::prelude::{VersionedLocation, VersionedResponse, VersionedXcm};

use super::{super::v0::Callback, *};

pub type QueryId = u64;

// Precompile index within the runtime
const PRECOMPILE: u16 = 5;

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
	fn execute(&self, message: Bytes, weight: Weight) -> Bytes;

	/// Returns the response to a message.
	///
	/// A non-existent message identifier will return an empty response, which could also be a valid
	/// response depending on the source message.
	///
	/// # Parameters
	/// - `message` - The message identifier.
	#[ink(message)]
	fn getResponse(&self, message: MessageId) -> Bytes;

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
	#[ink(message, selector = 0x5a8db3bd)]
	fn new_query(&self, responder: Bytes, timeout: BlockNumber) -> QueryId;

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
	#[ink(message, selector = 0xc0ca060b)]
	fn new_query_with_callback(
		&self,
		responder: Bytes,
		timeout: BlockNumber,
		callback: Callback,
	) -> QueryId;

	/// Polls the status of a message.
	///
	/// # Parameters
	/// - `message` - The message identifier to poll.
	#[ink(message)]
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
	#[ink(message, selector = 0xcdd80f3b)]
	fn remove_many(&self, messages: Vec<MessageId>);

	/// Send an XCM from a given origin.
	///
	/// # Parameters
	/// - `destination` - The SCALE-encoded versioned location for the destination of the message.
	/// - `message` - A SCALE-encoded versioned XCM message.
	///
	/// # Returns
	/// A SCALE-encoded dispatch result.
	#[ink(message)]
	fn send(&self, destination: Bytes, message: Bytes) -> Bytes;
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
pub fn execute<Call: Encode>(message: VersionedXcm<Call>, weight: Weight) -> Bytes {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Xcm, Pop, Sol) = address.into();
	precompile.execute(message.encode(), weight)
}

/// Returns the response to a message.
///
/// A non-existent message identifier will return an empty response, which could also be a valid
/// response depending on the source message.
///
/// # Parameters
/// - `message` - The message identifier.
#[inline]
pub fn get_response(message: MessageId) -> Bytes {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Xcm, Pop, Sol) = address.into();
	precompile.getResponse(message)
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
	responder: VersionedLocation,
	timeout: BlockNumber,
	callback: Option<Callback>,
) -> QueryId {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Xcm, Pop, Sol) = address.into();
	match callback {
		None => precompile.new_query(responder.encode(), timeout),
		Some(callback) => precompile.new_query_with_callback(responder.encode(), timeout, callback),
	}
}

/// Polls the status of a message.
///
/// # Parameters
/// - `message` - The message identifier to poll.
#[inline]
pub fn poll_status(message: MessageId) -> MessageStatus {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Xcm, Pop, Sol) = address.into();
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
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Xcm, Pop, Sol) = address.into();
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
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Xcm, Pop, Sol) = address.into();
	precompile.remove_many(messages)
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
pub fn send<Call: Encode>(destination: VersionedLocation, message: VersionedXcm<Call>) -> Bytes {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Xcm, Pop, Sol) = address.into();
	precompile.send(destination.encode(), message.encode())
}

#[ink::trait_definition]
pub trait OnResponse {
	// pop-api::messaging::xcm::OnResponse::on_response
	#[ink(message, selector = 0x641b0b03)]
	fn on_response(&mut self, id: MessageId, response: Bytes);
}

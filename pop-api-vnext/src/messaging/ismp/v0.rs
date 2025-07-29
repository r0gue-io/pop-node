pub use errors::{Error, Error::*};
use ink::{Address, SolBytes};

use super::{super::v0::Callback, *};

mod errors;

// Precompile index within the runtime
const PRECOMPILE: u16 = 4;
/// The address of the ISMP precompile.
pub const PRECOMPILE_ADDRESS: Address = fixed_address(PRECOMPILE);

/// The ISMP precompile offers a streamlined interface for messaging using the Interoperable State
/// Machine Protocol.
#[ink::trait_definition]
pub trait Ismp {
	/// Submit a new ISMP `Get` request.
	///
	/// # Parameters
	/// - `request` - The ISMP `Get` message containing query details.
	/// - `fee` - The fee to be paid to relayers.
	///
	/// # Returns
	/// A unique message identifier.
	#[ink(message)]
	fn get(&self, request: Get, fee: U256) -> Result<MessageId, Error>;

	/// Submit a new ISMP `Post` request.
	///
	/// Sends a `Post` message through ISMP with arbitrary data.
	///
	/// # Parameters
	/// - `request` - The ISMP `Post` message containing the payload.
	/// - `fee` - The fee to be paid to relayers.
	///
	/// # Returns
	/// A unique message identifier.
	#[ink(message)]
	fn post(&self, request: Post, fee: U256) -> Result<MessageId, Error>;
}

/// The ISMP precompile offers a streamlined interface for messaging using the Interoperable State
/// Machine Protocol.
#[ink::trait_definition]
pub trait IsmpCallback {
	/// Submit a new ISMP `Get` request.
	///
	/// # Parameters
	/// - `request` - The ISMP `Get` message containing query details.
	/// - `fee` - The fee to be paid to relayers.
	/// - `callback` - The callback to execute upon receiving a response.
	///
	/// # Returns
	/// A unique message identifier.
	#[ink(message)]
	fn get(&self, request: Get, fee: U256, callback: Callback) -> Result<MessageId, Error>;

	/// Submit a new ISMP `Post` request.
	///
	/// Sends a `Post` message through ISMP with arbitrary data.
	///
	/// # Parameters
	/// - `request` - The ISMP `Post` message containing the payload.
	/// - `fee` - The fee to be paid to relayers.
	/// - `callback` - The callback to execute upon receiving a response.
	///
	/// # Returns
	/// A unique message identifier.
	#[ink(message)]
	fn post(&self, request: Post, fee: U256, callback: Callback) -> Result<MessageId, Error>;
}

/// The messaging interface of the ISMP precompile offers a general interface for cross-chain
/// messaging operations.
///
/// This convenience trait simply provides access to general cross-chain messaging operations via
/// the ISMP precompile, so that users need only use a single precompile if desired.
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
	fn getResponse(&self, message: MessageId) -> Bytes;

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
	fn remove(&self, message: MessageId) -> Result<(), Error>;

	/// Remove a batch of completed or timed-out messages.
	///
	/// Allows users to clean up storage and reclaim deposits for messages that have concluded.
	///
	/// # Parameters
	/// - `messages` - A set of identifiers of messages to remove (bounded by `MaxRemovals`).
	#[ink(message)]
	#[allow(non_snake_case)]
	fn removeMany(&self, messages: Vec<MessageId>) -> Result<(), Error>;
}

/// Submit a new ISMP `Get` request.
///
/// # Parameters
/// - `request` - The ISMP `Get` message containing query details.
/// - `fee` - The fee to be paid to relayers.
/// - `callback` - An optional callback to execute upon receiving a response.
///
/// # Returns
/// A unique message identifier.
#[inline]
pub fn get(request: Get, fee: U256, callback: Option<Callback>) -> Result<MessageId, Error> {
	match callback {
		None => {
			let precompile: contract_ref!(Ismp, Pop, Sol) = PRECOMPILE_ADDRESS.into();
			precompile.get(request, fee)
		},
		Some(callback) => {
			let precompile: contract_ref!(IsmpCallback, Pop, Sol) = PRECOMPILE_ADDRESS.into();
			precompile.get(request, fee, callback)
		},
	}
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

/// Polls the status of a message.
///
/// # Parameters
/// - `message` - The message identifier to poll.
#[inline]
pub fn poll_status(message: MessageId) -> MessageStatus {
	let precompile: contract_ref!(Messaging, Pop, Sol) = PRECOMPILE_ADDRESS.into();
	precompile.pollStatus(message)
}

/// Submit a new ISMP `Post` request.
///
/// Sends a `Post` message through ISMP with arbitrary data.
///
/// # Parameters
/// - `request` - The ISMP `Post` message containing the payload.
/// - `fee` - The fee to be paid to relayers.
/// - `callback` - An optional callback to execute upon receiving a response.
///
/// # Returns
/// A unique message identifier.
#[inline]
pub fn post(request: Post, fee: U256, callback: Option<Callback>) -> Result<MessageId, Error> {
	match callback {
		None => {
			let precompile: contract_ref!(Ismp, Pop, Sol) = PRECOMPILE_ADDRESS.into();
			precompile.post(request, fee)
		},
		Some(callback) => {
			let precompile: contract_ref!(IsmpCallback, Pop, Sol) = PRECOMPILE_ADDRESS.into();
			precompile.post(request, fee, callback)
		},
	}
}

/// Remove a completed or timed-out message.
///
/// Allows users to clean up storage and reclaim deposits for messages that have concluded.
///
/// # Parameters
/// - `message` - The identifier of the message to remove.
#[inline]
pub fn remove(message: MessageId) -> Result<(), Error> {
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
pub fn remove_many(messages: Vec<MessageId>) -> Result<(), Error> {
	let precompile: contract_ref!(Messaging, Pop, Sol) = PRECOMPILE_ADDRESS.into();
	precompile.removeMany(messages)
}

/// A GET request, intended to be used for sending outgoing requests.
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(ink::SolDecode, ink::SolEncode)]
pub struct Get {
	/// The destination state machine of this request.
	pub destination: u32,
	/// Height at which to read the state machine.
	pub height: u64,
	/// Relative from the current timestamp at which this request expires in seconds.
	pub timeout: u64,
	/// Some application-specific metadata relating to this request.
	pub context: Bytes,
	/// Raw Storage keys that would be used to fetch the values from the counterparty.
	pub keys: Vec<Bytes>,
}

impl Get {
	/// Creates a new GET request, intended to be used for sending outgoing requests.
	///
	/// # Parameters
	/// - `destination` - The destination state machine of this request.
	/// - `height` - Height at which to read the state machine.
	/// - `timeout` - Relative from the current timestamp at which this request expires in seconds.
	/// - `context` - Some application-specific metadata relating to this request.
	/// - `keys` - Raw Storage keys that would be used to fetch the values from the counterparty.
	pub fn new(
		destination: u32,
		height: u64,
		timeout: u64,
		context: Bytes,
		keys: Vec<Bytes>,
	) -> Self {
		Self { destination, height, timeout, context, keys }
	}
}

/// A POST request, intended to be used for sending outgoing requests.
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(ink::SolDecode, ink::SolEncode)]
pub struct Post {
	/// The destination state machine of this request.
	pub destination: u32,
	/// Relative from the current timestamp at which this request expires in seconds.
	pub timeout: u64,
	/// Encoded request data.
	pub data: Bytes,
}

impl Post {
	/// Creates a new POST request, intended to be used for sending outgoing requests.
	///
	/// # Parameters
	/// - `destination` - The destination state machine of this request.
	/// - `timeout` - Relative from the current timestamp at which this request expires in seconds.
	/// - `data` - Encoded request data.
	pub fn new(destination: u32, timeout: u64, data: Vec<u8>) -> Self {
		Self { destination, timeout, data: SolBytes(data) }
	}
}

/// A verified storage value.
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, ink::SolDecode, ink::SolEncode)]
pub struct StorageValue {
	/// The request storage key.
	pub key: Bytes,
	/// The verified value.
	pub value: Option<Bytes>,
}

/// A callback for handling responses to ISMP `Get` requests.
#[ink::trait_definition]
pub trait OnGetResponse {
	/// Handles a response to an ISMP `Get` request.
	///
	/// # Parameters
	/// - `id` - The identifier of the originating message.
	/// - `response` - The values derived from the state proof.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn onGetResponse(&mut self, id: MessageId, response: Vec<StorageValue>);
}

/// A callback for handling responses to ISMP `Post` requests.
#[ink::trait_definition]
pub trait OnPostResponse {
	/// Handles a response to an ISMP `Post` request.
	///
	/// # Parameters
	/// - `id` - The identifier of the originating message.
	/// - `response` - The response message.
	#[ink(message)]
	#[allow(non_snake_case)]
	fn onPostResponse(&mut self, id: MessageId, response: Bytes);
}

/// Event emitted when a ISMP `Get` request is completed.
#[ink::event]
pub struct IsmpGetCompleted {
	/// The identifier of the originating message.
	#[ink(topic)]
	pub id: MessageId,
	/// The values derived from the state proof.
	pub response: Vec<StorageValue>,
}

/// Event emitted when a ISMP `Post` request is completed.
#[ink::event]
pub struct IsmpPostCompleted {
	/// The identifier of the originating message.
	#[ink(topic)]
	pub id: MessageId,
	/// The response message.
	pub response: Bytes,
}

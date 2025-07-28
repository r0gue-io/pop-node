pub use errors::{Error, Error::*};

use super::*;

mod errors;

// Precompile index within the runtime
const PRECOMPILE: u16 = 3;
/// The address of the messaging precompile.
pub const PRECOMPILE_ADDRESS: Address = fixed_address(PRECOMPILE);

/// The messaging precompile offers a general interface for cross-chain messaging operations.
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

/// A message callback.
#[ink::scale_derive(Decode, Encode, TypeInfo)]
#[derive(ink::SolDecode, ink::SolEncode)]
pub struct Callback {
	/// The contract address to which the callback should be sent.
	destination: Address,
	/// The encoding used for the data going to the contract.
	encoding: Encoding,
	/// The message selector to be used for the callback.
	selector: FixedBytes<4>,
	/// The pre-paid weight used as a gas limit for the callback.
	weight: Weight,
}

impl Callback {
	/// Creates a new callback with the specified encoding, selector, and weight.
	///
	/// # Parameters
	/// - `destination` - The contract address to which the callback should be sent.
	/// - `encoding` - The encoding used for the data going to the contract.
	/// - `selector` - The message selector to be used for the callback.
	/// - `weight` - The pre-paid weight used as a gas limit for the callback.
	pub fn new(destination: Address, encoding: Encoding, selector: u32, weight: Weight) -> Self {
		Self { destination, encoding, selector: SolBytes(selector.to_be_bytes()), weight }
	}
}

/// The specificiation of how data must be encoded before being sent to a contract.
#[derive(Copy, Clone, ink::SolDecode, ink::SolEncode)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
#[repr(u8)]
pub enum Encoding {
	/// SCALE (Simple Concatenated Aggregate Little-Endian) encoding.
	Scale,
	/// Solidity ABI (Application Binary Interface) encoding,
	SolidityAbi,
}

/// The status of a message.
#[derive(Copy, Clone, ink::SolDecode, ink::SolEncode, PartialEq)]
#[ink::scale_derive(Decode, Encode, TypeInfo)]
#[cfg_attr(feature = "std", derive(Debug))]
#[repr(u8)]
pub enum MessageStatus {
	NotFound = 0,
	Pending = 1,
	Complete = 2,
	Timeout = 3,
}

/// One or more messages have been removed for the account.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Removed {
	/// The origin of the messages.
	#[ink(topic)]
	pub account: Address,
	/// The messages which were removed.
	#[ink(topic)]
	pub messages: Vec<MessageId>,
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

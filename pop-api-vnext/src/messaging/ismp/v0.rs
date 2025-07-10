use super::{super::v0::Callback, *};

// Precompile index within the runtime
const PRECOMPILE: u16 = 4;

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
	fn get(&self, request: Get, fee: U256) -> MessageId;

	/// Submit a new ISMP `Get` request.
	///
	/// # Parameters
	/// - `request` - The ISMP `Get` message containing query details.
	/// - `fee` - The fee to be paid to relayers.
	/// - `callback` - The callback to execute upon receiving a response.
	///
	/// # Returns
	/// A unique message identifier.
	#[ink(message, selector = 0x39f75435)]
	fn get_with_callback(&self, request: Get, fee: U256, callback: Callback) -> MessageId;

	/// Returns the response to a message.
	///
	/// A non-existent message identifier will return an empty response, which could also be a valid
	/// response depending on the source message.
	///
	/// # Parameters
	/// - `message` - The message identifier.
	#[ink(message, selector = 0xada86798)]
	fn get_response(&self, message: MessageId) -> Bytes;

	/// Polls the status of a message.
	///
	/// # Parameters
	/// - `message` - The message identifier to poll.
	#[ink(message)]
	fn pollStatus(&self, message: MessageId) -> MessageStatus;

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
	fn post(&self, request: Post, fee: U256) -> MessageId;

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
	#[ink(message, selector = 0xeb0f21f1)]
	fn post_with_callback(&self, request: Post, fee: U256, callback: Callback) -> MessageId;

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
pub fn get(request: Get, fee: U256, callback: Option<Callback>) -> MessageId {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Ismp, Pop, Sol) = address.into();
	match callback {
		None => precompile.get(request, fee),
		Some(callback) => precompile.get_with_callback(request, fee, callback),
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
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Ismp, Pop, Sol) = address.into();
	precompile.get_response(message)
}

/// Polls the status of a message.
///
/// # Parameters
/// - `message` - The message identifier to poll.
#[inline]
pub fn poll_status(message: MessageId) -> MessageStatus {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Ismp, Pop, Sol) = address.into();
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
pub fn post(request: Post, fee: U256, callback: Option<Callback>) -> MessageId {
	let address = fixed_address(PRECOMPILE);
	let precompile: contract_ref!(Ismp, Pop, Sol) = address.into();
	match callback {
		None => precompile.post(request, fee),
		Some(callback) => precompile.post_with_callback(request, fee, callback),
	}
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
	let precompile: contract_ref!(Ismp, Pop, Sol) = address.into();
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
	let precompile: contract_ref!(Ismp, Pop, Sol) = address.into();
	precompile.remove_many(messages)
}

/// A GET request, intended to be used for sending outgoing requests
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Get {
	/// The destination state machine of this request.
	pub destination: u32,
	/// Height at which to read the state machine.
	pub height: u64,
	/// Relative from the current timestamp at which this request expires in seconds.
	pub timeout: u64,
	/// Some application-specific metadata relating to this request.
	pub context: Vec<u8>,
	/// Raw Storage keys that would be used to fetch the values from the counterparty.
	pub keys: Vec<Vec<u8>>,
}

impl Get {
	// TODO: docs
	pub fn new(
		destination: u32,
		height: u64,
		timeout: u64,
		context: Vec<u8>,
		keys: Vec<Vec<u8>>,
	) -> Self {
		Self { destination, height, timeout, context, keys }
	}
}

impl SolDecode for Get {
	type SolType = (u32, u64, u64, Vec<u8>, Vec<Vec<u8>>);

	fn from_sol_type(value: Self::SolType) -> Self {
		Self {
			destination: value.0,
			height: value.1,
			timeout: value.2,
			context: value.3,
			keys: value.4,
		}
	}
}
impl<'a> SolEncode<'a> for Get {
	type SolType = (&'a u32, &'a u64, &'a u64, &'a Vec<u8>, &'a Vec<Vec<u8>>);

	fn to_sol_type(&'a self) -> Self::SolType {
		(&self.destination, &self.height, &self.timeout, &self.context, &self.keys)
	}
}

/// A POST request, intended to be used for sending outgoing requests.
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Post {
	/// The destination state machine of this request.
	pub destination: u32,
	/// Relative from the current timestamp at which this request expires in seconds.
	pub timeout: u64,
	/// Encoded request data.
	pub data: Vec<u8>,
}

impl Post {
	// TODO: docs
	pub fn new(destination: u32, timeout: u64, data: Vec<u8>) -> Self {
		Self { destination, timeout, data }
	}
}

impl SolDecode for Post {
	type SolType = (u32, u64, Vec<u8>);

	fn from_sol_type(value: Self::SolType) -> Self {
		Self { destination: value.0, timeout: value.1, data: value.2 }
	}
}
impl<'a> SolEncode<'a> for Post {
	type SolType = (&'a u32, &'a u64, &'a Vec<u8>);

	fn to_sol_type(&'a self) -> Self::SolType {
		(&self.destination, &self.timeout, &self.data)
	}
}

/// A verified storage value.
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[derive(Debug)]
pub struct StorageValue {
	/// The request storage key.
	pub key: Vec<u8>,
	/// The verified value.
	pub value: Option<Vec<u8>>,
}

impl SolDecode for StorageValue {
	type SolType = (Vec<u8>, (bool, Vec<u8>));

	fn from_sol_type(value: Self::SolType) -> Self {
		let key = value.0;
		let value = match value.1 .0 {
			true => Some(value.1 .1),
			false => None,
		};
		Self { key, value }
	}
}
impl<'a> SolEncode<'a> for StorageValue {
	type SolType = (&'a Vec<u8>, bool, &'a [u8]);

	fn to_sol_type(&'a self) -> Self::SolType {
		const EMPTY: [u8; 0] = [];
		(
			&self.key,
			self.value.is_some(),
			&self.value.as_ref().map_or(EMPTY.as_slice(), |v| v.as_slice()),
		)
	}
}

#[ink::trait_definition]
pub trait OnGetResponse {
	// pop-api::messaging::ismp::OnGetResponse::on_response
	#[ink(message, selector = 0x57ad942b)]
	fn on_response(&mut self, id: MessageId, values: Vec<StorageValue>);
}

#[ink::trait_definition]
pub trait OnPostResponse {
	// pop-api::messaging::ismp::OnPostResponse::on_response
	#[ink(message, selector = 0xcfb0a1d2)]
	fn on_response(&mut self, id: MessageId, response: Vec<u8>);
}

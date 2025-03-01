use super::*;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Get {
	pub dest: u32,
	pub height: u32,
	pub timeout: u64,
	// TODO: Option
	pub context: Vec<u8>,
	pub keys: Vec<Vec<u8>>,
}

impl Get {
	pub fn new(dest: u32, height: u32, timeout: u64, context: Vec<u8>, keys: Vec<Vec<u8>>) -> Self {
		// TODO: validate: dest para id, ensure at least one key
		Self { dest, height, timeout, context, keys }
	}
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Post {
	pub dest: u32,
	pub timeout: u64,
	pub data: Vec<u8>,
}

impl Post {
	pub fn new(dest: u32, timeout: u64, data: Vec<u8>) -> Self {
		// TODO: validate: dest para id, ensure data not empty
		Self { dest, timeout, data }
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

#[inline]
pub fn get(id: MessageId, request: Get, fee: Balance, callback: Option<Callback>) -> Result<()> {
	build_dispatch(ISMP_GET)
		.input::<(MessageId, Get, Balance, Option<Callback>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, request, fee, callback))
}

#[inline]
pub fn post(id: MessageId, request: Post, fee: Balance, callback: Option<Callback>) -> Result<()> {
	build_dispatch(ISMP_POST)
		.input::<(MessageId, Post, Balance, Option<Callback>)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, request, fee, callback))
}

#[ink::trait_definition]
pub trait OnGetResponse {
	// pop-api::messaging::ismp::OnGetResponse::on_response
	#[ink(message, selector = 0x57ad942b)]
	fn on_response(&mut self, id: MessageId, values: Vec<StorageValue>) -> Result<()>;
}

#[ink::trait_definition]
pub trait OnPostResponse {
	// pop-api::messaging::ismp::OnPostResponse::on_response
	#[ink(message, selector = 0xcfb0a1d2)]
	fn on_response(&mut self, id: MessageId, response: Vec<u8>) -> Result<()>;
}

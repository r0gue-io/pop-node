use super::*;

#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Get {
	dest: u32,
	height: u32,
	timeout: u64,
	// TODO: Option
	context: Vec<u8>,
	keys: Vec<Vec<u8>>,
}

impl Get {
	pub fn new(dest: u32, height: u32, timeout: u64, context: Vec<u8>, keys: Vec<Vec<u8>>) -> Self {
		// TODO: validate: dest para id, ensure at least one key
		Self { dest, height, timeout, context, keys }
	}
}

#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub struct Post {
	dest: u32,
	timeout: u64,
	data: Vec<u8>,
}

impl Post {
	pub fn new(dest: u32, timeout: u64, data: Vec<u8>) -> Self {
		// TODO: validate: dest para id, ensure data not empty
		Self { dest, timeout, data }
	}
}

#[inline]
pub fn get(id: RequestId, request: Get, fee: Balance) -> Result<()> {
	build_dispatch(ISMP_GET)
		.input::<(RequestId, Get, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, request, fee))
}

#[inline]
pub fn post(id: RequestId, request: Post, fee: Balance) -> Result<()> {
	build_dispatch(ISMP_POST)
		.input::<(RequestId, Post, Balance)>()
		.output::<Result<()>, true>()
		.handle_error_code::<StatusCode>()
		.call(&(id, request, fee))
}

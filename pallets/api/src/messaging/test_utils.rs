use ::xcm::latest::{Junctions, Location};
use codec::Encode;
use frame_benchmarking::{account, v2::*};
use frame_support::{dispatch::RawOrigin, traits::Currency, BoundedVec};
use ismp::{
	host::StateMachine,
	module::IsmpModule,
	router::{
		GetRequest, GetResponse, PostRequest, PostResponse, Response as IsmpResponse, StorageValue,
	},
};
use sp_runtime::traits::{One, Zero};
use sp_std::vec;

use crate::messaging::Config;

pub fn ismp_get_request(key_len: usize, keys_len: usize, context_len: usize) -> GetRequest {
	GetRequest {
		source: StateMachine::Polkadot(2000),
		dest: StateMachine::Polkadot(2001),
		nonce: 100u64,
		from: [1u8; 32].to_vec(),
		keys: vec![vec![1u8; key_len]; keys_len],
		height: 1,
		context: [1u8].repeat(context_len),
		timeout_timestamp: 100_000,
	}
}

pub fn ismp_post_request(body_len: usize) -> PostRequest {
	PostRequest {
		source: StateMachine::Polkadot(2000),
		dest: StateMachine::Polkadot(2001),
		nonce: 100u64,
		from: [1u8; 32].to_vec(),
		to: [1u8; 32].to_vec(),
		timeout_timestamp: 100_000,
		body: [1u8].repeat(body_len),
	}
}

pub fn ismp_get_response(
	key_len: usize,
	keys_len: usize,
	context_len: usize,
	response_len: usize,
) -> GetResponse {
	let r = get_storage_value();
	let r_encoded_size = r.encoded_size();
	let mut values = vec![];

	let iterations = response_len / r_encoded_size;
	for i in 0..iterations - 1 {
		values.push(r.clone());
	}

	debug_assert!(values.encoded_size() < response_len);
	GetResponse { get: ismp_get_request(key_len, keys_len, context_len), values }
}

pub fn ismp_post_response(body_len: usize, response_len: usize) -> PostResponse {
	let response = [1u8].repeat(response_len - 2);
	PostResponse { post: ismp_post_request(body_len), response, timeout_timestamp: 100_001 }
}

pub fn get_storage_value() -> StorageValue {
	StorageValue { key: [1u8; 1].to_vec(), value: Some([1u8; 1].to_vec()) }
}

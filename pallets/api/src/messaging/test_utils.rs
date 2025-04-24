use ismp::{host::StateMachine, router::PostRequest};
use sp_std::vec;

#[cfg(any(feature = "runtime-benchmarks", test))]
/// Constructs a dummy `PostRequest` used for testing or benchmarking.
pub fn ismp_post_request(body_len: usize) -> PostRequest {
	PostRequest {
		source: StateMachine::Polkadot(2000),
		dest: StateMachine::Polkadot(2001),
		nonce: 100u64,
		from: [1u8; 32].to_vec(),
		to: [1u8; 32].to_vec(),
		timeout_timestamp: 100_000,
		body: vec![1u8; body_len],
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub use benchmark_helpers::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmark_helpers {
	use codec::Encode;
	use ismp::{
		host::StateMachine,
		router::{GetRequest, GetResponse, PostResponse, StorageValue},
	};
	use sp_std::{vec, vec::Vec};

	use super::ismp_post_request;

	/// Constructs a dummy `GetRequest` with specified dimensions.
	pub fn ismp_get_request(key_len: usize, keys_len: usize, context_len: usize) -> GetRequest {
		GetRequest {
			source: StateMachine::Polkadot(2000),
			dest: StateMachine::Polkadot(2001),
			nonce: 100u64,
			from: vec![1u8; 32],
			keys: vec![vec![1u8; key_len]; keys_len],
			height: 1,
			context: vec![1u8; context_len],
			timeout_timestamp: 100_000,
		}
	}

	/// Constructs a dummy `GetResponse` of approximately the requested size.
	pub fn ismp_get_response(
		key_len: usize,
		keys_len: usize,
		context_len: usize,
		response_len: usize,
	) -> GetResponse {
		let r = get_storage_value();
		let r_encoded_size = r.encoded_size();
		let iterations = response_len / r_encoded_size;

		let values = (0..iterations.saturating_sub(1)).map(|_| r.clone()).collect::<Vec<_>>();

		debug_assert!(values.encoded_size() < response_len);

		GetResponse { get: ismp_get_request(key_len, keys_len, context_len), values }
	}

	/// Constructs a dummy `PostResponse` with a body of the requested length.
	pub fn ismp_post_response(body_len: usize, response_len: usize) -> PostResponse {
		let response = vec![1u8; response_len.saturating_sub(2)];
		PostResponse { post: ismp_post_request(body_len), response, timeout_timestamp: 100_001 }
	}

	/// Constructs a dummy `StorageValue`.
	pub fn get_storage_value() -> StorageValue {
		StorageValue { key: vec![1u8; 1], value: Some(vec![1u8; 1]) }
	}
}

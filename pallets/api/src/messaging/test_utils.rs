use ::xcm::latest::{Junctions, Location};
use frame_benchmarking::{account, v2::*};
use frame_support::{dispatch::RawOrigin, traits::Currency, BoundedVec};
use ismp::{
	host::StateMachine,
	module::IsmpModule,
	router::{GetRequest, GetResponse, PostRequest, PostResponse, Response as IsmpResponse},
};
use sp_runtime::traits::{One, Zero};

pub fn ismp_get_request() -> GetRequest {
	GetRequest {
		source: StateMachine::Polkadot(2000),
		dest: StateMachine::Polkadot(2001),
		nonce: 100u64,
		from: vec![],
		keys: vec![vec![]],
		height: 1,
		context: vec![],
		timeout_timestamp: 10000,
	}
}

pub fn ismp_post_request() -> PostRequest {
	PostRequest {
		source: StateMachine::Polkadot(2000),
		dest: StateMachine::Polkadot(2001),
		nonce: 100u64,
		from: vec![],
		to: vec![],
		timeout_timestamp: 10000,
		body: vec![],
	}
}

pub fn ismp_get_response() -> GetResponse {
	GetResponse { get: ismp_get_request(), values: vec![] }
}

pub fn ismp_post_response() -> PostResponse {
	PostResponse { post: ismp_post_request(), response: Default::default(), timeout_timestamp: 0 }
}

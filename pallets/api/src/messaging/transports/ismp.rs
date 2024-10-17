use core::marker::PhantomData;

pub(crate) use ::ismp::dispatcher::{FeeMetadata, IsmpDispatcher};
use ::ismp::{
	dispatcher::{
		DispatchGet,
		DispatchRequest::{self},
	},
	host::StateMachine,
};
use codec::{Decode, Encode};
use frame_support::{
	pallet_prelude::Weight, CloneNoBound, DebugNoBound, EqNoBound, PartialEqNoBound,
};
use ismp::{
	dispatcher::DispatchPost,
	module::IsmpModule,
	router::{GetResponse, PostRequest, PostResponse, Request::*, Response, Timeout},
	Error,
};
use pallet_ismp::weights::IsmpModuleWeight;
use scale_info::TypeInfo;
use sp_core::{keccak_256, Get as GetT, H256};
use sp_runtime::BoundedVec;

use crate::messaging::{
	pallet::{Config, Event, IsmpRequests, Pallet, Requests, Responses},
	MaxContextLenOf, MaxDataLenOf, MaxKeyLenOf, MaxKeysOf, Status,
};

pub const ID: [u8; 3] = *b"pop";

type DbWeightOf<T> = <T as frame_system::Config>::DbWeight;
pub(crate) type GetOf<T> = Get<MaxContextLenOf<T>, MaxKeysOf<T>, MaxKeyLenOf<T>>;
pub(crate) type PostOf<T> = Post<MaxDataLenOf<T>>;

#[derive(Encode, EqNoBound, CloneNoBound, DebugNoBound, Decode, PartialEqNoBound, TypeInfo)]
#[scale_info(skip_type_params(MaxContextLen, MaxKeys, MaxKeyLen, MaxDataLen))]
pub enum Request<
	MaxContextLen: GetT<u32>,
	MaxKeys: GetT<u32>,
	MaxKeyLen: GetT<u32>,
	MaxDataLen: GetT<u32>,
> {
	Get(Get<MaxContextLen, MaxKeys, MaxKeyLen>),
	Post(Post<MaxDataLen>),
}

impl<MaxContextLen: GetT<u32>, MaxKeys: GetT<u32>, MayKeyLen: GetT<u32>, MaxDataLen: GetT<u32>>
	From<Request<MaxContextLen, MaxKeys, MayKeyLen, MaxDataLen>> for DispatchRequest
{
	fn from(value: Request<MaxContextLen, MaxKeys, MayKeyLen, MaxDataLen>) -> Self {
		match value {
			Request::Get(get) => Self::Get(get.into()),
			Request::Post(post) => Self::Post(post.into()),
		}
	}
}

#[derive(Encode, EqNoBound, CloneNoBound, DebugNoBound, Decode, PartialEqNoBound, TypeInfo)]
#[scale_info(skip_type_params(MaxContextLen, MaxKeys, MaxKeyLen))]
pub struct Get<MaxContextLen: GetT<u32>, MaxKeys: GetT<u32>, MaxKeyLen: GetT<u32>> {
	// TODO: Option<u32> to support relay?
	pub(crate) dest: u32,
	pub(crate) height: u32,
	pub(crate) timeout: u64,
	pub(crate) context: BoundedVec<u8, MaxContextLen>,
	pub(crate) keys: BoundedVec<BoundedVec<u8, MaxKeyLen>, MaxKeys>,
}

impl<MaxContextLen: GetT<u32>, MaxKeys: GetT<u32>, MayKeyLen: GetT<u32>>
	From<Get<MaxContextLen, MaxKeys, MayKeyLen>> for DispatchGet
{
	fn from(value: Get<MaxContextLen, MaxKeys, MayKeyLen>) -> Self {
		DispatchGet {
			dest: StateMachine::Polkadot(value.dest),
			from: ID.into(),
			keys: value.keys.into_iter().map(|key| key.into_inner()).collect(),
			height: value.height.into(),
			context: value.context.into_inner(),
			timeout: value.timeout,
		}
	}
}

#[derive(Encode, EqNoBound, CloneNoBound, DebugNoBound, Decode, PartialEqNoBound, TypeInfo)]
#[scale_info(skip_type_params(MaxDataLen))]
pub struct Post<MaxDataLen: GetT<u32>> {
	// TODO: Option<u32> to support relay?
	pub(crate) dest: u32,
	pub(crate) timeout: u64,
	pub(crate) data: BoundedVec<u8, MaxDataLen>,
}

impl<MaxDataLen: GetT<u32>> From<Post<MaxDataLen>> for DispatchPost {
	fn from(value: Post<MaxDataLen>) -> Self {
		DispatchPost {
			dest: StateMachine::Polkadot(value.dest),
			from: ID.into(),
			to: ID.into(),
			timeout: value.timeout,
			body: value.data.into_inner(),
		}
	}
}

pub struct Handler<T>(PhantomData<T>);
impl<T> Handler<T> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<T: Config> IsmpModule for Handler<T> {
	fn on_accept(&self, _request: PostRequest) -> Result<(), Error> {
		todo!()
	}

	fn on_response(&self, response: Response) -> Result<(), Error> {
		let ((requestor, id), response) = match response {
			Response::Get(GetResponse { get, values }) => {
				// hash request to determine key for original request id lookup
				(
					IsmpRequests::<T>::get(H256::from(keccak_256(
						&ismp::router::Request::Get(get).encode(),
					)))
					.ok_or(Error::Custom("request not found".into()))?,
					values.encode(),
				)
			},
			Response::Post(PostResponse { post, response, .. }) => {
				// hash request to determine key for original request id lookup
				(
					IsmpRequests::<T>::get(H256::from(keccak_256(&Post(post).encode())))
						.ok_or(Error::Custom("request not found".into()))?,
					response,
				)
			},
		};

		// Store values for later retrieval
		let response: BoundedVec<u8, T::MaxResponseLen> =
			response.try_into().map_err(|_| Error::Custom("response exceeds max".into()))?;
		Requests::<T>::try_mutate(&requestor, &id, |v| {
			let Some((status, ..)) = v else {
				return Err(Error::Custom("response exceeds max".into()))
			};
			*status = Status::Complete;
			Ok(())
		})?;
		Responses::<T>::insert(&requestor, &id, response);
		Pallet::<T>::deposit_event(Event::<T>::ResponseReceived { requestor, id });
		Ok(())
	}

	fn on_timeout(&self, request: Timeout) -> Result<(), Error> {
		match request {
			Timeout::Request(request) => {
				// hash request to determine key for original request id lookup
				let id = match request {
					Get(get) => H256::from(keccak_256(&get.encode())),
					Post(post) => H256::from(keccak_256(&post.encode())),
				};
				let key =
					IsmpRequests::<T>::get(id).ok_or(Error::Custom("request not found".into()))?;
				Requests::<T>::try_mutate(key.0, key.1, |v| {
					let Some((status, ..)) = v else {
						return Err(Error::Custom("response exceeds max".into()))
					};
					*status = Status::TimedOut;
					Ok(())
				})?;
				Ok(())
			},
			Timeout::Response(_response) => {
				todo!()
			},
		}
	}
}

// TODO: replace with benchmarked weight functions
impl<T: Config> IsmpModuleWeight for Pallet<T> {
	fn on_accept(&self, _request: &PostRequest) -> Weight {
		todo!()
	}

	fn on_timeout(&self, _request: &Timeout) -> Weight {
		DbWeightOf::<T>::get().reads_writes(2, 1)
	}

	fn on_response(&self, _response: &Response) -> Weight {
		DbWeightOf::<T>::get().reads_writes(2, 2)
	}
}

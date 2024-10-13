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
use sp_core::{keccak_256, Get, H256};
use sp_runtime::BoundedVec;

use super::{
	pallet::{Config, Event, IsmpRequests, Pallet, Requests, Responses},
	Status,
};

pub const ID: [u8; 3] = *b"pop";

type DbWeightOf<T> = <T as frame_system::Config>::DbWeight;

#[derive(Encode, EqNoBound, CloneNoBound, DebugNoBound, Decode, PartialEqNoBound, TypeInfo)]
#[scale_info(skip_type_params(MaxContextLen, MaxKeys, MaxKeyLen, MaxDataLen))]
pub enum Request<
	MaxContextLen: Get<u32>,
	MaxKeys: Get<u32>,
	MaxKeyLen: Get<u32>,
	MaxDataLen: Get<u32>,
> {
	Get {
		para: u32,
		height: u32,
		timeout: u64,
		context: BoundedVec<u8, MaxContextLen>,
		keys: BoundedVec<BoundedVec<u8, MaxKeyLen>, MaxKeys>,
	},
	Post {
		para: u32,
		timeout: u64,
		data: BoundedVec<u8, MaxDataLen>,
	},
}

impl<MaxContextLen: Get<u32>, MaxKeys: Get<u32>, MayKeyLen: Get<u32>, MaxDataLen: Get<u32>>
	From<Request<MaxContextLen, MaxKeys, MayKeyLen, MaxDataLen>> for DispatchRequest
{
	fn from(value: Request<MaxContextLen, MaxKeys, MayKeyLen, MaxDataLen>) -> Self {
		match value {
			Request::Get { para, height, timeout, context, keys } => Self::Get(DispatchGet {
				dest: StateMachine::Polkadot(para),
				from: ID.into(),
				keys: keys.into_iter().map(|key| key.into_inner()).collect(),
				height: height.into(),
				context: context.into_inner(),
				timeout,
			}),
			Request::Post { para, timeout, data } => Self::Post(DispatchPost {
				dest: StateMachine::Polkadot(para),
				from: ID.into(),
				to: ID.into(),
				timeout,
				body: data.into_inner(),
			}),
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
		let (id, response) = match response {
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
		Requests::<T>::try_mutate(&id.0, &id.1, |v| {
			let Some((status, ..)) = v else {
				return Err(Error::Custom("response exceeds max".into()))
			};
			*status = Status::Complete;
			Ok(())
		})?;
		Responses::<T>::insert(&id.0, &id.1, response);
		Pallet::<T>::deposit_event(Event::<T>::ResponseReceived { id });
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

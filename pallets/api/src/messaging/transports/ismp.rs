use core::marker::PhantomData;

pub(crate) use ::ismp::dispatcher::{FeeMetadata, IsmpDispatcher};
use ::ismp::{
	dispatcher::{
		DispatchGet,
		DispatchRequest::{self},
	},
	host::StateMachine,
};
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	pallet_prelude::Weight, traits::Get as _, CloneNoBound, DebugNoBound, EqNoBound,
	PartialEqNoBound, ensure,
};
use ismp::{
	dispatcher::DispatchPost,
	module::IsmpModule,
	router::{GetResponse, PostRequest, PostResponse, Request::*, Response, Timeout},
	Error,
};
use pallet_ismp::weights::IsmpModuleWeight;
use scale_info::TypeInfo;
use sp_core::{keccak_256, H256};
use sp_runtime::{BoundedVec, Saturating};

use crate::messaging::{
	pallet::{Config, Event, IsmpRequests, Messages, Pallet},
	AccountIdOf, MessageId, Vec,
};

pub const ID: [u8; 3] = *b"pop";

type DbWeightOf<T> = <T as frame_system::Config>::DbWeight;

#[derive(Encode, EqNoBound, CloneNoBound, DebugNoBound, Decode, PartialEqNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub enum Message<T: Config> {
	Get(Get<T>),
	Post(Post<T>),
}

impl<T: Config> From<Message<T>> for DispatchRequest {
	fn from(value: Message<T>) -> Self {
		match value {
			Message::Get(get) => get.into(),
			Message::Post(post) => post.into(),
		}
	}
}

#[derive(
	Encode, EqNoBound, CloneNoBound, DebugNoBound, Decode, PartialEqNoBound, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub struct Get<T: Config> {
	// TODO: Option<u32> to support relay?
	pub(crate) dest: u32,
	pub(crate) height: u32,
	pub(crate) timeout: u64,
	pub(crate) context: BoundedVec<u8, T::MaxContextLen>,
	pub(crate) keys: BoundedVec<BoundedVec<u8, T::MaxKeyLen>, T::MaxKeys>,
}

impl<T: Config> From<Get<T>> for DispatchGet {
	fn from(value: Get<T>) -> Self {
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

impl<T: Config> From<Get<T>> for DispatchRequest {
	fn from(value: Get<T>) -> Self {
		DispatchRequest::Get(value.into())
	}
}

#[derive(
	Encode, EqNoBound, CloneNoBound, DebugNoBound, Decode, PartialEqNoBound, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(T))]
pub struct Post<T: Config> {
	// TODO: Option<u32> to support relay?
	pub(crate) dest: u32,
	pub(crate) timeout: u64,
	pub(crate) data: BoundedVec<u8, T::MaxDataLen>,
}

impl<T: Config> From<Post<T>> for DispatchPost {
	fn from(value: Post<T>) -> Self {
		DispatchPost {
			dest: StateMachine::Polkadot(value.dest),
			from: ID.into(),
			to: ID.into(),
			timeout: value.timeout,
			body: value.data.into_inner(),
		}
	}
}

impl<T: Config> From<Post<T>> for DispatchRequest {
	fn from(value: Post<T>) -> Self {
		DispatchRequest::Post(value.into())
	}
}

pub struct Handler<T>(PhantomData<T>);
impl<T> Handler<T> {
	pub fn new() -> Self {
		Self(PhantomData)
	}
}

impl<T: Config> IsmpModule for Handler<T> {
	fn on_accept(&self, _request: PostRequest) -> Result<(), anyhow::Error> {
		Err(Error::Custom("pop-net is not accepting post requests at this time!".to_string()).into())
	}

	fn on_response(&self, response: Response) -> Result<(), anyhow::Error> {
		// Hash request to determine key for message lookup.
		match response {
			Response::Get(GetResponse { get, values }) => {
				log::debug!(target: "pop-api::extension", "StorageValue={:?}", values[0]);
				// TODO: This should be bound to the hasher used in the ismp dispatcher.
				let commitment = H256::from(keccak_256(&ismp::router::Request::Get(get).encode()));
				process_response(
					&commitment,
					&values,
					|| values.encode(),
					|dest, id| Event::<T>::IsmpGetResponseReceived { dest, id, commitment },
				)
			},
			Response::Post(PostResponse { post, response, .. }) => {
				let commitment = H256::from(keccak_256(&Post(post).encode()));
				process_response(
					&commitment,
					&response,
					| | response.clone(), 
					|dest, id| Event::<T>::IsmpPostResponseReceived { dest, id, commitment },
				)
			},
		}
	}

	fn on_timeout(&self, timeout: Timeout) -> Result<(), anyhow::Error> {
		match timeout {
			Timeout::Request(request) => {
				// hash request to determine key for original request id lookup
				let id = match request {
					// TODO: This should be bound to the hasher used in the ismp dispatcher.
					Get(get) => H256::from(keccak_256(&get.encode())),
					Post(post) => H256::from(keccak_256(&post.encode())),
				};
				let key =
					IsmpRequests::<T>::get(id).ok_or(Error::Custom("request not found".into()))?;
				Messages::<T>::try_mutate(key.0, key.1, |message| {
					let Some(super::super::Message::Ismp { commitment, deposit, .. }) = message
					else {
						return Err(Error::Custom("message not found".into()))
					};
					 *message = Some(super::super::Message::IsmpTimeout {
					 	deposit: *deposit,
					 	commitment: *commitment,
					 });

					 Ok(())
				})?;
				Ok(())
			},
			Timeout::Response(_response) => {
				// We have received a response which has timedout, which should be acted on as to not bloat storage. 
				// We have an opportunity either to return the deposit not and mark as timed out to await removal.

				todo!("Quick chat with peter.")

			},
		}
	}
}

impl<T: Config> IsmpModuleWeight for Pallet<T> {

	// Static as not in use.
	fn on_accept(&self, _request: &PostRequest) -> Weight {
		DbWeightOf::<T>::get().reads_writes(2, 1)
	}

	fn on_timeout(&self, _timeout: &Timeout) -> Weight {
		DbWeightOf::<T>::get().reads_writes(2, 1)
	}

	fn on_response(&self, _response: &Response) -> Weight {
		DbWeightOf::<T>::get().reads_writes(2, 2)
	}
}


fn process_response<T: Config>(
	commitment: &H256,
	encode: &impl Encode,
	store: impl Fn() -> Vec<u8>,
	event: impl Fn(AccountIdOf<T>, MessageId) -> Event<T>,
) -> Result<(), anyhow::Error> {
	let (origin, id) =
		IsmpRequests::<T>::get(commitment).ok_or(Error::Custom("request not found".into()))?;

	let Some(super::super::Message::Ismp { commitment, callback, deposit }) =
		Messages::<T>::get(&origin, &id)
	else {
		return Err(Error::Custom("message not found".into()).into())
	};

	// Attempt callback with result if specified.
	if let Some(callback) = callback {
		// ensure!(encode.len() <= T::MaxResponseLen::get(), Error::Custom("Response length is too long!"));
		// TODO: check response length
		// TODO: update status if failed
		if Pallet::<T>::call(&origin, callback, &id, &encode).is_ok() {
			Pallet::<T>::deposit_event(event(origin, id));
			return Ok(());
		}
	}

	// Otherwise store response for manual retrieval and removal.
	let response: BoundedVec<u8, T::MaxResponseLen> =
		store().try_into().map_err(|_| Error::Custom("response exceeds max".into()))?;
	Messages::<T>::insert(
		&origin,
		&id,
		super::super::Message::IsmpResponse { commitment, deposit, response },
	);
	Pallet::<T>::deposit_event(event(origin, id));
	Ok(())
}

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
	PartialEqNoBound,
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
use sp_runtime::{BoundedVec, SaturatedConversion, Saturating};

use crate::messaging::{
	pallet::{Config, Event, IsmpRequests, Messages, Pallet},
	BalanceOf, CalculateDeposit, MessageId,
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

#[derive(Encode, EqNoBound, CloneNoBound, DebugNoBound, Decode, PartialEqNoBound, TypeInfo)]
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

impl<T: Config> CalculateDeposit<BalanceOf<T>> for Get<T> {
	fn calculate_deposit(&self) -> BalanceOf<T> {
		let len = self.dest.encoded_size() +
			self.height.encoded_size() +
			self.timeout.encoded_size() +
			self.context.len() +
			self.keys.iter().map(|k| k.len()).sum::<usize>();
		calculate_deposit::<T>(T::IsmpByteFee::get() * len.saturated_into())
	}
}

#[derive(Encode, EqNoBound, CloneNoBound, DebugNoBound, Decode, PartialEqNoBound, TypeInfo)]
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

impl<T: Config> CalculateDeposit<BalanceOf<T>> for Post<T> {
	fn calculate_deposit(&self) -> BalanceOf<T> {
		let len = self.dest.encoded_size() + self.timeout.encoded_size() + self.data.len();
		calculate_deposit::<T>(T::IsmpByteFee::get() * len.saturated_into())
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
		let ((origin, id), response) = match response {
			Response::Get(GetResponse { get, values }) => {
				// hash request to determine key for original message id lookup
				let id = IsmpRequests::<T>::get(H256::from(keccak_256(
					&ismp::router::Request::Get(get).encode(),
				)))
				.ok_or(Error::Custom("request not found".into()))?;
				Pallet::<T>::deposit_event(Event::<T>::IsmpGetResponseReceived {
					dest: id.0.clone(),
					id: id.1,
				});
				(id, values.encode())
			},
			Response::Post(PostResponse { post, response, .. }) => {
				// hash request to determine key for original message id lookup
				let id = IsmpRequests::<T>::get(H256::from(keccak_256(&Post(post).encode())))
					.ok_or(Error::Custom("request not found".into()))?;
				Pallet::<T>::deposit_event(Event::<T>::IsmpPostResponseReceived {
					dest: id.0.clone(),
					id: id.1,
				});
				(id, response)
			},
		};

		// Store values for later retrieval
		let response: BoundedVec<u8, T::MaxResponseLen> =
			response.try_into().map_err(|_| Error::Custom("response exceeds max".into()))?;
		Messages::<T>::try_mutate(&origin, &id, |message| {
			let Some(super::super::Message::Ismp { deposit, commitment }) = message else {
				return Err(Error::Custom("message not found".into()))
			};
			*message = Some(super::super::Message::IsmpResponse {
				deposit: *deposit,
				commitment: *commitment,
				response,
			});
			Ok(())
		})?;
		Ok(())
	}

	fn on_timeout(&self, timeout: Timeout) -> Result<(), Error> {
		match timeout {
			Timeout::Request(request) => {
				// hash request to determine key for original request id lookup
				let id = match request {
					Get(get) => H256::from(keccak_256(&get.encode())),
					Post(post) => H256::from(keccak_256(&post.encode())),
				};
				let key =
					IsmpRequests::<T>::get(id).ok_or(Error::Custom("request not found".into()))?;
				Messages::<T>::try_mutate(key.0, key.1, |message| {
					let Some(super::super::Message::Ismp { deposit, commitment }) = message else {
						return Err(Error::Custom("message not found".into()))
					};
					*message = Some(super::super::Message::IsmpTimedOut {
						deposit: *deposit,
						commitment: *commitment,
					});
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

	fn on_timeout(&self, _timeout: &Timeout) -> Weight {
		DbWeightOf::<T>::get().reads_writes(2, 1)
	}

	fn on_response(&self, _response: &Response) -> Weight {
		DbWeightOf::<T>::get().reads_writes(2, 2)
	}
}

fn calculate_deposit<T: Config>(mut deposit: BalanceOf<T>) -> BalanceOf<T> {
	// Add amount for `IsmpRequests` lookup.
	let key_len: BalanceOf<T> =
		(T::AccountId::max_encoded_len() + MessageId::max_encoded_len()).saturated_into();
	deposit.saturating_accrue(
		T::ByteFee::get() * (H256::max_encoded_len().saturated_into::<BalanceOf<T>>() + key_len),
	);

	deposit
}
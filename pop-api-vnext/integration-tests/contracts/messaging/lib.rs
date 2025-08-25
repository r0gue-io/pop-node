#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{abi::Sol, contract_ref, prelude::vec::Vec, sol::DynBytes, storage::Mapping, U256};
use pop_api::{
	messaging::{
		self as api,
		ismp::{
			self, Get, Ismp, IsmpCallback, IsmpGetCompleted, IsmpPostCompleted, OnGetResponse,
			OnPostResponse, Post, StorageValue,
		},
		xcm::{self, OnQueryResponse, QueryId, Xcm, XcmCallback, XcmCompleted},
		Callback, Error, MessageId, MessageStatus, Weight,
	},
	Pop,
};

#[ink::contract]
pub mod messaging {
	use super::*;

	#[ink(storage)]
	pub struct Messaging {
		/// Successful responses.
		responses: Mapping<MessageId, Response>,
	}

	impl Messaging {
		#[ink(constructor, default, payable)]
		#[allow(clippy::new_without_default)]
		pub fn new() -> Self {
			Self { responses: Mapping::new() }
		}
	}

	impl api::Messaging for Messaging {
		#[ink(message)]
		fn getResponse(&self, message: MessageId) -> DynBytes {
			api::get_response(message)
		}

		#[ink(message)]
		fn id(&self) -> u32 {
			api::id()
		}

		#[ink(message)]
		fn pollStatus(&self, message: MessageId) -> MessageStatus {
			api::poll_status(message)
		}

		#[ink(message)]
		fn remove(&self, message: MessageId) -> Result<(), Error> {
			api::remove(message)?;
			self.responses.remove(&message);
			Ok(())
		}

		#[ink(message)]
		fn removeMany(&self, messages: Vec<MessageId>) -> Result<(), Error> {
			for message in &messages {
				self.responses.remove(message);
			}
			api::remove_many(messages)?;
			Ok(())
		}
	}

	impl Ismp for Messaging {
		#[ink(message)]
		fn get(&self, request: Get, fee: U256) -> Result<MessageId, ismp::Error> {
			ismp::get(request, fee, None)
		}

		#[ink(message)]
		fn post(&self, request: Post, fee: U256) -> Result<MessageId, ismp::Error> {
			ismp::post(request, fee, None)
		}
	}

	impl IsmpCallback for Messaging {
		#[ink(message)]
		fn get(
			&self,
			request: Get,
			fee: U256,
			callback: Callback,
		) -> Result<MessageId, ismp::Error> {
			ismp::get(request, fee, Some(callback))
		}

		#[ink(message)]
		fn post(
			&self,
			request: Post,
			fee: U256,
			callback: Callback,
		) -> Result<MessageId, ismp::Error> {
			ismp::post(request, fee, Some(callback))
		}
	}

	impl OnGetResponse for Messaging {
		#[ink(message)]
		fn onGetResponse(&self, id: MessageId, response: Vec<StorageValue>) {
			// Adding state requires storage deposit limit to be defined on callback. Deposit is
			// moved from caller to contract and placed on hold. Deposit is claimed by anyone that
			// removes state, so adequate controls should be implemented by contract as desired.
			self.responses.insert(id, &Response::IsmpGet(response.clone()));
			self.env().emit_event(IsmpGetCompleted { id, response });
		}
	}

	impl OnPostResponse for Messaging {
		#[ink(message)]
		fn onPostResponse(&self, id: MessageId, response: DynBytes) {
			// Adding state requires storage deposit limit to be defined on callback. Deposit is
			// moved from caller to contract and placed on hold. Deposit is claimed by anyone that
			// removes state, so adequate controls should be implemented by contract as desired.
			self.responses.insert(id, &Response::IsmpPost(response.0.clone()));
			self.env().emit_event(IsmpPostCompleted { id, response });
		}
	}

	impl Xcm for Messaging {
		#[ink(message)]
		fn execute(&self, message: DynBytes, weight: Weight) -> DynBytes {
			let precompile: contract_ref!(Xcm, Pop, Sol) = xcm::PRECOMPILE_ADDRESS.into();
			precompile.execute(message, weight)
		}

		#[ink(message)]
		fn newQuery(&self, responder: DynBytes, timeout: BlockNumber) -> (MessageId, QueryId) {
			let precompile: contract_ref!(Xcm, Pop, Sol) = xcm::PRECOMPILE_ADDRESS.into();
			precompile.newQuery(responder, timeout)
		}

		#[ink(message)]
		fn send(&self, destination: DynBytes, message: DynBytes) -> DynBytes {
			let precompile: contract_ref!(Xcm, Pop, Sol) = xcm::PRECOMPILE_ADDRESS.into();
			precompile.send(destination, message)
		}
	}

	impl XcmCallback for Messaging {
		#[ink(message)]
		fn newQuery(
			&self,
			responder: DynBytes,
			timeout: BlockNumber,
			callback: Callback,
		) -> (MessageId, QueryId) {
			let precompile: contract_ref!(XcmCallback, Pop, Sol) = xcm::PRECOMPILE_ADDRESS.into();
			precompile.newQuery(responder, timeout, callback)
		}
	}

	impl OnQueryResponse for Messaging {
		#[ink(message)]
		fn onQueryResponse(&self, id: MessageId, response: DynBytes) {
			// Adding state requires storage deposit limit to be defined on callback. Deposit is
			// moved from caller to contract and placed on hold. Deposit is claimed by anyone that
			// removes state, so adequate controls should be implemented by contract as desired.
			self.responses.insert(id, &Response::XcmQuery(response.0.clone()));
			self.env().emit_event(XcmCompleted { id, result: response });
		}
	}

	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
	enum Response {
		IsmpGet(Vec<StorageValue>),
		IsmpPost(Vec<u8>),
		XcmQuery(Vec<u8>),
	}
}

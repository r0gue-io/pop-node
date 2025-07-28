#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{abi::Sol, contract_ref, prelude::vec::Vec, U256};
use pop_api::{
	messaging::{
		self as api,
		ismp::{
			self, Get, Ismp, IsmpCallback, IsmpGetCompleted, IsmpPostCompleted, OnGetResponse,
			OnPostResponse, Post, StorageValue,
		},
		xcm::{self, OnQueryResponse, QueryId, Xcm, XcmCallback, XcmCompleted},
		Bytes, Callback, Error, MessageId, MessageStatus, Weight,
	},
	Pop,
};

#[ink::contract]
pub mod messaging {
	use super::*;

	#[ink(storage)]
	pub struct Messaging;

	impl Messaging {
		#[ink(constructor, default, payable)]
		#[allow(clippy::new_without_default)]
		pub fn new() -> Self {
			Self {}
		}
	}

	impl api::Messaging for Messaging {
		#[ink(message)]
		fn getResponse(&self, message: MessageId) -> Bytes {
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
			api::remove(message)
		}

		#[ink(message)]
		fn removeMany(&self, messages: Vec<MessageId>) -> Result<(), Error> {
			api::remove_many(messages)
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
		fn onGetResponse(&mut self, id: MessageId, response: Vec<StorageValue>) {
			self.env().emit_event(IsmpGetCompleted { id, response });
		}
	}

	impl OnPostResponse for Messaging {
		#[ink(message)]
		fn onPostResponse(&mut self, id: MessageId, response: Bytes) {
			self.env().emit_event(IsmpPostCompleted { id, response });
		}
	}

	impl Xcm for Messaging {
		#[ink(message)]
		fn execute(&self, message: Bytes, weight: Weight) -> Bytes {
			let precompile: contract_ref!(Xcm, Pop, Sol) = xcm::PRECOMPILE_ADDRESS.into();
			precompile.execute(message, weight)
		}

		#[ink(message)]
		fn newQuery(&self, responder: Bytes, timeout: BlockNumber) -> (MessageId, QueryId) {
			let precompile: contract_ref!(Xcm, Pop, Sol) = xcm::PRECOMPILE_ADDRESS.into();
			precompile.newQuery(responder, timeout)
		}

		#[ink(message)]
		fn send(&self, destination: Bytes, message: Bytes) -> Bytes {
			let precompile: contract_ref!(Xcm, Pop, Sol) = xcm::PRECOMPILE_ADDRESS.into();
			precompile.send(destination, message)
		}
	}

	impl XcmCallback for Messaging {
		#[ink(message)]
		fn newQuery(
			&self,
			responder: Bytes,
			timeout: BlockNumber,
			callback: Callback,
		) -> (MessageId, QueryId) {
			let precompile: contract_ref!(XcmCallback, Pop, Sol) = xcm::PRECOMPILE_ADDRESS.into();
			precompile.newQuery(responder, timeout, callback)
		}
	}

	impl OnQueryResponse for Messaging {
		#[ink(message)]
		fn onQueryResponse(&mut self, id: MessageId, response: Bytes) {
			self.env().emit_event(XcmCompleted { id, result: response });
		}
	}
}

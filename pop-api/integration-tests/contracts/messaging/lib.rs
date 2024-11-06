#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{env::debug_println, prelude::vec::Vec};
use pop_api::{
	messaging::{
		self as api,
		ismp::{Get, Post},
		xcm::{Location, QueryId, Response},
		Callback, MessageId, Status,
	},
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod messaging {
	use ink::xcm::prelude::Weight;
	use pop_api::messaging::ismp::StorageValue;

	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Contract;

	impl Contract {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			Default::default()
		}

		#[ink(message)]
		pub fn ismp_get(
			&mut self,
			id: MessageId,
			request: Get,
			fee: Balance,
			callback: bool,
		) -> Result<()> {
			debug_println!(
				"messaging::ismp_get id={id}, dest={}, height={}, timeout={}, fee={fee}, \
				 callback={callback}",
				request.dest,
				request.height,
				request.timeout
			);
			api::ismp::get(
				id,
				request,
				fee,
				callback.then_some(
					// See `api::ismp::OnGetResponse` impl below
					Callback::to(0x57ad942b, Weight::from_parts(600_000_000, 150_000)),
				),
			)?;
			Ok(())
		}

		#[ink(message)]
		pub fn ismp_post(
			&mut self,
			id: MessageId,
			request: Post,
			fee: Balance,
			callback: bool,
		) -> Result<()> {
			debug_println!(
				"messaging::ismp_post id={id}, dest={}, timeout={}, fee={fee}, callback={callback}",
				request.dest,
				request.timeout
			);
			api::ismp::post(
				id,
				request,
				fee,
				callback.then_some(
					// See `api::ismp::OnPostResponse` impl below
					Callback::to(0xcfb0a1d2, Weight::from_parts(600_000_000, 150_000)),
				),
			)?;
			Ok(())
		}

		#[ink(message)]
		pub fn xcm_new_query(
			&mut self,
			id: MessageId,
			responder: Location,
			timeout: BlockNumber,
			callback: bool,
		) -> Result<Option<QueryId>> {
			debug_println!(
				"messaging::xcm_new_query id={id}, responder={responder:?}, timeout={timeout}, \
				 callback={callback}"
			);
			api::xcm::new_query(
				id,
				responder,
				timeout,
				callback.then_some(
					// See api::xcm::OnResponse impl below
					Callback::to(0x641b0b03, Weight::from_parts(600_000_000, 200_000)),
				),
			)
		}

		#[ink(message)]
		pub fn poll(&self, id: MessageId) -> Result<Option<Status>> {
			debug_println!("messaging::poll id={id}");
			api::poll((self.env().account_id(), id))
		}

		#[ink(message)]
		pub fn get(&self, id: MessageId) -> Result<Option<Vec<u8>>> {
			debug_println!("messaging::get id={id}");
			api::get((self.env().account_id(), id))
		}

		#[ink(message)]
		pub fn remove(&mut self, id: MessageId) -> Result<()> {
			debug_println!("messaging::remove id={id}");
			api::remove([id].to_vec())?;
			Ok(())
		}
	}

	impl api::ismp::OnGetResponse for Contract {
		#[ink(message)]
		fn on_response(&mut self, id: MessageId, values: Vec<StorageValue>) -> Result<()> {
			debug_println!("messaging::ismp::get::on_response id={id}, values={values:?});");
			self.env().emit_event(IsmpGetCompleted { id, values });
			Ok(())
		}
	}

	impl api::ismp::OnPostResponse for Contract {
		#[ink(message)]
		fn on_response(&mut self, id: MessageId, response: Vec<u8>) -> Result<()> {
			debug_println!("messaging::ismp::post::on_response id={id}, response={response:?});");
			self.env().emit_event(IsmpPostCompleted { id, response });
			Ok(())
		}
	}

	impl api::xcm::OnResponse for Contract {
		#[ink(message)]
		fn on_response(&mut self, id: MessageId, response: Response) -> Result<()> {
			debug_println!("messaging::xcm::on_response id={id}, response={response:?}");
			match response {
				Response::Null => {},
				Response::Assets(_) => {},
				Response::ExecutionResult(_) => {},
				Response::Version(_) => {},
				Response::PalletsInfo(_) => {},
				Response::DispatchResult(_) => {},
			}
			self.env().emit_event(XcmCompleted { id, result: response });
			Ok(())
		}
	}

	#[ink::event]
	pub struct IsmpGetCompleted {
		#[ink(topic)]
		pub id: MessageId,
		pub values: Vec<StorageValue>,
	}

	#[ink::event]
	pub struct IsmpPostCompleted {
		#[ink(topic)]
		pub id: MessageId,
		pub response: Vec<u8>,
	}

	#[ink::event]
	pub struct XcmCompleted {
		#[ink(topic)]
		pub id: MessageId,
		pub result: Response,
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			Contract::new();
		}
	}
}

#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
	env::hash::{Blake2x256, CryptoHash},
	prelude::vec::Vec,
	scale::{Compact, Encode},
	xcm::{
		prelude::{All, Asset, Junction::Parachain, Location, Weight, WeightLimit, Xcm},
		VersionedXcm,
	},
};
use pop_api::{
	cross_chain::{self as api, ismp, Request, RequestId, Status},
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod cross_chain {
	use ink::xcm::{
		prelude::{AccountId32, OriginKind, QueryId, QueryResponseInfo, XcmHash},
		DoubleEncoded,
	};

	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct CrossChain {
		para: u32,
		id: RequestId,
	}

	impl CrossChain {
		#[ink(constructor, payable)]
		pub fn new(para: u32) -> Self {
			Self { para, id: 0 }
		}

		#[ink(message)]
		pub fn get(&mut self, key: Vec<u8>, height: u32) -> Result<()> {
			self.id = self.id.saturating_add(1);
			api::request(Request::Ismp {
				id: self.id,
				request: ismp::Request::Get {
					para: self.para,
					height,
					timeout: 0,
					context: Vec::default(),
					keys: Vec::from([key.clone()]),
				},
				fee: 0,
			})?;
			self.env().emit_event(IsmpRequested { id: self.id, key, height });
			Ok(())
		}

		#[ink(message, payable)]
		pub fn fund(&mut self) -> Result<()> {
			let dest = Location::new(1, Parachain(self.para));

			// Reserve transfer specified assets to contract account on destination.
			let asset: Asset = (Location::parent(), self.env().transferred_value()).into();
			let beneficiary = hashed_account(4_001, self.env().account_id()); // todo: para id getter
			let message: Xcm<()> = Xcm::builder_unsafe()
				.withdraw_asset(asset.clone().into())
				.initiate_reserve_withdraw(
					asset.clone().into(),
					dest.clone(),
					Xcm::builder_unsafe()
						.buy_execution(asset.clone(), WeightLimit::Unlimited)
						.deposit_asset(
							All.into(),
							Location::new(0, AccountId32 { network: None, id: beneficiary.0 }),
						)
						.build(),
				)
				.build();
			self.env().xcm_execute(&VersionedXcm::V4(message)).unwrap(); // todo: handle error
			self.env().emit_event(Funded {
				account_id: beneficiary,
				value: self.env().transferred_value(),
			});
			Ok(())
		}

		#[ink(message, payable)]
		pub fn transact(&mut self, call: DoubleEncoded<()>, weight: Weight) -> Result<()> {
			// Request query id, used to report transact status.
			self.id = self.id.saturating_add(1);
			let dest = Location::new(1, Parachain(self.para));
			// TODO: merge query into local implementation of api::request as an additional
			// instruction to make this a oneliner
			api::request(Request::Xcm {
				id: self.id,
				responder: dest.clone().into_versioned(),
				timeout: self.env().block_number().saturating_add(100),
			})?;
			let query_id = api::query_id((self.env().account_id(), self.id))?.unwrap();

			let query_response = QueryResponseInfo {
				// Route back to this parachain.
				destination: Location::new(1, Parachain(4_001)),
				query_id,
				// TODO: provide an api function for determining this value for processing the
				// reported transact status on the local chain
				max_weight: Weight::from_parts(1_000_000, 5_000),
			};

			// Send transact message.
			let asset: Asset = (Location::parent(), self.env().transferred_value()).into();
			let beneficiary = hashed_account(4_001, self.env().account_id()); // todo: para id getter
			let message: Xcm<()> = Xcm::builder_unsafe()
				.withdraw_asset(asset.clone().into())
				.buy_execution(asset, WeightLimit::Unlimited)
				.set_appendix(
					Xcm::builder_unsafe()
						.refund_surplus()
						.deposit_asset(
							All.into(),
							Location::new(0, AccountId32 { network: None, id: beneficiary.0 }),
						)
						.build(),
				)
				.set_error_handler(
					Xcm::builder_unsafe().report_error(query_response.clone()).build(),
				)
				.transact(OriginKind::SovereignAccount, weight, call)
				.report_transact_status(query_response)
				.build();
			let hash =
				self.env().xcm_send(&dest.into_versioned(), &VersionedXcm::V4(message)).unwrap(); // todo: handle error
			self.env().emit_event(XcmRequested { id: self.id, query_id, hash });
			Ok(())
		}

		#[ink(message)]
		pub fn complete(&mut self, request: RequestId) -> Result<()> {
			if let Ok(Some(status)) = api::poll((self.env().account_id(), request)) {
				if status == Status::Complete {
					let result = api::get((self.env().account_id(), request))?;
					api::remove([request].to_vec())?;
					self.env().emit_event(Completed { id: request, result });
				}
			}
			Ok(())
		}
	}

	#[ink::event]
	pub struct IsmpRequested {
		#[ink(topic)]
		pub id: RequestId,
		pub key: Vec<u8>,
		pub height: BlockNumber,
	}

	#[ink::event]
	pub struct Funded {
		#[ink(topic)]
		pub account_id: AccountId,
		pub value: Balance,
	}

	#[ink::event]
	pub struct XcmRequested {
		#[ink(topic)]
		pub id: RequestId,
		#[ink(topic)]
		pub query_id: QueryId,
		#[ink(topic)]
		pub hash: XcmHash,
	}

	#[ink::event]
	pub struct Completed {
		#[ink(topic)]
		pub id: RequestId,
		pub result: Option<Vec<u8>>,
	}

	// todo: make hasher generic and move to pop-api
	fn hashed_account(para_id: u32, account_id: AccountId) -> AccountId {
		let location = (
			b"SiblingChain",
			Compact::<u32>::from(para_id),
			(b"AccountId32", account_id.0).encode(),
		)
			.encode();
		let mut output = [0u8; 32];
		Blake2x256::hash(&location, &mut output);
		AccountId::from(output)
	}

	#[cfg(test)]
	mod tests {

		use super::*;

		#[ink::test]
		fn default_works() {
			CrossChain::new(1_000);
		}

		#[test]
		fn it_works() {
			let account_id: [u8; 32] = [
				27, 2, 24, 17, 104, 5, 173, 98, 25, 32, 36, 0, 82, 159, 11, 212, 178, 11, 39, 219,
				14, 178, 226, 179, 216, 62, 19, 85, 226, 17, 80, 179,
			];
			let location = (
				b"SiblingChain",
				Compact::<u32>::from(4001),
				(b"AccountId32", account_id).encode(),
			)
				.encode();
			let mut output = [0u8; 32];
			Blake2x256::hash(&location, &mut output);
			println!("{output:?}")
		}
	}
}

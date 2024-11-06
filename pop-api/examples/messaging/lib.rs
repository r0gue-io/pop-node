#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
	env::hash::{Blake2x256, CryptoHash},
	prelude::vec::Vec,
	scale::{Compact, Encode},
	xcm::{
		prelude::{
			AccountId32, All, Asset, Junction::Parachain, Location, OriginKind, QueryId,
			QueryResponseInfo, Weight, WeightLimit, Xcm, XcmHash,
		},
		DoubleEncoded, VersionedXcm,
	},
};
use pop_api::{
	incentives,
	messaging::{self as api, ismp, ismp::Get, xcm::Response, MessageId, Status},
	StatusCode,
};

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod messaging {
	use pop_api::messaging::Callback;

	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Messaging {
		para: u32,
		id: MessageId,
	}

	impl Messaging {
		#[ink(constructor, payable)]
		pub fn new(para: u32) -> Result<Self> {
			let instance = Self { para, id: 0 };
			incentives::register(instance.env().account_id())?;
			Ok(instance)
		}

		#[ink(message)]
		pub fn get(&mut self, key: Vec<u8>, height: u32) -> Result<()> {
			self.id = self.id.saturating_add(1);
			ismp::get(
				self.id,
				Get::new(self.para, height, 0, Vec::default(), Vec::from([key.clone()])),
				0,
				None,
			)?;
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
			api::xcm::execute(&VersionedXcm::V4(message)).unwrap(); // todo: handle error

			self.env().emit_event(Funded {
				account_id: beneficiary,
				value: self.env().transferred_value(),
			});
			Ok(())
		}

		#[ink(message, payable)]
		pub fn transact(&mut self, call: DoubleEncoded<()>, weight: Weight) -> Result<()> {
			let dest = Location::new(1, Parachain(self.para));

			// Register a new query for receiving a response, used to report transact status.
			self.id = self.id.saturating_add(1);
			let query_id = api::xcm::new_query(
				self.id,
				dest.clone(),
				self.env().block_number().saturating_add(100),
				// callback
				Some(Callback::to(0x641b0b03, Weight::from_parts(800_000_000, 500_000))),
			)?
			.unwrap(); // TODO: handle error

			// TODO: provide an api function for determining the local para id and max weight value
			// for processing the reported transact status on the local chain.
			let response = QueryResponseInfo {
				// Route back to this parachain.
				destination: Location::new(1, Parachain(4_001)),
				query_id,
				max_weight: Weight::from_parts(1_000_000, 5_000),
			};

			// Send transact message.
			let fees: Asset = (Location::parent(), self.env().transferred_value()).into();
			let message: Xcm<()> = self._transact(call, weight, fees, response);
			let hash = api::xcm::send(&dest.into_versioned(), &VersionedXcm::V4(message)).unwrap(); // todo: handle error

			self.env().emit_event(XcmRequested { id: self.id, query_id, hash });
			Ok(())
		}

		#[ink(message)]
		pub fn complete(&mut self, id: MessageId) -> Result<()> {
			if let Ok(Some(status)) = api::poll((self.env().account_id(), id)) {
				if status == Status::Complete {
					let result = api::get((self.env().account_id(), id))?;
					api::remove([id].to_vec())?;
					self.env().emit_event(Completed { id, result });
				}
			}
			Ok(())
		}

		fn _transact(
			&self,
			call: DoubleEncoded<()>,
			weight: Weight,
			fees: Asset,
			response: QueryResponseInfo,
		) -> Xcm<()> {
			let beneficiary = hashed_account(4_001, self.env().account_id()); // todo: para id getter
			Xcm::builder_unsafe()
				.withdraw_asset(fees.clone().into())
				.buy_execution(fees, WeightLimit::Unlimited)
				.set_appendix(
					Xcm::builder_unsafe()
						.refund_surplus()
						.deposit_asset(
							All.into(),
							Location::new(0, AccountId32 { network: None, id: beneficiary.0 }),
						)
						.build(),
				)
				.set_error_handler(Xcm::builder_unsafe().report_error(response.clone()).build())
				.transact(OriginKind::SovereignAccount, weight, call)
				.report_transact_status(response)
				.build()
		}
	}

	impl api::xcm::OnResponse for Messaging {
		#[ink(message)]
		fn on_response(&mut self, id: MessageId, response: Response) -> Result<()> {
			// todo: ensure caller is self (runtime)
			self.env().emit_event(XcmCompleted { id, result: response });
			Ok(())
		}
	}

	#[ink::event]
	pub struct IsmpRequested {
		#[ink(topic)]
		pub id: MessageId,
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
		pub id: MessageId,
		#[ink(topic)]
		pub query_id: QueryId,
		#[ink(topic)]
		pub hash: XcmHash,
	}

	#[ink::event]
	pub struct Completed {
		#[ink(topic)]
		pub id: MessageId,
		pub result: Option<Vec<u8>>,
	}

	#[ink::event]
	pub struct XcmCompleted {
		#[ink(topic)]
		pub id: MessageId,
		pub result: Response,
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
			Messaging::new(1_000);
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

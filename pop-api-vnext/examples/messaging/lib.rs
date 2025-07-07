#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
	env::hash::{Blake2x256, CryptoHash},
	prelude::vec::Vec,
	scale::{DecodeAll, Encode},
	xcm::{
		prelude::{
			AccountId32, All, Asset, Junction::Parachain, Location, OriginKind, QueryId,
			QueryResponseInfo, WeightLimit, Xcm, XcmHash,
		},
		DoubleEncoded, VersionedXcm,
	},
	U256,
};
use pop_api::{
	messaging::{
		self as api, hashed_account, ismp,
		ismp::{Get, StorageValue},
		xcm::VersionedLocation,
		Bytes, Callback, Encoding, MessageId,
		MessageStatus::Complete,
		Weight,
	},
	revert,
};

#[ink::contract]
mod messaging {

	use super::*;

	// TODO: use manifest metadata to determine
	const ENCODING: Encoding = Encoding::SolidityAbi;
	const UNAUTHORIZED: u32 = u32::MAX;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Messaging {
		para: u32,
		id: MessageId,
	}

	impl Messaging {
		#[ink(constructor, payable)]
		pub fn new(para: u32) -> Self {
			Self { para, id: 0 }
		}

		#[ink(message)]
		pub fn get(&mut self, key: Vec<u8>, height: u32) {
			// self.id = self.id.saturating_add(1);
			let id = ismp::get(
				Get::new(self.para, height, 0, Vec::default(), Vec::from([key.clone()])),
				U256::zero(),
				Some(Callback::new(
					self.env().address(),
					ENCODING,
					0x57ad942b,
					Weight::from_parts(800_000_000, 500_000),
				)),
			);
			self.env().emit_event(IsmpRequested { id: self.id, key, height });
		}

		#[ink(message, payable)]
		pub fn fund(&mut self) {
			let dest = Location::new(1, Parachain(self.para));

			// Reserve transfer specified assets to contract account on destination.
			let asset: Asset =
				(Location::parent(), u128::try_from(self.env().transferred_value()).unwrap())
					.into();
			let beneficiary = hashed_account(4_001, self.env().account_id()); // todo: para id getter
			let message: Xcm<()> = Xcm::builder_unsafe()
				.withdraw_asset(asset.clone())
				.initiate_reserve_withdraw(
					asset.clone(),
					dest.clone(),
					Xcm::builder_unsafe()
						.buy_execution(asset.clone(), WeightLimit::Unlimited)
						.deposit_asset(
							All,
							Location::new(0, AccountId32 { network: None, id: beneficiary.0 }),
						)
						.build(),
				)
				.build();
			let result = api::xcm::execute(VersionedXcm::V5(message), Weight::MAX);
			// TODO: check result

			self.env().emit_event(Funded {
				account_id: beneficiary,
				value: self.env().transferred_value(),
			});
		}

		#[ink(message, payable)]
		pub fn transact(&mut self, call: Vec<u8>, weight: Weight) {
			let dest = Location::new(1, Parachain(self.para));
			let call = DoubleEncoded::<()>::decode_all(&mut &call[..]).unwrap();

			// Register a new query for receiving a response, used to report transact status.
			self.id = self.id.saturating_add(1);
			let query_id = api::xcm::new_query(
				VersionedLocation::V5(dest.clone()),
				self.env().block_number().saturating_add(100),
				Some(Callback::new(
					self.env().address(),
					ENCODING,
					0x641b0b03,
					Weight::from_parts(800_000_000, 500_000),
				)),
			);

			// TODO: provide an api function for determining the local para id and max weight value
			// for processing the reported transact status on the local chain.
			let response = QueryResponseInfo {
				// Route back to this parachain.
				destination: Location::new(1, Parachain(4_001)),
				query_id,
				max_weight: Weight::from_parts(1_000_000, 5_000),
			};

			// Send transact message.
			let fees: Asset =
				(Location::parent(), u128::try_from(self.env().transferred_value()).unwrap())
					.into();
			let message: Xcm<()> = self._transact(call, weight, fees, response);
			let mut hash = [0u8; 32];
			Blake2x256::hash(&message.encode(), &mut hash);
			let result = api::xcm::send(dest.into_versioned(), VersionedXcm::V5(message));

			self.env().emit_event(XcmRequested { id: self.id, query_id, hash });
		}

		#[ink(message)]
		pub fn complete(&mut self, id: MessageId) {
			if api::poll_status(id) == Complete {
				let result = api::get_response(id);
				api::remove(id);
				self.env().emit_event(Completed { id, result });
			}
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
				.withdraw_asset(fees.clone())
				.buy_execution(fees, WeightLimit::Unlimited)
				.set_appendix(
					Xcm::builder_unsafe()
						.refund_surplus()
						.deposit_asset(
							All,
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

	impl api::ismp::OnGetResponse for Messaging {
		#[ink(message)]
		fn on_response(&mut self, id: MessageId, values: Vec<StorageValue>) {
			if self.env().caller() != self.env().address() {
				revert(&UNAUTHORIZED);
			}
			self.env().emit_event(GetCompleted { id, values });
		}
	}

	impl api::xcm::OnResponse for Messaging {
		#[ink(message)]
		fn on_response(&mut self, id: MessageId, response: Vec<u8>) {
			if self.env().caller() != self.env().address() {
				revert(&UNAUTHORIZED);
			}
			self.env().emit_event(XcmCompleted { id, result: response });
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
		pub value: U256,
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
		pub result: Vec<u8>,
	}

	#[ink::event]
	pub struct XcmCompleted {
		#[ink(topic)]
		pub id: MessageId,
		pub result: Bytes,
	}

	#[ink::event]
	pub struct GetCompleted {
		#[ink(topic)]
		pub id: MessageId,
		pub values: Vec<StorageValue>,
	}
}

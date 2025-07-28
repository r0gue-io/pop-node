#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
	env::hash::{Blake2x256, CryptoHash},
	prelude::vec::Vec,
	scale::{DecodeAll, Encode},
	xcm::{
		prelude::{
			AccountId32, All, Asset, OriginKind, Parachain, QueryId, QueryResponseInfo,
			WeightLimit, Xcm, XcmHash,
		},
		DoubleEncoded, VersionedXcm,
	},
	SolBytes, U256,
};
use pop_api::{
	ensure,
	messaging::{
		self as api, hashed_account,
		ismp::{self, Get, OnGetResponse, StorageValue},
		xcm::{self, Location, OnQueryResponse},
		Bytes, Callback, Encoding, MessageId,
		MessageStatus::Complete,
		Weight,
	},
	revert,
};

// NOTE: requires `cargo-contract` built from `master`

#[ink::contract]
mod messaging {

	use self::Error::*;
	use super::*;

	// TODO: use manifest metadata to determine
	const ENCODING: Encoding = Encoding::SolidityAbi;
	const UNAUTHORIZED: u32 = u32::MAX;

	/// A contract for interacting with chains.
	#[ink(storage)]
	pub struct Messaging {
		/// The owner of the contract.
		owner: Address,
		/// The weight to be used for callback responses to this contract.
		weight: Weight,
	}

	impl Messaging {
		/// Instantiate the contract.
		#[ink(constructor, payable)]
		#[allow(clippy::new_without_default)]
		pub fn new() -> Self {
			Self { owner: Self::env().caller(), weight: Weight::from_parts(800_000_000, 500_000) }
		}

		/// Request some state from a chain at the specified keys and block height.
		///
		/// # Parameters
		/// - `dest` - The identifier of the destination chain.
		/// - `keys` - The storage key(s) used to query state from the remote parachain.
		/// - `height` - The block height at which to read the state.
		#[ink(message)]
		pub fn get(&mut self, dest: u32, keys: Vec<Bytes>, height: u64) -> Result<(), ismp::Error> {
			let id = ismp::get(
				Get::new(dest, height, 0, SolBytes(Vec::default()), keys.clone()),
				U256::zero(),
				Some(Callback::new(self.env().address(), ENCODING, 0x9bf78ffb, self.weight)),
			)?;
			self.env().emit_event(IsmpRequested { id, keys, height });
			Ok(())
		}

		/// Funds the contract's account on the destination chain.
		///
		/// # Parameters
		/// - `dest` - The identifier of the destination chain.
		#[ink(message, payable)]
		pub fn fund(&mut self, dest: u32) -> Result<(), Error> {
			let dest = Location::new(1, Parachain(dest));

			// Reserve transfer specified assets to contract account on destination.
			let amount = u128::try_from(self.env().transferred_value()).map_err(|_| Overflow)?;
			let asset: Asset = (Location::parent(), amount).into();
			let beneficiary = hashed_account(api::id(), self.env().account_id());
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
			let result = xcm::execute(VersionedXcm::V5(message), Weight::MAX);
			// TODO: check result

			self.env().emit_event(Funded {
				account_id: beneficiary,
				value: self.env().transferred_value(),
			});
			Ok(())
		}

		#[ink(message, payable)]
		pub fn transact(&mut self, dest: u32, call: Vec<u8>, weight: Weight) -> Result<(), Error> {
			let dest = Location::new(1, Parachain(dest));
			let call =
				DoubleEncoded::<()>::decode_all(&mut &call[..]).map_err(|_| DecodingFailed)?;

			// Register a new query for receiving a response, used to report transact status.
			let (id, query_id) = xcm::new_query(
				dest.clone(),
				self.env().block_number().saturating_add(100),
				Some(Callback::new(self.env().address(), ENCODING, 0x97dbf9fb, self.weight)),
			);

			// TODO: provide an api function for determining the max weight value for processing the
			// reported transact status on the local chain.
			let response = QueryResponseInfo {
				// Route back to this parachain.
				destination: Location::new(1, Parachain(api::id())),
				query_id,
				max_weight: Weight::from_parts(1_000_000, 5_000),
			};

			// Send transact message.
			let amount = u128::try_from(self.env().transferred_value()).map_err(|_| Overflow)?;
			let fees: Asset = (Location::parent(), amount).into();
			let message: Xcm<()> = self._transact(call, weight, fees, response);
			let mut hash = [0u8; 32];
			Blake2x256::hash(&message.encode(), &mut hash);
			let result = api::xcm::send(dest.into_versioned(), VersionedXcm::V5(message));

			self.env().emit_event(XcmRequested { id, query_id, hash });
			Ok(())
		}

		#[ink(message)]
		pub fn complete(&mut self, id: MessageId) -> Result<(), api::Error> {
			if api::poll_status(id) == Complete {
				let result = api::get_response(id);
				api::remove(id)?;
				self.env().emit_event(Completed { id, result });
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
			let beneficiary = hashed_account(api::id(), self.env().account_id());
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

		/// Transfer the ownership of the contract to another account.
		///
		/// # Parameters
		/// - `owner` - New owner account.
		///
		/// NOTE: the specified owner account is not checked, allowing the zero address to be
		/// specified if desired..
		#[ink(message)]
		pub fn transfer_ownership(&mut self, owner: Address) -> Result<(), Error> {
			ensure!(self.env().caller() == self.owner, NoPermission);
			self.owner = owner;
			Ok(())
		}

		/// Sets the weight used for callback responses.
		///
		/// # Parameters
		/// - `owner` - New owner account.
		///
		/// NOTE: the specified owner account is not checked, allowing the zero address to be
		/// specified if desired..
		#[ink(message)]
		pub fn set_weight(&mut self, weight: Weight) -> Result<(), Error> {
			ensure!(self.env().caller() == self.owner, NoPermission);
			self.weight = weight;
			Ok(())
		}
	}

	impl OnGetResponse for Messaging {
		#[ink(message)]
		fn onGetResponse(&mut self, id: MessageId, values: Vec<StorageValue>) {
			if self.env().caller() != self.env().address() {
				revert(&UNAUTHORIZED);
			}
			self.env().emit_event(GetCompleted { id, values });
		}
	}

	impl OnQueryResponse for Messaging {
		#[ink(message)]
		fn onQueryResponse(&mut self, id: MessageId, response: Bytes) {
			if self.env().caller() != self.env().address() {
				revert(&UNAUTHORIZED);
			}
			self.env().emit_event(XcmCompleted { id, result: response });
		}
	}

	#[cfg_attr(feature = "std", derive(Debug, PartialEq))]
	#[derive(ink::SolErrorDecode, ink::SolErrorEncode)]
	#[ink::scale_derive(Decode, Encode, TypeInfo)]
	pub enum Error {
		DecodingFailed,
		NoPermission,
		Overflow,
	}

	#[ink::event]
	pub struct IsmpRequested {
		#[ink(topic)]
		pub id: MessageId,
		pub keys: Vec<Bytes>,
		pub height: u64,
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
		pub result: Bytes,
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

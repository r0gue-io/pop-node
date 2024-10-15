#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

use ink::{prelude::vec, xcm::prelude::*};
use pop_api::primitives::AccountId;

#[ink::contract]
pub mod transfer_contract {
	use super::*;
	/// No storage is needed for this simple contract.
	#[ink(storage)]
	pub struct TransferContract {}

	#[derive(Debug, PartialEq, Eq)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub enum Error {
		TransferFailed,
		AHTransferFailed,
		AHApiTransferFailed,
	}

	impl TransferContract {
		/// Creates a new instance of this contract.
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			Self {}
		}

		/// Transfers `value` amount of tokens to the specified `account`.
		///
		/// # Arguments
		///
		/// * `account` - The account to which the funds should be transferred.
		/// * `value` - The amount of tokens to transfer.
		///
		/// # Errors
		///
		/// - Panics if the contract does not have sufficient balance.
		/// - Panics if the transfer fails.
		#[ink(message)]
		pub fn transfer(&mut self, account: AccountId, value: Balance) -> Result<(), Error> {
			self.env().transfer(account, value).map_err(|_| Error::TransferFailed)
		}

		#[ink(message)]
		pub fn ah_transfer(
			&mut self,
			to: AccountId,
			value: Balance,
			fee: Balance,
		) -> Result<(), Error> {
			let asset_hub_para_id: u32 = 1000;
			let destination = Location::new(1, Parachain(asset_hub_para_id));
			let beneficiary = Location::new(0, AccountId32 { network: None, id: to.0.into() });

			// Define the assets
			let asset: Asset = (Location::parent(), value).into();
			let fee_asset: Asset = (Location::parent(), fee).into();

			// XCM instructions to be executed on AssetHub
			let xcm_on_destination = Xcm(vec![
				BuyExecution { fees: fee_asset.clone(), weight_limit: WeightLimit::Unlimited },
				DepositAsset { assets: Wild(All.into()), beneficiary: beneficiary.clone() },
			]);

			// Construct the full XCM message
			let message: Xcm<()> = Xcm(vec![
				// Withdraw the total amount (value + fee) from the contract's account
				WithdrawAsset((vec![asset.clone(), fee_asset.clone()]).into()),
				// Initiate the reserve-based transfer
				InitiateReserveWithdraw {
					assets: vec![asset.clone()].into(),
					reserve: destination.clone(),
					xcm: xcm_on_destination,
				},
			]);

			let hash = self
				.env()
				.xcm_execute(&VersionedXcm::V4(message))
				.map_err(|_| Error::AHTransferFailed)?;

			Ok(())
		}

		#[ink(message)]
		pub fn api_ah_transfer(
			&mut self,
			to: AccountId,
			value: Balance,
			fee: Balance,
		) -> Result<(), Error> {
			pop_api::v0::xc::asset_hub_transfer(to, value, fee)
				.map_err(|_| Error::AHApiTransferFailed)
		}
	}
}

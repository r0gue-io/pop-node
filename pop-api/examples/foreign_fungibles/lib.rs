#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{prelude::vec, xcm::prelude::*};
use pop_api::{
	primitives::{AccountId, TokenId},
	v0::fungibles as api,
};

#[cfg(test)]
mod tests;

#[ink::contract]
mod fungibles {
	use ink::env::Error as EnvError;

	use super::*;

	#[ink(storage)]
	pub struct Fungible {
		id: TokenId,
	}

	#[derive(Debug, PartialEq, Eq)]
	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub enum Error {
		TransferFailed,
		AHTransferFailed,
	}

	// Constants for USDC and Asset Hub
	const ASSET_HUB_PARA_ID: u32 = 1000;
	const ASSET_HUB_ASSETS_PALLET: u8 = 50;
	const USDC_ASSET_ID_ON_AH: u128 = 1337;
	const USDC_ASSET_ID_ON_POP: TokenId = 1;

	impl Fungible {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			Self { id: USDC_ASSET_ID_ON_POP }
		}

		#[ink(message)]
		pub fn ah_transfer(
			&mut self,
			to: AccountId,
			value: Balance,
			fee: Balance,
		) -> Result<(), Error> {
			// Define the destination (Asset Hub)
			let destination = Location::new(1, [Parachain(ASSET_HUB_PARA_ID)]);

			// Define the beneficiary (the 'to' account on Asset Hub)
			let beneficiary = Location::new(0, [AccountId32 { network: None, id: to.0.into() }]);

			// Define USDC asset location as seen from Pop Network (foreign asset from Asset Hub)
			let usdc_location_on_pop = Location::new(
				1,
				[
					Parachain(ASSET_HUB_PARA_ID),
					PalletInstance(ASSET_HUB_ASSETS_PALLET),
					GeneralIndex(USDC_ASSET_ID_ON_AH),
				],
			);
			let usdc_asset_on_pop: Asset = (AssetId(usdc_location_on_pop.clone()), value).into();

			// Define USDC asset location as seen from Asset Hub (local asset)
			let usdc_location_on_ah = Location::new(
				0,
				[PalletInstance(ASSET_HUB_ASSETS_PALLET), GeneralIndex(USDC_ASSET_ID_ON_AH)],
			);
			let usdc_asset_on_ah: Asset = (AssetId(usdc_location_on_ah.clone()), value).into();

			let fee_asset: Asset = (AssetId(Location::parent()), fee).into();

			// XCM instructions to be executed on Asset Hub
			let xcm_on_destination = Xcm(vec![
				// Buy execution with the DOT
				BuyExecution { fees: fee_asset.clone(), weight_limit: WeightLimit::Unlimited },
				// Deposit USDC to the beneficiary
				DepositAsset { assets: Wild(All.into()), beneficiary: beneficiary.clone() },
			]);

			// Construct the full XCM message from Pop Network
			let message: Xcm<()> = Xcm(vec![
				// Withdraw the USDC and DOT.
				WithdrawAsset((vec![usdc_asset_on_pop.clone(), fee_asset.clone()]).into()),
				// Initiate the reserve-based transfer to Asset Hub
				InitiateReserveWithdraw {
					assets: vec![usdc_asset_on_pop.clone(), fee_asset].into(),
					reserve: destination.clone(),
					xcm: xcm_on_destination,
				},
			]);

			// Execute the XCM message
			self.env()
				.xcm_execute(&VersionedXcm::V4(message))
				.map_err(|_| Error::AHTransferFailed)?;

			Ok(())
		}

		#[ink(message)]
		pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<(), Error> {
			api::transfer(self.id, to, value).map_err(|_| Error::TransferFailed)?;
			Ok(())
		}
	}
}

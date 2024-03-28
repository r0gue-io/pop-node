/// As Pop Network uses the relay chain token as native, the dev accounts are not funded by default.
/// Therefore, after network launch there needs to be a reserve transfer from the relay chain
/// to the dev accounts.
///
/// This script performs these reserve transfers to fund the dev accounts from the relay chain.
///
use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::{dev, Keypair};

use std::time::Duration;

#[cfg(feature = "paseo")]
mod paseo_interface;
#[cfg(not(feature = "paseo"))]
mod rococo_interface;

mod pop_interface;

const PARA_ID: u32 = 4385;

#[cfg(not(feature = "paseo"))]
mod relay {
	use super::*;
	pub(crate) use crate::rococo_interface::api as runtime;
	pub(crate) type RuntimeCall = runtime::runtime_types::rococo_runtime::RuntimeCall;
	pub(crate) const UNIT: u128 = 1_000_000_000_000;

	use runtime::runtime_types::{
		staging_xcm::v4::{
			asset::Fungibility::Fungible,
			asset::{Asset, AssetId, Assets},
			junction::Junction,
			junctions::Junctions,
			junctions::Junctions::X1,
			location::Location,
		},
		xcm::{v3::WeightLimit, VersionedAssets, VersionedLocation},
	};

	// generate XCM message to reserve transfer funds to a designated account on
	// Pop Parachain
	pub(crate) fn gen_account_fund_message_call(account: Keypair) -> RuntimeCall {
		let pop_location = VersionedLocation::V4(Location {
			parents: 0,
			interior: X1([Junction::Parachain(PARA_ID)]),
		});
		let pop_beneficiary = VersionedLocation::V4(Location {
			parents: 0,
			interior: X1([Junction::AccountId32 { network: None, id: account.public_key().0 }]),
		});
		let amount = Fungible(AMOUNT_TO_FUND);
		let assets = VersionedAssets::V4(Assets {
			0: vec![Asset {
				id: AssetId { 0: Location { parents: 0, interior: Junctions::Here } },
				fun: amount,
			}],
		});

		RuntimeCall::XcmPallet(
			crate::relay::runtime::xcm_pallet::Call::limited_reserve_transfer_assets {
				dest: Box::new(pop_location),
				beneficiary: Box::new(pop_beneficiary),
				assets: Box::new(assets),
				fee_asset_item: 0,
				weight_limit: WeightLimit::Unlimited,
			},
		)
	}
}

#[cfg(feature = "paseo")]
mod relay {
	use super::*;
	pub(crate) use crate::paseo_interface::api as runtime;

	pub(crate) type RuntimeCall = runtime::runtime_types::paseo_runtime::RuntimeCall;
	pub(crate) const UNIT: u128 = 10_000_000_000;

	use runtime::runtime_types::{
		staging_xcm::v3::multilocation::MultiLocation as Location,
		xcm::{
			v3::{
				junction::Junction,
				junctions::Junctions,
				junctions::Junctions::X1,
				multiasset::{
					AssetId::Concrete, Fungibility::Fungible, MultiAsset as Asset,
					MultiAssets as Assets,
				},
				WeightLimit,
			},
			VersionedMultiAssets as VersionedAssets, VersionedMultiLocation as VersionedLocation,
		},
	};

	// generate XCM message to reserve transfer funds to a designated account on
	// Pop Parachain
	pub(crate) fn gen_account_fund_message_call(account: Keypair) -> RuntimeCall {
		let pop_location = VersionedLocation::V3(Location {
			parents: 0,
			interior: X1(Junction::Parachain(PARA_ID)),
		});
		let pop_beneficiary = VersionedLocation::V3(Location {
			parents: 0,
			interior: X1(Junction::AccountId32 { network: None, id: account.public_key().0 }),
		});
		let amount = Fungible(AMOUNT_TO_FUND);

		let assets = VersionedAssets::V3(Assets {
			0: vec![Asset {
				id: Concrete(Location { parents: 0, interior: Junctions::Here }),
				fun: amount,
			}],
		});

		RuntimeCall::XcmPallet(
			crate::relay::runtime::xcm_pallet::Call::limited_reserve_transfer_assets {
				dest: Box::new(pop_location),
				beneficiary: Box::new(pop_beneficiary),
				assets: Box::new(assets),
				fee_asset_item: 0,
				weight_limit: WeightLimit::Unlimited,
			},
		)
	}
}

use relay::*;
const AMOUNT_TO_FUND: u128 = UNIT * 1_000_000;

use pop_interface::api as pop;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let relay_api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:8833").await?;
	let pop_api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:9944").await?;

	let dev_accounts = vec![dev::alice(), dev::bob(), dev::charlie()];
	let fund_pop_accounts_calls: Vec<RuntimeCall> =
		dev_accounts.iter().map(|a| gen_account_fund_message_call(a.clone())).collect();

	let set_alice_balance = RuntimeCall::Balances(runtime::balances::Call::force_set_balance {
		who: dev::alice().public_key().into(),
		new_free: UNIT * 1_000_000_000,
	});

	// set alice's balance first so the account is guaranteed to have enough funds
	let sudo_set_balance = runtime::tx().sudo().sudo(set_alice_balance);
	let from = dev::alice();
	let _ = relay_api
		.tx()
		.sign_and_submit_then_watch_default(&sudo_set_balance, &from)
		.await?
		.wait_for_finalized_success()
		.await?;

	let batch_tx = runtime::tx().utility().batch(fund_pop_accounts_calls);

	let from = dev::alice();
	let batch_events = relay_api
		.tx()
		.sign_and_submit_then_watch_default(&batch_tx, &from)
		.await?
		.wait_for_finalized_success()
		.await?;

	let xcm_event = batch_events.find_first::<runtime::xcm_pallet::events::Sent>()?;
	if let Some(event) = xcm_event {
		println!("XCM messages sent {event:?}");
	}
	println!("Checking Pop Balances");

	// simple system to wait up to 8 block intervals total (not per account)
	let max_wait = 8;
	let mut waited = 0;
	for account in dev_accounts {
		let query = pop::storage().system().account(&account.public_key().0.into());

		let mut result = None;

		while waited < max_wait {
			result = pop_api.storage().at_latest().await?.fetch(&query).await?;
			if result.is_some() {
				break;
			}

			// if result is none, wait for 6 seconds. Timeout in 8 * 6 seconds
			tokio::time::sleep(Duration::from_millis(6000)).await;
			waited += 1;
		}

		// check accounts were funded. 2 UNIT threshold to account for fees
		assert!(result.expect("account does not exist").data.free >= AMOUNT_TO_FUND - UNIT * 2);
		println!("Account: {:?} Funded", account);
	}
	Ok(())
}

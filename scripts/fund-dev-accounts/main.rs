use sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec};
use sp_core::sr25519::Pair;
use sp_runtime::MultiSigner;
use subxt::{
	utils::{AccountId32, MultiAddress},
	OnlineClient, PolkadotConfig,
};
use subxt_signer::sr25519::{dev, Keypair};

use sp_core::Encode;
use subxt::tx::Signer;

#[subxt::subxt(runtime_metadata_path = "../metadata/rococo.scale")]
pub mod rococo {}

type RococoCall = rococo::runtime_types::rococo_runtime::RuntimeCall;

use crate::rococo::runtime_types::staging_xcm::v4::asset::Fungibility::Fungible;
use crate::rococo::runtime_types::staging_xcm::v4::asset::{Asset, AssetId, Assets};
use crate::rococo::runtime_types::staging_xcm::v4::junction::Junction;
use crate::rococo::runtime_types::staging_xcm::v4::junctions::Junctions;
use crate::rococo::runtime_types::staging_xcm::v4::junctions::Junctions::X1;
use crate::rococo::runtime_types::staging_xcm::v4::location::Location;
use crate::rococo::runtime_types::xcm::v3::WeightLimit;
use crate::rococo::runtime_types::xcm::{VersionedAssetId, VersionedAssets, VersionedLocation};
use rococo::runtime_types::polkadot_parachain_primitives::primitives::Id;

const ROC_UNIT: u128 = 1_000_000_000_000;
const PAS_UNIT: u128 = 10_000_000_000;

// generate XCM message to reserve transfer funds to a designated account on
// Pop Parachain
fn gen_account_fund_message_call(account: Keypair) -> RococoCall {
	let pop_location =
		VersionedLocation::V4(Location { parents: 0, interior: X1([Junction::Parachain(9090)]) });
	let pop_beneficiary = VersionedLocation::V4(Location {
		parents: 0,
		interior: X1([Junction::AccountId32 { network: None, id: account.public_key().0 }]),
	});
	let amount = Fungible(ROC_UNIT * 1_000_000);
	let assets = VersionedAssets::V4(Assets {
		0: vec![Asset {
			id: AssetId { 0: Location { parents: 0, interior: Junctions::Here } },
			fun: amount,
		}],
	});

	RococoCall::XcmPallet(rococo::xcm_pallet::Call::limited_reserve_transfer_assets {
		dest: Box::new(pop_location),
		beneficiary: Box::new(pop_beneficiary),
		assets: Box::new(assets),
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	})
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	use rococo::runtime_types::staging_xcm::v4::location::Location;
	use rococo::runtime_types::xcm::VersionedLocation;

	use rococo::runtime_types::polkadot_parachain_primitives::primitives::{
		HeadData, Id, ValidationCode,
	};

	let dev_accounts = vec![dev::alice(), dev::bob(), dev::charlie()];
	let fund_pop_accounts =
		dev_accounts.iter().map(|a| gen_account_fund_message_call(a.clone())).collect();

	let rococo_api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:8833").await?;

	let set_alice_balance = RococoCall::Balances(rococo::balances::Call::force_set_balance {
		who: dev::alice().public_key().into(),
		new_free: ROC_UNIT * 1_000_000_000,
	});

	// set alice's balance first so the account is guaranteed to have enough funds
	let sudo_set_balance = rococo::tx().sudo().sudo(set_alice_balance);
	let from = dev::alice();
	let batch_events = rococo_api
		.tx()
		.sign_and_submit_then_watch_default(&sudo_set_balance, &from)
		.await?
		.wait_for_finalized_success()
		.await?;

	let batch_tx = rococo::tx().utility().batch(fund_pop_accounts);

	let from = dev::alice();
	let batch_events = rococo_api
		.tx()
		.sign_and_submit_then_watch_default(&batch_tx, &from)
		.await?
		.wait_for_finalized_success()
		.await?;

	Ok(())
}

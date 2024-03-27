use subxt::{OnlineClient, PolkadotConfig};
use subxt_signer::sr25519::{dev, Keypair};

#[cfg(not(feature = "paseo"))]
mod relay {
	#[subxt::subxt(runtime_metadata_path = "../metadata/rococo.scale")]
	pub(crate) mod runtime {}
	pub(crate) type RuntimeCall = runtime::runtime_types::rococo_runtime::RuntimeCall;
	pub(crate) const UNIT: u128 = 1_000_000_000_000;
}

#[cfg(feature = "paseo")]
mod relay {
	#[subxt::subxt(runtime_metadata_path = "../metadata/paseo.scale")]
	pub(crate) mod runtime {}
	pub(crate) type RuntimeCall = relay::runtime_types::paseo_runtime::RuntimeCall;
	pub(crate) const UNIT: u128 = 10_000_000_000;
}

use relay::RuntimeCall;
use relay::UNIT;

use relay::runtime::runtime_types::{
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
fn gen_account_fund_message_call(account: Keypair) -> RuntimeCall {
	let pop_location =
		VersionedLocation::V4(Location { parents: 0, interior: X1([Junction::Parachain(9090)]) });
	let pop_beneficiary = VersionedLocation::V4(Location {
		parents: 0,
		interior: X1([Junction::AccountId32 { network: None, id: account.public_key().0 }]),
	});
	let amount = Fungible(UNIT * 1_000_000);
	let assets = VersionedAssets::V4(Assets {
		0: vec![Asset {
			id: AssetId { 0: Location { parents: 0, interior: Junctions::Here } },
			fun: amount,
		}],
	});

	RuntimeCall::XcmPallet(relay::runtime::xcm_pallet::Call::limited_reserve_transfer_assets {
		dest: Box::new(pop_location),
		beneficiary: Box::new(pop_beneficiary),
		assets: Box::new(assets),
		fee_asset_item: 0,
		weight_limit: WeightLimit::Unlimited,
	})
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let dev_accounts = vec![dev::alice(), dev::bob(), dev::charlie()];
	let fund_pop_accounts =
		dev_accounts.iter().map(|a| gen_account_fund_message_call(a.clone())).collect();

	let rococo_api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:8833").await?;

	let set_alice_balance =
		RuntimeCall::Balances(relay::runtime::balances::Call::force_set_balance {
			who: dev::alice().public_key().into(),
			new_free: UNIT * 1_000_000_000,
		});

	// set alice's balance first so the account is guaranteed to have enough funds
	let sudo_set_balance = relay::runtime::tx().sudo().sudo(set_alice_balance);
	let from = dev::alice();
	let _ = rococo_api
		.tx()
		.sign_and_submit_then_watch_default(&sudo_set_balance, &from)
		.await?
		.wait_for_finalized_success()
		.await?;

	let batch_tx = relay::runtime::tx().utility().batch(fund_pop_accounts);

	let from = dev::alice();
	let _ = rococo_api
		.tx()
		.sign_and_submit_then_watch_default(&batch_tx, &from)
		.await?
		.wait_for_finalized_success()
		.await?;

	Ok(())
}

use sp_core::crypto::{Ss58AddressFormatRegistry, Ss58Codec};
use sp_core::sr25519::Pair;
use sp_runtime::MultiSigner;
use subxt::{
	utils::{AccountId32, MultiAddress},
	OnlineClient, PolkadotConfig,
};
use subxt_signer::sr25519::dev;

use sp_core::Encode;

#[subxt::subxt(runtime_metadata_path = "./metadata/rococo.scale")]
pub mod rococo {}

#[subxt::subxt(runtime_metadata_path = "./metadata/pop-net.scale")]
pub mod pop {}

type RococoCall = rococo::runtime_types::rococo_runtime::RuntimeCall;

use rococo::runtime_types::polkadot_parachain_primitives::primitives::Id;

pub fn calculate_sovereign_account<Pair>(
	para_id: u32,
) -> Result<AccountId32, Box<dyn std::error::Error>>
where
	Pair: sp_core::Pair,
	Pair::Public: Into<MultiSigner>,
{
	let id = Id(para_id);
	let prefix = hex::encode("para");
	let encoded_id = hex::encode(id.encode());
	let encoded_key = "0x".to_owned() + &prefix + &encoded_id;
	let public_str = format!("{:0<width$}", encoded_key, width = 64 + 2);

	let public = array_bytes::hex2bytes(&public_str).expect("Failed to convert hex to bytes");
	let public_key = Pair::Public::try_from(&public)
		.map_err(|_| "Failed to construct public key from given hex")?;
	let to_parse =
		public_key.to_ss58check_with_version(Ss58AddressFormatRegistry::SubstrateAccount.into());
	Ok(to_parse.parse().unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	use rococo::runtime_types::pallet_broker::coretime_interface::CoreAssignment;
	use rococo::runtime_types::polkadot_parachain_primitives::primitives::{
		HeadData, Id, ValidationCode,
	};
	use rococo::runtime_types::polkadot_runtime_parachains::assigner_coretime::PartsOf57600;

	let pop_api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:9944").await?;
	let rococo_api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:8833").await?;

	let head_data =
		std::fs::read_to_string("./integration-tests/artifacts/para-2000-genesis-state")?;
	let validation_code =
		std::fs::read_to_string("./integration-tests/artifacts/para-2000-genesis-code")?;
	let para_id_tx = rococo::tx().registrar().reserve();

	let force_register = RococoCall::Registrar(rococo::registrar::Call::force_register {
		who: dev::charlie().public_key().into(),
		deposit: 100000000000000,
		id: Id(2000),
		genesis_head: HeadData(hex::decode(&head_data[2..])?),
		validation_code: ValidationCode(hex::decode(&validation_code[2..])?),
	});

	let assign_core = RococoCall::Coretime(rococo::coretime::Call::assign_core {
		begin: 0,
		core: 2,
		assignment: vec![(CoreAssignment::Pool, PartsOf57600(57600))],
		end_hint: None,
	});

	let fund_sovereign_account = RococoCall::Balances(rococo::balances::Call::force_set_balance {
		who: subxt::utils::MultiAddress::Id(
			calculate_sovereign_account::<Pair>(9090)
				.expect("Failed to calculate sovereign account"),
		),
		new_free: 1000000000000000,
	});

	let batch = RococoCall::Utility(rococo::utility::Call::batch {
		calls: vec![force_register, assign_core, fund_sovereign_account],
	});

	let sudo_batch = rococo::tx().sudo().sudo(batch);

	let from = dev::charlie();
	let events = rococo_api
		.tx()
		.sign_and_submit_then_watch_default(&para_id_tx, &from)
		.await?
		.wait_for_finalized_success()
		.await?;

	let reserved_event = events.find_first::<rococo::registrar::events::Reserved>()?;
	if let Some(event) = reserved_event {
		println!("Para ID reserved success: {event:?}");
	}

	let from = dev::alice();
	let batch_events = rococo_api
		.tx()
		.sign_and_submit_then_watch_default(&sudo_batch, &from)
		.await?
		.wait_for_finalized_success()
		.await?;

	let registered_event = batch_events.find_first::<rococo::registrar::events::Registered>()?;
	if let Some(event) = registered_event {
		println!("Para thread registered: {event:?}");
	}

	let log = std::fs::File::create("para-2000.log").expect("Failed to create log file");
	let mut collator =
		std::process::Command::new("./integration-tests/artifacts/parachain-template-node")
			.arg("--alice")
			.arg("--collator")
			.arg("--force-authoring")
			.arg("--base-path")
			.arg("/tmp/parachain/alice")
			.arg("--port")
			.arg("40336")
			.arg("--rpc-port")
			.arg("8811")
			.arg("--rpc-cors")
			.arg("all")
			.arg("--unsafe-rpc-external")
			.arg("--rpc-methods")
			.arg("unsafe")
			.arg("--chain")
			.arg("./integration-tests/artifacts/para-2000-raw-spec.json")
			.arg("--execution")
			.arg("wasm")
			.arg("--")
			.arg("--execution")
			.arg("wasm")
			.arg("--chain")
			.arg("./integration-tests/artifacts/rococo-local.json")
			.arg("--rpc-port")
			.arg("9947")
			.arg("--port")
			.arg("30338")
			.stdout(log)
			.spawn()
			.expect("Failed to execute command");

	collator.wait().expect("Failed to wait for collator");
	Ok(())
}

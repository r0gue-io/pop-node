use subxt::{OnlineClient, PolkadotConfig, utils::{AccountId32, MultiAddress},};
use subxt_signer::sr25519::dev;

#[subxt::subxt(runtime_metadata_path = "./metadata/rococo.scale")]
pub mod rococo {}

#[subxt::subxt(runtime_metadata_path = "./metadata/pop-net.scale")]
pub mod pop {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rococo::runtime_types::polkadot_parachain_primitives::primitives::{Id, HeadData, ValidationCode};
    let pop_api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:56939").await?;
    let rococo_api = OnlineClient::<PolkadotConfig>::from_url("ws://127.0.0.1:56931").await?;

    // Build a balance transfer extrinsic.
    let para_id_tx = rococo::tx().registrar().reserve();
    let head_data  = HeadData(vec![0u8; 32]);
    let validation_code  = ValidationCode(vec![0u8; 32]);
    let para_register_tx = rococo::tx().registrar().register(Id(2000), head_data, validation_code);
    log::debug!("para_id_tx: {:?}", para_id_tx);

    let from = dev::alice();
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

    let events = rococo_api
        .tx()
        .sign_and_submit_then_watch_default(&para_register_tx, &from)
        .await?
        .wait_for_finalized_success()
        .await?;

    let registered_event = events.find_first::<rococo::registrar::events::Registered>()?;
    if let Some(event) = registered_event {
        println!("Para Thread registered success: {event:?}");
    }


    Ok(())
}
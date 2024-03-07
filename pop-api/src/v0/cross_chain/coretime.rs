use crate::RuntimeCall;
use crate::dispatch;
use crate::v0::cross_chain::XcmCalls;

pub fn place_spot_order(
    para_id: u32,
    max_price: u128
) -> Result<(), E> {

    // craft encoded message to be sent via XCM
    // Needs help from runtime to double encode the call.
    // Like in https://github.com/paritytech/polkadot-sdk/blob/629506ce061db76d31d4f7a81f4a497752b27259/cumulus/parachains/runtimes/coretime/coretime-rococo/src/coretime.rs#L97
    // My intention was adding a handle_xcm function in the extension
    // that returned a tuple, (destination Location, encoded call)
    // If handle_send_xcm() receives Relay::OnDemand(OnDemandCall::PlaceOrderKeepAlive) it has enough info
    // to encode the Location and the message.

    Ok(())

}

#[derive(scale::Encode)]
pub(crate) enum OnDemandCall {
    #[codec(index = 1)]
    PlaceOrderKeepAlive {
        max_amount: u128,
        para_id: u32,
    },
}
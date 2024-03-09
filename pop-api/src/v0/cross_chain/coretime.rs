use crate::{
    handle_xcm,
    primitives::xc_indices::{XCLocationIndices, RelayIndices, OnDemandCall}
};

pub fn place_spot_order(
    max_amount: u128,
    para_id: u32,
) -> crate::cross_chain::Result<()> {
   Ok(handle_xcm(XCLocationIndices::Relay(RelayIndices::OnDemand(OnDemandCall::PlaceOrderKeepAlive {
    max_amount,
    para_id,
   })))?)
}
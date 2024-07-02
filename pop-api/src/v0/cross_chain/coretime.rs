// use crate::{
// 	primitives::cross_chain::{CrossChainMessage, OnDemand, RelayChainMessage},
// 	send_xcm,
// };
//
// /// Send a cross-chain message to place a sport order for instantaneous coretime.
// pub fn place_spot_order(max_amount: u128, para_id: u32) -> crate::cross_chain::Result<()> {
// 	Ok(send_xcm(CrossChainMessage::Relay(RelayChainMessage::OnDemand(
// 		OnDemand::PlaceOrderKeepAlive { max_amount, para_id },
// 	)))?)
// }

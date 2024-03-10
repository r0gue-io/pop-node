use scale::{Decode, Encode, MaxEncodedLen};

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum CrossChainMessage {
	Relay(RelayChainMessage),
}

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum RelayChainMessage {
	// Rococo index: https://github.com/paritytech/polkadot-sdk/blob/629506ce061db76d31d4f7a81f4a497752b27259/polkadot/runtime/rococo/src/lib.rs#L1423
	#[codec(index = 66)]
	OnDemand(OnDemand),
}

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum OnDemand {
	#[codec(index = 1)]
	PlaceOrderKeepAlive { max_amount: u128, para_id: u32 },
}

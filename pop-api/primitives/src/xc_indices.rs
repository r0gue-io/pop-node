use scale::{Decode, Encode, MaxEncodedLen};


#[derive(scale::Encode, Decode, Debug, MaxEncodedLen)]
pub enum XCLocationIndices {
    Relay(RelayIndices),
}

#[derive(scale::Encode, Decode, Debug, MaxEncodedLen)]
pub enum RelayIndices {
    // Rococo index: https://github.com/paritytech/polkadot-sdk/blob/629506ce061db76d31d4f7a81f4a497752b27259/polkadot/runtime/rococo/src/lib.rs#L1423
    #[codec(index = 66)]
    OnDemand(OnDemandCall),
}

#[derive(scale::Encode, Decode, Debug, MaxEncodedLen)]
pub enum OnDemandCall {
    #[codec(index = 1)]
    PlaceOrderKeepAlive {
        max_amount: u128,
        para_id: u32,
    },
}
use scale::{Encode, Decode};

#[derive(Encode, Decode, Debug)]
pub enum RuntimeStateKeys {
    ParachainSystem(ParachainSystemKeys),
}

#[derive(Encode, Decode, Debug)]
pub enum ParachainSystemKeys {
    LastRelayChainBlockNumber
}
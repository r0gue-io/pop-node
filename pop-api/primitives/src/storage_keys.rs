use scale::{Decode, Encode, MaxEncodedLen};

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum RuntimeStateKeys {
	ParachainSystem(ParachainSystemKeys),
}

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum ParachainSystemKeys {
	LastRelayChainBlockNumber,
}

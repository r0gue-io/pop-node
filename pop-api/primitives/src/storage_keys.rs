use scale::{Decode, Encode, MaxEncodedLen};

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum RuntimeStateKeys {
	Nfts(NftsKeys),
	ParachainSystem(ParachainSystemKeys),
}

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum ParachainSystemKeys {
	LastRelayChainBlockNumber,
}

// https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/nfts/src/impl_nonfungibles.rs
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum NftsKeys {
	Owner,
	CollectionOwner,
	Attribute,
	CustomAttribute,
	SystemAttribute,
	CollectionAttribute,
}

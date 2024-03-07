use crate::{
	primitives::storage_keys::{ParachainSystemKeys, RuntimeStateKeys},
	BlockNumber, PopApiError,
};

pub mod balances;
pub mod nfts;
pub mod state;
pub mod cross_chain;

pub fn relay_chain_block_number() -> Result<BlockNumber, PopApiError> {
	state::read(RuntimeStateKeys::ParachainSystem(ParachainSystemKeys::LastRelayChainBlockNumber))
}

#[derive(scale::Encode)]
pub(crate) enum RuntimeCall {
	#[codec(index = 10)]
	Balances(balances::BalancesCall),
	#[codec(index = 31)]
	Xcm(cross_chain::XcmCalls),
	#[codec(index = 50)]
	Nfts(nfts::NftCalls),
}

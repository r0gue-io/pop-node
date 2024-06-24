#[cfg(feature = "assets")]
pub mod assets;
#[cfg(feature = "balances")]
pub mod balances;
#[cfg(feature = "cross-chain")]
pub mod cross_chain;
#[cfg(feature = "nfts")]
pub mod nfts;
pub mod state;

#[derive(scale::Encode)]
pub(crate) enum RuntimeCall {
	// #[codec(index = 10)]
	// #[cfg(feature = "balances")]
	// Balances(balances::BalancesCall),
	// #[codec(index = 50)]
	// #[cfg(feature = "nfts")]
	// Nfts(nfts::NftCalls),
	#[codec(index = 52)]
	#[cfg(feature = "assets")]
	Assets(assets::AssetsCall),
}

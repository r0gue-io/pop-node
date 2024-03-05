pub mod balances;
pub mod nfts;
pub mod state;

#[derive(scale::Encode)]
pub(crate) enum RuntimeCall {
    #[codec(index = 10)]
    Balances(balances::BalancesCall),
    #[codec(index = 50)]
    Nfts(nfts::NftCalls),
}

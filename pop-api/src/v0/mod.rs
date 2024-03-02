pub mod balances;
pub mod nfts;

#[derive(scale::Encode)]
pub(crate) enum RuntimeCall {
    #[codec(index = 50)]
    Nfts(nfts::NftCalls),
}

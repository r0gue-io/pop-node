pub mod balances;
pub mod nfts;
pub mod polkadot_xcm;

#[derive(scale::Encode)]
pub(crate) enum RuntimeCall {
    #[codec(index = 50)]
    Nfts(nfts::NftCalls),
    #[codec(index = 31)]
    PolkadotXcm(polkadot_xcm::XcmCalls),
}

use crate::interfaces::nfts::{Nft, NftCalls};
use pop_runtime::{AccountId, Balance, BlockNumber, assets_config::{KeyLimit, StringLimit, MaxTips}, Signature};
use parachains_common::{CollectionId, ItemId};

pub struct Pop {
   pub api: RuntimeCall 
}

enum RuntimeCall {
    Nfts(NftCalls<AccountId, Balance, BlockNumber, CollectionId, ItemId, KeyLimit, StringLimit, MaxTips, Signature>)
}
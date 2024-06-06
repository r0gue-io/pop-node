use super::*;

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum RuntimeStateKeys {
	Nfts(NftsKeys),
	ParachainSystem(ParachainSystemKeys),
	#[cfg(feature = "devnet")]
	Assets(AssetsKeys),
}

#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum ParachainSystemKeys {
	/// Get the last relay chain block number seen by the parachain.
	LastRelayChainBlockNumber,
}

// https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/nfts/src/impl_nonfungibles.rs
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum NftsKeys {
	// Get the details of a collection.
	Collection(CollectionId),
	/// Get the owner of the collection, if the collection exists.
	CollectionOwner(CollectionId),
	// Get the details of an item.
	Item(CollectionId, ItemId),
	/// Get the owner of the item, if the item exists.
	Owner(CollectionId, ItemId),
	/// Get the attribute value of `item` of `collection` corresponding to `key`.
	Attribute(CollectionId, ItemId, BoundedVec<u8, KeyLimit>),
	/// Get the system attribute value of `item` of `collection` corresponding to `key`
	SystemAttribute(CollectionId, Option<ItemId>, BoundedVec<u8, KeyLimit>),
	/// Get the attribute value of `item` of `collection` corresponding to `key`.
	CollectionAttribute(CollectionId, BoundedVec<u8, KeyLimit>),
}

/// The required input for state queries in pallet assets.
#[cfg(feature = "devnet")]
#[derive(Encode, Decode, Debug, MaxEncodedLen)]
pub enum AssetsKeys {
	Allowance(AssetId, AccountId, AccountId),
	/// Check if the asset exists.
	AssetExists(AssetId),
	/// Check balance.
	BalanceOf(AssetId, AccountId),
	/// Returns the total token supply for a given asset ID.
	TotalSupply(AssetId),
}

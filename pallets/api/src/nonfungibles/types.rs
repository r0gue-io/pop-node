use frame_support::traits::{nonfungibles_v2::Inspect, Currency};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet_nfts::{
	AttributeNamespace, CollectionConfig, CollectionDetails, CollectionSetting, CollectionSettings,
	DestroyWitness, ItemDeposit, ItemDetails, ItemSetting, MintSettings, MintType, MintWitness,
};

// Type aliases for pallet-nfts.
pub(super) type NftsOf<T> = pallet_nfts::Pallet<T>;
pub(super) type NftsWeightInfoOf<T> = <T as pallet_nfts::Config>::WeightInfo;
// Type aliases for pallet-nfts storage items.
pub(super) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub(super) type BalanceOf<T, I = ()> = <<T as pallet_nfts::Config<I>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
pub(super) type NextCollectionIdOf<T, I = ()> = pallet_nfts::NextCollectionId<T, I>;
pub(super) type CollectionIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;
pub(super) type ItemIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;
pub(super) type ItemPriceOf<T, I = ()> = BalanceOf<T, I>;
// TODO: Multi-instances.
pub(super) type CollectionDetailsFor<T, I = ()> =
	CollectionDetails<AccountIdOf<T>, BalanceOf<T, I>>;
pub(super) type AttributeNamespaceOf<T> = AttributeNamespace<AccountIdOf<T>>;
pub(super) type CollectionConfigFor<T, I = ()> =
	CollectionConfig<ItemPriceOf<T, I>, BlockNumberFor<T>, CollectionIdOf<T>>;

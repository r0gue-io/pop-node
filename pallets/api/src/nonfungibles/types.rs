use super::*;

pub(super) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub(super) type NftsOf<T> = pallet_nfts::Pallet<T, NftsInstanceOf<T>>;
pub(super) type NftsErrorOf<T> = pallet_nfts::Error<T, NftsInstanceOf<T>>;
pub(super) type NftsWeightInfoOf<T> = <T as pallet_nfts::Config<NftsInstanceOf<T>>>::WeightInfo;
pub(super) type NftsInstanceOf<T> = <T as Config>::NftsInstance;
pub(super) type BalanceOf<T> =
	<<T as pallet_nfts::Config<NftsInstanceOf<T>>>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;
pub(super) type CollectionIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;
pub(super) type ItemIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;
pub(super) type ItemPriceOf<T> = BalanceOf<T>;
pub(super) type CollectionDetailsFor<T> = CollectionDetails<AccountIdOf<T>, BalanceOf<T>>;
pub(super) type AttributeNamespaceOf<T> = AttributeNamespace<AccountIdOf<T>>;
pub(super) type CollectionConfigFor<T> =
	CollectionConfig<ItemPriceOf<T>, BlockNumberFor<T>, CollectionIdOf<T>>;
// Public due to pop-api integration tests crate.
pub type AccountBalanceOf<T> = pallet_nfts::AccountBalance<T, NftsInstanceOf<T>>;
pub type AttributeOf<T> = pallet_nfts::Attribute<T, NftsInstanceOf<T>>;
pub type AttributeKey<T> = BoundedVec<u8, <T as pallet_nfts::Config<NftsInstanceOf<T>>>::KeyLimit>;
pub type AttributeValue<T> =
	BoundedVec<u8, <T as pallet_nfts::Config<NftsInstanceOf<T>>>::ValueLimit>;
pub type CollectionOf<T> = pallet_nfts::Collection<T, NftsInstanceOf<T>>;
pub type CollectionConfigOf<T> = pallet_nfts::CollectionConfigOf<T, NftsInstanceOf<T>>;
pub type NextCollectionIdOf<T> = pallet_nfts::NextCollectionId<T, NftsInstanceOf<T>>;
pub type MetadataData<T> =
	BoundedVec<u8, <T as pallet_nfts::Config<NftsInstanceOf<T>>>::StringLimit>;

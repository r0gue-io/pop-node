use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{nonfungibles_v2::Inspect, Currency};
use frame_system::pallet_prelude::BlockNumberFor;
pub use pallet_nfts::{
	AttributeNamespace, CollectionDetails, DestroyWitness, ItemDeposit, ItemDetails, MintType,
	MintWitness,
};
use scale_info::TypeInfo;
use sp_runtime::{BoundedBTreeMap, RuntimeDebug};

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
pub(super) type ApprovalsOf<T> = BoundedBTreeMap<
	AccountIdOf<T>,
	Option<BlockNumberFor<T>>,
	<T as pallet_nfts::Config>::ApprovalsLimit,
>;
pub(super) type ItemPriceOf<T, I = ()> = BalanceOf<T, I>;
// TODO: Multi-instances.
pub(super) type ItemDepositOf<T, I = ()> = ItemDeposit<BalanceOf<T, I>, AccountIdOf<T>>;
pub(super) type CollectionDetailsFor<T, I = ()> =
	CollectionDetails<AccountIdOf<T>, BalanceOf<T, I>>;
pub(super) type ItemDetailsFor<T, I = ()> =
	ItemDetails<AccountIdOf<T>, ItemDepositOf<T, I>, ApprovalsOf<T>>;
pub(super) type AttributeNamespaceOf<T> = AttributeNamespace<AccountIdOf<T>>;
pub(super) type CreateCollectionConfigFor<T, I = ()> =
	CreateCollectionConfig<ItemPriceOf<T, I>, BlockNumberFor<T>, CollectionIdOf<T>>;

#[derive(Clone, Copy, Decode, Encode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct CreateCollectionConfig<Price, BlockNumber, CollectionId> {
	pub max_supply: Option<u32>,
	pub mint_type: MintType<CollectionId>,
	pub price: Option<Price>,
	pub start_block: Option<BlockNumber>,
	pub end_block: Option<BlockNumber>,
}

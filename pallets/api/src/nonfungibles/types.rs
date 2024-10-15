use frame_support::traits::{nonfungibles_v2::Inspect, Currency};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_nfts::{CollectionDetails, ItemDeposit, ItemDetails};
use sp_runtime::BoundedBTreeMap;

// Type aliases for pallet-nfts.
pub(super) type NftsOf<T> = pallet_nfts::Pallet<T>;
pub(super) type NftsWeightInfoOf<T> = <T as pallet_nfts::Config>::WeightInfo;
// Type aliases for pallet-nfts storage items.
pub(super) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub(super) type BalanceOf<T, I = ()> = <<T as pallet_nfts::Config<I>>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
pub(super) type CollectionIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;
pub(super) type ItemIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;
type ApprovalsOf<T> = BoundedBTreeMap<
	AccountIdOf<T>,
	Option<BlockNumberFor<T>>,
	<T as pallet_nfts::Config>::ApprovalsLimit,
>;
// TODO: Multi-instances.
pub(super) type ItemDepositOf<T, I = ()> = ItemDeposit<BalanceOf<T, I>, AccountIdOf<T>>;
pub(super) type CollectionDetailsFor<T, I = ()> =
	CollectionDetails<AccountIdOf<T>, BalanceOf<T, I>>;
pub(super) type ItemDetailsFor<T, I = ()> =
	ItemDetails<AccountIdOf<T>, ItemDepositOf<T, I>, ApprovalsOf<T>>;

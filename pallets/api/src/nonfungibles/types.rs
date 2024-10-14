use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{nonfungibles_v2::Inspect, Currency};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_runtime::BoundedBTreeMap;

use super::*;

pub(super) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub(super) type NftsOf<T> = pallet_nfts::Pallet<T>;
/// Weight information for extrinsics in this pallet.
pub(super) type NftsWeightInfoOf<T> = <T as pallet_nfts::Config>::WeightInfo;
/// A type alias for the collection ID.
pub(super) type CollectionIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;
/// A type alias for the collection item ID.
pub(super) type ItemIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;
/// A type alias for handling balance deposits.
pub(super) type BalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;
/// A type alias for keeping track of approvals used by a single item.
pub(super) type ApprovalsOf<T> = BoundedBTreeMap<
	AccountIdOf<T>,
	Option<BlockNumberFor<T>>,
	<T as pallet_nfts::Config>::ApprovalsLimit,
>;

pub(super) type ItemDetailsFor<T> = ItemDetails<AccountIdOf<T>, BalanceOf<T>, ApprovalsOf<T>>;
pub(super) type CollectionDetailsFor<T> = CollectionDetails<AccountIdOf<T>, BalanceOf<T>>;

/// Information concerning the ownership of a single unique item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ItemDetails<AccountId, Deposit, Approvals> {
	/// The owner of this item.
	pub owner: AccountId,
	/// The approved transferrer of this item, if one is set.
	pub approvals: Approvals,
	/// The amount held in the pallet's default account for this item. Free-hold items will have
	/// this as zero.
	pub deposit: Deposit,
}
/// Information about the reserved item deposit.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct ItemDeposit<DepositBalance, AccountId> {
	/// A depositor account.
	account: AccountId,
	/// An amount that gets reserved.
	amount: DepositBalance,
}
/// Information about a collection.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CollectionDetails<AccountId, DepositBalance> {
	/// Collection's owner.
	pub owner: AccountId,
	/// The total balance deposited by the owner for all the storage data associated with this
	/// collection. Used by `destroy`.
	pub owner_deposit: DepositBalance,
	/// The total number of outstanding items of this collection.
	pub items: u32,
	/// The total number of outstanding item metadata of this collection.
	pub item_metadatas: u32,
	/// The total number of outstanding item configs of this collection.
	pub item_configs: u32,
	/// The total number of attributes for this collection.
	pub attributes: u32,
}

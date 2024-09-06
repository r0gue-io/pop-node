use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::{nonfungibles_v2::Inspect, Currency};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_runtime::BoundedBTreeMap;

use super::Config;

pub(super) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub(super) type NftsOf<T> = pallet_nfts::Pallet<T>;

/// Weight information for extrinsics in this pallet.
pub(super) type NftsWeightInfoOf<T> = <T as pallet_nfts::Config>::WeightInfo;

/// A type alias for the collection ID.
pub(super) type CollectionIdOf<T> =
	<pallet_nfts::Pallet<T> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;

/// A type alias for the collection item ID.
pub(super) type ItemIdOf<T> =
	<pallet_nfts::Pallet<T> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;

// TODO: Even though this serves the `allowance` method, it creates the maintenance cost.

/// A type that holds the deposit for a single item.
pub(super) type ItemDepositOf<T> =
	ItemDeposit<DepositBalanceOf<T>, <T as frame_system::Config>::AccountId>;

/// A type alias for handling balance deposits.
pub(super) type DepositBalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

/// A type alias for keeping track of approvals used by a single item.
pub(super) type ApprovalsOf<T> = BoundedBTreeMap<
	AccountIdOf<T>,
	Option<BlockNumberFor<T>>,
	<T as pallet_nfts::Config>::ApprovalsLimit,
>;

/// Information concerning the ownership of a single unique item.
#[derive(Clone, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub(super) struct ItemDetails<T: Config> {
	/// The owner of this item.
	pub(super) owner: AccountIdOf<T>,
	/// The approved transferrer of this item, if one is set.
	pub(super) approvals: ApprovalsOf<T>,
	/// The amount held in the pallet's default account for this item. Free-hold items will have
	/// this as zero.
	pub(super) deposit: ItemDepositOf<T>,
}

/// Information about the reserved item deposit.
#[derive(Clone, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct ItemDeposit<DepositBalance, AccountId> {
	/// A depositor account.
	pub(super) account: AccountId,
	/// An amount that gets reserved.
	pub(super) amount: DepositBalance,
}

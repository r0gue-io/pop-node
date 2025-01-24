use frame_support::dispatch::DispatchResult;
use frame_system::pallet_prelude::*;

use super::{pallet::*, AccountIdLookupOf, CollectionIdOf, ItemIdOf, NftsOf};

impl<T: Config> Pallet<T> {
	// Approves the transfer of a specific item or all collection items owned by the `owner` to an
	// `operator`.
	///
	// # Parameters
	// - `owner` - The owner of the specified collection item(s).
	// - `collection` - The collection identifier.
	// - `maybe_item` - The optional item of the collection to be approved for delegated transfer.
	//   If `None`, the approval applies to all `owner`'s collection items.
	// - `operator` - The account to delegate permission to.
	pub(crate) fn do_approve(
		owner: OriginFor<T>,
		collection: CollectionIdOf<T>,
		maybe_item: Option<ItemIdOf<T>>,
		operator: AccountIdLookupOf<T>,
	) -> DispatchResult {
		match maybe_item {
			Some(item) => NftsOf::<T>::approve_transfer(owner, collection, item, operator, None),
			None => NftsOf::<T>::approve_collection_transfer(owner, collection, operator, None),
		}
	}

	// Cancel an approval to transfer a specific item or all items within a collection owned by
	// the `owner`.
	///
	// # Parameters
	// - `owner` - The owner of the specified collection item(s).
	// - `collection` - The collection identifier.
	// - `maybe_item` - The optional item of the collection that the operator has an approval to
	//   transfer. If not provided, an approval to transfer all `owner`'s collection items will be
	//   cancelled.
	// - `operator` - The account that had permission to transfer a specified collection item or all
	//   collection items owned by the `owner`.
	pub(crate) fn do_cancel_approval(
		owner: OriginFor<T>,
		collection: CollectionIdOf<T>,
		maybe_item: Option<ItemIdOf<T>>,
		operator: AccountIdLookupOf<T>,
	) -> DispatchResult {
		match maybe_item {
			Some(item) => NftsOf::<T>::cancel_approval(owner, collection, item, operator),
			None => NftsOf::<T>::cancel_collection_approval(owner, collection, operator),
		}
	}
}

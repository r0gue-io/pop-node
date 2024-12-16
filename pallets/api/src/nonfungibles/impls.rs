use frame_support::dispatch::{DispatchResultWithPostInfo, WithPostDispatchInfo};
use frame_system::pallet_prelude::*;
use pallet_nfts::WeightInfo as NftsWeightInfoTrait;
use sp_runtime::traits::StaticLookup;

use super::{pallet::*, AccountIdOf, CollectionIdOf, ItemIdOf, NftsOf, NftsWeightInfoOf};

impl<T: Config> Pallet<T> {
	/// Approves the transfer of a specific item or all collection items owned by the origin to
	/// an operator.
	///
	/// # Parameters
	/// - `owner` - The owner of the specified collection item(s).
	/// - `collection` - The identifier of the collection.
	/// - `maybe_item` - The optional item of the collection to be approved for delegated transfer.
	///   If `None`, the approval applies to all `owner`'s collection items.
	/// - `operator`: The account that will be allowed to take control of the specified item or all
	///   owner's collection items.
	pub(crate) fn do_approve(
		owner: OriginFor<T>,
		collection: CollectionIdOf<T>,
		maybe_item: Option<ItemIdOf<T>>,
		operator: &AccountIdOf<T>,
	) -> DispatchResultWithPostInfo {
		Ok(Some(match maybe_item {
			Some(item) => {
				NftsOf::<T>::approve_transfer(
					owner,
					collection,
					item,
					T::Lookup::unlookup(operator.clone()),
					None,
				)
				.map_err(|e| e.with_weight(NftsWeightInfoOf::<T>::approve_transfer()))?;
				NftsWeightInfoOf::<T>::approve_transfer()
			},
			None => {
				NftsOf::<T>::approve_collection_transfer(
					owner,
					collection,
					T::Lookup::unlookup(operator.clone()),
					None,
				)
				.map_err(|e| e.with_weight(NftsWeightInfoOf::<T>::approve_collection_transfer()))?;
				NftsWeightInfoOf::<T>::approve_collection_transfer()
			},
		})
		.into())
	}

	/// Cancel an approval to transfer a specific item or all items within a collection owned by
	/// the origin.
	///
	/// # Parameters
	/// - `owner` - The owner of the specified collection item(s).
	/// - `collection` - The identifier of the collection.
	/// - `maybe_item` - The optional item of the collection that the operator has an approval to
	///   transfer. If not provided, an approval to transfer all `owner`'s collection items will be
	///   cancelled.
	/// - `operator` - The account that had permission to transfer the sepcified item or all owner's
	///   collection items.
	pub(crate) fn do_cancel_approval(
		owner: OriginFor<T>,
		collection: CollectionIdOf<T>,
		maybe_item: Option<ItemIdOf<T>>,
		operator: &AccountIdOf<T>,
	) -> DispatchResultWithPostInfo {
		Ok(Some(match maybe_item {
			Some(item) => {
				NftsOf::<T>::cancel_approval(
					owner,
					collection,
					item,
					T::Lookup::unlookup(operator.clone()),
				)
				.map_err(|e| e.with_weight(NftsWeightInfoOf::<T>::cancel_approval()))?;
				NftsWeightInfoOf::<T>::cancel_approval()
			},
			None => {
				NftsOf::<T>::cancel_collection_approval(
					owner,
					collection,
					T::Lookup::unlookup(operator.clone()),
				)
				.map_err(|e| e.with_weight(NftsWeightInfoOf::<T>::cancel_collection_approval()))?;
				NftsWeightInfoOf::<T>::cancel_collection_approval()
			},
		})
		.into())
	}
}

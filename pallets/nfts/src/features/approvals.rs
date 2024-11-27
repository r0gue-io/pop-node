// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module contains helper functions for the approval logic implemented in the NFTs pallet.
//! The bitflag [`PalletFeature::Approvals`] needs to be set in [`Config::Features`] for NFTs
//! to have the functionality defined in this module.

use frame_support::pallet_prelude::*;

use crate::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Approves the transfer of an item to a delegate.
	///
	/// This function is used to approve the transfer of the specified `item` in the `collection` to
	/// a `delegate`. If `maybe_check_origin` is specified, the function ensures that the
	/// `check_origin` account is the owner of the item, granting them permission to approve the
	/// transfer. The `delegate` is the account that will be allowed to take control of the item.
	/// Optionally, a `deadline` can be specified to set a time limit for the approval. The
	/// `deadline` is expressed in block numbers and is added to the current block number to
	/// determine the absolute deadline for the approval. After approving the transfer, the function
	/// emits the `TransferApproved` event.
	///
	/// - `maybe_check_origin`: The optional account that is required to be the owner of the item,
	///   granting permission to approve the transfer. If `None`, no permission check is performed.
	/// - `collection`: The identifier of the collection containing the item to be transferred.
	/// - `item`: The identifier of the item to be transferred.
	/// - `delegate`: The account that will be allowed to take control of the item.
	/// - `maybe_deadline`: The optional deadline (in block numbers) specifying the time limit for
	///   the approval.
	pub(crate) fn do_approve_transfer(
		maybe_check_origin: Option<T::AccountId>,
		collection: T::CollectionId,
		item: T::ItemId,
		delegate: T::AccountId,
		maybe_deadline: Option<frame_system::pallet_prelude::BlockNumberFor<T>>,
	) -> DispatchResult {
		ensure!(
			Self::is_pallet_feature_enabled(PalletFeature::Approvals),
			Error::<T, I>::MethodDisabled
		);
		let mut details = Item::<T, I>::get(collection, item).ok_or(Error::<T, I>::UnknownItem)?;

		let collection_config = Self::get_collection_config(&collection)?;
		ensure!(
			collection_config.is_setting_enabled(CollectionSetting::TransferableItems),
			Error::<T, I>::ItemsNonTransferable
		);

		if let Some(check_origin) = maybe_check_origin {
			ensure!(check_origin == details.owner, Error::<T, I>::NoPermission);
		}

		let now = frame_system::Pallet::<T>::block_number();
		let deadline = maybe_deadline.map(|d| d.saturating_add(now));

		details
			.approvals
			.try_insert(delegate.clone(), deadline)
			.map_err(|_| Error::<T, I>::ReachedApprovalLimit)?;
		Item::<T, I>::insert(collection, item, &details);

		Self::deposit_event(Event::TransferApproved {
			collection,
			item: Some(item),
			owner: details.owner,
			delegate,
			deadline,
		});
		Ok(())
	}

	/// Cancels the approval for the transfer of an item to a delegate.
	///
	/// This function is used to cancel the approval for the transfer of the specified `item` in the
	/// `collection` to a `delegate`. If `maybe_check_origin` is specified, the function ensures
	/// that the `check_origin` account is the owner of the item or that the approval is past its
	/// deadline, granting permission to cancel the approval. After canceling the approval, the
	/// function emits the `ApprovalCancelled` event.
	///
	/// - `maybe_check_origin`: The optional account that is required to be the owner of the item or
	///   that the approval is past its deadline, granting permission to cancel the approval. If
	///   `None`, no permission check is performed.
	/// - `collection`: The identifier of the collection containing the item.
	/// - `item`: The identifier of the item.
	/// - `delegate`: The account that was previously allowed to take control of the item.
	pub(crate) fn do_cancel_approval(
		maybe_check_origin: Option<T::AccountId>,
		collection: T::CollectionId,
		item: T::ItemId,
		delegate: T::AccountId,
	) -> DispatchResult {
		let mut details = Item::<T, I>::get(collection, item).ok_or(Error::<T, I>::UnknownItem)?;

		ensure!(
			!CollectionApprovals::<T, I>::contains_key((collection, &details.owner, &delegate)),
			Error::<T, I>::DelegateApprovalConflict
		);

		let maybe_deadline = details.approvals.get(&delegate).ok_or(Error::<T, I>::NotDelegate)?;

		let is_past_deadline = if let Some(deadline) = maybe_deadline {
			let now = frame_system::Pallet::<T>::block_number();
			now > *deadline
		} else {
			false
		};

		if !is_past_deadline {
			if let Some(check_origin) = maybe_check_origin {
				ensure!(check_origin == details.owner, Error::<T, I>::NoPermission);
			}
		}

		details.approvals.remove(&delegate);
		Item::<T, I>::insert(collection, item, &details);

		Self::deposit_event(Event::ApprovalCancelled {
			collection,
			item: Some(item),
			owner: details.owner,
			delegate,
		});

		Ok(())
	}

	/// Clears all transfer approvals for an item.
	///
	/// This function is used to clear all transfer approvals for the specified `item` in the
	/// `collection`. If `maybe_check_origin` is specified, the function ensures that the
	/// `check_origin` account is the owner of the item, granting permission to clear all transfer
	/// approvals. After clearing all approvals, the function emits the `AllApprovalsCancelled`
	/// event.
	///
	/// - `maybe_check_origin`: The optional account that is required to be the owner of the item,
	///   granting permission to clear all transfer approvals. If `None`, no permission check is
	///   performed.
	/// - `collection`: The collection ID containing the item.
	/// - `item`: The item ID for which transfer approvals will be cleared.
	pub(crate) fn do_clear_all_transfer_approvals(
		maybe_check_origin: Option<T::AccountId>,
		collection: T::CollectionId,
		item: T::ItemId,
	) -> DispatchResult {
		let collection_details =
			Collection::<T, I>::get(collection).ok_or(Error::<T, I>::UnknownCollection)?;

		ensure!(
			CollectionApprovalCount::<T, I>::get(collection, Some(collection_details.owner)) == 0,
			Error::<T, I>::DelegateApprovalConflict
		);

		let mut details =
			Item::<T, I>::get(collection, item).ok_or(Error::<T, I>::UnknownCollection)?;

		if let Some(check_origin) = maybe_check_origin {
			ensure!(check_origin == details.owner, Error::<T, I>::NoPermission);
		}

		details.approvals.clear();
		Item::<T, I>::insert(collection, item, &details);

		Self::deposit_event(Event::AllApprovalsCancelled {
			collection,
			item: Some(item),
			owner: details.owner,
		});

		Ok(())
	}

	/// Approves the transfer of items in the collection that owned by the origin to a delegate.
	///
	/// This function is used to approve the transfer of items in the `collection` that owned by the
	/// `origin` to a `delegate`.The `delegate` is the account that will be allowed to take control
	/// of items in the collection that owned by the `origin`. Optionally, a `deadline` can be
	/// specified to set a time limit for the approval. The `deadline` is expressed in block
	/// numbers and is added to the current block number to determine the absolute deadline for the
	/// approval. After approving the transfer, the function emits the `TransferApproved` event.
	///
	/// - `origin`: The account grants permission to approve the transfer.
	/// - `collection`: The identifier of the collection.
	/// - `delegate`: The account that will be allowed to take control of items in the collection
	///   that owned by the `origin`.
	/// - `maybe_deadline`: The optional deadline (in block numbers) specifying the time limit for
	///   the approval.
	pub(crate) fn do_approve_collection_transfer(
		origin: T::AccountId,
		collection: T::CollectionId,
		delegate: T::AccountId,
		maybe_deadline: Option<frame_system::pallet_prelude::BlockNumberFor<T>>,
	) -> DispatchResult {
		ensure!(
			Self::is_pallet_feature_enabled(PalletFeature::Approvals),
			Error::<T, I>::MethodDisabled
		);

		let collection_config = Self::get_collection_config(&collection)?;
		ensure!(
			collection_config.is_setting_enabled(CollectionSetting::TransferableItems),
			Error::<T, I>::ItemsNonTransferable
		);
		let now = frame_system::Pallet::<T>::block_number();
		let deadline = maybe_deadline.map(|d| d.saturating_add(now));

		CollectionApprovals::<T, I>::try_mutate_exists(
			(&collection, &origin, &delegate),
			|maybe_approval| -> Result<(), DispatchError> {
				if maybe_approval.is_none() {
					// Increment approval counts for the `origin`, ensuring limits are respected.
					CollectionApprovalCount::<T, I>::try_mutate(
						collection,
						Some(&origin),
						|approvals| -> Result<(), DispatchError> {
							ensure!(
								*approvals < T::ApprovalsLimit::get(),
								Error::<T, I>::ReachedApprovalLimit
							);
							approvals.saturating_inc();
							Ok(())
						},
					)?;

					// Increment the total approval count for the collection.
					CollectionApprovalCount::<T, I>::mutate(
						collection,
						Option::<T::AccountId>::None,
						|approvals| approvals.saturating_inc(),
					);
				}
				*maybe_approval = Some(deadline);

				Ok(())
			},
		)?;

		Self::deposit_event(Event::TransferApproved {
			collection,
			item: None,
			owner: origin,
			delegate,
			deadline,
		});

		Ok(())
	}

	/// Cancels the transfer of items in the collection that owned by the origin to
	/// a delegate.
	///
	/// This function is used to cancel the approval for the transfer of items in the `collection`
	/// that owned by the `origin` to a `delegate`. After canceling the approval, the function emits
	/// the `ApprovalCancelled` event.
	///
	/// - `origin`: The account grants permission to cancel the transfer.
	/// - `collection`: The identifier of the collection.
	/// - `delegate`: The account that was previously allowed to take control of items in the
	///   collection that owned by the origin.
	pub(crate) fn do_cancel_collection_approval(
		origin: T::AccountId,
		collection: T::CollectionId,
		delegate: T::AccountId,
	) -> DispatchResult {
		CollectionApprovals::<T, I>::take((&collection, &origin, &delegate))
			.ok_or(Error::<T, I>::UnknownCollection)?;
		CollectionApprovalCount::<T, I>::mutate(collection, Some(&origin), |approvals| {
			approvals.saturating_dec();
		});
		CollectionApprovalCount::<T, I>::mutate(
			collection,
			Option::<T::AccountId>::None,
			|approvals| approvals.saturating_dec(),
		);

		Self::deposit_event(Event::ApprovalCancelled {
			collection,
			owner: origin,
			item: None,
			delegate,
		});

		Ok(())
	}

	/// Clears all collection approvals.
	///
	/// This function is used to clear all approvals to transfer items in the `collection` that
	/// owned by the `origin` to a `delegate`. After clearing all approvals, the function emits the
	/// `AllApprovalsCancelled` event.
	///
	/// - `origin`: The account grants permission to clear the transfer.
	/// - `collection`: The collection ID containing the item.
	/// - `witness_approvals`: Information on the collection approvals cleared. This must be
	///   correct.
	pub(crate) fn do_clear_all_collection_approvals(
		origin: T::AccountId,
		collection: T::CollectionId,
		witness_approvals: u32,
	) -> DispatchResult {
		let approvals = CollectionApprovalCount::<T, I>::take(collection, Some(&origin));
		ensure!(approvals == witness_approvals, Error::<T, I>::BadWitness);
		let _ = CollectionApprovals::<T, I>::clear_prefix((collection, &origin), approvals, None);
		CollectionApprovalCount::<T, I>::mutate(
			collection,
			Option::<T::AccountId>::None,
			|total_approvals| *total_approvals = total_approvals.saturating_sub(approvals),
		);

		Self::deposit_event(Event::AllApprovalsCancelled { collection, item: None, owner: origin });

		Ok(())
	}

	/// Checks whether the `delegate` has the necessary allowance to transfer items in the
	/// `collection` that owned by the `account`. If the `delegate` has an approval to
	/// transfer items in the collection that owned by the `account`, they can transfer every item
	/// without requiring explicit approval for that item.
	///
	/// - `collection`: The identifier of the collection
	/// - `account`: The account that granted the permission for `delegate` to transfer items in the
	///   `collection`.
	/// - `delegate`: The account that was previously allowed to take control of items in the
	///   collection that owned by the `account`.
	fn check_collection_approval(
		collection: &T::CollectionId,
		account: &T::AccountId,
		delegate: &T::AccountId,
	) -> Result<(), DispatchError> {
		let maybe_deadline = CollectionApprovals::<T, I>::get((&collection, &account, &delegate))
			.ok_or(Error::<T, I>::NoPermission)?;
		if let Some(deadline) = maybe_deadline {
			let block_number = frame_system::Pallet::<T>::block_number();
			ensure!(block_number <= deadline, Error::<T, I>::ApprovalExpired);
		}
		Ok(())
	}

	/// Checks whether the `delegate` has the necessary allowance to transfer items within the
	/// collection or a specific item in the collection. If the `delegate` has an approval to
	/// transfer items in the collection that owned by the `account`, they can transfer every item
	/// without requiring explicit approval for that item.
	///
	/// - `collection`: The identifier of the collection
	/// - `maybe_item`: The optional item of the collection that the delegated account has an
	///   approval to transfer. If not provided, an approval to transfer items in the collection
	///   that owned by the `account` will be checked.
	/// - `account`: The account that granted the permission for `delegate` to transfer items in the
	///   `collection` or the owner of the specified collection item.
	/// - `delegate`: The account that was previously allowed to take control of items in the
	///   collection that owned by the `owner`.
	pub fn check_approval(
		collection: &T::CollectionId,
		maybe_item: &Option<T::ItemId>,
		account: &T::AccountId,
		delegate: &T::AccountId,
	) -> Result<(), DispatchError> {
		// Check if a `delegate` has a permission to transfer items in the collection that owned by
		// the `owner`.
		let error = match Self::check_collection_approval(collection, account, delegate) {
			Ok(()) => return Ok(()),
			Err(error) => error,
		};
		// Check if a `delegate` has a permission to transfer the collection item.
		if let Some(item) = maybe_item {
			let details = Item::<T, I>::get(collection, item).ok_or(Error::<T, I>::UnknownItem)?;

			let maybe_deadline =
				details.approvals.get(delegate).ok_or(Error::<T, I>::NoPermission)?;
			if let Some(deadline) = maybe_deadline {
				let block_number = frame_system::Pallet::<T>::block_number();
				ensure!(block_number <= *deadline, Error::<T, I>::ApprovalExpired);
			}
			return Ok(());
		};
		Err(error)
	}
}

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
use frame_system::pallet_prelude::BlockNumberFor;

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
		maybe_deadline: Option<BlockNumberFor<T>>,
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

		// Cannot revoke approval for a specific collection item if the delegate already has
		// permission to transfer all items owned by the origin in the collection.
		ensure!(
			!CollectionApprovals::<T, I>::contains_key((collection, &details.owner, &delegate)),
			Error::<T, I>::DelegateApprovalConflict
		);

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
		let mut details =
			Item::<T, I>::get(collection, item).ok_or(Error::<T, I>::UnknownCollection)?;

		if let Some(check_origin) = maybe_check_origin {
			ensure!(
				CollectionApprovals::<T, I>::iter_prefix((collection, &check_origin))
					.take(1)
					.next()
					.is_none(),
				Error::<T, I>::DelegateApprovalConflict
			);
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
	/// `origin` to a `delegate`. The `delegate` is the account that will be allowed to take control
	/// of items in the collection. Optionally, a `deadline` can be specified to set a time limit
	/// for the approval. The `deadline` is expressed in block numbers and is added to the current
	/// block number to determine the absolute deadline for the approval. After approving the
	/// transfer, the function emits the `TransferApproved` event.
	///
	/// This function reserves the required deposit from the `origin` account. If an approval
	/// already exists, the new amount is added to such existing approval.
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
		maybe_deadline: Option<BlockNumberFor<T>>,
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
			|maybe_approval| -> DispatchResult {
				let deposit_required = T::CollectionApprovalDeposit::get();
				let mut current_deposit = match maybe_approval.take() {
					Some((_, deposit)) => deposit,
					None => Zero::zero(),
				};

				if current_deposit < deposit_required {
					T::Currency::reserve(&origin, deposit_required - current_deposit)?;
					current_deposit = deposit_required;
				}
				*maybe_approval = Some((deadline, current_deposit));
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

	/// Cancels the transfer of items in the collection that owned by the origin to a delegate.
	///
	/// This function is used to cancel the approval for the transfer of items in the `collection`
	/// that owned by the `origin` to a `delegate`. After canceling the approval, the function
	/// returns the `origin` back the deposited fund and emits the `ApprovalCancelled` event.
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
		let (_, deposit) = CollectionApprovals::<T, I>::take((&collection, &origin, &delegate))
			.ok_or(Error::<T, I>::UnknownCollection)?;

		T::Currency::unreserve(&origin, deposit);

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
	/// owned by the `origin` to a `delegate`. After clearing all approvals, the function returns
	/// the `origin` back the deposited fund of each collection approval and emits the
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
		let mut removed_approvals: u32 = 0;
		// Iterate and remove each collection approval, return the deposited fund back to the
		// `origin`.
		for (_, (_, deposit)) in CollectionApprovals::<T, I>::drain_prefix((collection, &origin)) {
			T::Currency::unreserve(&origin, deposit);
			removed_approvals.saturating_inc();
		}
		ensure!(removed_approvals == witness_approvals, Error::<T, I>::BadWitness);
		Self::deposit_event(Event::AllApprovalsCancelled { collection, item: None, owner: origin });
		Ok(())
	}

	/// Checks whether the `delegate` is approved to transfer items in the `collection` that owned
	/// by the `account`.
	///
	/// - `collection`: The identifier of the collection.
	/// - `account`: The account that granted the permission for `delegate` to transfer items.
	/// - `delegate`: The account that was previously allowed to take control of items in the
	///   `collection` that owned by the `account`.
	fn check_collection_approval(
		collection: &T::CollectionId,
		account: &T::AccountId,
		delegate: &T::AccountId,
	) -> DispatchResult {
		let (maybe_deadline, _) =
			CollectionApprovals::<T, I>::get((&collection, &account, &delegate))
				.ok_or(Error::<T, I>::NoPermission)?;
		if let Some(deadline) = maybe_deadline {
			let block_number = frame_system::Pallet::<T>::block_number();
			ensure!(block_number <= deadline, Error::<T, I>::ApprovalExpired);
		}
		Ok(())
	}

	/// Checks whether the `delegate` is approved by the `account` to transfer items that owned by
	/// the `account` or a specific item in the collection. If the `delegate` has
	/// an approval to transfer items in the collection that owned by the `account`, they can
	/// transfer every item without requiring explicit approval for that item.
	///
	/// - `collection`: The identifier of the collection.
	/// - `maybe_item`: The optional item of the collection that the delegated account has an
	///   approval to transfer. If not provided, an approval to transfer items in the collection
	///   that owned by the `account` will be checked.
	/// - `account`: The account that granted the permission for `delegate` to transfer items in the
	///   `collection` or the owner of the specified collection item.
	/// - `delegate`: The account that was previously allowed to take control of items in the
	///   collection that owned by the `account` or the specified collection item.
	pub fn check_approval(
		collection: &T::CollectionId,
		maybe_item: &Option<T::ItemId>,
		account: &T::AccountId,
		delegate: &T::AccountId,
	) -> DispatchResult {
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

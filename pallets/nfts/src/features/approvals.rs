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

		let deadline =
			maybe_deadline.map(|d| d.saturating_add(frame_system::Pallet::<T>::block_number()));

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

		// Cannot revoke approval for a specific collection item if the delegate has
		// permission to transfer all collection items owned by the origin.
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
	/// - `collection`: The identifier of the collection.
	/// - `item`: The item ID for which transfer approvals will be cleared.
	pub(crate) fn do_clear_all_transfer_approvals(
		maybe_check_origin: Option<T::AccountId>,
		collection: T::CollectionId,
		item: T::ItemId,
	) -> DispatchResult {
		let mut details =
			Item::<T, I>::get(collection, item).ok_or(Error::<T, I>::UnknownCollection)?;

		if let Some(check_origin) = maybe_check_origin {
			// Cannot revoke approvals for individual items when there are existing approvals to
			// transfer all items in the collection owned by the origin.
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
			item,
			owner: details.owner,
		});

		Ok(())
	}

	/// Approves the transfer of all collection items of `owner` to a `delegate`.
	///
	/// This function is used to approve the transfer of all (current and future) collection items
	/// of `owner` to a `delegate`. The `delegate` account will be allowed to take control of the
	/// items. Optionally, a `deadline` can be specified to set a time limit for the approval. The
	/// `deadline` is expressed in block numbers and is added to the current block number to
	/// determine the absolute deadline for the approval. After approving the transfer, the
	/// function emits the `TransferApproved` event.
	///
	/// This function reserves the necessary deposit from the owner account. If an approval already
	/// exists, additional funds are reserved only if the updated deposit exceeds the currently
	/// reserved amount. The existing approval's deadline is also updated.
	///
	/// - `owner`: The owner of the collection items.
	/// - `collection`: The identifier of the collection.
	/// - `delegate`: The account that will be approved to take control of the collection items.
	/// - `deposit`: The reserved amount for granting a collection approval.
	/// - `maybe_deadline`: The optional deadline (in block numbers) specifying the time limit for
	///   the approval.
	pub(crate) fn do_approve_collection_transfer(
		owner: T::AccountId,
		collection: T::CollectionId,
		delegate: T::AccountId,
		deposit: DepositBalanceOf<T, I>,
		maybe_deadline: Option<BlockNumberFor<T>>,
	) -> DispatchResult {
		ensure!(
			Self::is_pallet_feature_enabled(PalletFeature::Approvals),
			Error::<T, I>::MethodDisabled
		);
		ensure!(
			AccountBalance::<T, I>::get(collection, &owner)
				.filter(|(balance, _)| !balance.is_zero())
				.is_some(),
			Error::<T, I>::NoItemOwned
		);

		let collection_config = Self::get_collection_config(&collection)?;
		ensure!(
			collection_config.is_setting_enabled(CollectionSetting::TransferableItems),
			Error::<T, I>::ItemsNonTransferable
		);
		let deadline =
			maybe_deadline.map(|d| d.saturating_add(frame_system::Pallet::<T>::block_number()));

		CollectionApprovals::<T, I>::try_mutate_exists(
			(&collection, &owner, &delegate),
			|maybe_approval| -> DispatchResult {
				let current_deposit =
					maybe_approval.map(|(_, deposit)| deposit).unwrap_or_default();
				T::Currency::reserve(&owner, deposit.saturating_sub(current_deposit))?;
				*maybe_approval = Some((deadline, deposit));
				Ok(())
			},
		)?;

		Self::deposit_event(Event::TransferApproved {
			collection,
			item: None,
			owner,
			delegate,
			deadline,
		});

		Ok(())
	}

	/// Cancels the transfer of all `collection` items of `owner` to a `delegate`.
	///
	/// This function is used to cancel the approval for the transfer of the collection items of
	/// `owner` to a `delegate`. After canceling the approval, the deposit is returned to the
	/// `owner` account and the `ApprovalCancelled` event is emitted.
	///
	/// - `owner`: The owner of the collection items.
	/// - `collection`: The identifier of the collection.
	/// - `delegate`: The account that had permission to transfer collection items.
	pub(crate) fn do_cancel_collection_approval(
		owner: T::AccountId,
		collection: T::CollectionId,
		delegate: T::AccountId,
	) -> DispatchResult {
		let (_, deposit) = CollectionApprovals::<T, I>::take((&collection, &owner, &delegate))
			.ok_or(Error::<T, I>::NotDelegate)?;

		T::Currency::unreserve(&owner, deposit);

		Self::deposit_event(Event::ApprovalCancelled { collection, owner, item: None, delegate });

		Ok(())
	}

	/// Clears all collection approvals.
	///
	/// This function is used to clear `limit` collection approvals for the
	/// collection items of `owner`. After clearing the approvals, the deposit of each collection
	/// approval is returned to the `owner` account and the `ApprovalsCancelled` event is
	/// emitted.
	///
	/// - `owner`: The owner of the collection items.
	/// - `collection`: The identifier of the collection.
	/// - `limit`: The amount of collection approvals that will be cleared.
	pub(crate) fn do_clear_collection_approvals(
		owner: T::AccountId,
		collection: T::CollectionId,
		limit: u32,
	) -> Result<u32, DispatchError> {
		if limit == 0 {
			return Ok(0);
		}
		let mut removed_approvals: u32 = 0;
		let mut deposits: BalanceOf<T, I> = Zero::zero();
		// Iterate and remove each collection approval, returning the deposit back to the `owner`.
		for (_, (_, deposit)) in
			CollectionApprovals::<T, I>::drain_prefix((collection, &owner)).take(limit as usize)
		{
			deposits.saturating_accrue(deposit);
			removed_approvals.saturating_inc();
		}

		T::Currency::unreserve(&owner, deposits);
		Self::deposit_event(Event::ApprovalsCancelled {
			collection,
			owner,
			approvals: removed_approvals,
		});
		Ok(removed_approvals)
	}

	/// Checks whether the `delegate` has permission to transfer collection items of `owner`.
	///
	/// - `collection`: The identifier of the collection.
	/// - `owner`: The owner of the collection items.
	/// - `delegate`: The account to check for permission to transfer collection items of `owner`.
	fn check_collection_approval_permission(
		collection: &T::CollectionId,
		owner: &T::AccountId,
		delegate: &T::AccountId,
	) -> DispatchResult {
		let (maybe_deadline, _) =
			CollectionApprovals::<T, I>::get((&collection, &owner, &delegate))
				.ok_or(Error::<T, I>::NoPermission)?;
		if let Some(deadline) = maybe_deadline {
			let block_number = frame_system::Pallet::<T>::block_number();
			ensure!(block_number <= deadline, Error::<T, I>::ApprovalExpired);
		}
		Ok(())
	}

	/// Checks whether the `delegate` has permission to transfer `owner`'s collection item(s).
	/// If the `delegate` has permission to transfer all `owner`'s collection items, they can
	/// transfer any item without needing explicit approval for each individual item.
	///
	/// - `collection`: The identifier of the collection.
	/// - `maybe_item`: The optional item of the collection that the delegated account has
	///   permission to transfer. If not provided, permission to transfer all `owner`'s collection
	///   items will be checked.
	/// - `owner`: The owner of the specified collection item.
	/// - `delegate`: The account to check for permission to transfer collection item(s) from the
	///   owner.
	pub fn check_approval_permission(
		collection: &T::CollectionId,
		maybe_item: &Option<T::ItemId>,
		owner: &T::AccountId,
		delegate: &T::AccountId,
	) -> DispatchResult {
		// Check if `delegate` has permission to transfer `owner`'s collection items.
		let Err(error) = Self::check_collection_approval_permission(collection, owner, delegate)
		else {
			return Ok(());
		};

		// Check if a `delegate` has permission to transfer the given collection item.
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

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
			AccountApprovals::<T, I>::get(collection, collection_details.owner) == 0,
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

	/// Approves the transfer of all items within the collection to a delegate.
	///
	/// This function is used to approve the transfer of all items within the `collection` to
	/// a `delegate`. If `maybe_check_origin` is specified, the function ensures that the
	/// `check_origin` account is the owner of the collection, granting them permission to approve
	/// the transfer. The `delegate` is the account that will be allowed to take control of all
	/// items within the collection. Optionally, a `deadline` can be specified to set a time limit
	/// for the approval. The `deadline` is expressed in block numbers and is added to the current
	/// block number to determine the absolute deadline for the approval. After approving the
	/// transfer, the function emits the `TransferApproved` event.
	///
	/// - `maybe_check_origin`: The optional account that is required to be the owner of the item,
	///   granting permission to approve the transfer. If `None`, no permission check is performed.
	/// - `collection`: The identifier of the collection.
	/// - `delegate`: The account that will be allowed to take control of all items within the
	///   collection.
	/// - `maybe_deadline`: The optional deadline (in block numbers) specifying the time limit for
	///   the approval.
	pub(crate) fn do_approve_collection(
		maybe_check_origin: Option<T::AccountId>,
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

		let (owner, deadline) = Collection::<T, I>::try_mutate(
			collection,
			|maybe_collection_details| -> Result<(T::AccountId, Option<BlockNumberFor<T>>), DispatchError> {
				let collection_details =
					maybe_collection_details.as_mut().ok_or(Error::<T, I>::UnknownCollection)?;
				let owner = collection_details.clone().owner;

				if let Some(check_origin) = maybe_check_origin {
					ensure!(check_origin == owner, Error::<T, I>::NoPermission);
				}
				let now = frame_system::Pallet::<T>::block_number();
				let deadline = maybe_deadline.map(|d| d.saturating_add(now));

				AccountApprovals::<T, I>::try_mutate(
					collection,
					&owner,
					|allowances| -> Result<(), DispatchError> {
						ensure!(
							*allowances < T::ApprovalsLimit::get(),
							Error::<T, I>::ReachedApprovalLimit
						);
						CollectionApprovals::<T, I>::insert((&collection, &owner, &delegate), deadline);
						allowances.saturating_inc();
						Ok(())
					},
				)?;
				Ok((owner, deadline))
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

	/// Cancels the approval for the transfer of all items within the collection to a delegate.
	///
	/// This function is used to cancel the approval for the transfer of all items in the
	/// `collection` to a `delegate`. If `maybe_check_origin` is specified, the function ensures
	/// that the `check_origin` account is the owner of the item or that the approval is past its
	/// deadline, granting permission to cancel the approval. After canceling the approval, the
	/// function emits the `ApprovalCancelled` event.
	///
	/// - `maybe_check_origin`: The optional account that is required to be the owner of the
	///   collection or that the approval is past its deadline, granting permission to cancel the
	///   approval. If `None`, no permission check is performed.
	/// - `collection`: The identifier of the collection
	/// - `delegate`: The account that was previously allowed to take control of all items within
	///   the collection.
	pub(crate) fn do_cancel_collection(
		maybe_check_origin: Option<T::AccountId>,
		collection: T::CollectionId,
		delegate: T::AccountId,
	) -> DispatchResult {
		let owner = Collection::<T, I>::try_mutate(
			collection,
			|maybe_collection_details| -> Result<T::AccountId, DispatchError> {
				let collection_details =
					maybe_collection_details.as_mut().ok_or(Error::<T, I>::UnknownCollection)?;
				let owner = collection_details.clone().owner;
				let maybe_deadline =
					CollectionApprovals::<T, I>::get((&collection, &owner, &delegate))
						.ok_or(Error::<T, I>::NotDelegate)?;

				let is_past_deadline = if let Some(deadline) = maybe_deadline {
					let now = frame_system::Pallet::<T>::block_number();
					now > deadline
				} else {
					false
				};

				if !is_past_deadline {
					if let Some(check_origin) = maybe_check_origin {
						ensure!(check_origin == owner, Error::<T, I>::NoPermission);
					}
				}

				CollectionApprovals::<T, I>::remove((&collection, &owner, &delegate));
				AccountApprovals::<T, I>::mutate(collection, &owner, |allowances| {
					allowances.saturating_dec();
				});
				Ok(owner)
			},
		)?;

		Self::deposit_event(Event::ApprovalCancelled { collection, owner, item: None, delegate });

		Ok(())
	}

	/// Clears all collection approvals.
	///
	/// This function is used to clear all approvals to transfer all items within the collections.
	/// If `maybe_check_origin` is specified, the function ensures that the `check_origin` account
	/// is the owner of the item, granting permission to clear all collection approvals. After
	/// clearing all approvals, the function emits the `AllApprovalsCancelled` event.
	///
	/// - `maybe_check_origin`: The optional account that is required to be the owner of the
	///   collection, granting permission to clear all collection approvals. If `None`, no
	///   permission check is performed.
	/// - `collection`: The collection ID containing the item.
	/// - `witness_allowances`: Information on the collection approvals cleared. This must be
	///   correct.
	pub(crate) fn do_clear_all_collection_approvals(
		maybe_check_origin: Option<T::AccountId>,
		collection: T::CollectionId,
		witness_allowances: u32,
	) -> DispatchResult {
		let owner = Collection::<T, I>::try_mutate(
			collection,
			|maybe_collection_details| -> Result<T::AccountId, DispatchError> {
				let collection_details =
					maybe_collection_details.as_mut().ok_or(Error::<T, I>::UnknownCollection)?;
				let owner = collection_details.clone().owner;

				if let Some(check_origin) = maybe_check_origin {
					ensure!(check_origin == owner.clone(), Error::<T, I>::NoPermission);
				}

				AccountApprovals::<T, I>::try_mutate(
					collection,
					&owner,
					|allowances| -> Result<(), DispatchError> {
						ensure!(*allowances == witness_allowances, Error::<T, I>::BadWitness);
						let _ = CollectionApprovals::<T, I>::clear_prefix(
							(collection, owner.clone()),
							*allowances,
							None,
						);
						*allowances = 0;

						Ok(())
					},
				)?;
				Ok(owner)
			},
		)?;

		Self::deposit_event(Event::AllApprovalsCancelled { collection, item: None, owner });

		Ok(())
	}

	/// Checks whether the `delegate` has the necessary allowance to transfer all items within the
	/// collection. If the `delegate` has approval to transfer all items in the collection, they can
	/// transfer every item without requiring explicit approval for that item.
	///
	/// - `collection`: The identifier of the collection
	/// - `owner`: The owner of the collection or the collection item.
	/// - `delegate`: The account that was previously allowed to take control of all items within
	///   the collection.
	fn check_collection_approval(
		collection: &T::CollectionId,
		owner: &T::AccountId,
		delegate: &T::AccountId,
	) -> Result<(), DispatchError> {
		let maybe_deadline = CollectionApprovals::<T, I>::get((&collection, &owner, &delegate))
			.ok_or(Error::<T, I>::NoPermission)?;
		if let Some(deadline) = maybe_deadline {
			let block_number = frame_system::Pallet::<T>::block_number();
			ensure!(block_number <= deadline, Error::<T, I>::ApprovalExpired);
		}
		Ok(())
	}

	/// Checks whether the `delegate` has the necessary allowance to transfer items within the
	/// collection or a specific item in the collection. If the `delegate` has approval to transfer
	/// all items in the collection, they can transfer every item without requiring explicit
	/// approval for that item.
	///
	/// - `collection`: The identifier of the collection
	/// - `maybe_item`: The optional item of the collection that the delegated account has an
	///   approval to transfer. If not provided, an approval to transfer all items within the
	///   collection will be checked.
	/// - `owner`: The owner of the collection or the collection item.
	/// - `delegate`: The account that was previously allowed to take control of all items within
	///   the collection.
	pub fn check_approval(
		collection: &T::CollectionId,
		maybe_item: &Option<T::ItemId>,
		owner: &T::AccountId,
		delegate: &T::AccountId,
	) -> Result<(), DispatchError> {
		// Check if a `delegate` has a permission to spend the collection.
		let check_collection_approval_error =
			match Self::check_collection_approval(collection, owner, delegate) {
				Ok(()) => return Ok(()),
				Err(error) => error,
			};
		// Check if a `delegate` has a permission to spend the collection item.
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
		Err(check_collection_approval_error)
	}
}

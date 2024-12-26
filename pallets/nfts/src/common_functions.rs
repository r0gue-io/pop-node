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

//! Various pieces of common functionality.

use alloc::vec::Vec;

use frame_support::{pallet_prelude::*, sp_runtime::ArithmeticError};

use crate::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Get the owner of the item, if the item exists.
	///
	/// - `collection`: The identifier of the collection.
	/// - `item`: The identifier of the collection item.
	pub fn owner(collection: T::CollectionId, item: T::ItemId) -> Option<T::AccountId> {
		Item::<T, I>::get(collection, item).map(|i| i.owner)
	}

	/// Get the owner of the collection, if the collection exists.
	///
	/// - `collection`: The identifier of the collection.
	pub fn collection_owner(collection: T::CollectionId) -> Option<T::AccountId> {
		Collection::<T, I>::get(collection).map(|i| i.owner)
	}

	/// Get the total number of items in the collection, if the collection exists.
	///
	/// - `collection`: The identifier of the collection.
	pub fn collection_items(collection: T::CollectionId) -> Option<u32> {
		Collection::<T, I>::get(collection).map(|i| i.items)
	}

	/// Get the metadata of the collection item.
	///
	/// - `collection`: The identifier of the collection.
	/// - `item`: The identifier of the collection item.
	pub fn item_metadata(
		collection: T::CollectionId,
		item: T::ItemId,
	) -> Option<BoundedVec<u8, T::StringLimit>> {
		ItemMetadataOf::<T, I>::get(collection, item).map(|metadata| metadata.data)
	}

	/// Validates the signature of the given data with the provided signer's account ID.
	///
	/// # Errors
	///
	/// This function returns a [`WrongSignature`](crate::Error::WrongSignature) error if the
	/// signature is invalid or the verification process fails.
	pub fn validate_signature(
		data: &Vec<u8>,
		signature: &T::OffchainSignature,
		signer: &T::AccountId,
	) -> DispatchResult {
		if signature.verify(&**data, signer) {
			return Ok(())
		}

		// NOTE: for security reasons modern UIs implicitly wrap the data requested to sign into
		// <Bytes></Bytes>, that's why we support both wrapped and raw versions.
		let prefix = b"<Bytes>";
		let suffix = b"</Bytes>";
		let mut wrapped: Vec<u8> = Vec::with_capacity(data.len() + prefix.len() + suffix.len());
		wrapped.extend(prefix);
		wrapped.extend(data);
		wrapped.extend(suffix);

		ensure!(signature.verify(&*wrapped, signer), Error::<T, I>::WrongSignature);

		Ok(())
	}

	pub(crate) fn set_next_collection_id(collection: T::CollectionId) {
		let next_id = collection.increment();
		NextCollectionId::<T, I>::set(next_id);
		Self::deposit_event(Event::NextCollectionIdIncremented { next_id });
	}

	/// Increment the number of items in the `collection` owned by the `owner`. If no entry exists
	/// for the `owner` in `AccountBalance`, create a new record and reserve `deposit_amount` from
	/// the `deposit_account`.
	pub(crate) fn increment_account_balance(
		collection: T::CollectionId,
		owner: &T::AccountId,
		(deposit_account, deposit_amount): (&T::AccountId, DepositBalanceOf<T, I>),
	) -> DispatchResult {
		AccountBalance::<T, I>::mutate(collection, owner, |maybe_balance| -> DispatchResult {
			match maybe_balance {
				None => {
					T::Currency::reserve(deposit_account, deposit_amount)?;
					*maybe_balance = Some((1, (deposit_account.clone(), deposit_amount)));
				},
				Some((balance, _deposit)) => {
					balance.saturating_inc();
				},
			}
			Ok(())
		})
	}

	/// Decrement the number of `collection` items owned by the `owner`. If the `owner`'s item
	/// count reaches zero after the reduction, remove the `AccountBalance` record and unreserve
	/// the deposited funds.
	pub(crate) fn decrement_account_balance(
		collection: T::CollectionId,
		owner: &T::AccountId,
	) -> DispatchResult {
		AccountBalance::<T, I>::try_mutate_exists(
			collection,
			owner,
			|maybe_balance| -> DispatchResult {
				let (balance, (deposit_account, deposit_amount)) =
					maybe_balance.as_mut().ok_or(Error::<T, I>::NoItemOwned)?;

				*balance = balance.checked_sub(1).ok_or(ArithmeticError::Underflow)?;
				if *balance == 0 {
					T::Currency::unreserve(deposit_account, *deposit_amount);
					*maybe_balance = None;
				}
				Ok(())
			},
		)
	}

	#[allow(missing_docs)]
	#[cfg(any(test, feature = "runtime-benchmarks"))]
	pub fn set_next_id(id: T::CollectionId) {
		NextCollectionId::<T, I>::set(Some(id));
	}

	#[cfg(test)]
	pub fn get_next_id() -> T::CollectionId {
		NextCollectionId::<T, I>::get()
			.or(T::CollectionId::initial_value())
			.expect("Failed to get next collection ID")
	}
}

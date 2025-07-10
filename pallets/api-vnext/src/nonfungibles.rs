use frame_support::{
	dispatch::{DispatchResult, DispatchResultWithPostInfo, WithPostDispatchInfo},
	traits::{nonfungibles_v2::Inspect, Incrementable},
	weights::Weight,
	BoundedVec,
};
use frame_system::pallet_prelude::OriginFor;
use pallet_nfts::{
	AccountBalance, Attribute, AttributeNamespace, CancelAttributesApprovalWitness,
	CollectionConfigFor, Config, DepositBalanceOf, DestroyWitness, MintWitness, NextCollectionId,
};

use super::*;

pub mod precompiles;
#[cfg(test)]
mod tests;

type BlockNumberFor<T, I = ()> = pallet_nfts::BlockNumberFor<T, I>;
type CollectionIdOf<T, I = ()> =
	<Nfts<T, I> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;
type ItemIdOf<T, I = ()> = <Nfts<T, I> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;

fn approve<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	operator: AccountIdOf<T>,
	item: Option<ItemIdOf<T, I>>,
	approved: bool,
	deadline: Option<BlockNumberFor<T, I>>,
) -> DispatchResultWithPostInfo {
	// TODO: weights
	let operator_lookup = T::Lookup::unlookup(operator.clone());
	let weight = if approved {
		match item {
			Some(item) => {
				Nfts::<T, I>::approve_transfer(origin, collection, item, operator_lookup, deadline)
					.map_err(|e| e.with_weight(Weight::zero()))?;
				Weight::zero()
			},
			None => {
				Nfts::<T, I>::approve_collection_transfer(
					origin,
					collection,
					operator_lookup,
					deadline,
				)
				.map_err(|e| e.with_weight(Weight::zero()))?;
				Weight::zero()
			},
		}
	} else {
		match item {
			Some(item) => {
				Nfts::<T, I>::cancel_approval(origin, collection, item, operator_lookup)
					.map_err(|e| e.with_weight(Weight::zero()))?;
				Weight::zero()
			},
			None => {
				Nfts::<T, I>::cancel_collection_approval(origin, collection, operator_lookup)
					.map_err(|e| e.with_weight(Weight::zero()))?;
				Weight::zero()
			},
		}
	};
	Ok(Some(weight).into())
}

fn transfer<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	to: AccountIdOf<T>,
	item: ItemIdOf<T, I>,
) -> DispatchResult {
	Nfts::<T, I>::transfer(origin, collection, item, T::Lookup::unlookup(to.clone()))
}

fn create<T: Config<I>, I>(
	origin: OriginFor<T>,
	admin: AccountIdOf<T>,
	config: CollectionConfigFor<T, I>,
) -> DispatchResult {
	Nfts::<T, I>::create(origin, T::Lookup::unlookup(admin.clone()), config)
}

fn destroy<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	witness: DestroyWitness,
) -> DispatchResultWithPostInfo {
	Nfts::<T, I>::destroy(origin, collection, witness)
}

fn set_attribute<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	item: Option<ItemIdOf<T, I>>,
	namespace: AttributeNamespace<AccountIdOf<T>>,
	key: BoundedVec<u8, T::KeyLimit>,
	value: BoundedVec<u8, T::ValueLimit>,
) -> DispatchResult {
	Nfts::<T, I>::set_attribute(origin, collection, item, namespace, key, value)
}

fn clear_attribute<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	item: Option<ItemIdOf<T, I>>,
	namespace: AttributeNamespace<AccountIdOf<T>>,
	key: BoundedVec<u8, T::KeyLimit>,
) -> DispatchResult {
	Nfts::<T, I>::clear_attribute(origin, collection, item, namespace, key)
}

fn set_metadata<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	item: ItemIdOf<T, I>,
	data: BoundedVec<u8, T::StringLimit>,
) -> DispatchResult {
	Nfts::<T, I>::set_metadata(origin, collection, item, data)
}

fn clear_metadata<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	item: ItemIdOf<T, I>,
) -> DispatchResult {
	Nfts::<T, I>::clear_metadata(origin, collection, item)
}

fn approve_item_attributes<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	item: ItemIdOf<T, I>,
	delegate: AccountIdOf<T>,
) -> DispatchResult {
	Nfts::<T, I>::approve_item_attributes(origin, collection, item, T::Lookup::unlookup(delegate))
}

fn cancel_item_attributes_approval<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	item: ItemIdOf<T, I>,
	delegate: AccountIdOf<T>,
	witness: CancelAttributesApprovalWitness,
) -> DispatchResult {
	Nfts::<T, I>::cancel_item_attributes_approval(
		origin,
		collection,
		item,
		T::Lookup::unlookup(delegate),
		witness,
	)
}

fn clear_all_transfer_approvals<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	item: ItemIdOf<T, I>,
) -> DispatchResult {
	Nfts::<T, I>::clear_all_transfer_approvals(origin, collection, item)
}

fn clear_collection_approvals<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	limit: u32,
) -> DispatchResultWithPostInfo {
	Nfts::<T, I>::clear_collection_approvals(origin, collection, limit)
}

fn mint<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	to: AccountIdOf<T>,
	item: ItemIdOf<T, I>,
	witness: Option<MintWitness<ItemIdOf<T, I>, DepositBalanceOf<T, I>>>,
) -> DispatchResult {
	Nfts::<T, I>::mint(origin, collection, item, T::Lookup::unlookup(to.clone()), witness)
}

fn burn<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	item: ItemIdOf<T, I>,
) -> DispatchResult {
	Nfts::<T, I>::burn(origin, collection, item)
}

fn balance_of<T: Config<I>, I>(collection: CollectionIdOf<T, I>, owner: AccountIdOf<T>) -> u32 {
	AccountBalance::<T, I>::get(collection, owner)
		.map(|(balance, _)| balance)
		.unwrap_or_default()
}

fn owner_of<T: Config<I>, I>(
	collection: CollectionIdOf<T, I>,
	item: ItemIdOf<T, I>,
) -> Option<AccountIdOf<T>> {
	Nfts::<T, I>::owner(collection, item)
}

fn allowance<T: Config<I>, I>(
	collection: CollectionIdOf<T, I>,
	owner: AccountIdOf<T>,
	operator: AccountIdOf<T>,
	item: Option<ItemIdOf<T, I>>,
) -> bool {
	Nfts::<T, I>::check_approval_permission(&collection, &item, &owner, &operator).is_ok()
}

fn total_supply<T: Config<I>, I>(collection: CollectionIdOf<T, I>) -> u32 {
	Nfts::<T, I>::collection_items(collection).unwrap_or_default()
}

fn get_attributes<T: Config<I>, I>(
	collection: CollectionIdOf<T, I>,
	item: Option<ItemIdOf<T, I>>,
	namespace: AttributeNamespace<AccountIdOf<T>>,
	key: BoundedVec<u8, T::KeyLimit>,
) -> Option<Vec<u8>> {
	Attribute::<T, I>::get((collection, item, namespace, key)).map(|attribute| attribute.0.into())
}

fn item_metadata<T: Config<I>, I>(
	collection: CollectionIdOf<T, I>,
	item: ItemIdOf<T, I>,
) -> Option<Vec<u8>> {
	Nfts::<T, I>::item_metadata(collection, item).map(|metadata| metadata.into())
}

fn set_collection_metadata<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	data: BoundedVec<u8, T::StringLimit>,
) -> DispatchResult {
	Nfts::<T, I>::set_collection_metadata(origin, collection, data)
}

fn clear_collection_metadata<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
) -> DispatchResult {
	Nfts::<T, I>::clear_collection_metadata(origin, collection)
}

fn set_collection_max_supply<T: Config<I>, I>(
	origin: OriginFor<T>,
	collection: CollectionIdOf<T, I>,
	max_supply: u32,
) -> DispatchResult {
	Nfts::<T, I>::set_collection_max_supply(origin, collection, max_supply)
}

fn next_collection_id<T: Config<I>, I>() -> Option<CollectionIdOf<T, I>> {
	NextCollectionId::<T, I>::get().or(T::CollectionId::initial_value())
}

// TODO: replace with type in pallet_nfts once available in future release
pub struct InlineCollectionIdExtractor;
impl CollectionIdExtractor for InlineCollectionIdExtractor {
	type CollectionId = u32;

	fn collection_id_from_address(addr: &[u8; 20]) -> Result<Self::CollectionId, Error> {
		let bytes: [u8; 4] = addr[0..4].try_into().expect("slice is 4 bytes; qed");
		let collection_index = u32::from_be_bytes(bytes);
		Ok(collection_index)
	}
}
/// Mean of extracting the asset id from the precompile address.
pub trait CollectionIdExtractor {
	type CollectionId;

	fn collection_id_from_address(address: &[u8; 20]) -> Result<Self::CollectionId, Error>;
}

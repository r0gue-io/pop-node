//! Benchmarking setup for pallet_api::nonfungibles

use frame_benchmarking::{account, v2::*};
use frame_support::BoundedVec;
use sp_runtime::traits::Zero;

use super::{
	AttributeNamespace, CollectionIdOf, Config, Inspect, ItemIdOf, NftsInstanceOf, Pallet, Read,
};
use crate::Read as _;

const SEED: u32 = 1;

#[benchmarks(
	where
	<pallet_nfts::Pallet<T, NftsInstanceOf<T>> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId: Zero,
	<pallet_nfts::Pallet<T, NftsInstanceOf<T>> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId: Zero,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	// Storage: `Collection`
	fn total_supply() {
		#[block]
		{
			Pallet::<T>::read(Read::TotalSupply(CollectionIdOf::<T>::zero()));
		}
	}

	#[benchmark]
	// Storage: `AccountBalance`
	fn balance_of() {
		#[block]
		{
			Pallet::<T>::read(Read::BalanceOf {
				collection: CollectionIdOf::<T>::zero(),
				owner: account("Alice", 0, SEED),
			});
		}
	}

	#[benchmark]
	// Storage: `Allowances`, `Item`
	fn allowance() {
		#[block]
		{
			Pallet::<T>::read(Read::Allowance {
				collection: CollectionIdOf::<T>::zero(),
				owner: account("Alice", 0, SEED),
				operator: account("Bob", 0, SEED),
				item: Some(ItemIdOf::<T>::zero()),
			});
		}
	}

	#[benchmark]
	// Storage: `Item`
	fn owner_of() {
		#[block]
		{
			Pallet::<T>::read(Read::OwnerOf {
				collection: CollectionIdOf::<T>::zero(),
				item: ItemIdOf::<T>::zero(),
			});
		}
	}

	#[benchmark]
	// Storage: `Attribute`
	fn get_attribute() {
		#[block]
		{
			Pallet::<T>::read(Read::GetAttribute {
				key: BoundedVec::default(),
				collection: CollectionIdOf::<T>::zero(),
				item: Some(ItemIdOf::<T>::zero()),
				namespace: AttributeNamespace::CollectionOwner,
			});
		}
	}

	#[benchmark]
	// Storage: `Collection`
	fn collection() {
		#[block]
		{
			Pallet::<T>::read(Read::Collection(CollectionIdOf::<T>::zero()));
		}
	}

	#[benchmark]
	// Storage: `NextCollectionId`
	fn next_collection_id() {
		#[block]
		{
			Pallet::<T>::read(Read::NextCollectionId);
		}
	}

	#[benchmark]
	// Storage: `ItemMetadata`
	fn item_metadata() {
		#[block]
		{
			Pallet::<T>::read(Read::ItemMetadata {
				collection: CollectionIdOf::<T>::zero(),
				item: ItemIdOf::<T>::zero(),
			});
		}
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}

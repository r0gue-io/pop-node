//! Benchmarking setup for pallet_api::nonfungibles

use frame_benchmarking::{account, v2::*};
use frame_support::{
	assert_ok,
	traits::tokens::nonfungibles_v2::{Create, Mutate},
	BoundedVec,
};
use frame_system::RawOrigin;
use sp_runtime::traits::Zero;

use super::{
	AttributeNamespace, CollectionIdOf, Config, ItemIdOf, NftsInstanceOf, NftsOf, Pallet, Read,
};
use crate::{
	nonfungibles::{
		AccountIdOf, Call, CollectionConfig, CollectionConfigFor, CollectionSettings, Inspect,
		ItemConfig, ItemSettings, MintSettings,
	},
	Read as _,
};

const SEED: u32 = 1;

#[benchmarks(
	where
	<pallet_nfts::Pallet<T, NftsInstanceOf<T>> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId: Zero,
	<pallet_nfts::Pallet<T, NftsInstanceOf<T>> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId: Zero,
)]
mod benchmarks {
	use super::*;

	// Parameter:
	// - 'a': whether `approved` is true or false.
	// - 'i': whether `item` is provided.
	#[benchmark]
	fn approve(a: Linear<0, 1>, i: Linear<0, 1>) -> Result<(), BenchmarkError> {
		let item_id = ItemIdOf::<T>::zero();
		let collection_id = CollectionIdOf::<T>::zero();
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let operator: AccountIdOf<T> = account("Bob", 0, SEED);

		assert_ok!(
			<NftsOf<T> as Create<AccountIdOf<T>, CollectionConfigFor<T>>>::create_collection(
				&owner,
				&owner,
				&CollectionConfig {
					settings: CollectionSettings::all_enabled(),
					max_supply: None,
					mint_settings: MintSettings::default(),
				}
			)
		);
		assert_ok!(<NftsOf<T> as Mutate<AccountIdOf<T>, ItemConfig>>::mint_into(
			&collection_id,
			&item_id,
			&owner,
			&ItemConfig { settings: ItemSettings::all_enabled() },
			false
		));

		let approved = a == 0;
		let maybe_item = if i == 0 { None } else { Some(item_id) };

		#[extrinsic_call]
		_(RawOrigin::Signed(owner.clone()), collection_id, maybe_item, operator.clone(), approved);

		assert!(
			NftsOf::<T>::check_approval_permission(
				&collection_id,
				&Some(item_id),
				&owner,
				&operator
			)
			.is_ok() == approved
		);

		Ok(())
	}

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
				item: ItemIdOf::<T>::zero(),
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
}

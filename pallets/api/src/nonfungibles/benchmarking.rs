//! Benchmarking setup for pallet_api::nonfungibles

use frame_benchmarking::{account, v2::*};
use frame_support::{
	assert_ok,
	traits::{
		tokens::nonfungibles_v2::{Create, Mutate},
		Currency,
	},
	BoundedVec,
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::traits::{Bounded, StaticLookup, Zero};

use super::{
	AccountIdOf, AttributeNamespace, BalanceOf, Call, CollectionConfig, CollectionConfigOf,
	CollectionIdOf, CollectionSettings, Config, Event, Inspect, ItemConfig, ItemIdOf, ItemSettings,
	MintSettings, NftsInstanceOf, NftsOf, Pallet, Read,
};
use crate::Read as _;

const SEED: u32 = 1;

// See if `generic_event` has been emitted.
fn assert_has_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

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
		let collection_id = CollectionIdOf::<T>::zero();
		let deadline = BlockNumberFor::<T>::max_value();
		let item_id = ItemIdOf::<T>::zero();
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let operator: AccountIdOf<T> = account("Bob", 0, SEED);
		let operator_lookup = T::Lookup::unlookup(operator.clone());
		let origin = RawOrigin::Signed(owner.clone());

		T::Currency::make_free_balance_be(&owner, BalanceOf::<T>::max_value());
		assert_ok!(
			<NftsOf<T> as Create<AccountIdOf<T>, CollectionConfigOf<T>>>::create_collection(
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

		let (approved, maybe_item) = match (a, i) {
			(0, 0) => {
				NftsOf::<T>::approve_collection_transfer(
					origin.clone().into(),
					collection_id,
					operator_lookup,
					None,
				)?;
				(false, None)
			},
			(1, 0) => (true, None),
			(0, 1) => {
				NftsOf::<T>::approve_transfer(
					origin.clone().into(),
					collection_id,
					item_id,
					operator_lookup,
					None,
				)?;
				(false, Some(item_id))
			},
			(1, 1) => (true, Some(item_id)),
			_ => unreachable!("values can only be 0 or 1"),
		};

		#[extrinsic_call]
		_(origin, collection_id, operator.clone(), maybe_item, approved, Some(deadline));

		assert_eq!(
			NftsOf::<T>::check_approval_permission(&collection_id, &maybe_item, &owner, &operator)
				.is_ok(),
			approved
		);

		assert_has_event::<T>(
			Event::Approval {
				owner,
				operator,
				collection: collection_id,
				item: maybe_item,
				approved,
			}
			.into(),
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

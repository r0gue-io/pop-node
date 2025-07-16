//! Benchmarking setup for pallet_api::nonfungibles::precompiles

use alloc::{string::String, vec};

use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok,
	pallet_prelude::IsType,
	sp_runtime::traits::StaticLookup,
	traits::{
		fungible::{Inspect, Mutate},
		Get, Time,
	},
	BoundedVec,
};
use frame_system::pallet_prelude::OriginFor;
use pallet_nfts::{AttributeNamespace, CollectionConfig, CollectionSettings, MintSettings};
use pallet_revive::{
	precompiles::{
		alloy::primitives as alloy,
		run::{CallSetup, WasmModule, H256, U256},
	},
	test_utils::{ALICE_ADDR, BOB_ADDR},
	AddressMapper as _, Origin,
};

use super::{
	precompiles::v0::{INonfungibles, INonfungiblesCalls},
	Config, NextCollectionId,
};
#[cfg(test)]
use crate::mock::ExtBuilder;
use crate::{call_precompile, fixed_address};

const NONFUNGIBLES: u16 = 101;
const ADDRESS: [u8; 20] = fixed_address(NONFUNGIBLES);

type AddressMapper<T> = <T as pallet_revive::Config>::AddressMapper;
type Nfts<T, I> = pallet_nfts::Pallet<T, I>;
type CollectionId<T, I> = <T as pallet_nfts::Config<I>>::CollectionId;
type ItemId<T, I> = <T as pallet_nfts::Config<I>>::ItemId;
type Balances<T> = <T as pallet_revive::Config>::Currency;
type Nonfungibles<T, I> = super::precompiles::v0::Nonfungibles<NONFUNGIBLES, T, I>;

#[instance_benchmarks(
    where
        // Precompiles
        T: pallet_revive::Config<
            Currency: Inspect<<T as frame_system::Config>::AccountId, Balance: Into<U256> + TryFrom<U256>>,
            Hash: IsType<H256>,
            Time: Time<Moment: Into<U256>>
        >,
        // Nonfungibles
        T: pallet_nfts::Config<I, CollectionId: Default + From<u32> + Into<u32>, ItemId: Default + From<u32>>
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn approve() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let operator = <AddressMapper<T>>::to_account_id(&BOB_ADDR);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::approve_1(INonfungibles::approve_1Call {
			collection: collection_id.into(),
			item: item_id.into(),
			operator: <AddressMapper<T>>::to_address(&operator).0.into(),
			approved: true,
			deadline: 0,
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}

		// Verify approval was set
		assert!(check_allowance::<T, I>(collection_id, owner, operator, Some(item_id)));
	}

	#[benchmark]
	fn transfer() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let to = <AddressMapper<T>>::to_account_id(&BOB_ADDR);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::transfer(INonfungibles::transferCall {
			collection: collection_id.into(),
			to: <AddressMapper<T>>::to_address(&to).0.into(),
			item: item_id.into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}

		// Verify transfer occurred
		assert_eq!(get_owner_of::<T, I>(collection_id, item_id), Some(to));
	}

	#[benchmark]
	fn create() {
		let admin = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let creator = <AddressMapper<T>>::to_account_id(&BOB_ADDR);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(creator.clone()));
		let mut ext = call_setup.ext().0;
		let config = default_collection_config::<T, I>();
		let config_bytes = codec::Encode::encode(&config);
		let input = INonfungiblesCalls::create(INonfungibles::createCall {
			admin: <AddressMapper<T>>::to_address(&admin).0.into(),
			config: config_bytes.into(),
		});

		#[block]
		{
			let collection_id =
				call_precompile::<Nonfungibles<T, I>, _, u32>(&mut ext, &ADDRESS, &input).unwrap();
			assert_eq!(collection_id, 0);
		}
	}

	#[benchmark]
	fn destroy() {
		let collection_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let witness =
			pallet_nfts::DestroyWitness { item_metadatas: 0, item_configs: 0, attributes: 0 };

		create_collection::<T, I>(owner.clone());

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::destroy(INonfungibles::destroyCall {
			collection: collection_id.into(),
			witness: codec::Encode::encode(&witness).into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn set_attribute() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let key = b"test_key";
		let value = b"test_value";
		let namespace: AttributeNamespace<AccountIdOf<T>> = AttributeNamespace::CollectionOwner;
		let namespace_bytes = codec::Encode::encode(&namespace);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::setAttribute_1(INonfungibles::setAttribute_1Call {
			collection: collection_id.into(),
			item: item_id.into(),
			namespace: namespace_bytes.clone().into(),
			key: key.to_vec().into(),
			value: value.to_vec().into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn clear_attribute() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let key = b"test_key";
		let namespace: AttributeNamespace<AccountIdOf<T>> = AttributeNamespace::CollectionOwner;
		let namespace_bytes = codec::Encode::encode(&namespace);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);
		set_item_attribute::<T, I>(collection_id, Some(item_id), key, b"test_value");

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::clearAttribute_1(INonfungibles::clearAttribute_1Call {
			collection: collection_id.into(),
			item: item_id.into(),
			namespace: namespace_bytes.clone().into(),
			key: key.to_vec().into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn set_metadata() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let metadata = b"test metadata";

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::setMetadata_0(INonfungibles::setMetadata_0Call {
			collection: collection_id.into(),
			item: item_id.into(),
			data: metadata.to_vec().into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn clear_metadata() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);
		set_item_metadata::<T, I>(owner.clone(), collection_id, item_id, b"test metadata");

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::clearMetadata_1(INonfungibles::clearMetadata_1Call {
			collection: collection_id.into(),
			item: item_id.into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn set_max_supply() {
		let collection_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let max_supply: u32 = 1000;

		create_collection::<T, I>(owner.clone());

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::setMaxSupply(INonfungibles::setMaxSupplyCall {
			collection: collection_id.into(),
			maxSupply: max_supply.into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn approve_item_attributes() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let delegate = <AddressMapper<T>>::to_account_id(&BOB_ADDR);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input =
			INonfungiblesCalls::approveItemAttributes(INonfungibles::approveItemAttributesCall {
				collection: collection_id.into(),
				item: item_id.into(),
				delegate: <AddressMapper<T>>::to_address(&delegate).0.into(),
			});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn cancel_item_attributes_approval() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let delegate = <AddressMapper<T>>::to_account_id(&BOB_ADDR);
		let witness = pallet_nfts::CancelAttributesApprovalWitness { account_attributes: 0 };

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::cancelItemAttributesApproval(
			INonfungibles::cancelItemAttributesApprovalCall {
				collection: collection_id.into(),
				item: item_id.into(),
				delegate: <AddressMapper<T>>::to_address(&delegate).0.into(),
				witness: codec::Encode::encode(&witness).into(),
			},
		);

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn clear_all_approvals() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::clearAllApprovals(INonfungibles::clearAllApprovalsCall {
			collection: collection_id.into(),
			item: item_id.into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn clear_collection_approvals(l: Linear<1, 1000>) {
		let collection_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);

		create_collection::<T, I>(owner.clone());

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::clearCollectionApprovals(
			INonfungibles::clearCollectionApprovalsCall {
				collection: collection_id.into(),
				limit: l.into(),
			},
		);

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn mint() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let to = <AddressMapper<T>>::to_account_id(&BOB_ADDR);
		let witness: pallet_nfts::MintWitness<_, <T as pallet_nfts::Config<I>>::Balance> =
			pallet_nfts::MintWitness { owned_item: Some(0), mint_price: None };

		create_collection::<T, I>(owner.clone());

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::mint(INonfungibles::mintCall {
			collection: collection_id.into(),
			to: <AddressMapper<T>>::to_address(&to).0.into(),
			item: item_id.into(),
			witness: codec::Encode::encode(&witness).into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn burn() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::burn(INonfungibles::burnCall {
			collection: collection_id.into(),
			item: item_id.into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, ()>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn balance_of() {
		let collection_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);

		create_collection::<T, I>(owner.clone());

		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::balanceOf(INonfungibles::balanceOfCall {
			collection: collection_id.into(),
			owner: <AddressMapper<T>>::to_address(&owner).0.into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, u32>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn owner_of() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);

		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::ownerOf(INonfungibles::ownerOfCall {
			collection: collection_id.into(),
			item: item_id.into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, alloy::Address>(
				&mut ext, &ADDRESS, &input
			));
		}
	}

	#[benchmark]
	fn allowance() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let operator = <AddressMapper<T>>::to_account_id(&BOB_ADDR);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);

		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::allowance_1(INonfungibles::allowance_1Call {
			collection: collection_id.into(),
			owner: <AddressMapper<T>>::to_address(&owner).0.into(),
			operator: <AddressMapper<T>>::to_address(&operator).0.into(),
			item: item_id.into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, bool>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn total_supply() {
		let collection_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);

		create_collection::<T, I>(owner.clone());

		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::totalSupply(INonfungibles::totalSupplyCall {
			collection: collection_id.into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, u32>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn get_attribute() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let key = b"test_key";
		let value = b"test_value";
		let namespace: AttributeNamespace<AccountIdOf<T>> = AttributeNamespace::CollectionOwner;
		let namespace_bytes = codec::Encode::encode(&namespace);

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);
		set_item_attribute::<T, I>(collection_id, Some(item_id), key, value);

		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::getAttribute_1(INonfungibles::getAttribute_1Call {
			collection: collection_id.into(),
			item: item_id.into(),
			namespace: namespace_bytes.clone().into(),
			key: key.to_vec().into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, String>(
				&mut ext, &ADDRESS, &input
			));
		}
	}

	#[benchmark]
	fn item_metadata() {
		let collection_id: u32 = 0;
		let item_id: u32 = 0;
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let metadata = b"test metadata";

		create_collection_and_mint::<T, I>(owner.clone(), collection_id, item_id);
		set_item_metadata::<T, I>(owner.clone(), collection_id, item_id, metadata);

		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = INonfungiblesCalls::itemMetadata(INonfungibles::itemMetadataCall {
			collection: collection_id.into(),
			item: item_id.into(),
		});

		#[block]
		{
			assert_ok!(call_precompile::<Nonfungibles<T, I>, _, String>(
				&mut ext, &ADDRESS, &input
			));
		}
	}

	impl_benchmark_test_suite!(pallet_nfts::Pallet, ExtBuilder::new().build(), crate::mock::Test);
}

// Helper functions
fn create_collection<
	T: Config<I> + pallet_nfts::Config<I, CollectionId: Default> + pallet_revive::Config,
	I: 'static,
>(
	owner: T::AccountId,
) -> CollectionId<T, I> {
	let collection_id = NextCollectionId::<T, I>::get().unwrap_or_default();
	<Balances<T>>::set_balance(&owner, u32::MAX.into());
	assert_ok!(<Nfts<T, I>>::create(
		OriginFor::<T>::Signed(owner.clone()),
		T::Lookup::unlookup(owner.clone()),
		default_collection_config::<T, I>(),
	));
	collection_id
}

fn create_collection_and_mint<
	T: Config<I>
		+ pallet_nfts::Config<I, CollectionId: From<u32>, ItemId: From<u32>>
		+ pallet_revive::Config,
	I: 'static,
>(
	owner: T::AccountId,
	collection_id: u32,
	item_id: u32,
) {
	create_collection::<T, I>(owner.clone());
	assert_ok!(<Nfts<T, I>>::mint(
		OriginFor::<T>::Signed(owner.clone()),
		collection_id.into(),
		item_id.into(),
		T::Lookup::unlookup(owner.clone()),
		None,
	));
}

fn default_collection_config<T: Config<I> + pallet_nfts::Config<I>, I: 'static>(
) -> CollectionConfigFor<T, I> {
	CollectionConfig {
		settings: CollectionSettings::all_enabled(),
		max_supply: None,
		mint_settings: MintSettings::default(),
	}
}

fn set_item_attribute<
	T: Config<I>
		+ pallet_nfts::Config<I, CollectionId: From<u32>, ItemId: From<u32>>
		+ pallet_revive::Config,
	I: 'static,
>(
	collection_id: u32,
	item_id: Option<u32>,
	key: &[u8],
	value: &[u8],
) {
	let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
	assert_ok!(<Nfts<T, I>>::set_attribute(
		OriginFor::<T>::Signed(owner.clone()),
		collection_id.into(),
		item_id.map(|id| id.into()),
		AttributeNamespace::CollectionOwner,
		BoundedVec::truncate_from(key.to_vec()),
		BoundedVec::truncate_from(value.to_vec()),
	));
}

fn set_item_metadata<
	T: Config<I> + pallet_nfts::Config<I, CollectionId: From<u32>, ItemId: From<u32>>,
	I: 'static,
>(
	owner: T::AccountId,
	collection_id: u32,
	item_id: u32,
	metadata: &[u8],
) {
	assert_ok!(<Nfts<T, I>>::set_metadata(
		OriginFor::<T>::Signed(owner.clone()),
		collection_id.into(),
		item_id.into(),
		BoundedVec::truncate_from(metadata.to_vec()),
	));
}

fn set_collection_metadata<
	T: Config<I> + pallet_nfts::Config<I, CollectionId: From<u32>>,
	I: 'static,
>(
	owner: T::AccountId,
	collection_id: u32,
	metadata: &[u8],
) {
	assert_ok!(<Nfts<T, I>>::set_collection_metadata(
		OriginFor::<T>::Signed(owner.clone()),
		collection_id.into(),
		BoundedVec::truncate_from(metadata.to_vec()),
	));
}

fn check_allowance<
	T: Config<I> + pallet_nfts::Config<I, CollectionId: From<u32>, ItemId: From<u32>>,
	I: 'static,
>(
	collection_id: u32,
	owner: T::AccountId,
	operator: T::AccountId,
	item_id: Option<u32>,
) -> bool {
	<Nfts<T, I>>::check_approval_permission(
		&collection_id.into(),
		&item_id.map(|id| id.into()),
		&owner,
		&operator,
	)
	.is_ok()
}

fn get_owner_of<
	T: Config<I> + pallet_nfts::Config<I, CollectionId: From<u32>, ItemId: From<u32>>,
	I: 'static,
>(
	collection_id: u32,
	item_id: u32,
) -> Option<T::AccountId> {
	<Nfts<T, I>>::owner(collection_id.into(), item_id.into())
}

fn set_up_call<
	T: pallet_revive::Config<
		Currency: Inspect<
			<T as frame_system::Config>::AccountId,
			Balance: Into<U256> + TryFrom<U256>,
		>,
		Hash: IsType<H256>,
		Time: Time<Moment: Into<U256>>,
	>,
>() -> CallSetup<T> {
	CallSetup::<T>::new(WasmModule::dummy())
}

// Type aliases for cleaner code
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type CollectionConfigFor<T, I> = pallet_nfts::CollectionConfigFor<T, I>;
type DepositBalanceOf<T, I> = pallet_nfts::DepositBalanceOf<T, I>;

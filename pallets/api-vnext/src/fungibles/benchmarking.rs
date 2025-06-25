//! Benchmarking setup for pallet_api::fungibles::precompiles

use alloc::{
	string::{String, ToString},
	vec,
};

use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok,
	pallet_prelude::IsType,
	traits::{
		fungible::{Inspect, Mutate as _},
		fungibles::{
			approvals::{self, Inspect as _},
			metadata::{self, Inspect as _},
			Create, Inspect as _,
		},
		Get, Time,
	},
};
use pallet_assets::{Asset, AssetStatus};
use pallet_revive::{
	precompiles::{
		alloy::primitives as alloy,
		run::{precompile, CallSetup, WasmModule, H256, U256},
	},
	test_utils::{ALICE_ADDR, BOB_ADDR, CHARLIE_ADDR},
	AddressMapper as _, Origin,
};

use super::{
	precompiles::{IFungibles, IFungiblesCalls, UintTryFrom, UintTryTo},
	Config, NextAssetId, Pallet,
};
#[cfg(test)]
use crate::mock::{ExtBuilder, Test};
use crate::{call_precompile, fixed_address};

const FUNGIBLES: u16 = 100;
const ADDRESS: [u8; 20] = fixed_address(FUNGIBLES);

type AddressMapper<T> = <T as pallet_revive::Config>::AddressMapper;
type Assets<T, I> = pallet_assets::Pallet<T, I>;
type AssetsBalance<T, I> = <T as pallet_assets::Config<I>>::Balance;
type AssetsStringLimit<T, I> = <T as pallet_assets::Config<I>>::StringLimit;
type Balances<T> = <T as pallet_revive::Config>::Currency;
type Fungibles<T, I> = super::Fungibles<FUNGIBLES, T, I>;
type TokenId<T, I> = <T as pallet_assets::Config<I>>::AssetId;

#[instance_benchmarks(
    where
        // Precompiles
        T: pallet_revive::Config<
            Currency: Inspect<<T as frame_system::Config>::AccountId, Balance: Into<U256> + TryFrom<U256>>,
            Hash: IsType<H256>,
            Time: Time<Moment: Into<U256>>
        >,
        // Fungibles
        T: pallet_assets::Config<I, AssetId: Default + From<u32> + Into<u32>>,
        alloy::U256: UintTryFrom<AssetsBalance<T, I>> + UintTryTo<AssetsBalance<T, I>>
)]
mod benchmarks {
	use super::*;

	// Parameter:
	// - 'a': whether `approve_transfer` is required.
	// - 'c': whether `cancel_approval` is required.
	#[benchmark]
	fn approve(a: Linear<0, 1>, c: Linear<0, 1>) -> Result<(), BenchmarkError> {
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let spender = <AddressMapper<T>>::to_account_id(&BOB_ADDR);
		let current_allowance = <AssetsBalance<T, I>>::from(u32::MAX / 2);
		let token = super::create::<T, I>(<AddressMapper<T>>::to_account_id(&CHARLIE_ADDR));
		// Set the `current_allowance`.
		<Balances<T>>::set_balance(&owner, u32::MAX.into());
		assert_ok!(<Assets<T, I> as approvals::Mutate<T::AccountId>>::approve(
			token.clone(),
			&owner,
			&spender,
			current_allowance,
		));
		let approval_value = match (a, c) {
			// Equal to the current allowance.
			(0, 0) => current_allowance,
			// Greater than the current allowance.
			(1, 0) => <AssetsBalance<T, I>>::from(u32::MAX),
			// Zero.
			(0, 1) => <AssetsBalance<T, I>>::from(0u32),
			// Smaller than the current allowance.
			(1, 1) => <AssetsBalance<T, I>>::from(u32::MAX / 4),
			_ => unreachable!("values can only be 0 or 1"),
		};

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::approve(IFungibles::approveCall {
			token: token.clone().into(),
			spender: <AddressMapper<T>>::to_address(&spender).0.into(),
			value: alloy::U256::from(approval_value),
		});

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}

		assert_eq!(<Assets<T, I>>::allowance(token.clone(), &owner, &spender), approval_value);
		if c == 1 {
			assert_has_event::<T, I>(
				pallet_assets::Event::ApprovalCancelled {
					asset_id: token.clone(),
					owner: owner.clone(),
					delegate: spender.clone(),
				}
				.into(),
			);
		}
		if a == 1 {
			let amount = match c {
				// When the allowance was cancelled and then approved with the new value.
				1 => approval_value,
				// When the allowance was increased.
				0 => approval_value - current_allowance,
				_ => unreachable!("`c` can only be 0 or 1"),
			};
			assert_has_event::<T, I>(
				pallet_assets::Event::ApprovedTransfer {
					asset_id: token,
					source: owner,
					delegate: spender,
					amount,
				}
				.into(),
			);
		}
		Ok(())
	}

	#[benchmark]
	fn create() {
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let admin = BOB_ADDR;
		let min_balance: AssetsBalance<T, I> = 1u8.into();

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::create(IFungibles::createCall {
			admin: admin.0.into(),
			minBalance: alloy::U256::from(min_balance),
		});

		let mut token = 0;
		#[block]
		{
			token = call_precompile::<Fungibles<T, I>, _, _>(&mut ext, &ADDRESS, &input).unwrap();
		}

		assert!(<Assets<T, I>>::asset_exists(token.into()));
	}

	#[benchmark]
	fn start_destroy() {
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let token = super::create::<T, I>(owner.clone());
		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::startDestroy(IFungibles::startDestroyCall {
			token: token.clone().into(),
		});

		assert_eq!(<Asset<T, I>>::get(token.clone()).unwrap().status, AssetStatus::Live);

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}

		assert_eq!(<Asset<T, I>>::get(token.clone()).unwrap().status, AssetStatus::Destroying);
	}

	#[benchmark]
	fn set_metadata() {
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let token = super::create::<T, I>(owner.clone());
		let max = AssetsStringLimit::<T, I>::get() as usize;
		let name = vec![42u8; max];
		let symbol = vec![42u8; max];
		let decimals = u8::MAX - 1;

		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::setMetadata(IFungibles::setMetadataCall {
			token: token.clone().into(),
			name: String::from_utf8_lossy(&name).to_string(),
			symbol: String::from_utf8_lossy(&symbol).to_string(),
			decimals,
		});

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}

		assert_eq!(<Assets<T, I>>::name(token.clone()), name);
		assert_eq!(<Assets<T, I>>::symbol(token.clone()), symbol);
		assert_eq!(<Assets<T, I>>::decimals(token), decimals);
	}

	#[benchmark]
	fn clear_metadata() {
		let owner = <AddressMapper<T>>::to_account_id(&ALICE_ADDR);
		let token = super::create::<T, I>(owner.clone());
		let mut call_setup = set_up_call();
		call_setup.set_origin(Origin::Signed(owner.clone()));
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::clearMetadata(IFungibles::clearMetadataCall {
			token: token.clone().into(),
		});

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}

		assert!(<Assets<T, I>>::name(token.clone()).is_empty());
		assert!(<Assets<T, I>>::symbol(token.clone()).is_empty());
		assert_eq!(<Assets<T, I>>::decimals(token), 0);
	}

	#[benchmark]
	fn total_supply() {
		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::totalSupply(IFungibles::totalSupplyCall { token: 0 });

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn balance_of() {
		let token = super::create::<T, I>(<AddressMapper<T>>::to_account_id(&ALICE_ADDR));
		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::balanceOf(IFungibles::balanceOfCall {
			token: token.into(),
			owner: ALICE_ADDR.0.into(),
		});

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn allowance() {
		let token = super::create::<T, I>(<AddressMapper<T>>::to_account_id(&ALICE_ADDR));
		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::allowance(IFungibles::allowanceCall {
			token: token.into(),
			owner: ALICE_ADDR.0.into(),
			spender: BOB_ADDR.0.into(),
		});

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn metadata_name() {
		let token = super::create::<T, I>(<AddressMapper<T>>::to_account_id(&ALICE_ADDR));
		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::name(IFungibles::nameCall { token: token.into() });

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn metadata_symbol() {
		let token = super::create::<T, I>(<AddressMapper<T>>::to_account_id(&ALICE_ADDR));
		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::symbol(IFungibles::symbolCall { token: token.into() });

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn metadata_decimals() {
		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::decimals(IFungibles::decimalsCall { token: 0 });

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}
	}

	#[benchmark]
	fn exists() {
		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = IFungiblesCalls::exists(IFungibles::existsCall { token: 0 });

		#[block]
		{
			assert_ok!(precompile::<Fungibles<T, I>, _>(&mut ext, &ADDRESS, &input));
		}
	}

	impl_benchmark_test_suite!(Pallet, ExtBuilder::new().build(), Test);
}

// Ensure `event` has been emitted.
fn assert_has_event<T: pallet_assets::Config<I>, I>(
	event: <T as pallet_assets::Config<I>>::RuntimeEvent,
) {
	frame_system::Pallet::<T>::assert_has_event(event.into());
}

fn create<T: Config<I> + pallet_assets::Config<I, AssetId: Default> + pallet_revive::Config, I>(
	owner: T::AccountId,
) -> TokenId<T, I> {
	let token = NextAssetId::<T, I>::get().unwrap_or_default();
	<Balances<T>>::set_balance(&owner, u32::MAX.into());
	assert_ok!(<Assets<T, I> as Create<T::AccountId>>::create(
		token.clone(),
		owner.clone(),
		true,
		1u32.into()
	));

	let max = AssetsStringLimit::<T, I>::get() as usize;
	assert_ok!(<Assets<T, I> as metadata::Mutate<T::AccountId>>::set(
		token.clone(),
		&owner,
		vec![255u8; max],
		vec![255u8; max],
		u8::MAX
	));

	token
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

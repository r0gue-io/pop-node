//! Benchmarking setup for pallet-api::fungibles

use super::{AccountIdOf, AssetIdOf, AssetsInstanceOf, AssetsOf, BalanceOf, Call, Config, Pallet};
use frame_benchmarking::{account, v2::*};
use frame_support::{
	assert_ok,
	traits::{
		fungibles::{
			approvals::{Inspect as ApprovalInspect, Mutate},
			Create, Inspect,
		},
		Currency,
	},
};
use frame_system::RawOrigin;
use sp_runtime::traits::Zero;

const SEED: u32 = 1;

// See if `generic_event` has been emitted.
fn assert_has_event<T: Config>(
	generic_event: <T as pallet_assets::Config<AssetsInstanceOf<T>>>::RuntimeEvent,
) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

#[benchmarks(
	where
	<pallet_assets::Pallet<T, AssetsInstanceOf<T>> as Inspect<<T as frame_system::Config>::AccountId>>::AssetId: Zero,
)]
mod benchmarks {
	use super::*;

	// Parameter:
	// - 'a': whether `approve_transfer` has been called.
	// - 'c': whether `cancel_approval` has been called.
	#[benchmark]
	fn approve(a: Linear<0, 1>, c: Linear<0, 1>) -> Result<(), BenchmarkError> {
		let asset_id = AssetIdOf::<T>::zero();
		let min_balance = <BalanceOf<T>>::from(1u32);
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let spender: AccountIdOf<T> = account("Bob", 0, SEED);
		let current_allowance = <BalanceOf<T>>::from(u32::MAX / 2);
		T::Currency::make_free_balance_be(&owner, u32::MAX.into());
		// Set the `current_allowance`.
		assert_ok!(<AssetsOf<T> as Create<AccountIdOf<T>>>::create(
			asset_id.clone(),
			owner.clone(),
			true,
			min_balance
		));
		assert_ok!(<AssetsOf<T> as Mutate<AccountIdOf<T>>>::approve(
			asset_id.clone(),
			&owner,
			&spender,
			current_allowance,
		));
		let approval_value = match (a, c) {
			// Equal to the current allowance.
			(0, 0) => current_allowance,
			// Greater than the current allowance.
			(1, 0) => <BalanceOf<T>>::from(u32::MAX),
			// Zero.
			(0, 1) => <BalanceOf<T>>::from(0u32),
			// Smaller than the current allowance.
			(1, 1) => <BalanceOf<T>>::from(u32::MAX / 4),
			_ => unreachable!("values can only be 0 or 1"),
		};

		#[extrinsic_call]
		_(RawOrigin::Signed(owner.clone()), asset_id.clone(), spender.clone(), approval_value);

		assert_eq!(AssetsOf::<T>::allowance(asset_id.clone(), &owner, &spender), approval_value);
		if c == 1 {
			assert_has_event::<T>(
				pallet_assets::Event::ApprovalCancelled {
					asset_id: asset_id.clone(),
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
			assert_has_event::<T>(
				pallet_assets::Event::ApprovedTransfer {
					asset_id,
					source: owner,
					delegate: spender,
					amount,
				}
				.into(),
			);
		}
		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}

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

	// The worst case scenario is when the allowance is set to a value which is lower than the
	// current allowance.
	#[benchmark]
	fn approve() -> Result<(), BenchmarkError> {
		let asset_id = AssetIdOf::<T>::zero();
		let decreased_value = <BalanceOf<T>>::from(50u32);
		let min_balance = <BalanceOf<T>>::from(1u32);
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let spender: AccountIdOf<T> = account("Bob", 0, SEED);
		let value = <BalanceOf<T>>::from(100u32);
		T::Currency::make_free_balance_be(&owner, u32::MAX.into());
		assert_ok!(<AssetsOf<T> as Create<AccountIdOf<T>>>::create(
			asset_id.clone().into(),
			owner.clone(),
			true,
			min_balance
		));
		assert_ok!(<AssetsOf<T> as Mutate<AccountIdOf<T>>>::approve(
			asset_id.clone(),
			&owner,
			&spender,
			value
		));

		#[extrinsic_call]
		_(RawOrigin::Signed(owner.clone()), asset_id.clone(), spender.clone(), decreased_value);

		assert_eq!(AssetsOf::<T>::allowance(asset_id.clone(), &owner, &spender), decreased_value);
		// To make sure both dispatchables have been successfully executed - i.e. worst case
		// scenario.
		assert_has_event::<T>(
			pallet_assets::Event::ApprovalCancelled {
				asset_id: asset_id.clone(),
				owner: owner.clone(),
				delegate: spender.clone(),
			}
			.into(),
		);
		assert_has_event::<T>(
			pallet_assets::Event::ApprovedTransfer {
				asset_id,
				source: owner,
				delegate: spender,
				amount: value,
			}
			.into(),
		);

		Ok(())
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}

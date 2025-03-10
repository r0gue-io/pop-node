//! Benchmarking setup for pallet_api::nonfungibles
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, v2::*};
use frame_support::{dispatch::RawOrigin, traits::Currency, BoundedVec};
use sp_runtime::traits::Zero;

use super::*;
use crate::Read as _;
const SEED: u32 = 1;

// See if `generic_event` has been emitted.
fn assert_has_event<T: Config>(generic_event: <T as crate::messaging::Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

#[benchmarks(
	where
	T: pallet_balances::Config
)]
mod messaging_benchmarks {
	use super::*;

	#[benchmark]
	// x: The number of removals required.
	fn remove(x: Linear<1, 255>) {
		let deposit: BalanceOf<T> = sp_runtime::traits::One::one();
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let mut message_ids: BoundedVec<MessageId, T::MaxRemovals> = BoundedVec::new();
		pallet_balances::Pallet::<T>::make_free_balance_be(&owner, u32::MAX.into());

		for i in 0..x {
			<T as crate::messaging::Config>::Deposit::hold(
				&HoldReason::Messaging.into(),
				&owner,
				deposit,
			)
			.unwrap();

			let message_id = H256::repeat_byte(i as u8);
			let commitment = H256::repeat_byte(i as u8);

			let good_message = Message::IsmpResponse {
				commitment: commitment.clone(),
				deposit,
				response: Default::default(),
				status: MessageStatus::Ok,
			};

			Messages::<T>::insert(&owner, &message_id.0, &good_message);
			IsmpRequests::<T>::insert(&commitment, (&owner, &message_id.0));
			message_ids.try_push(message_id.0).unwrap()
		}
		#[extrinsic_call]
		Pallet::<T>::remove(RawOrigin::Signed(owner.clone()), message_ids.clone());

		assert_has_event::<T>(
			crate::messaging::Event::Removed { origin: owner, messages: message_ids.to_vec() }
				.into(),
		)
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}

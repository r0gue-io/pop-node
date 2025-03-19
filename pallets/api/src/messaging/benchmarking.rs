//! Benchmarking setup for pallet_api::nonfungibles
#![cfg(feature = "runtime-benchmarks")]

use ::xcm::latest::{Junctions, Location};
use frame_benchmarking::{account, v2::*};
use frame_support::{dispatch::RawOrigin, traits::Currency, BoundedVec};
use sp_runtime::traits::{One, Zero};

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

	/// x: The number of removals required.
	#[benchmark]
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

	#[benchmark]
	fn xcm_new_query() {
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let message_id: [u8; 32] = [0; 32];
		let responder = Location { parents: 1, interior: Junctions::Here };
		let timeout = <BlockNumberOf<T> as One>::one() + frame_system::Pallet::<T>::block_number();
		let callback =
			Callback { selector: [0; 4], weight: 100.into(), spare_weight_creditor: owner.clone() };

		pallet_balances::Pallet::<T>::make_free_balance_be(&owner, u32::MAX.into());

		#[extrinsic_call]
		Pallet::<T>::xcm_new_query(
			RawOrigin::Signed(owner.clone()),
			message_id.clone(),
			responder.clone(),
			timeout,
			Some(callback.clone()),
		);

		assert_has_event::<T>(
			crate::messaging::Event::XcmQueryCreated {
				origin: owner,
				id: message_id,
				query_id: 0,
				callback: Some(callback),
			}
			.into(),
		)
	}

	/// x: Wether a successfully executing callback is provided.
	#[benchmark]
	fn xcm_response(x: Linear<0, 1>) {
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let message_id: [u8; 32] = [0; 32];
		let responder = Location { parents: 1, interior: Junctions::Here };
		let timeout = <BlockNumberOf<T> as One>::one() + frame_system::Pallet::<T>::block_number();
		let callback = None;
		let response = Response::ExecutionResult(None);

		if x == 1 {
			// The mock will always assume successfull callback.
			let callback = Some(Callback {
				selector: [0; 4],
				weight: 100.into(),
				spare_weight_creditor: owner.clone(),
			});
		}

		pallet_balances::Pallet::<T>::make_free_balance_be(&owner, u32::MAX.into());

		Pallet::<T>::xcm_new_query(
			RawOrigin::Signed(owner.clone()).into(),
			message_id.clone(),
			responder.clone(),
			timeout,
			callback.clone(),
		)
		.unwrap();

		#[extrinsic_call]
		Pallet::<T>::xcm_response(RawOrigin::Root, 0, response.clone());

		assert_has_event::<T>(
			crate::messaging::Event::XcmResponseReceived {
				dest: owner.clone(),
				id: message_id,
				query_id: 0,
				response,
			}
			.into(),
		);
		assert!(Messages::<T>::get(&owner, &message_id).is_none());
		assert!(XcmQueries::<T>::get(0).is_none());
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}

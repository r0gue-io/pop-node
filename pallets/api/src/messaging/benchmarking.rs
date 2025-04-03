//! Benchmarking setup for pallet_api::nonfungibles
#![cfg(feature = "runtime-benchmarks")]

use ::ismp::{
	host::StateMachine,
	module::IsmpModule,
	router::{
		GetRequest, GetResponse, PostRequest, PostResponse, Request, Request::*,
		Response as IsmpResponse, Timeout,
	},
	// dispatcher::DispatchRequest::{Post, Get},
};
use ::xcm::latest::{Junctions, Location};
use frame_benchmarking::{account, v2::*};
use frame_support::{dispatch::RawOrigin, traits::Currency, BoundedVec, sp_runtime::traits::Hash, sp_runtime::RuntimeDebug,};
use sp_core::{bounded_vec, keccak_256, H256, Get};
use sp_runtime::traits::{One, Zero};
use sp_io::hashing::blake2_256;


use super::*;
use crate::{messaging::test_utils::*, Read as _};
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
	fn remove(x: Linear<1, { T::MaxRemovals::get() }>) {
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
		
			// Generate unique message_id and commitment using hashing
			let message_id = H256::from(blake2_256(&(i.to_le_bytes())));
			let commitment = H256::from(blake2_256(&(i.to_le_bytes())));
		
			let good_message = Message::IsmpResponse {
				commitment: commitment.clone(),
				deposit,
				response: Default::default(),
			};
		
			Messages::<T>::insert(&owner, &message_id.0, &good_message);
			IsmpRequests::<T>::insert(&commitment, (&owner, &message_id.0));
			message_ids.try_push(message_id.0).unwrap();
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
		let callback = Callback {
			selector: [0; 4],
			weight: 100.into(),
			spare_weight_creditor: owner.clone(),
			abi: Abi::Scale,
		};

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

	/// x: Whether a successfully executing callback is provided.
	#[benchmark]
	fn xcm_response(x: Linear<0, 1>) {
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let message_id: [u8; 32] = [0; 32];
		let responder = Location { parents: 1, interior: Junctions::Here };
		let timeout = <BlockNumberOf<T> as One>::one() + frame_system::Pallet::<T>::block_number();
		let response = Response::ExecutionResult(None);

		let callback = if x == 1 {
			// The mock will always assume successfull callback.
			Some(Callback {
				selector: [0; 4],
				weight: 100.into(),
				spare_weight_creditor: owner.clone(),
				abi: Abi::Scale,
			})
		} else {
			None
		};

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

	/// x: Is it a get. (example: 1 = get, 0 = post)
	/// y: the response has a callback.
	#[benchmark]
	fn ismp_on_response(x: Linear<0, 1>, y: Linear<0, 1>) {
		let origin: T::AccountId = account("alice", 0, SEED);
 		let message_id = [1; 32];
		let callback = if y == 1 {
			// The mock will always assume successfull callback.
			Some(Callback {
				selector: [0; 4],
				weight: 100.into(),
				spare_weight_creditor: origin.clone(),
				abi: Abi::Scale,
			})
		} else {
			None
		};

		let (response, event, commitment) = if x == 1 {
			// get response
			let get = ismp_get_response(T::MaxKeyLen::get() as usize, T::MaxKeys::get() as usize, T::MaxContextLen::get() as usize, T::MaxResponseLen::get() as usize);
			let commitment = H256::from(keccak_256(&Get(get.get.clone()).encode()));
			(
				IsmpResponse::Get(get.clone()),
				crate::messaging::Event::<T>::IsmpGetResponseReceived {
					dest: origin.clone(),
					id: message_id,
					commitment,
				},
				commitment,
			)
		} else {
			// post response
			let post = ismp_post_response(T::MaxDataLen::get() as usize, T::MaxResponseLen::get() as usize);
			let commitment = H256::from(keccak_256(&Post(post.post.clone()).encode()));
			(
				IsmpResponse::Post(post.clone()),
				crate::messaging::Event::<T>::IsmpPostResponseReceived {
					dest: origin.clone(),
					id: message_id,
					commitment,
				},
				commitment,
			)
		};

		let message = Message::Ismp { commitment, callback, deposit: One::one() };

		IsmpRequests::<T>::insert(&commitment, (&origin, &message_id));
		Messages::<T>::insert(&origin, &message_id, &message);

		let handler = crate::messaging::ismp::Handler::<T>::new();

		#[block]
		{
			handler.on_response(response.clone()).unwrap();
		}

		assert_has_event::<T>(event.into())
	}

	/// x: is it a Request::Post, Request::Get or Response::Post.
	/// x = 0: Post request.
	/// x = 1: Get request.
	/// x = 2: Post response.
	/// y = 1: There is a callback
	#[benchmark]
	fn ismp_on_timeout(x: Linear<0, 2>, y: Linear<0, 1>) {
		let commitment = H256::repeat_byte(2u8);
		let origin: T::AccountId = account("alice", 0, SEED);
		let message_id = [1; 32];
		
		let callback = if y == 1 {
			Some(Callback {
				selector: [1; 4],
				weight: 100.into(),
				spare_weight_creditor: origin.clone(),
				abi: Abi::Scale,
			})
		} else {
			None
		};

		let (timeout_message, commitment) = if x == 0 {
			let post_request = ismp_post_request(T::MaxDataLen::get() as usize);
			let commitment = H256::from(keccak_256(&Post(post_request.clone()).encode()));
			(Timeout::Request(Request::Post(post_request.clone())), commitment)
		} else if x == 1 {
			let get_request = ismp_get_request(T::MaxKeyLen::get() as usize, T::MaxKeys::get() as usize, T::MaxContextLen::get() as usize);
			let commitment = H256::from(keccak_256(&Get(get_request.clone()).encode()));
			(Timeout::Request(Request::Get(get_request.clone())), commitment)
		} else {
			let post_response = ismp_post_response(T::MaxDataLen::get() as usize, T::MaxResponseLen::get() as usize);
			let commitment = H256::from(keccak_256(&Post(post_response.post.clone()).encode()));

			(Timeout::Response(post_response), commitment)
		};

		let input_message =
			Message::Ismp { commitment, callback, deposit: One::one() };

		IsmpRequests::<T>::insert(&commitment, (&origin, &message_id));
		Messages::<T>::insert(&origin, &message_id, &input_message);

		let handler = crate::messaging::ismp::Handler::<T>::new();
		#[block]
		{
			handler.on_timeout(timeout_message).unwrap();
		}
	}

	/// x: Maximum context len: bound to u16::MAX T::MaxContextLen
	/// y: Maximum length of keys (inner) len: bound to u16::MAX T::MaxKeyLen
	/// z: Maximum amount of keys (outer) len: bound to u16::MAX T::MaxKeys
	#[benchmark]
	fn ismp_get(
		x: Linear<0, { T::MaxContextLen::get() }>,
		y: Linear<0, { T::MaxKeyLen::get() }>,
		z: Linear<0, { T::MaxKeys::get() }>,
		a: Linear<0, 1>,
	) {
		let origin: T::AccountId = account("alice", 0, SEED);
		let message_id = [1; 32];
		let inner_keys: BoundedVec<u8, T::MaxKeyLen> =
			vec![1u8].repeat(y as usize).try_into().unwrap();
		let mut outer_keys = vec![];
		for k in (0..z) {
			outer_keys.push(inner_keys.clone())
		}

		let callback = if a == 1 {
			Some(Callback {
				selector: [1; 4],
				weight: 100.into(),
				spare_weight_creditor: origin.clone(),
				abi: Abi::Scale,
			})
		} else {
			None
		};

		let get = crate::messaging::ismp::Get::<T> {
			dest: 0,
			height: 0,
			timeout: 0,
			context: vec![1u8].repeat(x as usize).try_into().unwrap(),
			keys: outer_keys.try_into().unwrap(),
		};

		pallet_balances::Pallet::<T>::make_free_balance_be(&origin, u32::MAX.into());

		#[extrinsic_call]
		Pallet::<T>::ismp_get(RawOrigin::Signed(origin.clone()), message_id, get, callback);
	}

	/// x: Maximun byte len of outgoing data. T::MaxDataLen
	/// y: is there a callback.
	#[benchmark]
	fn ismp_post(x: Linear<0, { T::MaxDataLen::get() }>, y: Linear<0, 1>) {
		let origin: T::AccountId = account("alice", 0, SEED);
		let message_id = [1; 32];
		let data = vec![1u8].repeat(x as usize).try_into().unwrap();

		let get = crate::messaging::ismp::Post::<T> { dest: 0, timeout: 0, data };

		let callback = if y == 1 {
			Some(Callback {
				selector: [1; 4],
				weight: 100.into(),
				spare_weight_creditor: origin.clone(),
				abi: Abi::Scale,
			})
		} else {
			None
		};

		pallet_balances::Pallet::<T>::make_free_balance_be(&origin, u32::MAX.into());

		#[extrinsic_call]
		Pallet::<T>::ismp_post(RawOrigin::Signed(origin.clone()), message_id, get, callback);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}

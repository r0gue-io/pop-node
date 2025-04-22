//! Benchmarking setup for pallet_api::messaging
#![cfg(feature = "runtime-benchmarks")]

use ::ismp::{
	messaging::hash_request,
	module::IsmpModule,
	router::{Request, Response as IsmpResponse, Timeout},
};
use ::xcm::latest::{Junctions, Location};
use frame_benchmarking::{account, v2::*};
use frame_support::{
	dispatch::RawOrigin,
	traits::{Currency, EnsureOrigin},
	BoundedVec,
};
use sp_core::{Get, H256};
use sp_io::hashing::blake2_256;
use sp_runtime::traits::One;
use sp_std::vec;

use super::*;
use crate::messaging::test_utils::*;

const SEED: u32 = 1;
type RuntimeOrigin<T> = <T as frame_system::Config>::RuntimeOrigin;

// See if `generic_event` has been emitted.
fn assert_has_event<T: Config>(generic_event: <T as crate::messaging::Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

#[benchmarks(
	where
	T: pallet_balances::Config + pallet_timestamp::Config,
)]
mod messaging_benchmarks {
	use super::*;

	/// # Parameters
	/// - `x`: `Linear<1, { T::MaxRemovals::get() }>`   The number of message removals to perform
	///   (bounded by `MaxRemovals`).
	#[benchmark]
	fn remove(x: Linear<1, { T::MaxRemovals::get() }>) {
		let deposit: BalanceOf<T> = sp_runtime::traits::One::one();
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let mut message_ids: BoundedVec<MessageId, T::MaxRemovals> = BoundedVec::new();
		pallet_balances::Pallet::<T>::make_free_balance_be(&owner, u32::MAX.into());

		for i in 0..x {
			<T as crate::messaging::Config>::Fungibles::hold(
				&HoldReason::Messaging.into(),
				&owner,
				deposit,
			)
			.unwrap();

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

	/// Submits a new XCM query message with an optional callback.
	///
	/// # Parameters
	/// - `x`: `Linear<0, 1>`   Whether a callback is supplied:
	///   - `0`: No callback
	///   - `1`: Callback attached
	#[benchmark]
	fn xcm_new_query(x: Linear<0, 1>) {
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let message_id: [u8; 32] = [0; 32];
		let responder = Location { parents: 1, interior: Junctions::Here };
		let timeout = frame_system::Pallet::<T>::block_number() + 1000u32.into();
		let callback = if x == 1 {
			Some(Callback {
				selector: [0; 4],
				weight: Weight::from_parts(100, 100),
				abi: Abi::Scale,
			})
		} else {
			None
		};

		pallet_balances::Pallet::<T>::make_free_balance_be(&owner, u32::MAX.into());

		#[extrinsic_call]
		Pallet::<T>::xcm_new_query(
			RawOrigin::Signed(owner.clone()),
			message_id.clone(),
			responder.clone(),
			timeout,
			callback.clone(),
		);

		assert_has_event::<T>(
			crate::messaging::Event::XcmQueryCreated {
				origin: owner,
				id: message_id,
				query_id: 0,
				callback,
			}
			.into(),
		)
	}

	/// Handles a response from an XCM query and executes a callback if present.
	///
	/// No benchmark input parameters. A mock response is created and processed.
	#[benchmark]
	fn xcm_response() {
		let owner: AccountIdOf<T> = account("Alice", 0, SEED);
		let id_data = b"xcmresponse";
		let encoded = id_data.encode();
		let message_id: [u8; 32] = H256::from(blake2_256(&encoded)).into();

		let responder = Location { parents: 1, interior: Junctions::Here };
		let timeout = <BlockNumberOf<T> as One>::one() + frame_system::Pallet::<T>::block_number();
		let response = Response::ExecutionResult(None);

		let callback = Some(Callback {
			selector: [0; 4],
			weight: Weight::from_parts(100, 100),
			abi: Abi::Scale,
		});

		pallet_balances::Pallet::<T>::make_free_balance_be(&owner, u32::MAX.into());

		Pallet::<T>::xcm_new_query(
			RawOrigin::Signed(owner.clone()).into(),
			message_id,
			responder.clone(),
			timeout,
			callback.clone(),
		)
		.unwrap();

		let response_origin = T::XcmResponseOrigin::try_successful_origin().unwrap();

		#[extrinsic_call]
		Pallet::<T>::xcm_response(response_origin as RuntimeOrigin<T>, 0, response.clone());

		assert_has_event::<T>(
			crate::messaging::Event::XcmResponseReceived {
				dest: owner.clone(),
				id: message_id,
				query_id: 0,
				response,
			}
			.into(),
		);
	}

	/// Handles a response to a previously submitted ISMP request.
	///
	/// # Parameters
	/// - `x`: `Linear<0, 1>`   The type of ISMP response:
	///   - `0`: `PostResponse`
	///   - `1`: `GetResponse`
	#[benchmark]
	fn ismp_on_response(x: Linear<0, 1>) {
		let origin: T::AccountId = account("alice", 0, SEED);
		pallet_balances::Pallet::<T>::make_free_balance_be(&origin, u32::MAX.into());

		let id_data = (x, b"ismp_response");
		let encoded = id_data.encode();
		let message_id: [u8; 32] = H256::from(blake2_256(&encoded)).into();

		let weight = Weight::from_parts(100_000, 100_000);
		let cb_gas_deposit = T::WeightToFee::weight_to_fee(&weight);
		T::Fungibles::hold(&HoldReason::CallbackGas.into(), &origin, cb_gas_deposit).unwrap();
		// also hold for message deposit
		let message_deposit = calculate_protocol_deposit::<T, T::OnChainByteFee>(
			ProtocolStorageDeposit::IsmpRequests,
		) + calculate_message_deposit::<T, T::OnChainByteFee>() +
			calculate_deposit_of::<T, T::OffChainByteFee, ismp::Get<T>>();

		// Take some extra so we dont need to complicate the benchmark further.
		T::Fungibles::hold(&HoldReason::Messaging.into(), &origin, message_deposit * 2u32.into())
			.unwrap();

		let callback = Some(Callback { selector: [0; 4], weight, abi: Abi::Scale });

		let (response, event, commitment) = if x == 1 {
			// get response
			let get_response = ismp_get_response(
				T::MaxKeyLen::get() as usize,
				T::MaxKeys::get() as usize,
				T::MaxContextLen::get() as usize,
				T::MaxResponseLen::get() as usize,
			);
			let commitment = hash_request::<T::Keccak256>(&Request::Get(get_response.get.clone()));
			let get = IsmpResponse::Get(get_response);

			(
				get,
				crate::messaging::Event::<T>::IsmpGetResponseReceived {
					dest: origin.clone(),
					id: message_id,
					commitment,
				},
				commitment,
			)
		} else {
			// post response
			let post_response = ismp_post_response(
				T::MaxDataLen::get() as usize,
				T::MaxResponseLen::get() as usize,
			);

			let commitment =
				hash_request::<T::Keccak256>(&Request::Post(post_response.post.clone().clone()));
			let post = IsmpResponse::Post(post_response);

			(
				post,
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
			handler.on_response(response.clone()).unwrap()
		}

		assert_has_event::<T>(event.into())
	}

	/// Handles timeout of a pending ISMP request or response.
	///
	/// # Parameters
	/// - `x`: `Linear<0, 2>`   Type of item that timed out:
	///   - `0`: `PostRequest`
	///   - `1`: `GetRequest`
	///   - `2`: `PostResponse`
	#[benchmark]
	fn ismp_on_timeout(x: Linear<0, 2>) {
		let origin: T::AccountId = account("alice", 0, SEED);
		let id_data = (x, b"ismp_timeout");
		let encoded = id_data.encode();
		let message_id: [u8; 32] = H256::from(blake2_256(&encoded)).into();

		let callback = Some(Callback {
			selector: [1; 4],
			weight: Weight::from_parts(100, 100),
			abi: Abi::Scale,
		});

		let (timeout_message, commitment) = if x == 0 {
			let post_request = Request::Post(ismp_post_request(T::MaxDataLen::get() as usize));
			let commitment = hash_request::<T::Keccak256>(&post_request);
			(Timeout::Request(post_request), commitment)
		} else if x == 1 {
			let get_request = Request::Get(ismp_get_request(
				T::MaxKeyLen::get() as usize,
				T::MaxKeys::get() as usize,
				T::MaxContextLen::get() as usize,
			));
			let commitment = hash_request::<T::Keccak256>(&get_request);
			(Timeout::Request(get_request), commitment)
		} else {
			let post_response = ismp_post_response(
				T::MaxDataLen::get() as usize,
				T::MaxResponseLen::get() as usize,
			);
			let commitment =
				hash_request::<T::Keccak256>(&Request::Post(post_response.post.clone()));
			(Timeout::Response(post_response), commitment)
		};

		let event = Event::<T>::IsmpTimedOut { commitment };

		let input_message = Message::Ismp { commitment, callback, deposit: One::one() };

		IsmpRequests::<T>::insert(&commitment, (&origin, &message_id));
		Messages::<T>::insert(&origin, &message_id, &input_message);

		let handler = crate::messaging::ismp::Handler::<T>::new();
		#[block]
		{
			handler.on_timeout(timeout_message).unwrap()
		}
		assert_has_event::<T>(event.into());
	}

	/// Sends a `Get` request using ISMP with varying context and key sizes.
	///
	/// # Parameters
	/// - `y`: `Linear<0, { T::MaxContextLen::get() }>`   Length of the context field (in bytes).
	/// - `z`: `Linear<0, { T::MaxKeys::get() }>`   Number of keys in the outer keys array.
	/// - `a`: `Linear<0, 1>`   Whether a callback is attached:
	///   - `0`: No callback
	///   - `1`: Callback attached
	#[benchmark]
	fn ismp_get(
		x: Linear<0, { T::MaxContextLen::get() }>,
		y: Linear<0, { T::MaxKeys::get() }>,
		a: Linear<0, 1>,
	) {
		pallet_timestamp::Pallet::<T>::set_timestamp(1u32.into());
		let origin: T::AccountId = account("alice", 0, SEED);
		let id_data = (x, y, a, "ismp_get");
		let encoded = id_data.encode();
		let message_id = H256::from(blake2_256(&encoded));

		let inner_keys: BoundedVec<u8, T::MaxKeyLen> =
			vec![1u8].repeat(T::MaxKeyLen::get() as usize).try_into().unwrap();

		let mut outer_keys = vec![];
		for _ in 0..y {
			outer_keys.push(inner_keys.clone())
		}

		let callback = if a == 1 {
			Some(Callback {
				selector: [1; 4],
				weight: Weight::from_parts(100, 100),
				abi: Abi::Scale,
			})
		} else {
			None
		};

		let get = crate::messaging::ismp::Get::<T> {
			dest: 2000,
			height: 100_000,
			timeout: 100_000,
			context: vec![1u8].repeat(x as usize).try_into().unwrap(),
			keys: outer_keys.try_into().unwrap(),
		};

		pallet_balances::Pallet::<T>::make_free_balance_be(
			&origin,
			pallet_balances::Pallet::<T>::total_issuance() / 2u32.into(),
		);

		#[extrinsic_call]
		Pallet::<T>::ismp_get(RawOrigin::Signed(origin.clone()), message_id.into(), get, callback);
	}

	/// Sends a `Post` request using ISMP with a variable-sized data payload.
	///
	/// # Parameters
	/// - `x`: `Linear<0, { T::MaxDataLen::get() }>`   Length of the `data` field (in bytes).
	/// - `y`: `Linear<0, 1>`   Whether a callback is attached:
	///   - `0`: No callback
	///   - `1`: Callback attached
	#[benchmark]
	fn ismp_post(x: Linear<0, { T::MaxDataLen::get() }>, y: Linear<0, 1>) {
		pallet_timestamp::Pallet::<T>::set_timestamp(1u32.into());

		let origin: T::AccountId = account("alice", 0, SEED);
		let id_data = (x, y, b"ismp_post");
		let encoded = id_data.encode();
		let message_id = H256::from(blake2_256(&encoded));

		let data = vec![1u8].repeat(x as usize).try_into().unwrap();

		let get = crate::messaging::ismp::Post::<T> { dest: 2000, timeout: 100_000, data };

		let callback = if y == 1 {
			Some(Callback {
				selector: [1; 4],
				weight: Weight::from_parts(100, 100),

				abi: Abi::Scale,
			})
		} else {
			None
		};

		pallet_balances::Pallet::<T>::make_free_balance_be(
			&origin,
			pallet_balances::Pallet::<T>::total_issuance() / 2u32.into(),
		);

		#[extrinsic_call]
		Pallet::<T>::ismp_post(RawOrigin::Signed(origin.clone()), message_id.into(), get, callback);
	}

	#[benchmark]
	fn top_up_callback_weight() {
		let origin: T::AccountId = account("alice", 0, SEED);
		let message_id = [0u8; 32];
		let initial_weight = Weight::from_parts(100_000, 100_000);
		let additional_weight = Weight::from_parts(150_000, 150_000);
		let callback = Callback { abi: Abi::Scale, weight: initial_weight, selector: [0u8; 4] };

		let message: Message<T> = Message::Ismp {
			commitment: [10u8; 32].into(),
			deposit: 100_000u32.into(),
			callback: Some(callback),
		};

		Messages::<T>::insert(&origin, message_id, &message);

		pallet_balances::Pallet::<T>::make_free_balance_be(
			&origin,
			pallet_balances::Pallet::<T>::total_issuance() / 2u32.into(),
		);

		#[extrinsic_call]
		Pallet::<T>::top_up_callback_weight(
			RawOrigin::Signed(origin.clone()),
			message_id,
			additional_weight,
		);

		assert_has_event::<T>(
			Event::<T>::CallbackGasIncreased {
				message_id,
				total_weight: initial_weight + additional_weight,
			}
			.into(),
		);
	}

	impl_benchmark_test_suite!(Pallet, crate::mock::new_test_ext(), crate::mock::Test);
}

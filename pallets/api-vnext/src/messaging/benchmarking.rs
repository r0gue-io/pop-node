//! Benchmarking setup for pallet_api::messaging

use alloc::{vec, vec::Vec};
use core::u64;

use ::ismp::{
	dispatcher::{DispatchGet, DispatchPost},
	host::StateMachine,
	module::IsmpModule,
	router::{
		GetRequest, GetResponse, PostRequest, PostResponse, Request, Response, StorageValue,
		Timeout,
	},
};
use ::xcm::latest::{Junctions, Location, Response::ExecutionResult};
use codec::Encode;
use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok,
	pallet_prelude::{IsType, Weight},
	traits::{
		fungible::{Inspect, Mutate, MutateHold},
		EnsureOrigin, Get, Time,
	},
};
use pallet_revive::{
	precompiles::{
		alloy::primitives as alloy,
		run::{H256, U256},
		Error,
	},
	test_utils::ALICE_ADDR,
	Origin::Signed,
};
use sp_io::hashing::blake2_256;
use sp_runtime::traits::Bounded;

use super::{
	call_precompile,
	precompiles::{
		ismp::v0::{self as ismp, IISMPCalls, IISMP},
		v0::{IMessaging, IMessagingCalls},
		xcm::v0::{self as xcm, BlockNumberOf, IXCMCalls, IXCM},
	},
	set_up_call,
	transports::{
		ismp::{get, post, Handler, ID},
		xcm::new_query,
	},
	Call, Callback, Config, Encoding, Event, HoldReason, IsmpRequests, Message, MessageId,
	Messages, Origin, Pallet,
};
#[cfg(test)]
use crate::mock::{ExtBuilder, Test};
use crate::{messaging::BalanceOf, TryConvert};

type Balances<T> = <T as Config>::Fungibles;
type HostStateMachine<T> = <T as pallet_ismp::Config>::HostStateMachine;
type Ismp<T> = super::precompiles::ismp::v0::Ismp<4, T>;
type Messaging<T> = super::precompiles::v0::Messaging<3, T>;
type Xcm<T> = super::precompiles::xcm::v0::Xcm<5, T>;

#[benchmarks(
	where
	    // Precompiles
        T: pallet_revive::Config<
            Currency: Inspect<<T as frame_system::Config>::AccountId, Balance: Into<U256> + TryFrom<U256> + TryFrom<alloy::U256>>,
            Hash: IsType<H256>,
            Time: Time<Moment: Into<U256>>
        >,
        // Messaging
        T: Config<Fungibles: Inspect<T::AccountId, Balance: Bounded + TryConvert<alloy::U256, Error = Error>>>,
        alloy::U256: TryConvert<<<T as Config>::Fungibles as Inspect<T::AccountId>>::Balance, Error = Error>,
        T: pallet_ismp::Config + pallet_xcm::Config + parachain_info::Config,
        u32: From<BlockNumberOf<T>>,
        // Timestamp
        T: pallet_timestamp::Config
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn block_number() {
		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = IXCMCalls::blockNumber(IXCM::blockNumberCall {});

		#[block]
		{
			assert_ok!(call_precompile::<Xcm<T>, _, u32>(&mut ext, &Xcm::<T>::address(), &input));
		}
	}

	#[benchmark]
	fn get_response() {
		let origin = Origin::from_address::<T>(ALICE_ADDR);
		let message = 1;

		Messages::<T>::insert(
			message,
			Message::ismp_response(
				origin.address,
				[255; 32].into(),
				BalanceOf::<T>::max_value(),
				vec![255; T::MaxResponseLen::get() as usize].try_into().unwrap(),
			),
		);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Signed(origin.account));
		let mut ext = call_setup.ext().0;
		let input = IMessagingCalls::getResponse(IMessaging::getResponseCall { message });

		#[block]
		{
			assert_ok!(call_precompile::<Messaging<T>, _, u32>(
				&mut ext,
				&Messaging::<T>::address(),
				&input
			));
		}
	}

	#[benchmark]
	fn id() {
		let mut call_setup = set_up_call();
		let mut ext = call_setup.ext().0;
		let input = IMessagingCalls::id(IMessaging::idCall {});

		#[block]
		{
			assert_ok!(call_precompile::<Messaging<T>, _, u32>(
				&mut ext,
				&Messaging::<T>::address(),
				&input
			));
		}
	}

	/// Sends a `Get` request using ISMP with varying context and key sizes.
	///
	/// # Parameters
	/// - `y`: `Linear<0, { T::MaxContextLen::get() }>`   Length of the context field (in bytes).
	/// - `z`: `Linear<0, { T::MaxKeys::get() }>`   Number of keys in the outer keys array.
	/// - `a`: `Linear<0, 1>`   Whether a callback is attached:
	///   - `0`: No callback
	///   - `1`: Callback attached
	// IMPORTANT NOTE: `skip_meta` and `pov_mode = Measured` currently used due to an issue with the
	// usage of the `RequestCommitments` storage item within `pallet_ismp`'s child trie, which
	// results in massive proof size values resulting from complexity parameters, preventing
	// compilation of the resulting weights. Reducing max limits on pallet config also avoids the
	// issue, but results in values which are too small to be useful. See `pallet-contracts` and
	// `pallet-revive` benchmarks for similar usage with contract child tries.
	#[benchmark(skip_meta, pov_mode = Measured)]
	fn ismp_get(
		x: Linear<0, { T::MaxContextLen::get() }>,
		y: Linear<0, { T::MaxKeys::get() }>,
		a: Linear<0, 1>,
	) -> Result<(), BenchmarkError> {
		let origin = Origin::from_address::<T>(ALICE_ADDR);
		let request = ismp::Get {
			destination: u32::MAX,
			height: u64::MAX,
			timeout: u64::MAX,
			context: vec![255; x as usize].into(),
			keys: vec![vec![255u8; T::MaxKeyLen::get() as usize].into(); y as usize].into(),
		};
		let fee = <Balances<T>>::minimum_balance()
			.try_convert()
			.map_err(|_| BenchmarkError::Stop("failed to convert minimum balance to fee"))?;

		silence_timestamp_genesis_warnings::<T>();
		<Balances<T>>::set_balance(&origin.account, <Balances<T>>::total_issuance() / 2u32.into());

		let mut call_setup = set_up_call();
		call_setup.set_origin(Signed(origin.account));
		let mut ext = call_setup.ext().0;
		let input = if x == 0 {
			IISMPCalls::get_0(IISMP::get_0Call { request, fee })
		} else {
			let callback = ismp::Callback {
				destination: [255; 20].into(),
				encoding: ismp::Encoding::SolidityAbi,
				selector: [255; 4].into(),
				gasLimit: ismp::Weight { refTime: 100_000, proofSize: 100_000 },
				storageDepositLimit: alloy::U256::from(100_000),
			};
			IISMPCalls::get_1(IISMP::get_1Call { request, fee, callback })
		};

		#[block]
		{
			assert_ok!(call_precompile::<Ismp<T>, _, ()>(&mut ext, &Ismp::<T>::address(), &input));
		}

		Ok(())
	}

	/// Handles a response to a previously submitted ISMP request.
	///
	/// # Parameters
	/// - `x`: `Linear<0, 1>`   The type of ISMP response:
	///   - `0`: `GetResponse`
	///   - `1`: `PostResponse`
	#[benchmark]
	fn ismp_on_response(x: Linear<0, 1>) {
		let origin = Origin::from_address::<T>(ALICE_ADDR);
		let fee = <Balances<T>>::minimum_balance();
		let callback = Callback::new(
			[255; 20].into(),
			Encoding::Scale,
			[255; 4],
			Weight::from_parts(100_000, 100_000),
			100_000u32.into(),
		);
		let handler = Handler::<T>::new();

		silence_timestamp_genesis_warnings::<T>();
		<Balances<T>>::set_balance(&origin.account, u32::MAX.into());

		let (message_id, commitment, response) =
			ismp_request::<T>(x, origin.clone(), fee, callback);

		#[block]
		{
			handler.on_response(response).unwrap()
		}

		let event = match x {
			0 =>
				Event::IsmpGetResponseReceived { dest: origin.address, id: message_id, commitment },
			_ =>
				Event::IsmpPostResponseReceived { dest: origin.address, id: message_id, commitment },
		};
		assert_has_event::<T>(event.into())
	}

	/// Handles timeout of a pending ISMP request or response.
	///
	/// # Parameters
	/// - `x`: `Linear<0, 2>`   Type of item that timed out:
	///   - `0`: `GetRequest`
	///   - `1`: `PostRequest`
	///   - `2`: `PostResponse`
	#[benchmark]
	fn ismp_on_timeout(x: Linear<0, 2>) -> Result<(), BenchmarkError> {
		let origin = Origin::from_address::<T>(ALICE_ADDR);
		let fee = <Balances<T>>::minimum_balance();
		let callback = Callback::new(
			[255; 20].into(),
			Encoding::Scale,
			[255; 4],
			Weight::from_parts(100_000, 100_000),
			100_000u32.into(),
		);
		let handler = Handler::<T>::new();

		silence_timestamp_genesis_warnings::<T>();
		<Balances<T>>::set_balance(&origin.account, u32::MAX.into());

		let (_, commitment, response) = ismp_request::<T>(x, origin, fee, callback);
		let timeout = match x {
			0 => {
				let Response::Get(response) = response else {
					return Err(BenchmarkError::Stop("unexpected response"));
				};
				Timeout::Request(Request::Get(response.get))
			},
			_ => {
				let Response::Post(response) = response else {
					return Err(BenchmarkError::Stop("unexpected response"));
				};
				match x {
					1 => Timeout::Request(Request::Post(response.post)),
					_ => Timeout::Response(response),
				}
			},
		};

		#[block]
		{
			handler.on_timeout(timeout).unwrap()
		}

		assert_has_event::<T>(Event::<T>::IsmpTimedOut { commitment }.into());
		Ok(())
	}

	/// Sends a `Post` request using ISMP with a variable-sized data payload.
	///
	/// # Parameters
	/// - `x`: `Linear<0, { T::MaxDataLen::get() }>`   Length of the `data` field (in bytes).
	/// - `y`: `Linear<0, 1>`   Whether a callback is attached:
	///   - `0`: No callback
	///   - `1`: Callback attached
	// IMPORTANT NOTE: `skip_meta` and `pov_mode = Measured` currently used due to an issue with the
	// usage of the `RequestCommitments` storage item within `pallet_ismp`'s child trie, which
	// results in massive proof size values resulting from complexity parameters, preventing
	// compilation of the resulting weights. Reducing max limits on pallet config also avoids the
	// issue, but results in values which are too small to be useful. See `pallet-contracts` and
	// `pallet-revive` benchmarks for similar usage with contract child tries.
	#[benchmark(skip_meta, pov_mode = Measured)]
	fn ismp_post(
		x: Linear<0, { T::MaxDataLen::get() }>,
		y: Linear<0, 1>,
	) -> Result<(), BenchmarkError> {
		let origin = Origin::from_address::<T>(ALICE_ADDR);
		let request = ismp::Post {
			destination: u32::MAX,
			timeout: u64::MAX,
			data: vec![255; x as usize].into(),
		};
		let fee = <Balances<T>>::minimum_balance()
			.try_convert()
			.map_err(|_| BenchmarkError::Stop("failed to convert minimum balance to fee"))?;

		silence_timestamp_genesis_warnings::<T>();
		<Balances<T>>::set_balance(&origin.account, <Balances<T>>::total_issuance() / 2u32.into());

		let mut call_setup = set_up_call();
		call_setup.set_origin(Signed(origin.account));
		let mut ext = call_setup.ext().0;
		let input = if y == 0 {
			IISMPCalls::post_0(IISMP::post_0Call { request, fee })
		} else {
			let callback = ismp::Callback {
				destination: [255; 20].into(),
				encoding: ismp::Encoding::SolidityAbi,
				selector: [255; 4].into(),
				gasLimit: ismp::Weight { refTime: 100_000, proofSize: 100_000 },
				storageDepositLimit: alloy::U256::from(100_000),
			};
			IISMPCalls::post_1(IISMP::post_1Call { request, fee, callback })
		};

		#[block]
		{
			assert_ok!(call_precompile::<Ismp<T>, _, ()>(&mut ext, &Ismp::<T>::address(), &input));
		}

		Ok(())
	}

	#[benchmark]
	fn poll_status() {
		let origin = Origin::from_address::<T>(ALICE_ADDR);
		let message = 1;

		Messages::<T>::insert(
			message,
			Message::ismp_response(
				origin.address,
				[255; 32].into(),
				BalanceOf::<T>::max_value(),
				vec![255; T::MaxResponseLen::get() as usize].try_into().unwrap(),
			),
		);

		let mut call_setup = set_up_call();
		call_setup.set_origin(Signed(origin.account));
		let mut ext = call_setup.ext().0;
		let input = IMessagingCalls::pollStatus(IMessaging::pollStatusCall { message });

		#[block]
		{
			assert_ok!(call_precompile::<Messaging<T>, _, u32>(
				&mut ext,
				&Messaging::<T>::address(),
				&input
			));
		}
	}

	/// # Parameters
	/// - `x`: `Linear<1, { T::MaxRemovals::get() }>`   The number of message removals to perform
	///   (bounded by `MaxRemovals`).
	#[benchmark]
	fn remove(x: Linear<1, { T::MaxRemovals::get() }>) {
		let message_deposit = 50_000u32.into();
		let callback_deposit = 100_000u32.into();
		let origin = Origin::from_address::<T>(ALICE_ADDR);
		let messages: Vec<MessageId> = (0..x as MessageId).collect();

		let mut call_setup = set_up_call();
		call_setup.set_origin(Signed(origin.account.clone()));
		let mut ext = call_setup.ext().0;

		<Balances<T>>::set_balance(&origin.account, u32::MAX.into());
		for i in &messages {
			assert_ok!(T::Fungibles::hold(
				&HoldReason::Messaging.into(),
				&origin.account,
				message_deposit
			));
			assert_ok!(T::Fungibles::hold(
				&HoldReason::CallbackGas.into(),
				&origin.account,
				callback_deposit
			));

			let commitment = H256::from(blake2_256(&(i.to_le_bytes())));

			// Timeout messages release callback deposit hence, are most expensive case for now.
			let good_message = Message::ismp_timeout(
				origin.address,
				commitment.clone(),
				message_deposit,
				Some(callback_deposit),
			);

			Messages::<T>::insert(&i, &good_message);
			IsmpRequests::<T>::insert(&commitment, i);
		}

		let input =
			IMessagingCalls::remove_1(IMessaging::remove_1Call { messages: messages.into() });

		#[block]
		{
			assert_ok!(call_precompile::<Messaging<T>, _, ()>(
				&mut ext,
				&Messaging::<T>::address(),
				&input
			));
		}
	}

	/// Submits a new XCM query message with an optional callback.
	///
	/// # Parameters
	/// - `x`: `Linear<0, 1>`   Whether a callback is supplied:
	///   - `0`: No callback
	///   - `1`: Callback attached
	#[benchmark]
	fn xcm_new_query(x: Linear<0, 1>) {
		let origin = Origin::from_address::<T>(ALICE_ADDR);
		let responder = Location { parents: 1, interior: Junctions::Here }.encode().into();
		let timeout = u32::from(frame_system::Pallet::<T>::block_number()) + 1_000;

		<Balances<T>>::set_balance(&origin.account, u32::MAX.into());

		let mut call_setup = set_up_call();
		call_setup.set_origin(Signed(origin.account));
		let mut ext = call_setup.ext().0;
		let input = if x == 0 {
			IXCMCalls::newQuery_0(IXCM::newQuery_0Call { responder, timeout })
		} else {
			let callback = xcm::Callback {
				destination: [255; 20].into(),
				encoding: xcm::Encoding::SolidityAbi,
				selector: [0; 4].into(),
				gasLimit: xcm::Weight { refTime: 100, proofSize: 100 },
				storageDepositLimit: alloy::U256::from(100),
			};
			IXCMCalls::newQuery_1(IXCM::newQuery_1Call { responder, timeout, callback })
		};

		#[block]
		{
			assert_ok!(call_precompile::<Xcm<T>, _, ()>(&mut ext, &Xcm::<T>::address(), &input));
		}
	}

	/// Handles a response from an XCM query and executes a callback if present.
	///
	/// No benchmark input parameters. A mock response is created and processed.
	#[benchmark]
	fn xcm_response() {
		let origin = Origin::from_address::<T>(ALICE_ADDR);
		let responder = Location { parents: 1, interior: Junctions::Here };
		let timeout = frame_system::Pallet::<T>::block_number() + 1u8.into();
		let callback = Some(Callback {
			destination: [255; 20].into(),
			encoding: Encoding::Scale,
			selector: [0; 4],
			gas_limit: Weight::from_parts(100, 100),
			storage_deposit_limit: 100u8.into(),
		});
		let response_origin = T::XcmResponseOrigin::try_successful_origin().unwrap();
		let response = ExecutionResult(None);

		<Balances<T>>::set_balance(&origin.account, u32::MAX.into());

		let (message_id, query_id) =
			new_query::<T>(origin.clone(), responder, timeout, callback).unwrap();

		#[extrinsic_call]
		Pallet::<T>::xcm_response(response_origin, query_id, response.clone());

		assert_has_event::<T>(
			Event::XcmResponseReceived { dest: origin.address, id: message_id, query_id, response }
				.into(),
		);
	}

	impl_benchmark_test_suite!(Pallet, ExtBuilder::new().build(), Test);
}

// See if `generic_event` has been emitted.
fn assert_has_event<T: Config>(generic_event: <T as frame_system::Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

fn ismp_request<T: Config + pallet_ismp::Config>(
	x: u32,
	origin: Origin<T::AccountId>,
	fee: BalanceOf<T>,
	callback: Callback<BalanceOf<T>>,
) -> (MessageId, H256, Response) {
	if x == 0 {
		// get response
		let request = DispatchGet {
			dest: StateMachine::Polkadot(u32::MAX),
			from: ID.to_vec(),
			keys: vec![vec![255u8; T::MaxKeyLen::get() as usize]; T::MaxKeys::get() as usize],
			height: u64::MAX,
			context: vec![255u8; T::MaxContextLen::get() as usize],
			timeout: u64::MAX,
		};

		let value = StorageValue { key: vec![255u8; 1], value: Some(vec![255u8; 1]) };
		let response = GetResponse {
			get: GetRequest {
				source: HostStateMachine::<T>::get(),
				dest: request.dest,
				nonce: 0,
				from: request.from.clone(),
				keys: request.keys.clone(),
				height: request.height,
				context: request.context.clone(),
				timeout_timestamp: request.timeout,
			},
			values: (0..(T::MaxResponseLen::get() as usize / value.encoded_size())
				.saturating_sub(1))
				.map(|_| value.clone())
				.collect::<Vec<_>>(),
		};

		let (id, commitment) = get::<T>(origin, request, fee, Some(callback)).unwrap();
		(id, commitment, Response::Get(response))
	} else {
		// post response
		let request = DispatchPost {
			dest: StateMachine::Polkadot(u32::MAX),
			from: ID.to_vec(),
			to: ID.to_vec(),
			timeout: u64::MAX,
			body: vec![255u8; T::MaxDataLen::get() as usize],
		};

		let response = PostResponse {
			post: PostRequest {
				source: HostStateMachine::<T>::get(),
				dest: request.dest,
				nonce: 0,
				from: request.from.clone(),
				to: request.to.clone(),
				timeout_timestamp: request.timeout,
				body: request.body.clone(),
			},
			response: vec![255u8; T::MaxResponseLen::get() as usize - 2],
			timeout_timestamp: request.timeout,
		};

		let (id, commitment) = post::<T>(origin, request, fee, Some(callback)).unwrap();
		(id, commitment, Response::Post(response))
	}
}

// Silence ``pallet_timestamp::UnixTime::now` is called at genesis, invalid value returned: 0`
// warnings
fn silence_timestamp_genesis_warnings<T: pallet_timestamp::Config>() {
	pallet_timestamp::Pallet::<T>::set_timestamp(1u32.into());
}

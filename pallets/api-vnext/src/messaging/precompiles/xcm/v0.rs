use ::xcm::{VersionedLocation, VersionedXcm, MAX_XCM_DECODE_DEPTH};
use codec::{DecodeAll, DecodeLimit};
use pallet_xcm::WeightInfo as _;
use sp_runtime::{
	traits::{Block, Header},
	TokenError,
};
pub(crate) use IXCM::*;

use super::*;
use crate::messaging::{transports::xcm::new_query, Config};

sol!(
	#![sol(extra_derives(Debug, PartialEq))]
	"src/messaging/precompiles/interfaces/v0/IXCM.sol"
);

pub(crate) type BlockNumberOf<T> =
	<<<T as frame_system::Config>::Block as Block>::Header as Header>::Number;

/// The XCM precompile offers a streamlined interface for messaging using Polkadot's Cross-Consensus
/// Messaging (XCM).
pub struct Xcm<const FIXED: u16, T>(PhantomData<T>);
impl<
		const FIXED: u16,
		T: frame_system::Config
			+ pallet_revive::Config
			+ pallet_xcm::Config
			+ parachain_info::Config
			+ Config,
	> Precompile for Xcm<FIXED, T>
where
	BlockNumberOf<T>: From<u32>,
	u32: From<BlockNumberOf<T>>,
{
	type Interface = IXCMCalls;
	type T = T;

	const HAS_CONTRACT_INFO: bool = false;
	const MATCHER: AddressMatcher =
		Fixed(NonZero::new(FIXED).expect("expected non-zero precompile address"));

	fn call(
		_address: &[u8; 20],
		input: &Self::Interface,
		env: &mut impl Ext<T = Self::T>,
	) -> Result<Vec<u8>, Error> {
		match input {
			IXCMCalls::blockNumber(blockNumberCall {}) => {
				env.charge(<T as Config>::WeightInfo::block_number())?;

				let block_number = u32::from(frame_system::Pallet::<T>::block_number());

				Ok(blockNumberCall::abi_encode_returns(&block_number))
			},
			IXCMCalls::execute(executeCall { message, weight }) => {
				// Based on https://github.com/paritytech/polkadot-sdk/blob/master/polkadot/xcm/pallet-xcm/src/precompiles.rs
				let weight = weight.into();
				let charged = env.charge(weight)?;
				let message = VersionedXcm::decode_all_with_depth_limit(
					MAX_XCM_DECODE_DEPTH,
					&mut &message[..],
				)
				.map_err(|_| Error::from(IXCM::DecodingFailed))?
				.into();

				let result = <pallet_xcm::Pallet<T>>::execute(
					<T as frame_system::Config>::RuntimeOrigin::signed(
						env.caller().account_id()?.clone(),
					),
					message,
					weight,
				);

				if let Ok(result) = result {
					// Adjust weight
					if let Some(actual_weight) = result.actual_weight {
						// TODO: replace with `env.adjust_gas(charged, result.weight);` once
						// #8693 lands
						env.gas_meter_mut()
							.adjust_gas(charged, RuntimeCosts::Precompile(actual_weight));
					}
				}

				Ok(executeCall::abi_encode_returns(&result.encode().into()))
			},
			IXCMCalls::getResponse(getResponseCall { message }) => {
				env.charge(<T as Config>::WeightInfo::get_response())?;

				let response = get::<T>(message).into();

				Ok(getResponseCall::abi_encode_returns(&response))
			},
			IXCMCalls::id(idCall {}) => {
				env.charge(<T as Config>::WeightInfo::id())?;

				let id = id::<T>();

				Ok(idCall::abi_encode_returns(&id))
			},
			IXCMCalls::newQuery_0(newQuery_0Call { responder, timeout }) => {
				env.charge(<T as Config>::WeightInfo::xcm_new_query(0))?;
				let origin: Origin<_> = env.caller().try_into()?;
				let location = Location::decode(&mut &responder[..])
					.map_err(|_| Error::from(IXCM::DecodingFailed))?;
				let address = origin.address;

				let (id, query_id) = new_query::<T>(origin, location, (*timeout).into(), None)
					.map_err(Self::map_err)?;

				let account = address.0.into();
				deposit_event(env, QueryCreated_0 { account, id, queryId: query_id })?;
				Ok(newQuery_0Call::abi_encode_returns(&newQuery_0Return { id, queryId: query_id }))
			},
			IXCMCalls::newQuery_1(newQuery_1Call { responder, timeout, callback }) => {
				env.charge(<T as Config>::WeightInfo::xcm_new_query(1))?;
				let origin: Origin<_> = env.caller().try_into()?;
				let location = Location::decode(&mut &responder[..])
					.map_err(|_| Error::from(IXCM::DecodingFailed))?;
				let address = origin.address;

				let (id, query_id) =
					new_query::<T>(origin, location, (*timeout).into(), Some(callback.into()))
						.map_err(Self::map_err)?;

				let account = address.0.into();
				let event =
					QueryCreated_1 { account, id, callback: callback.clone(), queryId: query_id };
				deposit_event(env, event)?;
				Ok(newQuery_1Call::abi_encode_returns(&newQuery_1Return { id, queryId: query_id }))
			},
			IXCMCalls::pollStatus(pollStatusCall { message }) => {
				env.charge(<T as Config>::WeightInfo::poll_status())?;

				let status = poll_status::<T>(message).into();

				Ok(pollStatusCall::abi_encode_returns(&status))
			},
			IXCMCalls::remove_0(remove_0Call { message }) => {
				env.charge(<T as Config>::WeightInfo::remove(1))?;
				let origin = Origin::try_from(env.caller())?;
				let address = origin.address;

				remove::<T>(origin, &[*message]).map_err(Self::map_err)?;

				let account = address.0.into();
				deposit_event(env, Removed { account, messages: vec![*message] })?;
				Ok(remove_0Call::abi_encode_returns(&remove_0Return {}))
			},
			IXCMCalls::remove_1(remove_1Call { messages }) => {
				let messages_len = messages
					.len()
					.try_into()
					.map_err(|_| DispatchError::from(ArithmeticError::Overflow))?;
				env.charge(<T as Config>::WeightInfo::remove(messages_len))?;
				let origin = Origin::try_from(env.caller())?;
				let address = origin.address;

				remove::<T>(origin, messages).map_err(Self::map_err)?;

				let account = address.0.into();
				deposit_event(env, Removed { account, messages: messages.clone() })?;
				Ok(remove_1Call::abi_encode_returns(&remove_1Return {}))
			},
			IXCMCalls::send(sendCall { destination, message }) => {
				// Based on https://github.com/paritytech/polkadot-sdk/blob/master/polkadot/xcm/pallet-xcm/src/precompiles.rs
				env.charge(<T as pallet_xcm::Config>::WeightInfo::send())?;
				let destination = VersionedLocation::decode_all(&mut &destination[..])
					.map_err(|_| Error::from(IXCM::DecodingFailed))?
					.into();
				let message = VersionedXcm::decode_all_with_depth_limit(
					MAX_XCM_DECODE_DEPTH,
					&mut &message[..],
				)
				.map_err(|_| Error::from(IXCM::DecodingFailed))?
				.into();

				let result = <pallet_xcm::Pallet<T>>::send(
					<T as frame_system::Config>::RuntimeOrigin::signed(
						env.caller().account_id()?.clone(),
					),
					destination,
					message,
				);

				Ok(sendCall::abi_encode_returns(&result.encode().into()))
			},
		}
	}
}

impl<const FIXED: u16, T: frame_system::Config> Xcm<FIXED, T> {
	/// The address of the precompile.
	pub const fn address() -> [u8; 20] {
		fixed_address(FIXED)
	}

	// Maps select, domain-specific dispatch errors to messaging errors. Anything not mapped results
	// in a `Error::Error(ExecError::DispatchError)` which results in trap rather than a revert.
	fn map_err(e: DispatchError) -> Error {
		use DispatchError::*;
		match e {
			Module(ModuleError { index, error, .. }) => {
				let index = Some(index as usize);
				if index == T::PalletInfo::index::<Pallet<T>>() {
					use messaging::Error::{self, *};

					match Error::<T>::decode(&mut error.as_slice()) {
						Ok(FutureTimeoutMandatory) => IXCM::FutureTimeoutMandatory.into(),
						Ok(MaxMessageTimeoutPerBlockReached) =>
							IXCM::MaxMessageTimeoutPerBlockReached.into(),
						Ok(MessageNotFound) => self::MessageNotFound.into(),
						Ok(OriginConversionFailed) => IXCM::OriginConversionFailed.into(),
						Ok(RequestPending) => self::RequestPending.into(),
						Ok(TooManyMessages) => self::TooManyMessages.into(),
						_ => e.into(),
					}
				} else {
					e.into()
				}
			},
			Token(TokenError::FundsUnavailable) => IXCM::FundsUnavailable.into(),
			_ => e.into(),
		}
	}
}

// Encoding of custom errors via `Error(String)`.
impl_from_sol_error! {
	IXCM::DecodingFailed,
	IXCM::FundsUnavailable,
	IXCM::FutureTimeoutMandatory,
	IXCM::MaxMessageTimeoutPerBlockReached,
	IXCM::OriginConversionFailed,
	MessageNotFound,
	RequestPending,
	TooManyMessages,
}

impl EncodeCallback for Response {
	fn encode(&self, encoding: messaging::Encoding, selector: [u8; 4], id: MessageId) -> Vec<u8> {
		use messaging::Encoding::*;
		// XCM responses are always SCALE-encoded.
		let response = codec::Encode::encode(&self);
		match encoding {
			Scale => [selector.to_vec(), (id, response).encode()].concat(),
			SolidityAbi => {
				let call = IQueryResponse::onQueryResponseCall { id, response: response.into() };
				let mut data = call.abi_encode();
				debug_assert_eq!(data[..4], selector);
				// Replace selector with that provided at request
				data.splice(0..4, selector);
				data
			},
		}
	}
}

impl From<&Callback> for super::Callback {
	fn from(callback: &Callback) -> Self {
		Self::new(
			(*callback.destination.0).into(),
			(&callback.encoding).into(),
			callback.selector.0,
			(&callback.weight).into(),
		)
	}
}

impl From<&Encoding> for super::Encoding {
	fn from(encoding: &Encoding) -> Self {
		match encoding {
			Encoding::Scale => Self::Scale,
			Encoding::SolidityAbi => Self::SolidityAbi,
			// TODO
			Encoding::__Invalid => unimplemented!(),
		}
	}
}

impl From<messaging::MessageStatus> for self::MessageStatus {
	fn from(value: messaging::MessageStatus) -> Self {
		use messaging::MessageStatus::*;
		match value {
			NotFound => Self::NotFound,
			Pending => Self::Pending,
			Complete => Self::Complete,
			Timeout => Self::Timeout,
		}
	}
}

impl From<&Weight> for super::Weight {
	fn from(weight: &Weight) -> Self {
		Self::from_parts(weight.refTime, weight.proofSize)
	}
}

impl From<super::Weight> for Weight {
	fn from(weight: super::Weight) -> Self {
		Self { refTime: weight.ref_time(), proofSize: weight.proof_size() }
	}
}

#[cfg(test)]
mod tests {
	use ::xcm::{
		latest::{Junction::Parachain, Response, Xcm},
		v5::WeightLimit,
	};
	use frame_support::{assert_ok, dispatch::PostDispatchInfo, weights::Weight};
	use mock::{messaging::RESPONSE_LOCATION, ExtBuilder, *};
	use pallet_revive::{
		precompiles::{
			alloy::sol_types::{SolInterface, SolType},
			Error,
		},
		test_utils::ALICE,
	};
	use pallet_xcm::ExecutionError;

	use super::{IXCMCalls::*, MessageStatus::*, *};

	type MaxXcmQueryTimeoutsPerBlock = <Test as Config>::MaxXcmQueryTimeoutsPerBlock;
	type MaxRemovals = <Test as Config>::MaxRemovals;
	type Messages = crate::messaging::Messages<Test>;
	type XcmQueryTimeouts = crate::messaging::XcmQueryTimeouts<Test>;

	const ADDRESS: [u8; 20] = fixed_address(XCM);
	const MESSAGE_DEPOSIT: u128 = 129_440;

	#[test]
	fn block_number_works() {
		let origin = ALICE;
		let block_number = 1;
		ExtBuilder::new().build().execute_with(|| {
			assert_eq!(
				call_precompile::<u32>(&origin, &blockNumber(blockNumberCall {})).unwrap(),
				block_number
			);
		});
	}

	#[test]
	fn execute_reverts_when_decoding_failed() {
		let origin = ALICE;
		let message = Vec::default().into();
		let weight = Weight::default().into();
		ExtBuilder::new().build().execute_with(|| {
			let input = execute(executeCall { message, weight });
			assert_revert!(call_precompile::<()>(&origin, &input), DecodingFailed);
		});
	}

	#[test]
	fn execute_works() {
		let origin = ALICE;
		let asset = (Location::here(), 100);
		let xcm = Xcm::<()>::builder()
			.withdraw_asset(asset.clone())
			.buy_execution(asset, WeightLimit::Unlimited)
			.build();
		let versioned_xcm = VersionedXcm::V5(xcm);
		let message = versioned_xcm.encode().into();
		let weight = Weight::from_parts(100_000, 100_000).into();
		ExtBuilder::new().build().execute_with(|| {
			let call = execute(executeCall { message, weight });
			let response = call_precompile::<Vec<u8>>(&origin, &call).unwrap();
			let result = Result::<PostDispatchInfo, DispatchErrorWithPostInfo>::decode(
				&mut response.as_slice(),
			)
			.unwrap();

			// No xcm-executor currently configured in mock runtime
			assert_eq!(
				result,
				Err(DispatchErrorWithPostInfo {
					post_info: PostDispatchInfo {
						actual_weight: Some(Weight::from_parts(100_000_000, 0)),
						pays_fee: Pays::Yes
					},
					error: pallet_xcm::Error::<Test>::LocalExecutionIncompleteWithError {
						index: 0,
						error: ExecutionError::Unimplemented
					}
					.into()
				})
			);
		});
	}

	#[test]
	fn get_response_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = 1;
		let response = Response::Null;
		ExtBuilder::new()
			.with_messages(vec![(
				origin.account.clone(),
				message,
				Message::xcm_response(origin.address, 0, 0, response.clone()),
				0,
			)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<Vec<u8>>(
						&origin.account,
						&getResponse(getResponseCall { message })
					)
					.unwrap(),
					Encode::encode(&response)
				);
			});
	}

	#[test]
	fn id_works() {
		let origin = ALICE;
		ExtBuilder::new().build().execute_with(|| {
			let para_id = u32::from(ParachainInfo::parachain_id());
			assert_eq!(
				call_precompile::<u32>(&origin, &IXCMCalls::id(idCall {})).unwrap(),
				para_id
			);
		});
	}

	#[test]
	fn new_query_reverts_when_decoding_failed() {
		let origin = ALICE;
		let responder = Vec::default().into();
		let timeout = 100;
		ExtBuilder::new().build().execute_with(|| {
			let input = newQuery_0(newQuery_0Call { responder, timeout });
			assert_revert!(call_precompile::<()>(&origin, &input), DecodingFailed);
		});
	}

	#[test]
	fn new_query_reverts_when_timeout_passed() {
		let origin = ALICE;
		let responder = RESPONSE_LOCATION.encode().into();
		let timeout = 0;
		ExtBuilder::new().build().execute_with(|| {
			let input = newQuery_0(newQuery_0Call { responder, timeout });
			assert_revert!(call_precompile::<()>(&origin, &input), FutureTimeoutMandatory);
		});
	}

	#[test]
	fn new_query_reverts_when_max_query_timeouts_reached() {
		let origin = ALICE;
		let responder = RESPONSE_LOCATION.encode().into();
		let timeout = 2;
		ExtBuilder::new().build().execute_with(|| {
			let current_block = frame_system::Pallet::<Test>::block_number();
			XcmQueryTimeouts::set(
				current_block + timeout,
				vec![0; MaxXcmQueryTimeoutsPerBlock::get() as usize].try_into().unwrap(),
			);

			let input = newQuery_0(newQuery_0Call { responder, timeout });
			assert_revert!(
				call_precompile::<()>(&origin, &input),
				MaxMessageTimeoutPerBlockReached
			);
		});
	}

	#[test]
	fn new_query_reverts_when_funds_unavailable() {
		let origin = ALICE;
		let responder = RESPONSE_LOCATION.encode().into();
		let timeout = 2;
		ExtBuilder::new().build().execute_with(|| {
			let input = newQuery_0(newQuery_0Call { responder, timeout });
			assert_revert!(call_precompile::<()>(&origin, &input), FundsUnavailable);
		});
	}

	#[test]
	fn new_query_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let responder = RESPONSE_LOCATION.encode().into();
		let timeout = 100;
		let message = 1;
		let query_id = 2;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), 1 * UNIT)]) // message deposit
			.with_message_id(message)
			.with_query_id(query_id)
			.build()
			.execute_with(|| {
			    let call = newQuery_0(newQuery_0Call { responder, timeout });
				assert_eq!(
					call_precompile::<MessageId>(&origin.account, &call).unwrap(),
					message
				);

				let account = origin.address.0.into();
				let event = QueryCreated_0 { account, id: message, queryId: query_id };
				assert_last_event(ADDRESS, event);
				assert!(matches!(
					Messages::get(message),
					Some(Message::XcmQuery { origin: o, query_id: qid, callback, message_deposit })
					    if o == origin && qid == query_id && callback.is_none() && message_deposit == MESSAGE_DEPOSIT)
				);
			});
	}

	#[test]
	fn new_query_with_callback_reverts_when_decoding_failed() {
		let origin = ALICE;
		let responder = Vec::default().into();
		let timeout = 100;
		let callback = Callback {
			destination: [255u8; 20].into(),
			encoding: super::Encoding::Scale,
			selector: [255u8; 4].into(),
			weight: super::Weight { refTime: 100, proofSize: 10 },
		};
		ExtBuilder::new().build().execute_with(|| {
			let input = newQuery_1(newQuery_1Call { responder, timeout, callback });
			assert_revert!(call_precompile::<()>(&origin, &input), DecodingFailed);
		});
	}

	#[test]
	fn new_query_with_callback_reverts_when_timeout_passed() {
		let origin = ALICE;
		let responder = RESPONSE_LOCATION.encode().into();
		let timeout = 0;
		let callback = Callback {
			destination: [255u8; 20].into(),
			encoding: super::Encoding::Scale,
			selector: [255u8; 4].into(),
			weight: super::Weight { refTime: 100, proofSize: 10 },
		};
		ExtBuilder::new().build().execute_with(|| {
			let input = newQuery_1(newQuery_1Call { responder, timeout, callback });
			assert_revert!(call_precompile::<()>(&origin, &input), FutureTimeoutMandatory);
		});
	}

	#[test]
	fn new_query_with_callback_reverts_when_max_query_timeouts_reached() {
		let origin = ALICE;
		let responder = RESPONSE_LOCATION.encode().into();
		let timeout = 2;
		let callback = Callback {
			destination: [255u8; 20].into(),
			encoding: super::Encoding::Scale,
			selector: [255u8; 4].into(),
			weight: super::Weight { refTime: 100, proofSize: 10 },
		};
		ExtBuilder::new().build().execute_with(|| {
			let current_block = frame_system::Pallet::<Test>::block_number();
			XcmQueryTimeouts::set(
				current_block + timeout,
				vec![0; MaxXcmQueryTimeoutsPerBlock::get() as usize].try_into().unwrap(),
			);

			let input = newQuery_1(newQuery_1Call { responder, timeout, callback });
			assert_revert!(
				call_precompile::<()>(&origin, &input),
				MaxMessageTimeoutPerBlockReached
			);
		});
	}

	#[test]
	fn new_query_with_callback_reverts_when_funds_unavailable() {
		let origin = ALICE;
		let responder = RESPONSE_LOCATION.encode().into();
		let timeout = 2;
		let callback = Callback {
			destination: [255u8; 20].into(),
			encoding: super::Encoding::Scale,
			selector: [255u8; 4].into(),
			weight: super::Weight { refTime: 100, proofSize: 10 },
		};
		ExtBuilder::new().build().execute_with(|| {
			let input = newQuery_1(newQuery_1Call { responder, timeout, callback });
			assert_revert!(call_precompile::<()>(&origin, &input), FundsUnavailable);
		});
	}

	#[test]
	fn new_query_with_callback_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let responder = RESPONSE_LOCATION.encode().into();
		let timeout = 100;
		let callback = Callback {
			destination: [255u8; 20].into(),
			encoding: super::Encoding::Scale,
			selector: [255u8; 4].into(),
			weight: super::Weight { refTime: 100, proofSize: 10 },
		};
		let message = 1;
		let query_id = 2;
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), 1 * UNIT)]) // message deposit
			.with_message_id(message)
			.with_query_id(query_id)
			.build()
			.execute_with(|| {
			    let call = newQuery_1(newQuery_1Call { responder, timeout, callback: callback.clone() });
				assert_eq!(
					call_precompile::<MessageId>(&origin.account, &call).unwrap(),
					message
				);

				let account = origin.address.0.into();
				let event = QueryCreated_1 { account, id: message, queryId: query_id, callback: callback.clone() };
				assert_last_event(ADDRESS, event);
				assert!(matches!(
					Messages::get(message),
					Some(Message::XcmQuery { origin: o, query_id: qid, callback: cb, message_deposit })
					    if o == origin && qid == query_id && cb == Some((&callback).into()) && message_deposit == MESSAGE_DEPOSIT)
				);
			});
	}

	#[test]
	fn poll_status_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let expected = [(0, NotFound), (1, Pending), (2, Complete), (3, Timeout)];
		let messages = [
			(1, Message::xcm_query(origin.clone(), 0, None, 0)),
			(2, Message::xcm_response(origin.address, 0, 0, Response::Null)),
			(3, Message::xcm_timeout(origin.address, 0, 0, None)),
		];
		ExtBuilder::new()
			.with_messages(messages.map(|(i, m)| (origin.account.clone(), i, m, 0)).to_vec())
			.build()
			.execute_with(|| {
				for (message, expected) in expected {
					assert_eq!(
						call_precompile::<super::MessageStatus>(
							&origin.account,
							&pollStatus(pollStatusCall { message })
						)
						.unwrap(),
						expected.into()
					);
				}
			});
	}

	#[test]
	fn remove_reverts_when_message_pending() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = 1;
		ExtBuilder::new()
			.with_messages(vec![(
				origin.account.clone(),
				message,
				Message::xcm_query(origin.clone(), 0, None, 0),
				0,
			)])
			.build()
			.execute_with(|| {
				assert_revert!(
					call_precompile::<()>(&origin.account, &remove_0(remove_0Call { message })),
					RequestPending
				);
			});
	}

	#[test]
	fn remove_reverts_when_message_not_found() {
		let origin = ALICE;
		let message = 1;
		ExtBuilder::new().build().execute_with(|| {
			assert_revert!(
				call_precompile::<()>(&origin, &remove_0(remove_0Call { message })),
				MessageNotFound
			);
		});
	}

	#[test]
	fn remove_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = 1;
		ExtBuilder::new()
			.with_messages(vec![(
				origin.account.clone(),
				message,
				Message::xcm_response(origin.address, 0, 0, Response::Null),
				0,
			)])
			.build()
			.execute_with(|| {
				assert_ok!(call_precompile::<()>(
					&origin.account,
					&remove_0(remove_0Call { message })
				));

				let account = origin.address.0.into();
				assert_last_event(ADDRESS, Removed { account, messages: vec![message] });
			});
	}

	#[test]
	fn remove_many_reverts_when_message_pending() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = Message::xcm_response(origin.address, 0, 0, Response::Null);
		let messages = <MaxRemovals as Get<u32>>::get() as u64;
		ExtBuilder::new()
			.with_messages(
				(0..messages - 1)
					.map(|i| (origin.account.clone(), i, message.clone(), 0))
					.chain(vec![(
						origin.account.clone(),
						messages,
						Message::xcm_query(origin.clone(), 0, None, 0),
						0,
					)])
					.collect(),
			)
			.build()
			.execute_with(|| {
				assert_revert!(
					call_precompile::<()>(
						&origin.account,
						&remove_1(remove_1Call { messages: (0..messages).collect() })
					),
					MessageNotFound
				);
			});
	}

	#[test]
	fn remove_many_reverts_when_message_not_found() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = Message::xcm_response(origin.address, 0, 0, Response::Null);
		let messages = <MaxRemovals as Get<u32>>::get() as u64;
		ExtBuilder::new()
			.with_messages(
				(0..messages - 1)
					.map(|i| (origin.account.clone(), i, message.clone(), 0))
					.collect(),
			)
			.build()
			.execute_with(|| {
				assert_revert!(
					call_precompile::<()>(
						&origin.account,
						&remove_1(remove_1Call { messages: (0..messages).collect() })
					),
					MessageNotFound
				);
			});
	}

	#[test]
	fn remove_many_reverts_when_too_many_messages() {
		let origin = ALICE;
		let messages = <MaxRemovals as Get<u32>>::get() as u64 + 1;
		ExtBuilder::new().build().execute_with(|| {
			assert_revert!(
				call_precompile::<()>(
					&origin,
					&remove_1(remove_1Call { messages: (0..messages).collect() })
				),
				TooManyMessages
			);
		});
	}

	#[test]
	fn remove_many_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let messages = 10;
		let message = Message::xcm_response(origin.address, 0, 0, Response::Null);
		ExtBuilder::new()
			.with_messages(
				(0..messages).map(|i| (origin.account.clone(), i, message.clone(), 0)).collect(),
			)
			.build()
			.execute_with(|| {
				let messages: Vec<_> = (0..messages).collect();
				assert_ok!(call_precompile::<()>(
					&origin.account,
					&remove_1(remove_1Call { messages: messages.clone() })
				));

				let account = origin.address.0.into();
				assert_last_event(ADDRESS, Removed { account, messages });
			});
	}

	#[test]
	fn send_reverts_when_decoding_failed() {
		let origin = ALICE;
		let destination = Location::new(1, [Parachain(1000).into()]);
		let versioned_location = VersionedLocation::V5(destination);
		let destination = versioned_location.encode().into();
		ExtBuilder::new().build().execute_with(|| {
			let input = send(sendCall {
				destination: Vec::default().into(),
				message: Vec::default().into(),
			});
			assert_revert!(call_precompile::<()>(&origin, &input), DecodingFailed);

			let input = send(sendCall { destination, message: Vec::default().into() });
			assert_revert!(call_precompile::<()>(&origin, &input), DecodingFailed);
		});
	}

	#[test]
	fn send_works() {
		let origin = ALICE;
		let destination = Location::new(1, [Parachain(1000).into()]);
		let versioned_location = VersionedLocation::V5(destination);
		let destination = versioned_location.encode().into();
		let asset = (Location::here(), 100);
		let xcm = Xcm::<()>::builder()
			.withdraw_asset(asset.clone())
			.buy_execution(asset, WeightLimit::Unlimited)
			.build();
		let versioned_xcm = VersionedXcm::V5(xcm);
		let message = versioned_xcm.encode().into();
		ExtBuilder::new().build().execute_with(|| {
			let call = send(sendCall { destination, message });
			let response = call_precompile::<Vec<u8>>(&origin, &call).unwrap();
			let result = Result::<(), DispatchError>::decode(&mut response.as_slice()).unwrap();
			// No xcm router currently configured in mock runtime
			assert_eq!(result, Err(pallet_xcm::Error::<Test>::Unreachable.into()));
		});
	}

	fn call_precompile<Output: SolValue + From<<Output::SolType as SolType>::RustType>>(
		origin: &AccountId,
		input: &IXCMCalls,
	) -> Result<Output, Error> {
		bare_call::<Test, Output>(
			RuntimeOrigin::signed(origin.clone()),
			ADDRESS.into(),
			0,
			Weight::MAX,
			DepositLimit::Balance(u128::MAX),
			input.abi_encode(),
		)
	}
}

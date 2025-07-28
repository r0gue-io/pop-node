use ::ismp::{
	dispatcher::{DispatchGet, DispatchPost},
	host::StateMachine,
};
use frame_support::{
	ensure,
	traits::{tokens::fungible::Inspect, Get as _},
};
pub(crate) use IISMP::{Get, *};

use super::*;
use crate::{
	messaging::transports::ismp::{get, post, ID},
	TryConvert,
};

sol!(
	#![sol(extra_derives(Debug, PartialEq))]
	"src/messaging/precompiles/interfaces/v0/IISMP.sol"
);

/// The ISMP precompile offers a streamlined interface for messaging using the Interoperable State
/// Machine Protocol.
pub struct Ismp<const FIXED: u16, T>(PhantomData<T>);
impl<
		const FIXED: u16,
		T: frame_system::Config
			+ pallet_revive::Config
			+ parachain_info::Config
			+ Config<Fungibles: Inspect<T::AccountId, Balance: TryConvert<U256, Error = Error>>>,
	> Precompile for Ismp<FIXED, T>
where
	U256: TryConvert<<<T as Config>::Fungibles as Inspect<T::AccountId>>::Balance, Error = Error>,
{
	type Interface = IISMPCalls;
	type T = T;

	const HAS_CONTRACT_INFO: bool = false;
	const MATCHER: AddressMatcher =
		Fixed(NonZero::new(FIXED).expect("expected non-zero precompile address"));

	fn call(
		_address: &[u8; 20],
		input: &Self::Interface,
		env: &mut impl Ext<T = Self::T>,
	) -> Result<Vec<u8>, pallet_revive::precompiles::Error> {
		match input {
			IISMPCalls::get_0(get_0Call { request, fee }) => {
				env.charge(<T as Config>::WeightInfo::ismp_get(
					request
						.context
						.len()
						.try_into()
						.map_err(|_| DispatchError::from(ArithmeticError::Overflow))?,
					request
						.keys
						.len()
						.try_into()
						.map_err(|_| DispatchError::from(ArithmeticError::Overflow))?,
					0,
				))?;
				let origin = Origin::try_from(env.caller())?;
				let message = try_get::<T>(request)?;
				let fee = (*fee).try_convert()?;
				let address = origin.address;

				let (id, commitment) =
					get::<T>(origin, message, fee, None).map_err(Self::map_err)?;

				let origin = address.0.into();
				let event = GetDispatched_0 { origin, id, commitment: commitment.0.into() };
				deposit_event(env, event)?;
				Ok(get_0Call::abi_encode_returns(&id))
			},
			IISMPCalls::get_1(get_1Call { request, fee, callback }) => {
				env.charge(<T as Config>::WeightInfo::ismp_get(
					request
						.context
						.len()
						.try_into()
						.map_err(|_| DispatchError::from(ArithmeticError::Overflow))?,
					request
						.keys
						.len()
						.try_into()
						.map_err(|_| DispatchError::from(ArithmeticError::Overflow))?,
					1,
				))?;
				let origin = Origin::try_from(env.caller())?;
				let message = try_get::<T>(request)?;
				let fee = (*fee).try_convert()?;
				let address = origin.address;

				let (id, commitment) =
					get::<T>(origin, message, fee, Some(callback.into())).map_err(Self::map_err)?;

				let origin = address.0.into();
				let commitment = commitment.0.into();
				let event = GetDispatched_1 { origin, id, commitment, callback: callback.clone() };
				deposit_event(env, event)?;
				Ok(get_1Call::abi_encode_returns(&id))
			},
			IISMPCalls::getResponse(getResponseCall { message }) => {
				env.charge(<T as Config>::WeightInfo::get_response())?;

				let response = super::get::<T>(message).into();

				Ok(getResponseCall::abi_encode_returns(&response))
			},
			IISMPCalls::id(idCall {}) => {
				env.charge(<T as Config>::WeightInfo::id())?;

				let id = id::<T>();

				Ok(idCall::abi_encode_returns(&id))
			},
			IISMPCalls::pollStatus(pollStatusCall { message }) => {
				env.charge(<T as Config>::WeightInfo::poll_status())?;

				let status = poll_status::<T>(message).into();

				Ok(pollStatusCall::abi_encode_returns(&status))
			},
			IISMPCalls::post_0(post_0Call { request, fee }) => {
				env.charge(<T as Config>::WeightInfo::ismp_post(
					request
						.data
						.len()
						.try_into()
						.map_err(|_| DispatchError::from(ArithmeticError::Overflow))?,
					0,
				))?;
				let origin: Origin<_> = env.caller().try_into()?;
				let message = try_post::<T>(request)?;
				let fee = (*fee).try_convert()?;
				let address = origin.address;

				let (id, commitment) =
					post::<T>(origin, message, fee, None).map_err(Self::map_err)?;

				let origin = address.0.into();
				let event = PostDispatched_0 { origin, id, commitment: commitment.0.into() };
				deposit_event(env, event)?;
				Ok(post_0Call::abi_encode_returns(&id))
			},
			IISMPCalls::post_1(post_1Call { request, fee, callback }) => {
				env.charge(<T as Config>::WeightInfo::ismp_post(
					request
						.data
						.len()
						.try_into()
						.map_err(|_| DispatchError::from(ArithmeticError::Overflow))?,
					1,
				))?;
				let origin: Origin<_> = env.caller().try_into()?;
				let message = try_post::<T>(request)?;
				let fee = (*fee).try_convert()?;
				let address = origin.address;

				let (id, commitment) = post::<T>(origin, message, fee, Some(callback.into()))
					.map_err(Self::map_err)?;

				let origin = address.0.into();
				let commitment = commitment.0.into();
				let event = PostDispatched_1 { origin, id, commitment, callback: callback.clone() };
				deposit_event(env, event)?;
				Ok(post_0Call::abi_encode_returns(&id))
			},
			IISMPCalls::remove_0(remove_0Call { message }) => {
				env.charge(<T as Config>::WeightInfo::remove(1))?;
				let origin = Origin::try_from(env.caller())?;
				let address = origin.address;

				remove::<T>(origin, &[*message]).map_err(Self::map_err)?;

				let account = address.0.into();
				deposit_event(env, Removed { account, messages: vec![*message] })?;
				Ok(remove_0Call::abi_encode_returns(&remove_0Return {}))
			},
			IISMPCalls::remove_1(remove_1Call { messages }) => {
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
		}
	}
}

impl<const FIXED: u16, T: frame_system::Config> Ismp<FIXED, T> {
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
						Ok(MessageNotFound) => self::MessageNotFound.into(),
						Ok(RequestPending) => self::RequestPending.into(),
						Ok(TooManyMessages) => self::TooManyMessages.into(),
						_ => e.into(),
					}
				} else {
					e.into()
				}
			},
			_ => e.into(),
		}
	}
}

// Encoding of custom errors via `Error(String)`.
impl_from_sol_error! {
	IISMP::MaxContextExceeded,
	IISMP::MaxDataExceeded,
	IISMP::MaxKeyExceeded,
	IISMP::MaxKeysExceeded,
	MessageNotFound,
	RequestPending,
	TooManyMessages,
}

fn try_get<T: Config>(value: &Get) -> Result<DispatchGet, Error> {
	ensure!(value.context.len() as u32 <= T::MaxContextLen::get(), IISMP::MaxContextExceeded);
	ensure!(value.keys.len() as u32 <= T::MaxKeys::get(), IISMP::MaxKeysExceeded);
	ensure!(
		value.keys.iter().all(|k| k.len() as u32 <= T::MaxKeyLen::get()),
		IISMP::MaxKeyExceeded
	);
	Ok(DispatchGet {
		dest: StateMachine::Polkadot(value.destination),
		from: ID.into(),
		keys: value.keys.iter().map(|key| key.to_vec()).collect(),
		height: value.height.into(),
		context: value.context.to_vec(),
		timeout: value.timeout,
	})
}

fn try_post<T: Config>(value: &Post) -> Result<DispatchPost, Error> {
	ensure!(value.data.len() as u32 <= T::MaxDataLen::get(), IISMP::MaxDataExceeded);
	Ok(DispatchPost {
		dest: StateMachine::Polkadot(value.destination),
		from: ID.into(),
		to: ID.into(),
		timeout: value.timeout,
		body: value.data.to_vec(),
	})
}

impl EncodeCallback for Vec<::ismp::router::StorageValue> {
	fn encode(&self, encoding: messaging::Encoding, selector: [u8; 4], id: MessageId) -> Vec<u8> {
		use messaging::Encoding::*;
		match encoding {
			Scale => [selector.to_vec(), (id, self).encode()].concat(),
			SolidityAbi => {
				// Use interface to encode call data
				let call = IGetResponse::onGetResponseCall {
					id,
					// Clones required for ABI encoding of dynamic bytes type.
					response: self
						.into_iter()
						.map(|v| IISMP::StorageValue {
							key: v.key.clone().into(),
							value: v.value.as_ref().map_or_else(
								|| IISMP::Value { exists: false, value: Default::default() },
								|v| IISMP::Value { exists: true, value: v.clone().into() },
							),
						})
						.collect(),
				};
				let mut data = call.abi_encode();
				debug_assert_eq!(data[..4], selector);
				// Replace selector with that provided at request
				data.splice(0..4, selector);
				data
			},
		}
	}
}

impl EncodeCallback for Vec<u8> {
	fn encode(&self, encoding: messaging::Encoding, selector: [u8; 4], id: MessageId) -> Vec<u8> {
		use messaging::Encoding::*;
		match encoding {
			Scale => [selector.to_vec(), (id, self).encode()].concat(),
			SolidityAbi => {
				// Use interface to encode call data. Clone required for ABI encoding of dynamic
				// bytes type.
				let call = IPostResponse::onPostResponseCall { id, response: self.clone().into() };
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

#[cfg(test)]
mod tests {
	use ::ismp::router::{GetRequest, PostRequest, Request};
	use frame_support::{
		assert_ok,
		traits::{Get, UnixTime},
		weights::Weight,
	};
	use mock::{ExtBuilder, *};
	use pallet_revive::{
		precompiles::{
			alloy::sol_types::{SolInterface, SolType},
			Error,
		},
		test_utils::{ALICE, ALICE_ADDR},
	};
	use sp_io::hashing::keccak_256;

	use super::{super::messaging::Origin, IISMPCalls::*, MessageStatus::*, *};

	type MaxContextLen = <Test as Config>::MaxContextLen;
	type MaxDataLen = <Test as Config>::MaxDataLen;
	type MaxKeyLen = <Test as Config>::MaxKeyLen;
	type MaxKeys = <Test as Config>::MaxKeys;
	type MaxRemovals = <Test as Config>::MaxRemovals;
	type Messages = crate::messaging::Messages<Test>;

	const ADDRESS: [u8; 20] = fixed_address(ISMP);
	const GET_MESSAGE_DEPOSIT: u128 = 131_290;
	const POST_MESSAGE_DEPOSIT: u128 = 134_415;

	#[test]
	fn get_reverts_when_max_context_exceeded() {
		let origin = ALICE;
		ExtBuilder::new().build().execute_with(|| {
			let request = IISMP::Get {
				destination: 0,
				height: u64::default(),
				timeout: u64::default(),
				context: vec![255u8; <MaxContextLen as Get<u32>>::get() as usize + 1].into(),
				keys: Vec::default(),
			};
			let input = get_0(get_0Call { request, fee: U256::ZERO });
			assert_revert!(call_precompile::<MessageId>(&origin, &input), MaxContextExceeded);
		});
	}

	#[test]
	fn get_reverts_when_max_keys_exceeded() {
		let origin = ALICE;
		ExtBuilder::new().build().execute_with(|| {
			let request = IISMP::Get {
				destination: 0,
				height: u64::default(),
				timeout: u64::default(),
				context: Vec::default().into(),
				keys: vec![vec![].into(); <MaxKeys as Get<u32>>::get() as usize + 1],
			};
			let input = get_0(get_0Call { request, fee: U256::ZERO });
			assert_revert!(call_precompile::<MessageId>(&origin, &input), MaxKeysExceeded);
		});
	}

	#[test]
	fn get_reverts_when_max_key_exceeded() {
		let origin = ALICE;
		ExtBuilder::new().build().execute_with(|| {
			let request = IISMP::Get {
				destination: 0,
				height: u64::default(),
				timeout: u64::default(),
				context: Vec::default().into(),
				keys: vec![vec![0u8; <MaxKeyLen as Get<u32>>::get() as usize + 1].into()],
			};
			let input = get_0(get_0Call { request, fee: U256::ZERO });
			assert_revert!(call_precompile::<MessageId>(&origin, &input), MaxKeyExceeded);
		});
	}

	#[test]
	fn get_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = 1;
		let request = IISMP::Get {
			destination: 1_000,
			height: u64::MAX,
			timeout: u64::MAX,
			context: vec![255u8; 64].into(),
			keys: vec![vec![255u8; 32].into()].into(),
		};
		let fee = U256::from(100);
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), 1 * UNIT)]) // message deposit
			.with_message_id(message)
			.build()
			.execute_with(|| {
				let commitment = get_hash(&request);

				assert_eq!(
					call_precompile::<MessageId>(&origin.account, &get_0(get_0Call { request, fee })).unwrap(),
					message
				);

				let event = GetDispatched_0 { origin: origin.address.0.into(), id: message, commitment: commitment.0.into() };
				assert_last_event(ADDRESS, event);
				assert!(matches!(
					Messages::get(message),
					Some(Message::Ismp { origin: o, commitment: c, callback, message_deposit })
					    if o == origin && c == commitment && callback.is_none() && message_deposit == GET_MESSAGE_DEPOSIT)
				);
			});
	}

	#[test]
	fn get_with_callback_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = 1;
		let request = IISMP::Get {
			destination: 1_000,
			height: u64::MAX,
			timeout: u64::MAX,
			context: vec![255u8; 64].into(),
			keys: vec![vec![255u8; 32].into()].into(),
		};
		let fee = U256::from(100);
		let callback = Callback {
			destination: [255u8; 20].into(),
			encoding: super::Encoding::Scale,
			selector: [255u8; 4].into(),
			weight: super::Weight { refTime: 100, proofSize: 10 },
		};
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), 1 * UNIT)]) // message deposit
			.with_message_id(message)
			.build()
			.execute_with(|| {
				let commitment = get_hash(&request);

				assert_eq!(
					call_precompile::<MessageId>(&origin.account, &get_1(get_1Call { request, fee, callback: callback.clone() })).unwrap(),
					message
				);

				let event = GetDispatched_1 { origin: origin.address.0.into(), id: message, commitment: commitment.0.into(), callback: callback.clone() };
				assert_last_event(ADDRESS, event);
				assert!(matches!(
					Messages::get(message),
					Some(Message::Ismp { origin: o, commitment: c, callback: cb, message_deposit })
					    if o == origin && c == commitment && cb == Some((&callback).into()) && message_deposit == GET_MESSAGE_DEPOSIT)
				);
			});
	}

	#[test]
	fn get_response_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = 1;
		let response = b"ismp response".to_vec();
		ExtBuilder::new()
			.with_messages(vec![(
				origin.account.clone(),
				message,
				Message::ismp_response(
					origin.address,
					H256::default(),
					0,
					response.clone().try_into().unwrap(),
				),
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
					response
				);
			});
	}

	#[test]
	fn poll_status_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let expected = [(0, NotFound), (1, Pending), (2, Complete), (3, Timeout)];
		let messages = [
			(1, Message::ismp(origin.clone(), H256::default(), None, 0)),
			(2, Message::ismp_response(origin.address, H256::default(), 0, BoundedVec::default())),
			(3, Message::ismp_timeout(origin.address, H256::default(), 0, None)),
		];
		ExtBuilder::new()
			.with_messages(messages.map(|(i, m)| (origin.account.clone(), i, m, 0)).to_vec())
			.build()
			.execute_with(|| {
				for (message, expected) in expected {
					assert_eq!(
						call_precompile::<MessageStatus>(
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
	fn post_reverts_when_max_data_exceeded() {
		let origin = ALICE;
		let request = IISMP::Post {
			destination: 0,
			timeout: u64::default(),
			data: vec![255u8; <MaxDataLen as Get<u32>>::get() as usize + 1].into(),
		};
		let fee = U256::from(100);
		ExtBuilder::new().build().execute_with(|| {
			let input = post_0(post_0Call { request, fee });
			assert_revert!(call_precompile::<MessageId>(&origin, &input), MaxDataExceeded);
		});
	}

	#[test]
	fn post_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = 1;
		let request =
			IISMP::Post { destination: 1_000, timeout: u64::MAX, data: vec![255u8; 1024].into() };
		let fee = U256::from(100);
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), 1 * UNIT)]) // message deposit
			.with_message_id(message)
			.build()
			.execute_with(|| {
				let commitment = post_hash(&request);

				assert_eq!(
					call_precompile::<MessageId>(&origin.account, &post_0(post_0Call { request, fee })).unwrap(),
					message
				);

				let event = PostDispatched_0 { origin: origin.address.0.into(), id: message, commitment: commitment.0.into() };
				assert_last_event(ADDRESS, event);
				assert!(matches!(
					Messages::get( message),
					Some(Message::Ismp { origin: o, commitment: c, callback, message_deposit })
					    if o == origin && c == commitment && callback.is_none() && message_deposit == POST_MESSAGE_DEPOSIT)
				);
			});
	}

	#[test]
	fn post_with_callback_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let message = 1;
		let request =
			IISMP::Post { destination: 1_000, timeout: u64::MAX, data: vec![255u8; 1024].into() };
		let fee = U256::from(100);
		let callback = Callback {
			destination: [255u8; 20].into(),
			encoding: super::Encoding::Scale,
			selector: [255u8; 4].into(),
			weight: super::Weight { refTime: 100, proofSize: 10 },
		};
		ExtBuilder::new()
			.with_balances(vec![(origin.account.clone(), 1 * UNIT)]) // message deposit
			.with_message_id(message)
			.build()
			.execute_with(|| {
				let commitment = post_hash(&request);

				assert_eq!(
					call_precompile::<MessageId>(&origin.account, &post_1(post_1Call { request, fee, callback: callback.clone() })).unwrap(),
					message
				);

				let event = PostDispatched_1 { origin: origin.address.0.into(), id: message, commitment: commitment.0.into(), callback: callback.clone() };
				assert_last_event(ADDRESS, event);
				assert!(matches!(
					Messages::get(message),
					Some(Message::Ismp { origin: o, commitment: c, callback: cb, message_deposit })
					    if o == origin && c == commitment && cb == Some((&callback).into()) && message_deposit == POST_MESSAGE_DEPOSIT)
				);
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
				Message::ismp_response(origin.address, H256::default(), 0, BoundedVec::default()),
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
		let message =
			Message::ismp_response(origin.address, H256::default(), 0, BoundedVec::default());
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

	fn call_precompile<Output: SolValue + From<<Output::SolType as SolType>::RustType>>(
		origin: &AccountId,
		input: &IISMPCalls,
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

	fn get_hash(request: &IISMP::Get) -> H256 {
		keccak_256(
			Request::Get(GetRequest {
				source: StateMachine::Polkadot(2_000),
				dest: StateMachine::Polkadot(request.destination),
				nonce: pallet_ismp::Nonce::<Test>::get(),
				from: ID.to_vec(),
				keys: request.keys.iter().map(|key| key.to_vec()).collect(),
				height: request.height,
				context: request.context.to_vec(),
				timeout_timestamp: Timestamp::now().as_secs() + request.timeout,
			})
			.encode()
			.as_ref(),
		)
		.into()
	}

	fn post_hash(request: &IISMP::Post) -> H256 {
		keccak_256(
			Request::Post(PostRequest {
				source: StateMachine::Polkadot(2_000),
				dest: StateMachine::Polkadot(request.destination),
				nonce: pallet_ismp::Nonce::<Test>::get(),
				from: ID.to_vec(),
				// TODO: check this is correct: https://github.com/r0gue-io/pop-node/blob/messaging-base/pallets/api/src/messaging/transports/ismp.rs#L105
				to: ID.to_vec(),
				timeout_timestamp: Timestamp::now().as_secs() + request.timeout,
				body: request.data.to_vec(),
			})
			.encode()
			.as_ref(),
		)
		.into()
	}
}

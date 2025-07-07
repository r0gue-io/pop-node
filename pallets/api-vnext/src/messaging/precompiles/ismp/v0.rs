use ::ismp::{
	dispatcher::{DispatchGet, DispatchPost},
	host::StateMachine,
};
use frame_support::traits::Get as _;
pub(crate) use IISMP::{Get, *};

use super::*;
use crate::messaging::transports::ismp::{get, post, ID};

sol!(
	#![sol(extra_derives(Debug, PartialEq))]
	"src/messaging/precompiles/interfaces/v0/IISMP.sol"
);

/// The ISMP precompile offers a streamlined interface for messaging using the Interoperable State
/// Machine Protocol.
pub struct Ismp<const FIXED: u16, T>(PhantomData<T>);
impl<const FIXED: u16, T: frame_system::Config + pallet_revive::Config + Config> Precompile
	for Ismp<FIXED, T>
where
	U256: UintTryTo<BalanceOf<T>>,
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
				let origin = env.caller();
				let origin = origin.account_id()?;
				let message = try_get::<T>(request)?;

				let (id, commitment) = get::<T>(origin, message, fee.saturating_to(), None)?;

				let origin = AddressMapper::<T>::to_address(origin).0.into();
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
				let origin = env.caller();
				let origin = origin.account_id()?;
				let message = try_get::<T>(request)?;

				let (id, commitment) = get::<T>(
					env.caller().account_id()?,
					message,
					fee.saturating_to(),
					Some(callback.into()),
				)?;

				let origin = AddressMapper::<T>::to_address(origin).0.into();
				let commitment = commitment.0.into();
				let event = GetDispatched_1 { origin, id, commitment, callback: callback.clone() };
				deposit_event(env, event)?;
				Ok(get_0Call::abi_encode_returns(&id))
			},
			IISMPCalls::getResponse(getResponseCall { message }) => {
				env.charge(<T as Config>::WeightInfo::get_response())?;

				let response = super::get::<T>((env.caller().account_id()?, message)).into();

				Ok(getResponseCall::abi_encode_returns(&response))
			},
			IISMPCalls::pollStatus(pollStatusCall { message }) => {
				env.charge(<T as Config>::WeightInfo::poll_status())?;

				let status = poll_status::<T>((env.caller().account_id()?, message)).into();

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
				let origin = env.caller();
				let origin = origin.account_id()?;
				let message = try_post::<T>(request)?;

				let (id, commitment) = post::<T>(origin, message, fee.saturating_to(), None)?;

				let origin = AddressMapper::<T>::to_address(origin).0.into();
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
				let origin = env.caller();
				let origin = origin.account_id()?;
				let message = try_post::<T>(request)?;

				let (id, commitment) = post::<T>(
					env.caller().account_id()?,
					message,
					fee.saturating_to(),
					Some(callback.into()),
				)?;

				let origin = AddressMapper::<T>::to_address(origin).0.into();
				let commitment = commitment.0.into();
				let event = PostDispatched_1 { origin, id, commitment, callback: callback.clone() };
				deposit_event(env, event)?;
				Ok(post_0Call::abi_encode_returns(&id))
			},
			IISMPCalls::remove_0(remove_0Call { message }) => {
				env.charge(<T as Config>::WeightInfo::remove(1))?;
				let origin = env.caller();
				let origin = origin.account_id()?;

				remove::<T>(origin, &[*message])?;

				// TODO: is the precompile emitting the event, or the pallet
				let account = AddressMapper::<T>::to_address(origin).0.into();
				deposit_event(env, Removed { account, messages: vec![*message] })?;
				Ok(remove_0Call::abi_encode_returns(&remove_0Return {}))
			},
			IISMPCalls::remove_1(remove_1Call { messages }) => {
				let messages_len = messages
					.len()
					.try_into()
					.map_err(|_| DispatchError::from(ArithmeticError::Overflow))?;
				env.charge(<T as Config>::WeightInfo::remove(messages_len))?;
				let origin = env.caller();
				let origin = origin.account_id()?;

				remove::<T>(origin, messages)?;

				// TODO: is the precompile emitting the event, or the pallet
				let account = AddressMapper::<T>::to_address(origin).0.into();
				deposit_event(env, Removed { account, messages: messages.clone() })?;
				Ok(remove_1Call::abi_encode_returns(&remove_1Return {}))
			},
		}
	}
}

fn try_get<T: Config>(value: &Get) -> Result<DispatchGet, DispatchError> {
	// TODO: additional error variants vs revive::DecodingFailed
	ensure!(value.context.len() as u32 <= T::MaxContextLen::get(), Error::<T>::TooManyMessages);
	ensure!(value.keys.len() as u32 <= T::MaxKeys::get(), Error::<T>::TooManyMessages);
	ensure!(
		value.keys.iter().all(|k| k.len() as u32 <= T::MaxKeyLen::get()),
		Error::<T>::TooManyMessages
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

fn try_post<T: Config>(value: &Post) -> Result<DispatchPost, DispatchError> {
	// TODO: additional error variants vs revive::DecodingFailed
	ensure!(value.data.len() as u32 <= T::MaxDataLen::get(), Error::<T>::TooManyMessages);
	Ok(DispatchPost {
		dest: StateMachine::Polkadot(value.destination),
		from: ID.into(),
		to: ID.into(),
		timeout: value.timeout,
		body: value.data.to_vec(),
	})
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
	use frame_support::{assert_ok, traits::UnixTime, weights::Weight};
	use mock::{ExtBuilder, *};
	use pallet_revive::{
		precompiles::alloy::sol_types::{SolInterface, SolType},
		test_utils::ALICE,
	};
	use sp_io::hashing::keccak_256;

	use super::{
		super::Message::{Ismp, *},
		IISMPCalls::*,
		MessageStatus::*,
		*,
	};

	type Messages = crate::messaging::Messages<Test>;

	const ADDRESS: [u8; 20] = fixed_address(ISMP);
	const MESSAGE_DEPOSIT: u128 = 129_540;

	#[test]
	fn get_works() {
		let origin = ALICE;
		let message = 1;
		let request = IISMP::Get {
			destination: 1_000,
			height: u32::MAX,
			timeout: u64::MAX,
			context: vec![255u8; 64].into(),
			keys: vec![vec![255u8; 32].into()].into(),
		};
		let fee = U256::from(100);
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), 1 * UNIT)]) // message deposit
			.with_message_id(&origin, message)
			.build()
			.execute_with(|| {
				let commitment = get_hash(&request);

				assert_eq!(
					call_precompile::<MessageId>(&origin, &get_0(get_0Call { request, fee })).unwrap(),
					message
				);

				let event = GetDispatched_0 { origin: to_address(&origin).0.into(), id: message, commitment: commitment.0.into() };
				assert_last_event(ADDRESS, event);
				assert!(matches!(
					Messages::get(&origin, message),
					Some(Message::Ismp { commitment: c, callback, message_deposit })
					    if c == commitment && callback.is_none() && message_deposit == MESSAGE_DEPOSIT)
				);
			});
	}

	#[test]
	fn get_with_callback_works() {
		let origin = ALICE;
		let message = 1;
		let request = IISMP::Get {
			destination: 1_000,
			height: u32::MAX,
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
			.with_balances(vec![(origin.clone(), 1 * UNIT)]) // message deposit
			.with_message_id(&origin, message)
			.build()
			.execute_with(|| {
				let commitment = get_hash(&request);

				assert_eq!(
					call_precompile::<MessageId>(&origin, &get_1(get_1Call { request, fee, callback: callback.clone() })).unwrap(),
					message
				);

				let event = GetDispatched_1 { origin: to_address(&origin).0.into(), id: message, commitment: commitment.0.into(), callback: callback.clone() };
				assert_last_event(ADDRESS, event);
				assert!(matches!(
					Messages::get(&origin, message),
					Some(Message::Ismp { commitment: c, callback: cb, message_deposit })
					    if c == commitment && cb == Some((&callback).into()) && message_deposit == MESSAGE_DEPOSIT)
				);
			});
	}

	#[test]
	fn get_response_works() {
		let origin = ALICE;
		let message = 1;
		let response = b"ismp response".to_vec();
		ExtBuilder::new()
			.with_messages(vec![(
				origin.clone(),
				message,
				IsmpResponse {
					commitment: H256::default(),
					message_deposit: 0,
					response: BoundedVec::truncate_from(response.clone()),
				},
			)])
			.build()
			.execute_with(|| {
				assert_eq!(
					call_precompile::<Vec<u8>>(&origin, &getResponse(getResponseCall { message }))
						.unwrap(),
					response
				);
			});
	}

	#[test]
	fn poll_status_works() {
		let origin = ALICE;
		let expected = [(0, NotFound), (1, Pending), (2, Complete), (3, Timeout)];
		let messages = [
			(1, Ismp { commitment: H256::default(), callback: None, message_deposit: 0 }),
			(
				2,
				IsmpResponse {
					commitment: H256::default(),
					message_deposit: 0,
					response: BoundedVec::default(),
				},
			),
			(
				3,
				IsmpTimeout {
					commitment: H256::default(),
					message_deposit: 0,
					callback_deposit: None,
				},
			),
		];
		ExtBuilder::new()
			.with_messages(messages.map(|(i, m)| (origin.clone(), i, m)).to_vec())
			.build()
			.execute_with(|| {
				for (message, expected) in expected {
					assert_eq!(
						call_precompile::<MessageStatus>(
							&origin,
							&pollStatus(pollStatusCall { message })
						)
						.unwrap(),
						expected.into()
					);
				}
			});
	}

	#[test]
	fn post_works() {
		let origin = ALICE;
		let message = 1;
		let request =
			IISMP::Post { destination: 1_000, timeout: u64::MAX, data: vec![255u8; 1024].into() };
		let fee = U256::from(100);
		ExtBuilder::new()
			.with_balances(vec![(origin.clone(), 1 * UNIT)]) // message deposit
			.with_message_id(&origin, message)
			.build()
			.execute_with(|| {
				let commitment = post_hash(&request);

				assert_eq!(
					call_precompile::<MessageId>(&origin, &post_0(post_0Call { request, fee })).unwrap(),
					message
				);

				let event = PostDispatched_0 { origin: to_address(&origin).0.into(), id: message, commitment: commitment.0.into() };
				assert_last_event(ADDRESS, event);
				assert!(matches!(
					Messages::get(&origin, message),
					Some(Message::Ismp { commitment: c, callback, message_deposit })
					    if c == commitment && callback.is_none() && message_deposit == MESSAGE_DEPOSIT)
				);
			});
	}

	#[test]
	fn post_with_callback_works() {
		let origin = ALICE;
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
			.with_balances(vec![(origin.clone(), 1 * UNIT)]) // message deposit
			.with_message_id(&origin, message)
			.build()
			.execute_with(|| {
				let commitment = post_hash(&request);

				assert_eq!(
					call_precompile::<MessageId>(&origin, &post_1(post_1Call { request, fee, callback: callback.clone() })).unwrap(),
					message
				);

				let event = PostDispatched_1 { origin: to_address(&origin).0.into(), id: message, commitment: commitment.0.into(), callback: callback.clone() };
				assert_last_event(ADDRESS, event);
				assert!(matches!(
					Messages::get(&origin, message),
					Some(Message::Ismp { commitment: c, callback: cb, message_deposit })
					    if c == commitment && cb == Some((&callback).into()) && message_deposit == MESSAGE_DEPOSIT)
				);
			});
	}

	#[test]
	fn remove_works() {
		let origin = ALICE;
		let message = 1;
		ExtBuilder::new()
			.with_messages(vec![(
				origin.clone(),
				message,
				IsmpResponse {
					commitment: H256::default(),
					message_deposit: 0,
					response: BoundedVec::default(),
				},
			)])
			.build()
			.execute_with(|| {
				assert_ok!(call_precompile::<()>(&origin, &remove_0(remove_0Call { message })));

				let account = to_address(&origin).0.into();
				assert_last_event(ADDRESS, Removed { account, messages: vec![message] });
			});
	}

	#[test]
	fn remove_many_works() {
		let origin = ALICE;
		let messages = 10;
		let message = IsmpResponse {
			commitment: H256::default(),
			message_deposit: 0,
			response: BoundedVec::default(),
		};
		ExtBuilder::new()
			.with_messages((0..messages).map(|i| (origin.clone(), i, message.clone())).collect())
			.build()
			.execute_with(|| {
				let messages: Vec<_> = (0..messages).collect();
				assert_ok!(call_precompile::<()>(
					&origin,
					&remove_1(remove_1Call { messages: messages.clone() })
				));

				let account = to_address(&origin).0.into();
				assert_last_event(ADDRESS, Removed { account, messages });
			});
	}

	fn call_precompile<Output: SolValue + From<<Output::SolType as SolType>::RustType>>(
		origin: &AccountId,
		input: &IISMPCalls,
	) -> Result<Output, DispatchError> {
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
				height: request.height as u64,
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

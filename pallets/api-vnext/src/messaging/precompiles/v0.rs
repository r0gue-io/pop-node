pub(crate) use IMessaging::*;
pub(crate) use IMessagingCalls;

use super::*;
use crate::messaging::{
	self,
	precompiles::v0::IMessaging::{
		getResponseCall, pollStatusCall, remove_0Call, remove_0Return, remove_1Call,
	},
	Config,
};

sol!(
	#![sol(extra_derives(Debug, PartialEq))]
	"src/messaging/precompiles/interfaces/v0/IMessaging.sol"
);

pub struct Messaging<const FIXED: u16, T>(PhantomData<T>);
impl<const FIXED: u16, T: frame_system::Config + pallet_revive::Config + Config> Precompile
	for Messaging<FIXED, T>
{
	type Interface = IMessagingCalls;
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
			IMessagingCalls::getResponse(getResponseCall { message }) => {
				env.charge(<T as Config>::WeightInfo::get_response())?;

				let response = get::<T>((env.caller().account_id()?, message)).into();

				Ok(getResponseCall::abi_encode_returns(&response))
			},
			IMessagingCalls::pollStatus(pollStatusCall { message }) => {
				env.charge(<T as Config>::WeightInfo::poll_status())?;

				let status = poll_status::<T>((env.caller().account_id()?, message)).into();

				Ok(pollStatusCall::abi_encode_returns(&status))
			},
			IMessagingCalls::remove_0(remove_0Call { message }) => {
				env.charge(<T as Config>::WeightInfo::remove(1))?;
				let origin = env.caller();
				let origin = origin.account_id()?;

				remove::<T>(origin, &[*message]).map_err(Self::map_err)?;

				// TODO: is the precompile emitting the event, or the pallet
				let account = AddressMapper::<T>::to_address(origin).0.into();
				deposit_event(env, Removed { account, messages: vec![*message] })?;
				Ok(remove_0Call::abi_encode_returns(&remove_0Return {}))
			},
			IMessagingCalls::remove_1(remove_1Call { messages }) => {
				let messages_len = messages
					.len()
					.try_into()
					.map_err(|_| DispatchError::from(ArithmeticError::Overflow))?;
				env.charge(<T as Config>::WeightInfo::remove(messages_len))?;
				let origin = env.caller();
				let origin = origin.account_id()?;

				remove::<T>(origin, messages).map_err(Self::map_err)?;

				// TODO: is the precompile emitting the event, or the pallet
				let account = AddressMapper::<T>::to_address(origin).0.into();
				deposit_event(env, Removed { account, messages: messages.clone() })?;
				Ok(remove_1Call::abi_encode_returns(&remove_1Return {}))
			},
		}
	}
}

impl<const FIXED: u16, T: frame_system::Config> Messaging<FIXED, T> {
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
						Ok(MessageNotFound) => IMessaging::MessageNotFound.into(),
						Ok(RequestPending) => IMessaging::RequestPending.into(),
						Ok(TooManyMessages) => IMessaging::TooManyMessages.into(),
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
	IMessaging::MessageNotFound,
	IMessaging::RequestPending,
	IMessaging::TooManyMessages,
}

impl From<messaging::MessageStatus> for IMessaging::MessageStatus {
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

#[cfg(test)]
mod tests {
	use ::xcm::latest::Response;
	use frame_support::{assert_ok, weights::Weight, BoundedVec};
	use mock::{ExtBuilder, *};
	use pallet_revive::{
		precompiles::{
			alloy::sol_types::{SolInterface, SolType},
			Error,
		},
		test_utils::ALICE,
	};

	use super::{
		super::{Message::*, MessageStatus::*},
		IMessagingCalls::*,
		*,
	};

	const ADDRESS: [u8; 20] = fixed_address(MESSAGING);

	type MaxRemovals = <Test as Config>::MaxRemovals;

	#[test]
	fn get_response_works() {
		let origin = ALICE;
		let expected =
			[(0, Vec::default()), (1, b"ismp response".to_vec()), (2, Response::Null.encode())];
		let messages = [
			(
				1,
				IsmpResponse {
					commitment: H256::default(),
					message_deposit: 0,
					response: BoundedVec::truncate_from(b"ismp response".to_vec()),
				},
			),
			(2, XcmResponse { query_id: 0, message_deposit: 0, response: Response::Null }),
		];
		ExtBuilder::new()
			.with_messages(messages.map(|(i, m)| (origin.clone(), i, m)).to_vec())
			.build()
			.execute_with(|| {
				for (message, expected) in expected {
					let input = getResponse(getResponseCall { message });
					assert_eq!(call_precompile::<Vec<u8>>(&origin, &input).unwrap(), expected);
				}
			});
	}

	#[test]
	fn poll_status_works() {
		let origin = ALICE;
		let expected = [(0, NotFound), (1, Pending), (2, Complete), (3, Timeout)];
		let messages = [
			(1, XcmQuery { query_id: 0, callback: None, message_deposit: 0 }),
			(2, XcmResponse { query_id: 0, message_deposit: 0, response: Response::Null }),
			(3, XcmTimeout { query_id: 0, message_deposit: 0, callback_deposit: None }),
		];
		ExtBuilder::new()
			.with_messages(messages.map(|(i, m)| (origin.clone(), i, m)).to_vec())
			.build()
			.execute_with(|| {
				for (message, expected) in expected {
					assert_eq!(
						call_precompile::<IMessaging::MessageStatus>(
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
	fn remove_reverts_when_message_pending() {
		let origin = ALICE;
		let message = 1;
		ExtBuilder::new()
			.with_messages(vec![(
				origin.clone(),
				message,
				XcmQuery { query_id: 0, callback: None, message_deposit: 0 },
			)])
			.build()
			.execute_with(|| {
				assert_revert!(
					call_precompile::<()>(&origin, &remove_0(remove_0Call { message })),
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
		let origin = ALICE;
		let message = 1;
		ExtBuilder::new()
			.with_messages(vec![(
				origin.clone(),
				message,
				XcmResponse { query_id: 0, message_deposit: 0, response: Response::Null },
			)])
			.build()
			.execute_with(|| {
				assert_ok!(call_precompile::<()>(&origin, &remove_0(remove_0Call { message })));

				let account = to_address(&origin).0.into();
				assert_last_event(ADDRESS, Removed { account, messages: vec![message] });
			});
	}

	#[test]
	fn remove_many_reverts_when_message_pending() {
		let origin = ALICE;
		let message = XcmResponse { query_id: 0, message_deposit: 0, response: Response::Null };
		let messages = <MaxRemovals as Get<u32>>::get() as u64;
		ExtBuilder::new()
			.with_messages(
				(0..messages - 1)
					.map(|i| (origin.clone(), i, message.clone()))
					.chain(vec![(
						origin.clone(),
						messages,
						XcmQuery { query_id: 0, callback: None, message_deposit: 0 },
					)])
					.collect(),
			)
			.build()
			.execute_with(|| {
				assert_revert!(
					call_precompile::<()>(
						&origin,
						&remove_1(remove_1Call { messages: (0..messages).collect() })
					),
					MessageNotFound
				);
			});
	}

	#[test]
	fn remove_many_reverts_when_message_not_found() {
		let origin = ALICE;
		let message = XcmResponse { query_id: 0, message_deposit: 0, response: Response::Null };
		let messages = <MaxRemovals as Get<u32>>::get() as u64;
		ExtBuilder::new()
			.with_messages(
				(0..messages - 1).map(|i| (origin.clone(), i, message.clone())).collect(),
			)
			.build()
			.execute_with(|| {
				assert_revert!(
					call_precompile::<()>(
						&origin,
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
		let origin = ALICE;
		let messages = 10;
		let message = XcmResponse { query_id: 0, message_deposit: 0, response: Response::Null };
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
		input: &IMessagingCalls,
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

pub(crate) use IMessaging::*;
pub(crate) use IMessagingCalls;

use super::*;
use crate::{
	messaging::{
		self,
		precompiles::v0::IMessaging::{
			getResponseCall, pollStatusCall, remove_0Call, remove_0Return, remove_1Call,
		},
		Config,
	},
	TryConvert,
};

sol!(
	#![sol(extra_derives(Debug, PartialEq))]
	"src/messaging/precompiles/interfaces/v0/IMessaging.sol"
);

pub struct Messaging<const FIXED: u16, T>(PhantomData<T>);
impl<
		const FIXED: u16,
		T: frame_system::Config + pallet_revive::Config + parachain_info::Config + Config,
	> Precompile for Messaging<FIXED, T>
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

				let response = get::<T>(message).into();

				Ok(getResponseCall::abi_encode_returns(&response))
			},
			IMessagingCalls::id(idCall {}) => {
				env.charge(<T as Config>::WeightInfo::id())?;

				let id = id::<T>();

				Ok(idCall::abi_encode_returns(&id))
			},
			IMessagingCalls::pollStatus(pollStatusCall { message }) => {
				env.charge(<T as Config>::WeightInfo::poll_status())?;

				let status = poll_status::<T>(message).into();

				Ok(pollStatusCall::abi_encode_returns(&status))
			},
			IMessagingCalls::remove_0(remove_0Call { message }) => {
				env.charge(<T as Config>::WeightInfo::remove(1))?;

				let account = (|| {
					let origin = Origin::try_from(env.caller())?;
					let address = origin.address();

					remove::<T>(origin, &[*message])?;

					Ok(address)
				})()
				.map_err(Self::map_err)?;

				deposit_event(env, Removed { account, messages: vec![*message] })?;
				Ok(remove_0Call::abi_encode_returns(&remove_0Return {}))
			},
			IMessagingCalls::remove_1(remove_1Call { messages }) => {
				let messages_len = messages.len().try_convert()?;
				env.charge(<T as Config>::WeightInfo::remove(messages_len))?;

				let account = (|| {
					let origin = Origin::try_from(env.caller())?;
					let address = origin.address();

					remove::<T>(origin, messages)?;

					Ok(address)
				})()
				.map_err(Self::map_err)?;

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
	use frame_support::{assert_ok, weights::Weight};
	use mock::{ExtBuilder, *};
	use pallet_revive::{
		precompiles::{
			alloy::sol_types::{SolInterface, SolType},
			Error,
		},
		test_utils::ALICE,
	};

	use super::{super::MessageStatus::*, IMessagingCalls::*, *};

	const ADDRESS: [u8; 20] = fixed_address(MESSAGING);

	type MaxRemovals = <Test as Config>::MaxRemovals;
	type Origin = super::Origin<Test>;

	#[test]
	fn get_response_works() {
		let origin = Origin::from((ALICE_ADDR, ALICE));
		let expected = [
			(0, Vec::default()),
			(1, b"ismp response".to_vec()),
			(2, Encode::encode(&Response::Null)),
		];
		let messages = [
			(
				1,
				Message::ismp_response(
					origin.address,
					H256::default(),
					0,
					b"ismp response".to_vec().try_into().unwrap(),
				),
			),
			(2, Message::xcm_response(origin.address, 0, 0, Response::Null)),
		];
		ExtBuilder::new()
			.with_messages(messages.map(|(i, m)| (origin.account.clone(), i, m, 0)).to_vec())
			.build()
			.execute_with(|| {
				for (message, expected) in expected {
					let input = getResponse(getResponseCall { message });
					assert_eq!(
						call_precompile::<Vec<u8>>(&origin.account, &input).unwrap(),
						expected
					);
				}
			});
	}

	#[test]
	fn id_works() {
		let origin = ALICE;
		ExtBuilder::new().build().execute_with(|| {
			let para_id = u32::from(ParachainInfo::parachain_id());
			assert_eq!(
				call_precompile::<u32>(&origin, &IMessagingCalls::id(idCall {})).unwrap(),
				para_id
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
						call_precompile::<IMessaging::MessageStatus>(
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

				let account = origin.address();
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

				let account = origin.address();
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

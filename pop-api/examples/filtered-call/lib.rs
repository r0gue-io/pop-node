#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// This is an example contract used for unit testing purposes.
/// It is intended to ensure that the call filter for the Pop API chain
/// extension works as expected.
/// An Ok(()) returned from `get_filtered` means that the filter successfully
/// BLOCKED the call. An Err() means that the filter did not block the call,
/// which is incorrect behavior.
/// The pop-api crate is not used because it does not support invalid calls ;)
use scale::Encode;

use ink::{env::Environment, prelude::vec::Vec};

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
	CallWasNotFiltered,
}

#[derive(Encode)]
pub enum SystemCalls {
	#[codec(index = 0)]
	Remark { remark: Vec<u8> },
}
#[derive(scale::Encode)]
pub enum RuntimeCall {
	#[codec(index = 0)]
	System(SystemCalls),
}

pub type Result<T> = core::result::Result<T, ContractError>;
#[ink::chain_extension(extension = 909)]
pub trait PopApi {
	type ErrorCode = ContractError;
	#[ink(function = 0)]
	fn dispatch(call: RuntimeCall) -> Result<()>;
}

impl From<ink::scale::Error> for ContractError {
	fn from(_: ink::scale::Error) -> Self {
		panic!("encountered unexpected invalid SCALE encoding")
	}
}

impl ink::env::chain_extension::FromStatusCode for ContractError {
	fn from_status_code(status_code: u32) -> core::result::Result<(), Self> {
		ink::env::debug_println!("get_filtered status_code: {:?}", status_code);
		match status_code {
			0 => Err(Self::CallWasNotFiltered),
			5 => Ok(()),
			_ => panic!("encountered unknown status code"),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[ink::scale_derive(TypeInfo)]
pub enum CustomEnvironment {}
impl Environment for CustomEnvironment {
	const MAX_EVENT_TOPICS: usize = <ink::env::DefaultEnvironment as Environment>::MAX_EVENT_TOPICS;

	type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
	type Balance = <ink::env::DefaultEnvironment as Environment>::Balance;
	type Hash = <ink::env::DefaultEnvironment as Environment>::Hash;
	type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;
	type BlockNumber = <ink::env::DefaultEnvironment as Environment>::BlockNumber;

	type ChainExtension = crate::PopApi;
}

#[ink::contract(env = crate::CustomEnvironment)]
mod pop_api_filtered_call {
	use super::{ContractError, RuntimeCall, SystemCalls, Vec};
	#[ink(storage)]
	#[derive(Default)]
	pub struct PopApiFilteredCall;

	impl PopApiFilteredCall {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("Contract::new");
			Default::default()
		}

		// Calls Pop API chain extension with non-allowed call.
		// If Ok(()) is returned the filter correctly blocked the call.
		#[ink(message)]
		pub fn get_filtered(&mut self) -> Result<(), ContractError> {
			ink::env::debug_println!("Contract::get_filtered");

			let blocked_call =
				RuntimeCall::System(SystemCalls::Remark { remark: Vec::from([0u8; 8]) });
			let res = self.env().extension().dispatch(blocked_call);

			ink::env::debug_println!("Contract::get_filtered res {:?}", res);

			res
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			PopApiFilteredCall::new();
		}
	}
}

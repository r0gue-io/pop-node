#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
	env::chain_extension::{ChainExtensionMethod, FromStatusCode},
	prelude::vec::Vec,
};

#[ink::contract]
mod proxy_contract {
	use super::*;

	// Simple contract for proxying a call to a chain extension.
	#[ink(storage)]
	#[derive(Default)]
	pub struct Proxy;

	impl Proxy {
		#[ink(constructor)]
		pub fn new() -> Self {
			ink::env::debug_println!("Proxy::new()");
			Default::default()
		}

		#[ink(message)]
		pub fn call(&self, func_id: u32, input: Vec<u8>) -> Result<Vec<u8>, StatusCode> {
			ink::env::debug_println!("Proxy::call() func_id={func_id}, input={input:?}");
			ChainExtensionMethod::build(func_id)
				.input::<Vec<u8>>()
				.output::<Result<Vec<u8>, StatusCode>, true>()
				.handle_error_code::<StatusCode>()
				.call(&input)
		}
	}

	#[ink::scale_derive(Encode, Decode, TypeInfo)]
	pub struct StatusCode(u32);
	impl FromStatusCode for StatusCode {
		fn from_status_code(status_code: u32) -> Result<(), Self> {
			match status_code {
				0 => Ok(()),
				_ => Err(StatusCode(status_code)),
			}
		}
	}

	impl From<ink::scale::Error> for StatusCode {
		fn from(_: ink::scale::Error) -> Self {
			StatusCode(u32::MAX)
		}
	}
}

/// We put `drink`-based tests as usual unit tests, into a test module.
#[cfg(test)]
mod tests {
	use codec::Encode;
	use core::fmt::Debug;
	use drink::{
		session::{Session, NO_ARGS, NO_SALT},
		AccountId32,
	};
	use frame_system::Call;
	use pop_sandbox::{
		utils::{call_function, ALICE},
		PopSandbox, RuntimeCall,
	};

	#[drink::contract_bundle_provider]
	enum BundleProvider {}

	#[drink::test(sandbox = PopSandbox)]
	fn deploy_contract_and_call(mut session: Session) -> Result<(), Box<dyn std::error::Error>> {
		let _ = env_logger::try_init();

		let contract_bundle = BundleProvider::local()?;
		let contract_address =
			session.deploy_bundle(contract_bundle, "new", NO_ARGS, NO_SALT, None)?;

		let call =
			RuntimeCall::System(Call::remark_with_event { remark: "pop".as_bytes().to_vec() });
		call_function(
			session,
			&contract_address,
			&ALICE,
			"call".to_string(),
			// DispatchCall::System::Call::remark_with_event.
			Some(vec![0.to_string(), serde_json::to_string(&call.encode()).unwrap()]),
			None,
		)?;
		// TOOD: Return CallFiltered

		Ok(())
	}
}
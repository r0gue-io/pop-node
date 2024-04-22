#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::balances;

#[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ContractError {
	BalancesError(balances::Error),
}

impl From<balances::Error> for ContractError {
	fn from(value: balances::Error) -> Self {
		ContractError::BalancesError(value)
	}
}

#[ink::contract(env = pop_api::Environment)]
mod pop_api_balances {
	use super::ContractError;

	#[ink(storage)]
	#[derive(Default)]
	pub struct PopApiBalances;

	impl PopApiBalances {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			ink::env::debug_println!("PopApiBalances::new");
			Default::default()
		}

		#[ink(message)]
		pub fn transfer_through_runtime(
			&mut self,
			receiver: AccountId,
			value: Balance,
		) -> Result<(), ContractError> {
			ink::env::debug_println!(
				"PopApiBalances::transfer_through_runtime: \nreceiver: {:?}, \nvalue: {:?}",
				receiver,
				value
			);

			pop_api::balances::transfer_keep_alive(receiver, value)?;

			ink::env::debug_println!("PopApiBalances::transfer_through_runtime end");
			Ok(())
		}
	}

	#[cfg(all(test, feature = "e2e-tests"))]
	mod e2e_tests {
		use super::*;
		use ink_e2e::{ChainBackend, ContractsBackend};

		use ink::{
			env::{test::default_accounts, DefaultEnvironment},
			primitives::AccountId,
		};

		type E2EResult<T> = Result<T, Box<dyn std::error::Error>>;

		/// The base number of indivisible units for balances on the
		/// `substrate-contracts-node`.
		const UNIT: Balance = 1_000_000_000_000;

		/// The contract will be given 1000 tokens during instantiation.
		const CONTRACT_BALANCE: Balance = 1_000 * UNIT;

		/// The receiver will get enough funds to have the required existential deposit.
		///
		/// If your chain has this threshold higher, increase the transfer value.
		const TRANSFER_VALUE: Balance = 1 / 10 * UNIT;

		/// An amount that is below the existential deposit, so that a transfer to an
		/// empty account fails.
		///
		/// Must not be zero, because such an operation would be a successful no-op.
		const INSUFFICIENT_TRANSFER_VALUE: Balance = 1;

		/// Positive case scenario:
		///  - the call is valid
		///  - the call execution succeeds
		#[ink_e2e::test]
		async fn transfer_with_call_runtime_works<Client: E2EBackend>(
			mut client: Client,
		) -> E2EResult<()> {
			// given
			let mut constructor = RuntimeCallerRef::new();
			let contract = client
				.instantiate("call-runtime", &ink_e2e::alice(), &mut constructor)
				.value(CONTRACT_BALANCE)
				.submit()
				.await
				.expect("instantiate failed");
			let mut call_builder = contract.call_builder::<RuntimeCaller>();

			let accounts = default_accounts::<DefaultEnvironment>();

			let receiver: AccountId = accounts.bob;

			let sender_balance_before = client
				.free_balance(accounts.alice)
				.await
				.expect("Failed to get account balance");
			let receiver_balance_before =
				client.free_balance(receiver).await.expect("Failed to get account balance");

			// when
			let transfer_message = call_builder.transfer_through_runtime(receiver, TRANSFER_VALUE);

			let call_res = client
				.call(&ink_e2e::alice(), &transfer_message)
				.submit()
				.await
				.expect("call failed");

			assert!(call_res.return_value().is_ok());

			// then
			let sender_balance_after = client
				.free_balance(accounts.alice)
				.await
				.expect("Failed to get account balance");
			let receiver_balance_after =
				client.free_balance(receiver).await.expect("Failed to get account balance");

			assert_eq!(contract_balance_before, contract_balance_after + TRANSFER_VALUE);
			assert_eq!(receiver_balance_before, receiver_balance_after - TRANSFER_VALUE);

			Ok(())
		}
	}
}

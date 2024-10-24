#![cfg_attr(not(feature = "std"), no_std, no_main)]

use pop_api::StatusCode;

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::contract]
mod environment {
	use super::*;

	#[ink(storage)]
	#[derive(Default)]
	pub struct Contract;

	impl Contract {
		#[ink(constructor, payable)]
		pub fn new() -> Self {
			Default::default()
		}

		#[ink(message)]
		pub fn caller(&self) -> AccountId {
			self.env().caller()
		}

		#[ink(message, payable)]
		pub fn transferred_value(&self) -> Balance {
			self.env().transferred_value()
		}

		#[ink(message)]
		pub fn weight_to_fee(&self, gas: u64) -> Balance {
			self.env().weight_to_fee(gas)
		}

		#[ink(message)]
		pub fn block_timestamp(&self) -> Timestamp {
			self.env().block_timestamp()
		}

		#[ink(message)]
		pub fn account_id(&self) -> AccountId {
			self.env().account_id()
		}

		#[ink(message)]
		pub fn balance(&self) -> Balance {
			self.env().balance()
		}

		#[ink(message)]
		pub fn block_number(&self) -> BlockNumber {
			self.env().block_number()
		}

		#[ink(message)]
		pub fn minimum_balance(&self) -> Balance {
			self.env().minimum_balance()
		}

		#[ink(message)]
		pub fn invoke_contract(&self) -> Result<()> {
			todo!()
		}

		#[ink(message)]
		pub fn invoke_contract_delegate(&self) -> Result<()> {
			todo!()
		}

		#[ink(message)]
		pub fn instantiate_contract(&self) -> Result<()> {
			todo!()
		}

		#[ink(message)]
		pub fn terminate_contract(&self, beneficiary: AccountId) {
			self.env().terminate_contract(beneficiary)
		}

		#[ink(message, payable)]
		pub fn transfer(&self, destination: AccountId) -> Result<()> {
			let transferred_value = self.env().transferred_value();
			self.env().transfer(destination, transferred_value).unwrap();
			self.env().emit_event(Transferred { transferred_value, destination });
			Ok(())
		}

		#[ink(message)]
		pub fn is_contract(&self, account: AccountId) -> bool {
			self.env().is_contract(&account)
		}

		#[ink(message)]
		pub fn caller_is_origin(&self) -> bool {
			self.env().caller_is_origin()
		}

		#[ink(message)]
		pub fn code_hash(&self, account: AccountId) -> Option<Hash> {
			self.env().code_hash(&account).ok()
		}

		#[ink(message)]
		pub fn own_code_hash(&self) -> Option<Hash> {
			self.env().own_code_hash().ok()
		}

		#[ink(message)]
		pub fn call_runtime(&self) -> Result<()> {
			todo!()
		}

		#[ink(message)]
		pub fn lock_delegate_dependency(&self) -> Result<()> {
			todo!()
		}

		#[ink(message)]
		pub fn unlock_delegate_dependency(&self) -> Result<()> {
			todo!()
		}

		#[ink(message)]
		pub fn xcm_execute(&self) -> Result<()> {
			todo!()
		}

		#[ink(message)]
		pub fn xcm_send(&self) -> Result<()> {
			todo!()
		}
	}

	#[ink::event]
	pub struct Transferred {
		pub transferred_value: Balance,
		pub destination: AccountId,
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[ink::test]
		fn default_works() {
			Contract::new();
		}
	}
}

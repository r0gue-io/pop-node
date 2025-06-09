#![cfg_attr(not(feature = "std"), no_std, no_main)]

// NOTE: requires `cargo-contract` built from `master`

#[ink::contract]
pub mod fungibles {
	use ink::U256;
	use pop_api::fungibles::{self as api, erc20, erc20::Transfer, TokenId as Id};

	#[ink(storage)]
	pub struct Fungible {
		id: Id,
		owner: Address,
	}

	impl Fungible {
		/// Instantiate the contract and create a new token. The token identifier will be stored
		/// in contract's storage.
		///
		/// # Parameters
		/// * - `min_balance` - The minimum balance required for accounts holding this token.
		// The `min_balance` ensures accounts hold a minimum amount of tokens, preventing tiny,
		// inactive balances from bloating the blockchain state and slowing down the network.
		#[ink(constructor, payable)]
		#[allow(clippy::new_without_default)]
		pub fn new(min_balance: U256) -> Self {
			let mut instance = Self { id: 0, owner: Self::env().caller() };
			instance.id = api::create(instance.env().address(), min_balance);
			instance
		}

		/// Creates `value` amount of tokens and assigns them to `account`, increasing the total
		/// supply.
		///
		/// # Parameters
		/// - `account` - The address to be credited with the created tokens.
		/// - `value` - The number of tokens to mint.
		#[ink(message)]
		pub fn mint(&mut self, account: Address, value: U256) {
			if let Err(e) = self.ensure_owner() {
				// Workaround until ink supports Error to Solidity custom error conversion; https://github.com/use-ink/ink/issues/2404
				revert(e)
			}
			// No-op if `value` is zero.
			if value == U256::zero() {
				return;
			}
			api::mint(self.id, account, value);
			self.env().emit_event(Transfer { from: Address::zero(), to: account, value });
		}

		/// Transfer the ownership of the contract to another account.
		///
		/// # Parameters
		/// - `owner` - New owner account.
		#[ink(message)]
		pub fn transfer_ownership(&mut self, owner: Address) {
			if let Err(e) = self.ensure_owner() {
				// Workaround until ink supports Error to Solidity custom error conversion; https://github.com/use-ink/ink/issues/2404
				revert(e)
			}
			self.owner = owner;
		}

		/// Check if the caller is the owner of the contract.
		fn ensure_owner(&self) -> Result<(), Error> {
			if self.owner != self.env().caller() {
				return Err(Error::NoPermission);
			}
			Ok(())
		}
	}

	// TODO: implement via Erc20 trait once Solidity support available
	impl Fungible {
		/// Returns the total token supply.
		#[ink(message)]
		#[allow(non_snake_case)] // Required to ensure message name results in correct sol selector
		pub fn totalSupply(&self) -> U256 {
			erc20::total_supply(self.id)
		}

		/// Returns the account balance for the specified `owner`.
		///
		/// # Parameters
		/// - `owner` - The address whose balance is being queried.
		#[ink(message)]
		#[allow(non_snake_case)]
		pub fn balanceOf(&self, owner: Address) -> U256 {
			erc20::balance_of(self.id, owner)
		}

		/// Returns the allowance for a `spender` approved by an `owner`.
		///
		/// # Parameters
		/// - `owner` - The account that owns the tokens.
		/// - `spender` - The account that is allowed to spend the tokens.
		#[ink(message)]
		pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
			erc20::allowance(self.id, owner, spender)
		}

		/// Transfers `value` amount of tokens from the contract to account `to` with
		/// additional `data` in unspecified format.
		///
		/// # Parameters
		/// - `to` - The recipient account.
		/// - `value` - The number of tokens to transfer.
		#[ink(message)]
		pub fn transfer(&mut self, to: Address, value: U256) -> bool {
			if let Err(e) = self.ensure_owner() {
				// Workaround until ink supports Error to Solidity custom error conversion
				revert(e)
			}
			let contract = self.env().address();

			// No-op if the contract and `to` is the same address or `value` is zero.
			if contract == to || value == U256::zero() {
				return true;
			}
			if !erc20::transfer(self.id, to, value) {
				revert(Error::TransferFailed)
			}
			self.env().emit_event(Transfer { from: contract, to, value });
			true
		}
	}

	pub enum Error {
		/// The signing account has no permission to do the operation.
		NoPermission,
		/// The transfer failed.
		TransferFailed,
	}

	fn revert(error: Error) -> ! {
		use ink::env::{return_value_solidity as return_value, ReturnFlags};
		use Error::*;

		const REVERT: ReturnFlags = ReturnFlags::REVERT;

		match error {
			NoPermission => return_value(REVERT, &(0x9d7b369d_u32.to_be_bytes())),
			TransferFailed => return_value(REVERT, &(0x90b8ec18_u32.to_be_bytes())),
		}
	}
}

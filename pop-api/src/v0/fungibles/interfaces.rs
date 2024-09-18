use super::*;

pub type Result<T> = core::result::Result<T, StatusCode>;

#[ink::trait_definition]
pub trait Psp22 {
	/// Returns the total token supply.
	#[ink(message)]
	fn total_supply(&self) -> Balance;

	/// Returns the account balance for the specified `owner`.
	///
	/// # Parameters
	/// - `owner` - The account whose balance is being queried.
	///
	/// Returns `0` if the account is non-existent.
	#[ink(message)]
	fn balance_of(&self, owner: AccountId) -> Balance;

	/// Returns the allowance for a `spender` approved by an `owner`.
	///
	/// # Parameters
	/// - `owner` - The account that owns the tokens.
	/// - `spender` - The account that is allowed to spend the tokens.
	///
	/// Returns `0` if no allowance has been set.
	#[ink(message)]
	fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

	/// Transfers `value` amount of tokens from the caller's account to account `to`
	/// with additional `data` in unspecified format.
	///
	/// # Parameters
	/// - `to` - The recipient account.
	/// - `value` - The number of tokens to transfer.
	/// - `data` - Additional data in unspecified format.
	///
	/// # Events
	/// On success a `Transfer` event is emitted.
	///
	/// No-op if the caller and `to` is the same address or `value` is zero, returns success
	/// and no events are emitted.
	///
	/// # Errors
	/// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
	#[ink(message)]
	fn transfer(&mut self, to: AccountId, value: Balance, data: Vec<u8>) -> Result<()>;

	/// Transfers `value` tokens on behalf of `from` to the account `to`
	/// with additional `data` in unspecified format.
	///
	/// # Parameters
	/// - `from` - The account from which the token balance will be withdrawn.
	/// - `to` - The recipient account.
	/// - `value` - The number of tokens to transfer.
	///
	/// If `from` and the caller are different addresses, the caller must be allowed
	/// by `from` to spend at least `value` tokens.
	///
	/// # Events
	/// On success a `Transfer` event is emitted.
	///
	/// No-op if `from` and `to` is the same address or `value` is zero, returns success
	/// and no events are emitted.
	///
	/// If `from` and the caller are different addresses, a successful transfer results
	/// in decreased allowance by `from` to the caller and an `Approval` event with
	/// the new allowance amount is emitted.
	///
	/// # Errors
	/// Reverts with `InsufficientBalance` if the `value` exceeds the balance of the account `from`.
	///
	/// Reverts with `InsufficientAllowance` if `from` and the caller are different addresses and
	/// the `value` exceeds the allowance granted by `from` to the caller.
	///
	/// If conditions for both `InsufficientBalance` and `InsufficientAllowance` errors are met,
	/// reverts with `InsufficientAllowance`.
	#[ink(message)]
	fn transfer_from(
		&mut self,
		from: AccountId,
		to: AccountId,
		value: Balance,
		data: Vec<u8>,
	) -> Result<()>;

	/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
	///
	/// Successive calls of this method overwrite previous values.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to approve.
	///
	/// # Events
	/// An `Approval` event is emitted.
	///
	/// No-op if the caller and `spender` is the same address, returns success and no events are emitted.
	#[ink(message)]
	fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()>;

	/// Increases the allowance of `spender` by `value` amount of tokens.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to increase the allowance by.
	///
	/// # Events
	/// An `Approval` event with the new allowance amount is emitted.
	///
	/// No-op if the caller and `spender` is the same address or `value` is zero, returns success
	/// and no events are emitted.
	#[ink(message)]
	fn increase_allowance(&mut self, spender: AccountId, value: Balance) -> Result<()>;

	/// Decreases the allowance of `spender` by `value` amount of tokens.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to decrease the allowance by.
	///
	/// # Events
	/// An `Approval` event with the new allowance amount is emitted.
	///
	/// No-op if the caller and `spender` is the same address or `value` is zero, returns success
	/// and no events are emitted.
	///
	/// # Errors
	/// Reverts with `InsufficientAllowance` if `spender` and the caller are different addresses and
	/// the `value` exceeds the allowance granted by the caller to `spender`.
	#[ink(message)]
	fn decrease_allowance(&mut self, spender: AccountId, value: Balance) -> Result<()>;
}

/// The PSP22 Metadata trait.
#[ink::trait_definition]
pub trait Psp22Managable {
	#[ink(message)]
	fn create(&mut self, id: TokenId, admin: AccountId, min_balance: Balance) -> Result<()>;

	#[ink(message)]
	fn start_destroy(&mut self, token: TokenId) -> Result<()>;

	#[ink(message)]
	fn set_metadata(
		&mut self,
		token: TokenId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> Result<()>;

	#[ink(message)]
	fn clear_metadata(&mut self, token: TokenId) -> Result<()>;

	#[ink(message)]
	fn token_exists(&self, token: TokenId) -> Result<bool>;
}

/// The PSP22 Metadata trait.
#[ink::trait_definition]
pub trait Psp22Metadata {
	/// Returns the token name.
	#[ink(message)]
	fn token_name(&self) -> Option<Vec<u8>>;

	/// Returns the token symbol.
	#[ink(message)]
	fn token_symbol(&self) -> Option<Vec<u8>>;

	/// Returns the token decimals.
	#[ink(message)]
	fn token_decimals(&self) -> u8;
}

/// The PSP22 Mintable trait.
#[ink::trait_definition]
pub trait Psp22Mintable {
	/// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
	///
	/// # Parameters
	/// - `account` - The account to be credited with the created tokens.
	/// - `value` - The number of tokens to mint.
	///
	/// # Events
	/// On success a `Transfer` event is emitted with `None` sender.
	///
	/// No-op if `value` is zero, returns success and no events are emitted.
	///
	/// # Errors
	/// Reverts with `Custom (max supply exceeded)` if the total supply increased by
	/// `value` exceeds maximal value of `u128` type.
	#[ink(message)]
	fn mint(&mut self, account: AccountId, amount: Balance) -> Result<()>;
}

/// The PSP22 Burnable trait.
#[ink::trait_definition]
pub trait Psp22Burnable {
	/// Destroys `value` amount of tokens from `account`, reducing the total supply.
	///
	/// # Parameters
	/// - `account` - The account from which the tokens will be destroyed.
	/// - `value` - The number of tokens to destroy.
	///
	/// # Events
	/// On success a `Transfer` event is emitted with `None` recipient.
	///
	/// No-op if `value` is zero, returns success and no events are emitted.
	///
	/// # Errors
	/// Reverts with `InsufficientBalance` if the `value` exceeds the caller's balance.
	#[ink(message)]
	fn burn(&mut self, account: AccountId, amount: Balance) -> Result<()>;
}

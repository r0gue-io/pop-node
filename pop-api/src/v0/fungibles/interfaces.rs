use super::*;

#[ink::trait_definition]
pub trait Psp22 {
	/// Returns the total token supply.
	#[ink(message)]
	fn total_supply(&self) -> Balance;

	/// Returns the account balance for the specified `owner`.
	///
	/// # Parameters
	/// - `owner` - The account whose balance is being queried.
	#[ink(message)]
	fn balance_of(&self, owner: AccountId) -> Balance;

	/// Returns the allowance for a `spender` approved by an `owner`.
	///
	/// # Parameters
	/// - `owner` - The account that owns the tokens.
	/// - `spender` - The account that is allowed to spend the tokens.
	#[ink(message)]
	fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

	/// Transfers `value` amount of tokens from the caller's account to account `to`
	/// with additional `data` in unspecified format.
	///
	/// # Parameters
	/// - `to` - The recipient account.
	/// - `value` - The number of tokens to transfer.
	/// - `data` - Additional data in unspecified format.
	#[ink(message)]
	fn transfer(&mut self, to: AccountId, value: Balance, data: Vec<u8>) -> PSP22Result<()>;

	/// Transfers `value` tokens on behalf of `from` to the account `to`
	/// with additional `data` in unspecified format.
	///
	/// # Parameters
	/// - `from` - The account from which the token balance will be withdrawn.
	/// - `to` - The recipient account.
	/// - `value` - The number of tokens to transfer.
	#[ink(message)]
	fn transfer_from(
		&mut self,
		from: AccountId,
		to: AccountId,
		value: Balance,
		data: Vec<u8>,
	) -> PSP22Result<()>;

	/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
	///
	/// Successive calls of this method overwrite previous values.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to approve.
	#[ink(message)]
	fn approve(&mut self, spender: AccountId, value: Balance) -> PSP22Result<()>;

	/// Increases the allowance of `spender` by `value` amount of tokens.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to increase the allowance by.
	#[ink(message)]
	fn increase_allowance(&mut self, spender: AccountId, value: Balance) -> PSP22Result<()>;

	/// Decreases the allowance of `spender` by `value` amount of tokens.
	///
	/// # Parameters
	/// - `spender` - The account that is allowed to spend the tokens.
	/// - `value` - The number of tokens to decrease the allowance by.
	#[ink(message)]
	fn decrease_allowance(&mut self, spender: AccountId, value: Balance) -> PSP22Result<()>;
}

/// The PSP22 Metadata trait.
#[ink::trait_definition]
pub trait Management {
	/// Creates a new token with the specified `id`, assigns `admin` as the administrator,
	/// and sets a minimum balance requirement.
	///
	/// # Parameters
	/// - `id`: The unique identifier for the token to be created.
	/// - `admin`: The account that will have administrative privileges over the token.
	/// - `min_balance`: The minimum balance required to hold the token.
	#[ink(message)]
	fn create(&mut self, id: TokenId, admin: AccountId, min_balance: Balance) -> PSP22Result<()>;

	/// Initiates the process to destroy the specified `token`.
	///
	/// # Parameters
	/// - `token`: The identifier of the token to be destroyed.
	#[ink(message)]
	fn start_destroy(&mut self, token: TokenId) -> PSP22Result<()>;

	/// Sets metadata for the specified `token`, including its name, symbol, and decimal places.
	///
	/// # Parameters
	/// - `token`: The identifier of the token to set metadata for.
	/// - `name`: The name of the token.
	/// - `symbol`: The symbol representing the token.
	/// - `decimals`: The number of decimal places the token uses.
	#[ink(message)]
	fn set_metadata(
		&mut self,
		token: TokenId,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> PSP22Result<()>;

	/// Clears the metadata for the specified `token`.
	///
	/// # Parameters
	/// - `token`: The identifier of the token to clear metadata for.
	#[ink(message)]
	fn clear_metadata(&mut self, token: TokenId) -> PSP22Result<()>;

	/// Checks whether a token with the specified `token` identifier exists.
	///
	/// # Parameters
	/// - `token`: The identifier of the token to check for existence.
	#[ink(message)]
	fn token_exists(&self, token: TokenId) -> PSP22Result<bool>;
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
	#[ink(message)]
	fn mint(&mut self, account: AccountId, amount: Balance) -> PSP22Result<()>;
}

/// The PSP22 Burnable trait.
#[ink::trait_definition]
pub trait Psp22Burnable {
	/// Destroys `value` amount of tokens from `account`, reducing the total supply.
	///
	/// # Parameters
	/// - `account` - The account from which the tokens will be destroyed.
	/// - `value` - The number of tokens to destroy.
	#[ink(message)]
	fn burn(&mut self, account: AccountId, amount: Balance) -> PSP22Result<()>;
}

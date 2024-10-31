# Fungibles API

The `fungibles` module provides an API for interacting and managing fungible tokens.

The API includes the following interfaces:
 1. PSP-22
 2. PSP-22 Metadata
 3. Management
 4. PSP-22 Mintable & Burnable
---

## Interface

#### PSP-22, PSP-22 Mintable & PSP-22 Burnable
The interface for transferring, delegating, minting and burning tokens. 
```rust
/// Returns the total token supply for a specified token.
///
/// # Parameters
/// - `token` - The token.
pub fn total_supply(token: TokenId) -> Result<Balance> {}

/// Returns the account balance for a specified `token` and `owner`. Returns `0` if
/// the account is non-existent.
///
/// # Parameters
/// - `token` - The token.
/// - `owner` - The account whose balance is being queried.
pub fn balance_of(token: TokenId, owner: AccountId) -> Result<Balance> {}

/// Returns the allowance for a `spender` approved by an `owner`, for a specified `token`. Returns
/// `0` if no allowance has been set.
///
/// # Parameters
/// - `token` - The token.
/// - `owner` - The account that owns the tokens.
/// - `spender` - The account that is allowed to spend the tokens.
pub fn allowance(token: TokenId, owner: AccountId, spender: AccountId) -> Result<Balance> {}

/// Transfers `value` amount of tokens from the caller's account to account `to`.
///
/// # Parameters
/// - `token` - The token to transfer.
/// - `to` - The recipient account.
/// - `value` - The number of tokens to transfer.
pub fn transfer(token: TokenId, to: AccountId, value: Balance) -> Result<()> {}

/// Transfers `value` amount tokens on behalf of `from` to account `to`.
///
/// # Parameters
/// - `token` - The token to transfer.
/// - `from` - The account from which the token balance will be withdrawn.
/// - `to` - The recipient account.
/// - `value` - The number of tokens to transfer.
pub fn transfer_from(token: TokenId, from: AccountId, to: AccountId, value: Balance) -> Result<()> {}

/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
///
/// # Parameters
/// - `token` - The token to approve.
/// - `spender` - The account that is allowed to spend the tokens.
/// - `value` - The number of tokens to approve.
pub fn approve(token: TokenId, spender: AccountId, value: Balance) -> Result<()> {}

/// Increases the allowance of `spender` by `value` amount of tokens.
///
/// # Parameters
/// - `token` - The token to have an allowance increased.
/// - `spender` - The account that is allowed to spend the tokens.
/// - `value` - The number of tokens to increase the allowance by.
pub fn increase_allowance(token: TokenId, spender: AccountId, value: Balance) -> Result<()> {}

/// Decreases the allowance of `spender` by `value` amount of tokens.
///
/// # Parameters
/// - `token` - The token to have an allowance decreased.
/// - `spender` - The account that is allowed to spend the tokens.
/// - `value` - The number of tokens to decrease the allowance by.
pub fn decrease_allowance(token: TokenId, spender: AccountId, value: Balance) -> Result<()> {}

/// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
///
/// # Parameters
/// - `token` - The token to mint.
/// - `account` - The account to be credited with the created tokens.
/// - `value` - The number of tokens to mint.
pub fn mint(token: TokenId, account: AccountId, value: Balance) -> Result<()> {}

/// Destroys `value` amount of tokens from `account`, reducing the total supply.
///
/// # Parameters
/// - `token` - the token to burn.
/// - `account` - The account from which the tokens will be destroyed.
/// - `value` - The number of tokens to destroy.
pub fn burn(token: TokenId, account: AccountId, value: Balance) -> Result<()> {}
```

#### PSP-22 Metadata
The PSP-22 compliant interface for querying metadata.
```rust
/// Returns the name of the specified token, if available.
///
/// # Parameters
/// - `token` - The token.
pub fn token_name(token: TokenId) -> Result<Option<Vec<u8>>> {}

/// Returns the symbol for the specified token, if available.
///
/// # Parameters
/// - `token` - The token.
pub fn token_symbol(token: TokenId) -> Result<Option<Vec<u8>>> {}

/// Returns the decimals for the specified token.
///
/// # Parameters
/// - `token` - The token.
pub fn token_decimals(token: TokenId) -> Result<u8> {}
```

#### PSP-22 Management
The interface for creating, managing and destroying fungible tokens.
```rust
/// Create a new token with a given identifier.
///
/// # Parameters
/// - `id` - The identifier of the token.
/// - `admin` - The account that will administer the token.
/// - `min_balance` - The minimum balance required for accounts holding this token.
pub fn create(id: TokenId, admin: AccountId, min_balance: Balance) -> Result<()> {}

/// Start the process of destroying a token.
///
/// # Parameters
/// - `token` - The token to be destroyed.
pub fn start_destroy(token: TokenId) -> Result<()> {}

/// Set the metadata for a token.
///
/// # Parameters
/// - `token`: The token to update.
/// - `name`: The user friendly name of this token.
/// - `symbol`: The exchange symbol for this token.
/// - `decimals`: The number of decimals this token uses to represent one unit.
pub fn set_metadata(
    token: TokenId,
    name: Vec<u8>,
    symbol: Vec<u8>,
    decimals: u8,
) -> Result<()> {}

/// Clear the metadata for a token.
///
/// # Parameters
/// - `token` - The token to update
pub fn clear_metadata(token: TokenId) -> Result<()> {}

/// Checks if a specified token exists.
///
/// # Parameters
/// - `token` - The token.
pub fn token_exists(token: TokenId) -> Result<bool> {}
```

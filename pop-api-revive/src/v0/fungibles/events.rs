//! A set of events for use in smart contracts interacting with the fungibles API.
//!
//! The `Transfer` and `Approval` events conform to the PSP-22 standard. The other events
//! (`Create`, `StartDestroy`, `SetMetadata`, `ClearMetadata`) are provided for convenience.
//!
//! These events are not emitted by the API itself but can be used in your contracts to
//! track token operations. Be mindful of the costs associated with emitting events.
//!
//! For more details, refer to [ink! events](https://use.ink/basics/events).

use super::*;

/// Event emitted when allowance by `owner` to `spender` changes.
// Differing style: event name abides by the PSP22 standard.
#[ink::event]
pub struct Approval {
	/// The owner providing the allowance.
	#[ink(topic)]
	pub owner: AccountId,
	/// The beneficiary of the allowance.
	#[ink(topic)]
	pub spender: AccountId,
	/// The new allowance amount.
	pub value: u128,
}

/// Event emitted when transfer of tokens occurs.
// Differing style: event name abides by the PSP22 standard.
#[ink::event]
pub struct Transfer {
	/// The source of the transfer. `None` when minting.
	#[ink(topic)]
	pub from: Option<AccountId>,
	/// The recipient of the transfer. `None` when burning.
	#[ink(topic)]
	pub to: Option<AccountId>,
	/// The amount transferred (or minted/burned).
	pub value: u128,
}

/// Event emitted when a token is created.
#[ink::event]
pub struct Created {
	/// The token identifier.
	#[ink(topic)]
	pub id: TokenId,
	/// The creator of the token.
	#[ink(topic)]
	pub creator: AccountId,
	/// The administrator of the token.
	#[ink(topic)]
	pub admin: AccountId,
}

/// Event emitted when a token is in the process of being destroyed.
#[ink::event]
pub struct DestroyStarted {
	/// The token.
	#[ink(topic)]
	pub token: TokenId,
}

/// Event emitted when new metadata is set for a token.
#[ink::event]
pub struct MetadataSet {
	/// The token.
	#[ink(topic)]
	pub token: TokenId,
	/// The name of the token.
	#[ink(topic)]
	pub name: Vec<u8>,
	/// The symbol of the token.
	#[ink(topic)]
	pub symbol: Vec<u8>,
	/// The decimals of the token.
	pub decimals: u8,
}

/// Event emitted when metadata is cleared for a token.
#[ink::event]
pub struct MetadataCleared {
	/// The token.
	#[ink(topic)]
	pub token: TokenId,
}

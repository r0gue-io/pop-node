//! A set of events for use in smart contracts interacting with the fungibles API.
//!
//! The `Transfer` and `Approval` events conform to the ERC-20 standard. The other events
//! (`Create`, `StartDestroy`, `SetMetadata`, `ClearMetadata`) are provided for convenience.
//!
//! These events are not emitted by the API itself but can be used in your contracts to
//! track token operations. Be mindful of the costs associated with emitting events.
//!
//! For more details, refer to [ink! events](https://use.ink/basics/events).

pub use erc20::{Approval, Transfer};

use super::*;

/// Event emitted when a token is created.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Created {
	/// The token identifier.
	#[ink(topic)]
	pub id: TokenId,
	/// The creator of the token.
	#[ink(topic)]
	pub creator: Address,
	/// The administrator of the token.
	#[ink(topic)]
	pub admin: Address,
}

/// Event emitted when a token is in the process of being destroyed.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct DestroyStarted {
	/// The token.
	#[ink(topic)]
	pub token: TokenId,
}

/// Event emitted when new metadata is set for a token.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MetadataSet {
	/// The token.
	#[ink(topic)]
	pub token: TokenId,
	/// The name of the token.
	#[ink(topic)]
	pub name: String,
	/// The symbol of the token.
	#[ink(topic)]
	pub symbol: String,
	/// The decimals of the token.
	pub decimals: u8,
}

/// Event emitted when metadata is cleared for a token.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct MetadataCleared {
	/// The token.
	#[ink(topic)]
	pub token: TokenId,
}

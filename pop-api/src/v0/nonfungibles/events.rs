//! A set of events for use in smart contracts interacting with the non-fungibles API.
//!
//! The `Transfer`, `Approval` and `AttributeSet` events conform to the PSP-34 standard.
//!
//! These events are not emitted by the API itself but can be used in your contracts to
//! track token operations. Be mindful of the costs associated with emitting events.
//!
//! For more details, refer to [ink! events](https://use.ink/basics/events).

use super::*;

/// Event emitted when a token transfer occurs.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Transfer {
	/// The source of the transfer. `None` when minting.
	#[ink(topic)]
	pub from: Option<AccountId>,
	/// The recipient of the transfer. `None` when burning.
	#[ink(topic)]
	pub to: Option<AccountId>,
	/// The item transferred (or minted/burned).
	pub item: ItemId,
}

/// Event emitted when a token approve occurs.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Approval {
	/// The owner providing the allowance.
	#[ink(topic)]
	pub owner: AccountId,
	/// The beneficiary of the allowance.
	#[ink(topic)]
	pub operator: AccountId,
	/// The item which is (dis)approved. `None` for all owner's items.
	pub item: Option<ItemId>,
	/// Whether allowance is set or removed.
	pub approved: bool,
}

/// Event emitted when an attribute is set for a token.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AttributeSet {
	/// The item which attribute is set.
	#[ink(topic)]
	pub item: Option<ItemId>,
	/// The key for the attribute.
	pub key: Vec<u8>,
	/// The data for the attribute.
	pub data: Vec<u8>,
}

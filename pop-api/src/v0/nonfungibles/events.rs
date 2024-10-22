//! A set of events for use in smart contracts interacting with the nonfungibles API.
//!
//! The `Transfer` and `Approval` events conform to the PSP-34 standard.
//!
//! These events are not emitted by the API itself but can be used in your contracts to
//! track token operations. Be mindful of the costs associated with emitting events.
//!
//! For more details, refer to [ink! events](https://use.ink/basics/events).

use super::*;

/// Event emitted when a token transfer occurs.
#[ink::event]
pub struct Transfer {
	/// The source of the transfer. `None` when minting.
	from: Option<AccountId>,
	/// The recipient of the transfer. `None` when burning.
	to: Option<AccountId>,
	/// The item transferred (or minted/burned).
	item: ItemId,
}

/// Event emitted when a token approve occurs.
#[ink::event]
pub struct Approval {
	/// The owner providing the allowance.
	owner: AccountId,
	/// The beneficiary of the allowance.
	operator: AccountId,
	/// The item which is (dis)approved. `None` for all owner's items.
	item: Option<ItemId>,
	/// Whether allowance is set or removed.
	approved: bool,
}

/// Event emitted when an attribute is set for a token.
#[ink::event]
pub struct AttributeSet {
	/// The item which attribute is set.
	item: ItemId,
	/// The key for the attribute.
	key: Vec<u8>,
	/// The data for the attribute.
	data: Vec<u8>,
}

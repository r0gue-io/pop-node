use super::*;

/// Emitted when the allowance of a `spender` for an `owner` is set by a call to
/// [`approve`]. `value` is the new allowance.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Approval {
	/// The owner providing the allowance.
	#[ink(topic)]
	pub owner: Address,
	/// The beneficiary of the allowance.
	#[ink(topic)]
	pub spender: Address,
	/// The new allowance amount.
	pub value: U256,
}

/// Emitted when `value` tokens are moved from one account (`from`) to another (`to`).
///
/// Note that `value` may be zero.
#[ink::event]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Transfer {
	/// The source of the transfer. The zero address when minting.
	#[ink(topic)]
	pub from: Address,
	/// The recipient of the transfer. The zero address when burning.
	#[ink(topic)]
	pub to: Address,
	/// The amount transferred (or minted/burned).
	pub value: U256,
}

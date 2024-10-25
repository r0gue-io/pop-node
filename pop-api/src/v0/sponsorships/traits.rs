//! Traits that can be used by contracts. Including standard compliant traits.

use super::*;

/// The CanSponsor trait.
#[ink::trait_definition]
pub trait IsSponsor {
	/// Registers a new sponsorship relation between the caller and an account.
	///
	/// # Parameters
	/// - `beneficiary` - The account to be sponsored.
	#[ink(message)]
	fn sponsor_account(&self, beneficiary: AccountId) -> Result<()>;

	/// Remove an account from the list of sponsored accounts managed by origin.
	///
	/// Parameters
	/// - `account`: Account to be removed from a sponsorship.
	#[ink(message)]
	fn remove_sponsorship(&self, account: AccountId) -> Result<()>;
}

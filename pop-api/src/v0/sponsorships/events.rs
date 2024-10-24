use super::*;

/// Event emitted when a new sponsorship is registered.
#[ink::event]
pub struct NewSponsorship {
	/// The account acting as sponsor.
	#[ink(topic)]
	pub sponsor: AccountId,
	/// The account beneficiary of the sponsorship.
	#[ink(topic)]
	pub beneficiary: AccountId,
}

/// Event emitted when a sponsorship is removed.
#[ink::event]
pub struct SponsorshipRemoved {
	/// Account no longer acting as sponsor.
	#[ink(topic)]
	pub was_sponsor: AccountId,
	/// Account no longer being sponsored.
	#[ink(topic)]
	pub was_beneficiary: AccountId,
}

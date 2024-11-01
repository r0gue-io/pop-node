use frame_support::parameter_types;

use crate::{deposit, Balance, Balances, Runtime, RuntimeEvent};

parameter_types! {
	pub const SponsorshipDeposit: Balance = deposit(1, 88);
}
impl pallet_sponsorships::Config for Runtime {
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type SponsorshipDeposit = SponsorshipDeposit;
	type WeightInfo = pallet_sponsorships::weights::SubstrateWeight<Self>;
}

use frame_support::parameter_types;

use crate::{deposit, Balance, Balances, Runtime, RuntimeCall, RuntimeEvent};

parameter_types! {
	// One storage item; key size is 32 + 32; value is size 4+4+16+32 bytes = 120 bytes.
	pub const DepositBase: Balance = deposit(1, 120);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u32 = 100;
}

impl pallet_multisig::Config for Runtime {
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

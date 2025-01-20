use frame_support::{pallet_prelude::ConstU32, traits::EqualPrivilegeOnly};

use crate::{
	deposit, parameter_types, AccountId, Balance, Balances, EnsureRoot, HoldConsideration,
	LinearStoragePrice, OriginCaller, Perbill, Preimage, Runtime, RuntimeBlockWeights, RuntimeCall,
	RuntimeEvent, RuntimeHoldReason, RuntimeOrigin, Weight,
};

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

parameter_types! {
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
	pub const PreimageBaseDeposit: Balance = deposit(2, 64);
	pub const PreimageByteDeposit: Balance = deposit(0, 1);
}

impl pallet_preimage::Config for Runtime {
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(60) *
		RuntimeBlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	#[cfg(feature = "runtime-benchmarks")]
	type MaxScheduledPerBlock = ConstU32<512>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MaxScheduledPerBlock = ConstU32<50>;
	type MaximumWeight = MaximumSchedulerWeight;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PalletsOrigin = OriginCaller;
	type Preimages = Preimage;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use frame_support::traits::Get;

	use super::*;

	#[test]
	fn utility_caller_origin_provided_by_runtime() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_utility::Config>::PalletsOrigin>(),
			TypeId::of::<OriginCaller>(),
		);
	}

	#[test]
	fn utility_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_utility::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn multisig_uses_balances_for_deposits() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_multisig::Config>::Currency>(),
			TypeId::of::<Balances>(),
		);
	}

	#[test]
	fn multisig_call_deposit_has_base_amount() {
		assert_eq!(
			<<Runtime as pallet_multisig::Config>::DepositBase as Get<Balance>>::get(),
			deposit(1, 120)
		);
	}

	#[test]
	fn multisig_call_deposit_has_additional_factor() {
		assert_eq!(
			<<Runtime as pallet_multisig::Config>::DepositFactor as Get<Balance>>::get(),
			deposit(0, 32)
		);
	}

	#[test]
	fn multisig_restricts_max_signatories() {
		assert_eq!(<<Runtime as pallet_multisig::Config>::MaxSignatories as Get<u32>>::get(), 100);
	}

	#[test]
	fn multisig_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_multisig::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn preimage_uses_balances_as_currency() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_preimage::Config>::Currency>(),
			TypeId::of::<Balances>(),
		);
	}

	#[test]
	fn preimage_manage_origin_is_root() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_preimage::Config>::ManagerOrigin>(),
			TypeId::of::<EnsureRoot<AccountId>>(),
		);
	}

	#[test]
	fn preimage_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_preimage::Config>::WeightInfo>(),
			TypeId::of::<()>()
		);
	}

	#[test]
	fn preimage_hold_reason_uses_linear_price() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_preimage::Config>::Consideration>(),
			TypeId::of::<
				HoldConsideration<
					AccountId,
					Balances,
					PreimageHoldReason,
					LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
				>,
			>()
		);
	}

	#[test]
	fn preimage_base_deposit() {
		assert_eq!(PreimageBaseDeposit::get(), deposit(2, 64));
	}

	#[test]
	fn preimage_byte_deposit() {
		assert_eq!(PreimageByteDeposit::get(), deposit(0, 1));
	}
}

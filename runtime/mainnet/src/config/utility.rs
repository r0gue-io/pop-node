use frame_support::parameter_types;
use crate::{Balance, Balances, OriginCaller, Runtime, RuntimeCall, RuntimeEvent, deposit};

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
}

use frame_support::traits::InstanceFilter;
use pop_runtime_common::proxy::{
	AnnouncementDepositBase, AnnouncementDepositFactor, MaxPending, MaxProxies, ProxyDepositBase,
	ProxyDepositFactor, ProxyType,
};
use sp_runtime::traits::BlakeTwo256;

use crate::{Balances, Runtime, RuntimeCall, RuntimeEvent};

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(c, RuntimeCall::Balances { .. }),
			ProxyType::CancelProxy => matches!(
				c,
				RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. }) |
					RuntimeCall::Utility { .. } |
					RuntimeCall::Multisig { .. }
			),
			ProxyType::Assets => {
				matches!(c, RuntimeCall::Utility { .. } | RuntimeCall::Multisig { .. })
			},
			ProxyType::AssetOwner => {
				matches!(c, RuntimeCall::Utility { .. } | RuntimeCall::Multisig { .. })
			},
			ProxyType::AssetManager => {
				matches!(c, RuntimeCall::Utility { .. } | RuntimeCall::Multisig { .. })
			},
			ProxyType::Collator => matches!(
				c,
				RuntimeCall::CollatorSelection { .. } |
					RuntimeCall::Utility { .. } |
					RuntimeCall::Multisig { .. }
			),
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		ProxyType::is_superset(self, o)
	}
}

impl pallet_proxy::Config for Runtime {
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type CallHasher = BlakeTwo256;
	type Currency = Balances;
	type MaxPending = MaxPending;
	type MaxProxies = MaxProxies;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type ProxyType = ProxyType;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Self>;
}

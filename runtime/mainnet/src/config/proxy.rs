use frame_support::traits::InstanceFilter;
use pop_runtime_common::proxy::{MaxPending, MaxProxies, ProxyType};

use crate::{
	config::monetary::deposit, parameter_types, Balance, Balances, BlakeTwo256, Runtime,
	RuntimeCall, RuntimeEvent,
};

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

parameter_types! {
	// One storage item; key size 32, value size 16.
	pub const ProxyDepositBase: Balance = deposit(1, 48);
	// Additional storage item size of AccountId 32 bytes + ProxyType 1 byte + BlockNum 4 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 37);
	// One storage item; key size 32, value size 16.
	pub const AnnouncementDepositBase: Balance = deposit(1, 48);
	// Additional storage item 32 bytes AccountId + 32 bytes Hash + 4 bytes BlockNum.
	pub const AnnouncementDepositFactor: Balance = deposit(0, 68);
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

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use frame_support::traits::Get;

	use super::*;

	#[test]
	fn proxy_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_proxy::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}

	#[test]
	fn pallet_proxy_uses_proxy_type() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_proxy::Config>::ProxyType>(),
			TypeId::of::<ProxyType>(),
		);
	}

	#[test]
	fn proxy_has_deposit_factor() {
		assert_eq!(
			<<Runtime as pallet_proxy::Config>::ProxyDepositFactor as Get<Balance>>::get(),
			deposit(0, 37),
		);
	}

	#[test]
	fn proxy_has_deposit_base() {
		assert_eq!(
			<<Runtime as pallet_proxy::Config>::ProxyDepositBase as Get<Balance>>::get(),
			deposit(1, 48),
		);
	}

	#[test]
	fn proxy_uses_balances_as_currency() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_proxy::Config>::Currency>(),
			TypeId::of::<Balances>(),
		);
	}

	#[test]
	fn proxy_configures_max_num_of_proxies() {
		assert_eq!(<<Runtime as pallet_proxy::Config>::MaxProxies>::get(), 32,);
	}

	#[test]
	fn proxy_configures_max_pending() {
		assert_eq!(<<Runtime as pallet_proxy::Config>::MaxPending>::get(), 32,);
	}

	#[test]
	fn proxy_uses_blaketwo256_as_hasher() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_proxy::Config>::CallHasher>(),
			TypeId::of::<BlakeTwo256>(),
		);
	}

	#[test]
	fn proxy_has_announcement_deposit_factor() {
		assert_eq!(
			<<Runtime as pallet_proxy::Config>::AnnouncementDepositFactor as Get<Balance>>::get(),
			deposit(0, 68),
		);
	}

	#[test]
	fn proxy_has_announcement_deposit_base() {
		assert_eq!(
			<<Runtime as pallet_proxy::Config>::AnnouncementDepositBase as Get<Balance>>::get(),
			deposit(1, 48),
		);
	}
}

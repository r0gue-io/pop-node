use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::InstanceFilter;
use pop_runtime_common::proxy::{MaxPending, MaxProxies};
use sp_runtime::RuntimeDebug;

use crate::{
	deposit, parameter_types, Balance, Balances, BlakeTwo256, Runtime, RuntimeCall, RuntimeEvent,
};

/// The type used to represent the kinds of proxying allowed.
// Mainnet will use this definition of ProxyType instead of the ones in
// `pop-common` crates until `pallet-assets` is integrated in the runtime.
// `ProxyType` in `pop-common` include Assets specific proxies which won't
// make much sense in this runtime.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	/// Fully permissioned proxy. Can execute any call on behalf of _proxied_.
	Any,
	/// Can execute any call that does not transfer funds or assets.
	NonTransfer,
	/// Proxy with the ability to reject time-delay proxy announcements.
	CancelProxy,
	/// Collator selection proxy. Can execute calls related to collator selection mechanism.
	Collator,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl ProxyType {
	/// Defines proxies permission hierarchy.
	// Example: A proxy that is not superset of another one won't be able to remove
	// that proxy relationship
	// src: https://github.com/paritytech/polkadot-sdk/blob/4cd07c56378291fddb9fceab3b508cf99034126a/substrate/frame/proxy/src/lib.rs#L802
	pub fn is_superset(s: &ProxyType, o: &ProxyType) -> bool {
		match (s, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, ProxyType::Collator) => true,
			_ => false,
		}
	}
}

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
	// One storage item; key size 32 + hash size 8.
	pub const ProxyDepositBase: Balance = deposit(1, 40);
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
	fn proxy_type_default_is_any() {
		assert_eq!(ProxyType::default(), ProxyType::Any);
	}

	#[test]
	fn proxy_type_superset_as_defined() {
		let all_proxies = vec![
			ProxyType::Any,
			ProxyType::NonTransfer,
			ProxyType::CancelProxy,
			ProxyType::Collator,
		];
		for proxy in all_proxies {
			// Every proxy is part of itself.
			assert!(ProxyType::is_superset(&proxy, &proxy));

			// Any contains all others, but is not contained.
			if proxy != ProxyType::Any {
				assert!(ProxyType::is_superset(&ProxyType::Any, &proxy));
				assert!(!ProxyType::is_superset(&proxy, &ProxyType::Any));
			}
			// CancelProxy does not contain any other proxy.
			if proxy != ProxyType::CancelProxy {
				assert!(!ProxyType::is_superset(&ProxyType::CancelProxy, &proxy));
			}
		}
		assert!(ProxyType::is_superset(&ProxyType::NonTransfer, &ProxyType::Collator));
		assert!(!ProxyType::is_superset(&ProxyType::Collator, &ProxyType::NonTransfer));
	}

	#[test]
	fn proxy_has_announcement_deposit_base() {
		assert_eq!(
			<<Runtime as pallet_proxy::Config>::AnnouncementDepositBase as Get<Balance>>::get(),
			deposit(1, 48),
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
	fn proxy_uses_blaketwo256_as_hasher() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_proxy::Config>::CallHasher>(),
			TypeId::of::<BlakeTwo256>(),
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
	fn proxy_configures_max_pending() {
		assert_eq!(<<Runtime as pallet_proxy::Config>::MaxPending>::get(), 32,);
	}

	#[test]
	fn proxy_configures_max_num_of_proxies() {
		assert_eq!(<<Runtime as pallet_proxy::Config>::MaxProxies>::get(), 32,);
	}

	#[test]
	fn proxy_has_deposit_base() {
		assert_eq!(
			<<Runtime as pallet_proxy::Config>::ProxyDepositBase as Get<Balance>>::get(),
			deposit(1, 40),
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
	fn pallet_proxy_uses_proxy_type() {
		assert_eq!(
			TypeId::of::<<Runtime as pallet_proxy::Config>::ProxyType>(),
			TypeId::of::<ProxyType>(),
		);
	}

	#[test]
	fn proxy_does_not_use_default_weights() {
		assert_ne!(
			TypeId::of::<<Runtime as pallet_proxy::Config>::WeightInfo>(),
			TypeId::of::<()>(),
		);
	}
}

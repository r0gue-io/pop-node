use frame_support::traits::InstanceFilter;
use pop_runtime_common::proxy::{
	AnnouncementDepositBase, AnnouncementDepositFactor, MaxPending, MaxProxies, ProxyDepositBase,
	ProxyDepositFactor, ProxyType,
};
use sp_runtime::traits::BlakeTwo256;

use super::assets::{NftsCall, TrustBackedAssetsCall};
use crate::{Balances, Runtime, RuntimeCall, RuntimeEvent};

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::Balances { .. } |
					RuntimeCall::Assets { .. } |
					RuntimeCall::Nfts { .. }
			),
			ProxyType::CancelProxy => matches!(
				c,
				RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. }) |
					RuntimeCall::Utility { .. } |
					RuntimeCall::Multisig { .. }
			),
			ProxyType::Assets => {
				matches!(
					c,
					RuntimeCall::Assets { .. } |
						RuntimeCall::Utility { .. } |
						RuntimeCall::Multisig { .. } |
						RuntimeCall::Nfts { .. }
				)
			},
			ProxyType::AssetOwner => matches!(
				c,
				RuntimeCall::Assets(TrustBackedAssetsCall::create { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::start_destroy { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::destroy_accounts { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::destroy_approvals { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::finish_destroy { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::transfer_ownership { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::set_team { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::set_metadata { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::clear_metadata { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::set_min_balance { .. }) |
					RuntimeCall::Nfts(NftsCall::create { .. }) |
					RuntimeCall::Nfts(NftsCall::destroy { .. }) |
					RuntimeCall::Nfts(NftsCall::redeposit { .. }) |
					RuntimeCall::Nfts(NftsCall::transfer_ownership { .. }) |
					RuntimeCall::Nfts(NftsCall::set_team { .. }) |
					RuntimeCall::Nfts(NftsCall::set_collection_max_supply { .. }) |
					RuntimeCall::Nfts(NftsCall::lock_collection { .. }) |
					RuntimeCall::Utility { .. } |
					RuntimeCall::Multisig { .. }
			),
			ProxyType::AssetManager => matches!(
				c,
				RuntimeCall::Assets(TrustBackedAssetsCall::mint { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::burn { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::freeze { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::block { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::thaw { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::freeze_asset { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::thaw_asset { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::touch_other { .. }) |
					RuntimeCall::Assets(TrustBackedAssetsCall::refund_other { .. }) |
					RuntimeCall::Nfts(NftsCall::force_mint { .. }) |
					RuntimeCall::Nfts(NftsCall::update_mint_settings { .. }) |
					RuntimeCall::Nfts(NftsCall::mint_pre_signed { .. }) |
					RuntimeCall::Nfts(NftsCall::set_attributes_pre_signed { .. }) |
					RuntimeCall::Nfts(NftsCall::lock_item_transfer { .. }) |
					RuntimeCall::Nfts(NftsCall::unlock_item_transfer { .. }) |
					RuntimeCall::Nfts(NftsCall::lock_item_properties { .. }) |
					RuntimeCall::Nfts(NftsCall::set_metadata { .. }) |
					RuntimeCall::Nfts(NftsCall::clear_metadata { .. }) |
					RuntimeCall::Nfts(NftsCall::set_collection_metadata { .. }) |
					RuntimeCall::Nfts(NftsCall::clear_collection_metadata { .. }) |
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

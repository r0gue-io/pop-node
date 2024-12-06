use frame_support::traits::InstanceFilter;
use pop_runtime_common::proxy::{
	AnnouncementDepositBase, AnnouncementDepositFactor, MaxPending, MaxProxies, ProxyDepositBase,
	ProxyDepositFactor, ProxyType,
};
use sp_runtime::traits::BlakeTwo256;

use super::assets::{TrustBackedAssetsCall, TrustBackedNftsCall};
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
					RuntimeCall::Nfts(TrustBackedNftsCall::create { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::destroy { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::redeposit { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::transfer_ownership { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::set_team { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::set_collection_max_supply { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::lock_collection { .. }) |
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
					RuntimeCall::Nfts(TrustBackedNftsCall::force_mint { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::update_mint_settings { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::mint_pre_signed { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::set_attributes_pre_signed { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::lock_item_transfer { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::unlock_item_transfer { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::lock_item_properties { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::set_metadata { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::clear_metadata { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::set_collection_metadata { .. }) |
					RuntimeCall::Nfts(TrustBackedNftsCall::clear_collection_metadata { .. }) |
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

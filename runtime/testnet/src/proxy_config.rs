use crate::{
	assets_config::TrustBackedAssetsCall, deposit, Balance, Balances, Runtime, RuntimeCall,
	RuntimeEvent,
};
use codec::{Decode, Encode, MaxEncodedLen};
use sp_runtime::{traits::BlakeTwo256, RuntimeDebug};

use frame_support::{parameter_types, traits::InstanceFilter};

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 40);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	// One storage item; key size 32, value size 16
	pub const AnnouncementDepositBase: Balance = deposit(1, 48);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
	pub const MaxPending: u16 = 32;
}

/// The type used to represent the kinds of proxying allowed.
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
	/// Assets proxy. Can execute any call from `assets`, **including asset transfers**.
	Assets,
	/// Owner proxy. Can execute calls related to asset ownership.
	AssetOwner,
	/// Asset manager. Can execute calls related to asset management.
	AssetManager,
	/// Collator selection proxy. Can execute calls related to collator selection mechanism.
	Collator,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}

impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::Balances { .. }
					| RuntimeCall::Assets { .. }
					| RuntimeCall::Nfts { .. }
			),
			ProxyType::CancelProxy => matches!(
				c,
				RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })
					| RuntimeCall::Utility { .. }
					| RuntimeCall::Multisig { .. }
			),
			ProxyType::Assets => {
				matches!(
					c,
					RuntimeCall::Assets { .. }
						| RuntimeCall::Utility { .. }
						| RuntimeCall::Multisig { .. }
						| RuntimeCall::Nfts { .. }
				)
			},
			ProxyType::AssetOwner => matches!(
				c,
				RuntimeCall::Assets(TrustBackedAssetsCall::create { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::start_destroy { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::destroy_accounts { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::destroy_approvals { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::finish_destroy { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::transfer_ownership { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::set_team { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::set_metadata { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::clear_metadata { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::set_min_balance { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::create { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::destroy { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::redeposit { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::transfer_ownership { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::set_team { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::set_collection_max_supply { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::lock_collection { .. })
					| RuntimeCall::Utility { .. }
					| RuntimeCall::Multisig { .. }
			),
			ProxyType::AssetManager => matches!(
				c,
				RuntimeCall::Assets(TrustBackedAssetsCall::mint { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::burn { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::freeze { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::block { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::thaw { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::freeze_asset { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::thaw_asset { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::touch_other { .. })
					| RuntimeCall::Assets(TrustBackedAssetsCall::refund_other { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::force_mint { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::update_mint_settings { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::mint_pre_signed { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::set_attributes_pre_signed { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::lock_item_transfer { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::unlock_item_transfer { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::lock_item_properties { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::set_metadata { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::clear_metadata { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::set_collection_metadata { .. })
					| RuntimeCall::Nfts(pallet_nfts::Call::clear_collection_metadata { .. })
					| RuntimeCall::Utility { .. }
					| RuntimeCall::Multisig { .. }
			),
			ProxyType::Collator => matches!(
				c,
				RuntimeCall::CollatorSelection { .. }
					| RuntimeCall::Utility { .. }
					| RuntimeCall::Multisig { .. }
			),
		}
	}

	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::Assets, ProxyType::AssetOwner) => true,
			(ProxyType::Assets, ProxyType::AssetManager) => true,
			(ProxyType::NonTransfer, ProxyType::Collator) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Self>;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

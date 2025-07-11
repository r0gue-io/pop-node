use frame_support::traits::InstanceFilter;
use pallet_nfts_sdk as pallet_nfts;
use pop_runtime_common::proxy::{MaxPending, MaxProxies, ProxyType};

use crate::{
	config::assets::TrustBackedAssetsCall, deposit, parameter_types, weights, Balance, Balances,
	BlakeTwo256, Runtime, RuntimeCall, RuntimeEvent, System,
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
				matches!(
					c,
					RuntimeCall::Assets { .. } |
						RuntimeCall::Multisig { .. } |
						RuntimeCall::Nfts { .. } |
						RuntimeCall::Utility { .. }
				)
			},
			ProxyType::AssetOwner => {
				matches!(
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
						RuntimeCall::Nfts(pallet_nfts::Call::create { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::destroy { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::redeposit { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::transfer_ownership { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::set_team { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::set_collection_max_supply { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::lock_collection { .. }) |
						RuntimeCall::Multisig { .. } |
						RuntimeCall::Utility { .. }
				)
			},
			ProxyType::AssetManager => {
				matches!(
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
						RuntimeCall::Nfts(pallet_nfts::Call::force_mint { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::update_mint_settings { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::mint_pre_signed { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::set_attributes_pre_signed { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::lock_item_transfer { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::unlock_item_transfer { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::lock_item_properties { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::set_metadata { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::clear_metadata { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::set_collection_metadata { .. }) |
						RuntimeCall::Nfts(pallet_nfts::Call::clear_collection_metadata { .. }) |
						RuntimeCall::Multisig { .. } |
						RuntimeCall::Utility { .. }
				)
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
	// One storage item; key size 32 + hash size 8.
	pub const ProxyDepositBase: Balance = deposit(1, 40);
	// Additional storage item size of AccountId 32 bytes + ProxyType 1 byte + BlockNum 4 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 37);
	// One storage item; key size 32, value size 16 + hash size 8.
	pub const AnnouncementDepositBase: Balance = deposit(1, 56);
	// Additional storage item 32 bytes AccountId + 32 bytes Hash + 4 bytes BlockNum.
	pub const AnnouncementDepositFactor: Balance = deposit(0, 68);
}

impl pallet_proxy::Config for Runtime {
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type BlockNumberProvider = System;
	type CallHasher = BlakeTwo256;
	type Currency = Balances;
	type MaxPending = MaxPending;
	type MaxProxies = MaxProxies;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type ProxyType = ProxyType;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_proxy::WeightInfo<Self>;
}

#[cfg(test)]
mod tests {
	use std::any::TypeId;

	use codec::MaxEncodedLen;
	use frame_support::{traits::Get, StorageHasher, Twox64Concat};
	use pallet_proxy::Config;
	use parachains_common::BlockNumber;
	use sp_runtime::{traits::Hash, MultiAddress};
	use ProxyType::*;

	use super::*;
	use crate::AccountId;

	#[test]
	fn proxy_type_assets_can_only_transfer_assets() {
		use sp_keyring::Sr25519Keyring::Alice;
		let alice_address = MultiAddress::Id(Alice.to_account_id());

		// Assert proxy type whitelists any asset related call.
		asset_transfer_calls().iter().for_each(|call| {
			assert!(Assets.filter(&call));
		});

		let balances_transfer_call =
			RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive {
				dest: alice_address.clone(),
				value: 0,
			});
		// Asset proxy type filters transfer calls from Balances pallet.
		assert!(!Assets.filter(&balances_transfer_call));
	}

	#[test]
	fn asset_owner_cannot_transfer_assets() {
		// AssetOwner won't be able to transfer assets.
		asset_transfer_calls().iter().for_each(|call| {
			assert!(!AssetOwner.filter(&call));
		});
	}

	#[test]
	fn asset_manager_cannot_transfer_assets() {
		// AssetManager won't be able to transfer assets.
		asset_transfer_calls().iter().for_each(|call| {
			assert!(!AssetManager.filter(&call));
		});
	}

	#[test]
	fn proxy_has_announcement_deposit_base() {
		// AnnouncementDepositBase #bytes.
		let base_bytes = Twox64Concat::max_len::<AccountId>() + Balance::max_encoded_len();
		assert_eq!(base_bytes, 56);

		assert_eq!(
			<<Runtime as Config>::AnnouncementDepositBase as Get<Balance>>::get(),
			deposit(1, 56),
		);
	}
	#[test]
	fn proxy_has_announcement_deposit_factor() {
		// AnnouncementDepositFactor #bytes.
		let factor_bytes = AccountId::max_encoded_len() +
			<<Runtime as Config>::CallHasher as Hash>::Output::max_encoded_len() +
			BlockNumber::max_encoded_len();
		assert_eq!(factor_bytes, 68);

		assert_eq!(
			<<Runtime as Config>::AnnouncementDepositFactor as Get<Balance>>::get(),
			deposit(0, 68),
		);
	}

	#[test]
	fn ensure_system_is_block_number_provider() {
		assert_eq!(
			TypeId::of::<<Runtime as Config>::BlockNumberProvider>(),
			TypeId::of::<System>(),
		);
	}

	#[test]
	fn proxy_uses_blaketwo256_as_hasher() {
		assert_eq!(TypeId::of::<<Runtime as Config>::CallHasher>(), TypeId::of::<BlakeTwo256>(),);
	}

	#[test]
	fn proxy_uses_balances_as_currency() {
		assert_eq!(TypeId::of::<<Runtime as Config>::Currency>(), TypeId::of::<Balances>(),);
	}

	#[test]
	fn proxy_configures_max_pending() {
		assert_eq!(<<Runtime as Config>::MaxPending>::get(), 32,);
	}

	#[test]
	fn proxy_configures_max_num_of_proxies() {
		assert_eq!(<<Runtime as Config>::MaxProxies>::get(), 32,);
	}

	#[test]
	fn proxy_has_deposit_base() {
		// ProxyDepositBase #bytes
		let base_bytes = Twox64Concat::max_len::<AccountId>();
		assert_eq!(base_bytes, 40);

		assert_eq!(<<Runtime as Config>::ProxyDepositBase as Get<Balance>>::get(), deposit(1, 40),);
	}

	#[test]
	fn proxy_has_deposit_factor() {
		// ProxyDepositFactor #bytes
		let factor_bytes = AccountId::max_encoded_len() +
			ProxyType::max_encoded_len() +
			BlockNumber::max_encoded_len();
		assert_eq!(factor_bytes, 37);

		assert_eq!(
			<<Runtime as Config>::ProxyDepositFactor as Get<Balance>>::get(),
			deposit(0, 37),
		);
	}

	#[test]
	fn pallet_proxy_uses_proxy_type() {
		assert_eq!(TypeId::of::<<Runtime as Config>::ProxyType>(), TypeId::of::<ProxyType>(),);
	}

	#[test]
	fn proxy_does_not_use_default_weights() {
		assert_ne!(TypeId::of::<<Runtime as Config>::WeightInfo>(), TypeId::of::<()>(),);
	}

	// Returns a list with some calls transferring assets.
	fn asset_transfer_calls() -> Vec<RuntimeCall> {
		use sp_keyring::Sr25519Keyring::Alice;
		let alice_address = MultiAddress::Id(Alice.to_account_id());

		vec![
			RuntimeCall::Assets(pallet_assets::Call::transfer_keep_alive {
				id: codec::Compact(0),
				target: alice_address.clone(),
				amount: 0,
			}),
			RuntimeCall::Nfts(pallet_nfts::Call::transfer {
				collection: 0,
				item: 0,
				dest: alice_address.clone(),
			}),
		]
	}
}

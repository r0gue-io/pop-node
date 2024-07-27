pub(crate) mod genesis;

use crate::chains::paseo::Paseo;
use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_assets_helpers_for_parachain, impl_foreign_assets_helpers_for_parachain,
	impl_xcm_helpers_for_parachain, impls::Parachain, xcm_emulator::decl_test_parachains,
};
use frame_support::traits::OnInitialize;

// AssetHubPaseo Parachain declaration
decl_test_parachains! {
	pub struct AssetHubPaseo {
		genesis = genesis::genesis(),
		on_init = {
			asset_hub_paseo_runtime::AuraExt::on_initialize(1);
		},
		runtime = asset_hub_paseo_runtime,
		core = {
			XcmpMessageHandler: asset_hub_paseo_runtime::XcmpQueue,
			LocationToAccountId: asset_hub_paseo_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_paseo_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: asset_hub_paseo_runtime::PolkadotXcm,
			Assets: asset_hub_paseo_runtime::Assets,
			ForeignAssets: asset_hub_paseo_runtime::ForeignAssets,
			Balances: asset_hub_paseo_runtime::Balances,
		}
	},
}

// AssetHubPaseo implementation
impl_accounts_helpers_for_parachain!(AssetHubPaseo);
impl_assert_events_helpers_for_parachain!(AssetHubPaseo);
impl_assets_helpers_for_parachain!(AssetHubPaseo, Paseo);
impl_foreign_assets_helpers_for_parachain!(AssetHubPaseo, Paseo);
impl_xcm_helpers_for_parachain!(AssetHubPaseo);

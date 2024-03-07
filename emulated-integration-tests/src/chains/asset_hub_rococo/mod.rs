pub mod genesis;

// Substrate
use frame_support::traits::OnInitialize;

// Cumulus
use crate::chains::rococo::Rococo;
use emulated_integration_tests_common::{
    impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
    impl_assets_helpers_for_parachain, impl_foreign_assets_helpers_for_parachain,
    impl_xcm_helpers_for_parachain, impls::Parachain, xcm_emulator::decl_test_parachains,
};

// AssetHubRococo Parachain declaration
decl_test_parachains! {
    pub struct AssetHubRococo {
        genesis = genesis::genesis(),
        on_init = {
            asset_hub_rococo_runtime::AuraExt::on_initialize(1);
        },
        runtime = asset_hub_rococo_runtime,
        core = {
            XcmpMessageHandler: asset_hub_rococo_runtime::XcmpQueue,
            LocationToAccountId: asset_hub_rococo_runtime::xcm_config::LocationToAccountId,
            ParachainInfo: asset_hub_rococo_runtime::ParachainInfo,
            MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
        },
        pallets = {
            PolkadotXcm: asset_hub_rococo_runtime::PolkadotXcm,
            Assets: asset_hub_rococo_runtime::Assets,
            ForeignAssets: asset_hub_rococo_runtime::ForeignAssets,
            PoolAssets: asset_hub_rococo_runtime::PoolAssets,
            AssetConversion: asset_hub_rococo_runtime::AssetConversion,
            Balances: asset_hub_rococo_runtime::Balances,
        }
    },
}

// AssetHubRococo implementation
impl_accounts_helpers_for_parachain!(AssetHubRococo);
impl_assert_events_helpers_for_parachain!(AssetHubRococo);
impl_assets_helpers_for_parachain!(AssetHubRococo, Rococo);
impl_foreign_assets_helpers_for_parachain!(AssetHubRococo, Rococo);
impl_xcm_helpers_for_parachain!(AssetHubRococo);

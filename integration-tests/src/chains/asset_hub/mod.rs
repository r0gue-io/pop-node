use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_assets_helpers_for_parachain, impl_foreign_assets_helpers_for_parachain,
	impl_xcm_helpers_for_parachain, impls::Parachain, xcm_emulator::decl_test_parachains,
};
use frame_support::traits::OnInitialize;
#[cfg(feature = "paseo")]
pub(crate) use {asset_hub_paseo_runtime as runtime, paseo_runtime_constants as constants};
#[cfg(feature = "westend")]
pub(crate) use {asset_hub_westend_runtime as runtime, westend_runtime_constants as constants};

use super::*;

pub(crate) mod genesis;
use genesis::*;

// AssetHub Parachain declaration.
decl_test_parachains! {
	pub struct AssetHub {
		genesis = genesis::genesis(),
		on_init = {
			runtime::AuraExt::on_initialize(1);
		},
		runtime = runtime,
		core = {
			XcmpMessageHandler: runtime::XcmpQueue,
			LocationToAccountId: runtime::xcm_config::LocationToAccountId,
			ParachainInfo: runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: runtime::PolkadotXcm,
			Assets: runtime::Assets,
			ForeignAssets: runtime::ForeignAssets,
			Balances: runtime::Balances,
		}
	},
}

// AssetHub implementation.
impl_accounts_helpers_for_parachain!(AssetHub);
impl_assert_events_helpers_for_parachain!(AssetHub);
impl_assets_helpers_for_parachain!(AssetHub);
impl_foreign_assets_helpers_for_parachain!(AssetHub, xcm::v5::Location);
impl_xcm_helpers_for_parachain!(AssetHub);

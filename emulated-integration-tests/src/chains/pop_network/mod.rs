pub(crate) mod genesis;

use crate::chains::rococo::Rococo;
use emulated_integration_tests_common::{
    impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
    impl_assets_helpers_for_parachain, impl_xcm_helpers_for_parachain, impls::Parachain,
    xcm_emulator::decl_test_parachains,
};
use frame_support::traits::OnInitialize;

// PopNetwork Parachain declaration
decl_test_parachains! {
    pub struct PopNetwork {
        genesis = genesis::genesis(),
        on_init = {
            pop_runtime::AuraExt::on_initialize(1);
        },
        runtime = pop_runtime,
        core = {
            XcmpMessageHandler: pop_runtime::XcmpQueue,
            LocationToAccountId: pop_runtime::xcm_config::LocationToAccountId,
            ParachainInfo: pop_runtime::ParachainInfo,
            MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
        },
        pallets = {
            PolkadotXcm: pop_runtime::PolkadotXcm,
            Assets: pop_runtime::Assets,
            Balances: pop_runtime::Balances,
        }
    },
}

// PopNetwork implementation
impl_accounts_helpers_for_parachain!(PopNetwork);
impl_assert_events_helpers_for_parachain!(PopNetwork);
impl_assets_helpers_for_parachain!(PopNetwork, Rococo);
impl_xcm_helpers_for_parachain!(PopNetwork);

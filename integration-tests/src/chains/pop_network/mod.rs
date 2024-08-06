pub(crate) mod genesis;

use crate::chains::paseo::Paseo;
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
			pop_runtime_devnet::AuraExt::on_initialize(1);
		},
		runtime = pop_runtime_devnet,
		core = {
			XcmpMessageHandler: pop_runtime_devnet::XcmpQueue,
			LocationToAccountId: pop_runtime_devnet::config::xcm::LocationToAccountId,
			ParachainInfo: pop_runtime_devnet::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: pop_runtime_devnet::PolkadotXcm,
			Assets: pop_runtime_devnet::Assets,
			Balances: pop_runtime_devnet::Balances,
		}
	},
}

// PopNetwork implementation
impl_accounts_helpers_for_parachain!(PopNetwork);
impl_assert_events_helpers_for_parachain!(PopNetwork);
impl_assets_helpers_for_parachain!(PopNetwork, Paseo);
impl_xcm_helpers_for_parachain!(PopNetwork);

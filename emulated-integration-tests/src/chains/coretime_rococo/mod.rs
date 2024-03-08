pub(crate) mod genesis;

use emulated_integration_tests_common::{
	impl_accounts_helpers_for_relay_chain, impl_assert_events_helpers_for_relay_chain,
	impl_hrmp_channels_helpers_for_relay_chain, impl_send_transact_helpers_for_relay_chain,
	xcm_emulator::decl_test_relay_chains,
};

// CoretimeRococo declaration
decl_test_relay_chains! {
	#[api_version(10)]
	pub struct CoretimeRococo {
		genesis = genesis::genesis(),
		on_init = (),
		runtime = coretime_rococo_runtime,
		core = {
			SovereignAccountOf: coretime_rococo_runtime::xcm_config::LocationConverter,
		},
		pallets = {
			XcmPallet: coretime_rococo_runtime::XcmPallet,
			Sudo: coretime_rococo_runtime::Sudo,
			Balances: coretime_rococo_runtime::Balances,
			Broker: coretime_rococo_runtime::Broker,
			Hrmp: coretime_rococo_runtime::Hrmp,
		}
	},
}

// CoretimeRococo implementation
impl_accounts_helpers_for_relay_chain!(CoretimeRococo);
impl_assert_events_helpers_for_relay_chain!(CoretimeRococo);
impl_hrmp_channels_helpers_for_relay_chain!(CoretimeRococo);
impl_send_transact_helpers_for_relay_chain!(CoretimeRococo);

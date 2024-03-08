pub(crate) mod genesis;

use emulated_integration_tests_common::{
	impl_accounts_helpers_for_relay_chain, impl_assert_events_helpers_for_relay_chain,
	impl_hrmp_channels_helpers_for_relay_chain, impl_send_transact_helpers_for_relay_chain,
	xcm_emulator::decl_test_relay_chains,
};

// Rococo declaration
decl_test_relay_chains! {
	#[api_version(10)]
	pub struct Rococo {
		genesis = genesis::genesis(),
		on_init = (),
		runtime = rococo_runtime,
		core = {
			SovereignAccountOf: rococo_runtime::xcm_config::LocationConverter,
		},
		pallets = {
			XcmPallet: rococo_runtime::XcmPallet,
			Sudo: rococo_runtime::Sudo,
			Balances: rococo_runtime::Balances,
			Hrmp: rococo_runtime::Hrmp,
		}
	},
}

// Rococo implementation
impl_accounts_helpers_for_relay_chain!(Rococo);
impl_assert_events_helpers_for_relay_chain!(Rococo);
impl_hrmp_channels_helpers_for_relay_chain!(Rococo);
impl_send_transact_helpers_for_relay_chain!(Rococo);

pub(crate) mod genesis;

use emulated_integration_tests_common::{
	impl_accounts_helpers_for_relay_chain, impl_assert_events_helpers_for_relay_chain,
	impl_hrmp_channels_helpers_for_relay_chain, impl_send_transact_helpers_for_relay_chain,
	xcm_emulator::decl_test_relay_chains,
};

// Paseo declaration
decl_test_relay_chains! {
	#[api_version(10)]
	pub struct Paseo {
		genesis = genesis::genesis(),
		on_init = (),
		runtime = polkadot_runtime,
		core = {
			SovereignAccountOf: polkadot_runtime::xcm_config::SovereignAccountOf,
		},
		pallets = {
			XcmPallet: polkadot_runtime::XcmPallet,
			Sudo: polkadot_runtime::Sudo,
			Balances: polkadot_runtime::Balances,
			Hrmp: polkadot_runtime::Hrmp,
		}
	},
}

// Paseo implementation
impl_accounts_helpers_for_relay_chain!(Paseo);
impl_assert_events_helpers_for_relay_chain!(Paseo);
impl_hrmp_channels_helpers_for_relay_chain!(Paseo);
impl_send_transact_helpers_for_relay_chain!(Paseo);

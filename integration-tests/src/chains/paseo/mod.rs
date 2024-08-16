pub(crate) mod genesis;
use polkadot_runtime as runtime;

use emulated_integration_tests_common::{
	impl_accounts_helpers_for_relay_chain, impl_assert_events_helpers_for_relay_chain,
	impl_hrmp_channels_helpers_for_relay_chain, impl_send_transact_helpers_for_relay_chain,
	xcm_emulator::decl_test_relay_chains,
};

// Paseo declaration
decl_test_relay_chains! {
	#[api_version(11)]
	pub struct Paseo {
		genesis = genesis::genesis(),
		on_init = (),
		runtime = runtime,
		core = {
			SovereignAccountOf: runtime::xcm_config::SovereignAccountOf,
		},
		pallets = {
			XcmPallet: runtime::XcmPallet,
			// Sudo: runtime::Sudo,
			Balances: runtime::Balances,
			Hrmp: runtime::Hrmp,
		}
	},
}

// Paseo implementation
impl_accounts_helpers_for_relay_chain!(Paseo);
impl_assert_events_helpers_for_relay_chain!(Paseo);
impl_hrmp_channels_helpers_for_relay_chain!(Paseo);
impl_send_transact_helpers_for_relay_chain!(Paseo);

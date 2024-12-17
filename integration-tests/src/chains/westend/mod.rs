use emulated_integration_tests_common::{
	impl_accounts_helpers_for_relay_chain, impl_assert_events_helpers_for_relay_chain,
	impl_hrmp_channels_helpers_for_relay_chain, impl_send_transact_helpers_for_relay_chain,
	xcm_emulator::decl_test_relay_chains,
};

pub(crate) mod genesis;

decl_test_relay_chains! {
	#[api_version(11)]
	pub struct Westend {
		genesis = genesis::genesis(),
		on_init = (),
		runtime = westend_runtime,
		core = {
			SovereignAccountOf: westend_runtime::xcm_config::LocationConverter,
		},
		pallets = {
			XcmPallet: westend_runtime::XcmPallet,
			Balances: westend_runtime::Balances,
			Hrmp: westend_runtime::Hrmp,
		}
	},
}

impl_accounts_helpers_for_relay_chain!(Westend);
impl_assert_events_helpers_for_relay_chain!(Westend);
impl_hrmp_channels_helpers_for_relay_chain!(Westend);
impl_send_transact_helpers_for_relay_chain!(Westend);

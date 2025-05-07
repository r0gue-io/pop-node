use emulated_integration_tests_common::{
	impl_accounts_helpers_for_relay_chain, impl_assert_events_helpers_for_relay_chain,
	impl_hrmp_channels_helpers_for_relay_chain, impl_send_transact_helpers_for_relay_chain,
	xcm_emulator::decl_test_relay_chains,
};
use polkadot_primitives::runtime_api::runtime_decl_for_parachain_host::ParachainHostV13;
#[cfg(feature = "paseo")]
pub(crate) use {
	paseo_runtime::{self as runtime, xcm_config::SovereignAccountOf},
	paseo_runtime_constants as constants,
};
#[cfg(feature = "westend")]
pub(crate) use {
	westend_runtime::{self as runtime, xcm_config::LocationConverter as SovereignAccountOf},
	westend_runtime_constants as constants,
};

pub(crate) mod genesis;

// Relay declaration.
decl_test_relay_chains! {
	#[api_version(11)]
	pub struct Relay {
		genesis = genesis::genesis(),
		on_init = (),
		runtime = runtime,
		core = {
			SovereignAccountOf: SovereignAccountOf,
		},
		pallets = {
			XcmPallet: runtime::XcmPallet,
			Balances: runtime::Balances,
			Hrmp: runtime::Hrmp,
		}
	},
}

// Relay implementation.
impl_accounts_helpers_for_relay_chain!(Relay);
impl_assert_events_helpers_for_relay_chain!(Relay);
impl_hrmp_channels_helpers_for_relay_chain!(Relay);
impl_send_transact_helpers_for_relay_chain!(Relay);

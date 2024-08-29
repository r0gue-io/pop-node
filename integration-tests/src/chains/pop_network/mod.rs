pub(crate) mod genesis;

use emulated_integration_tests_common::{
	impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_assets_helpers_for_parachain, impl_xcm_helpers_for_parachain, impls::Parachain,
	xcm_emulator::decl_test_parachains,
};
use frame_support::traits::OnInitialize;
#[cfg(not(feature = "mainnet"))]
use pop_runtime_devnet as runtime;
#[cfg(feature = "mainnet")]
use pop_runtime_mainnet as runtime;

// PopNetwork Parachain declaration
#[cfg(not(feature = "mainnet"))]
decl_test_parachains! {
	pub struct PopNetwork {
		genesis = genesis::genesis(),
		on_init = {
			runtime::AuraExt::on_initialize(1);
		},
		runtime = runtime,
		core = {
			XcmpMessageHandler: runtime::XcmpQueue,
			LocationToAccountId: runtime::config::xcm::LocationToAccountId,
			ParachainInfo: runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: runtime::PolkadotXcm,
			Balances: runtime::Balances,
			Assets: runtime::Assets,
		}
	},
}

#[cfg(feature = "mainnet")]
decl_test_parachains! {
	pub struct PopNetwork {
		genesis = genesis::genesis(),
		on_init = {
			runtime::AuraExt::on_initialize(1);
		},
		runtime = runtime,
		core = {
			XcmpMessageHandler: runtime::XcmpQueue,
			LocationToAccountId: runtime::config::xcm::LocationToAccountId,
			ParachainInfo: runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			PolkadotXcm: runtime::PolkadotXcm,
			Balances: runtime::Balances,
		}
	},
}

// PopNetwork implementation
impl_accounts_helpers_for_parachain!(PopNetwork);
impl_assert_events_helpers_for_parachain!(PopNetwork);
impl_xcm_helpers_for_parachain!(PopNetwork);
#[cfg(not(feature = "mainnet"))]
impl_assets_helpers_for_parachain!(PopNetwork);

use emulated_integration_tests_common::{build_genesis_storage, collators};
use pop_runtime_common::Balance;
use sp_core::storage::Storage;

use super::runtime;

#[cfg(not(feature = "mainnet"))]
pub(crate) const ED: Balance = runtime::EXISTENTIAL_DEPOSIT;
#[cfg(feature = "mainnet")]
pub(crate) const ED: Balance = runtime::EXISTENTIAL_DEPOSIT * 100;
const PARA_ID: u32 = 9090;
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

pub(crate) fn genesis() -> Storage {
	let genesis_config = runtime::RuntimeGenesisConfig {
		system: runtime::SystemConfig::default(),
		balances: runtime::BalancesConfig { ..Default::default() },
		parachain_info: runtime::ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
			..Default::default()
		},
		collator_selection: runtime::CollatorSelectionConfig {
			invulnerables: collators::invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: runtime::SessionConfig {
			keys: collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                   // account id
						acc,                           // validator id
						runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
			..Default::default()
		},
		polkadot_xcm: runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
	)
}

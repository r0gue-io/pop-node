use emulated_integration_tests_common::{accounts, build_genesis_storage, collators};
use pop_runtime_common::Balance;
use sp_core::storage::Storage;

pub(crate) const ED: Balance = pop_runtime_devnet::EXISTENTIAL_DEPOSIT;
const PARA_ID: u32 = 9090;
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

pub(crate) fn genesis() -> Storage {
	let genesis_config = pop_runtime_devnet::RuntimeGenesisConfig {
		system: pop_runtime_devnet::SystemConfig::default(),
		balances: pop_runtime_devnet::BalancesConfig {
			balances: accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096 * 4096))
				.collect(),
		},
		parachain_info: pop_runtime_devnet::ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
			..Default::default()
		},
		collator_selection: pop_runtime_devnet::CollatorSelectionConfig {
			invulnerables: collators::invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: pop_runtime_devnet::SessionConfig {
			keys: collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                       // account id
						acc,                               // validator id
						pop_runtime_devnet::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		polkadot_xcm: pop_runtime_devnet::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		pop_runtime_devnet::WASM_BINARY.expect("WASM binary was not built, please build it!"),
	)
}

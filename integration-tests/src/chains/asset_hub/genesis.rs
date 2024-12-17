use cumulus_primitives_core::relay_chain::Balance;
use emulated_integration_tests_common::{
	accounts, build_genesis_storage, collators::invulnerables, SAFE_XCM_VERSION,
};
use sp_core::storage::Storage;

use crate::chains::asset_hub::{
	constants::currency::EXISTENTIAL_DEPOSIT,
	runtime::{
		BalancesConfig, CollatorSelectionConfig, ParachainInfoConfig, PolkadotXcmConfig,
		RuntimeGenesisConfig, SessionConfig, SessionKeys, SystemConfig, WASM_BINARY,
	},
};

pub(crate) const ED: Balance = EXISTENTIAL_DEPOSIT;
pub(crate) const PARA_ID: u32 = 1000;

pub(crate) fn genesis() -> Storage {
	let genesis_config = RuntimeGenesisConfig {
		system: SystemConfig::default(),
		balances: BalancesConfig {
			balances: accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096 * 4096))
				.collect(),
		},
		parachain_info: ParachainInfoConfig { parachain_id: PARA_ID.into(), ..Default::default() },
		collator_selection: CollatorSelectionConfig {
			invulnerables: invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: SessionConfig {
			keys: invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),          // account id
						acc,                  // validator id
						SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		WASM_BINARY.expect("WASM binary was not built, please build it!"),
	)
}

// In case we want to add Polkadot in the future.
//
// #[cfg(feature = "polkadot")]
// use emulated_integration_tests_common::{get_account_id_from_seed, get_from_seed};
// use sp_core::sr25519;
// use polkadot_primitives::{AccountId, Balance};
//
// type AuraId = sp_consensus_aura::ed25519::AuthorityId;
// pub fn invulnerables() -> Vec<(AccountId, AuraId)> {
// 	vec![
// 		(get_account_id_from_seed::<sr25519::Public>("Alice"), get_from_seed::<AuraId>("Alice")),
// 		(get_account_id_from_seed::<sr25519::Public>("Bob"), get_from_seed::<AuraId>("Bob")),
// 	]
// }

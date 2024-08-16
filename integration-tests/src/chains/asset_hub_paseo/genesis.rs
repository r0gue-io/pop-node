use emulated_integration_tests_common::{
	accounts, build_genesis_storage, get_account_id_from_seed, get_from_seed, SAFE_XCM_VERSION,
};
use polkadot_primitives::{AccountId, Balance};
use sp_core::{sr25519, storage::Storage};

pub(crate) const PARA_ID: u32 = 1000;
pub(crate) const ED: Balance = polkadot_runtime_constants::currency::EXISTENTIAL_DEPOSIT / 10;

pub(crate) fn genesis() -> Storage {
	let genesis_config = asset_hub_polkadot_runtime::RuntimeGenesisConfig {
		system: asset_hub_polkadot_runtime::SystemConfig::default(),
		balances: asset_hub_polkadot_runtime::BalancesConfig {
			balances: accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096 * 4096))
				.collect(),
		},
		parachain_info: asset_hub_polkadot_runtime::ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
			..Default::default()
		},
		collator_selection: asset_hub_polkadot_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: asset_hub_polkadot_runtime::SessionConfig {
			keys: invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                      // account id
						acc,                                              // validator id
						asset_hub_polkadot_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		polkadot_xcm: asset_hub_polkadot_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(
		&genesis_config,
		asset_hub_polkadot_runtime::WASM_BINARY
			.expect("WASM binary was not built, please build it!"),
	)
}

type AuraId = sp_consensus_aura::ed25519::AuthorityId;
pub fn invulnerables() -> Vec<(AccountId, AuraId)> {
	vec![
		(get_account_id_from_seed::<sr25519::Public>("Alice"), get_from_seed::<AuraId>("Alice")),
		(get_account_id_from_seed::<sr25519::Public>("Bob"), get_from_seed::<AuraId>("Bob")),
	]
}

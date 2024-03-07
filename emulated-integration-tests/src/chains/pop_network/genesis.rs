// Substrate
use sp_core::storage::Storage;

// Cumulus
use emulated_integration_tests_common::{
    accounts, build_genesis_storage, collators, SAFE_XCM_VERSION,
};
use parachains_common::Balance;

pub const PARA_ID: u32 = 909;
pub const ED: Balance = rococo_runtime_constants::currency::EXISTENTIAL_DEPOSIT / 10;

pub fn genesis() -> Storage {
    let genesis_config = pop_runtime::RuntimeGenesisConfig {
        system: pop_runtime::SystemConfig::default(),
        balances: pop_runtime::BalancesConfig {
            balances: accounts::init_balances()
                .iter()
                .cloned()
                .map(|k| (k, ED * 4096 * 4096))
                .collect(),
        },
        parachain_info: pop_runtime::ParachainInfoConfig {
            parachain_id: PARA_ID.into(),
            ..Default::default()
        },
        collator_selection: pop_runtime::CollatorSelectionConfig {
            invulnerables: collators::invulnerables()
                .iter()
                .cloned()
                .map(|(acc, _)| acc)
                .collect(),
            candidacy_bond: ED * 16,
            ..Default::default()
        },
        session: pop_runtime::SessionConfig {
            keys: collators::invulnerables()
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                       // account id
                        acc,                               // validator id
                        pop_runtime::SessionKeys { aura }, // session keys
                    )
                })
                .collect(),
        },
        polkadot_xcm: pop_runtime::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
            ..Default::default()
        },
        ..Default::default()
    };

    build_genesis_storage(
        &genesis_config,
        pop_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
    )
}

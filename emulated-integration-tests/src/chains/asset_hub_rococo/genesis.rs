use emulated_integration_tests_common::{
    accounts, build_genesis_storage, collators, SAFE_XCM_VERSION,
};
use pop_runtime::Balance;
use sp_core::storage::Storage;

pub(crate) const PARA_ID: u32 = 1000;
pub(crate) const ED: Balance = rococo_runtime_constants::currency::EXISTENTIAL_DEPOSIT / 10;

pub(crate) fn genesis() -> Storage {
    let genesis_config = asset_hub_rococo_runtime::RuntimeGenesisConfig {
        system: asset_hub_rococo_runtime::SystemConfig::default(),
        balances: asset_hub_rococo_runtime::BalancesConfig {
            balances: accounts::init_balances()
                .iter()
                .cloned()
                .map(|k| (k, ED * 4096 * 4096))
                .collect(),
        },
        parachain_info: asset_hub_rococo_runtime::ParachainInfoConfig {
            parachain_id: PARA_ID.into(),
            ..Default::default()
        },
        collator_selection: asset_hub_rococo_runtime::CollatorSelectionConfig {
            invulnerables: collators::invulnerables()
                .iter()
                .cloned()
                .map(|(acc, _)| acc)
                .collect(),
            candidacy_bond: ED * 16,
            ..Default::default()
        },
        session: asset_hub_rococo_runtime::SessionConfig {
            keys: collators::invulnerables()
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                                    // account id
                        acc,                                            // validator id
                        asset_hub_rococo_runtime::SessionKeys { aura }, // session keys
                    )
                })
                .collect(),
        },
        polkadot_xcm: asset_hub_rococo_runtime::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
            ..Default::default()
        },
        ..Default::default()
    };

    build_genesis_storage(
        &genesis_config,
        asset_hub_rococo_runtime::WASM_BINARY.expect("WASM binary was not built, please build it!"),
    )
}

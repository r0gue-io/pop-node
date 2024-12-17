use cumulus_primitives_core::relay_chain::Balance;
use emulated_integration_tests_common::{
	accounts, build_genesis_storage, get_from_seed, get_host_config, validators,
};
use polkadot_primitives::{AssignmentId, ValidatorId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::ecdsa_crypto::AuthorityId as BeefyId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::storage::Storage;
use westend_runtime_constants::currency::UNITS as WND;

pub(crate) const ED: Balance = westend_runtime_constants::currency::EXISTENTIAL_DEPOSIT;
const ENDOWMENT: u128 = 1_000_000 * WND;

pub(crate) fn genesis() -> Storage {
	let genesis_config = westend_runtime::RuntimeGenesisConfig {
		system: westend_runtime::SystemConfig::default(),
		balances: westend_runtime::BalancesConfig {
			balances: accounts::init_balances().iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
		},
		session: westend_runtime::SessionConfig {
			keys: validators::initial_authorities()
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
							get_from_seed::<BeefyId>("Alice"),
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		babe: westend_runtime::BabeConfig {
			authorities: Default::default(),
			epoch_config: westend_runtime::BABE_GENESIS_EPOCH_CONFIG,
			..Default::default()
		},
		configuration: westend_runtime::ConfigurationConfig { config: get_host_config() },
		registrar: westend_runtime::RegistrarConfig {
			next_free_para_id: polkadot_primitives::LOWEST_PUBLIC_ID,
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(&genesis_config, westend_runtime::WASM_BINARY.unwrap())
}

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
	beefy: BeefyId,
) -> westend_runtime::SessionKeys {
	westend_runtime::SessionKeys {
		babe,
		grandpa,
		para_validator,
		para_assignment,
		authority_discovery,
		beefy,
	}
}

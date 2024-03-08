use emulated_integration_tests_common::{
	accounts, build_genesis_storage, get_account_id_from_seed, get_from_seed, get_host_config,
	validators,
};
use polkadot_primitives::{AssignmentId, ValidatorId};
use pop_runtime::Balance;
use coretime_rococo_runtime_constants::currency::UNITS as ROC;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::ecdsa_crypto::AuthorityId as BeefyId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, storage::Storage};

pub(crate) const ED: Balance = coretime_rococo_runtime_constants::currency::EXISTENTIAL_DEPOSIT;
const ENDOWMENT: u128 = 1_000_000 * ROC;

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
	beefy: BeefyId,
) -> coretime_rococo_runtime::SessionKeys {
	coretime_rococo_runtime::SessionKeys {
		babe,
		grandpa,
		para_validator,
		para_assignment,
		authority_discovery,
		beefy,
	}
}

pub(crate) fn genesis() -> Storage {
	let genesis_config = coretime_rococo_runtime::RuntimeGenesisConfig {
		system: coretime_rococo_runtime::SystemConfig::default(),
		balances: coretime_rococo_runtime::BalancesConfig {
			balances: accounts::init_balances().iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
		},
		session: coretime_rococo_runtime::SessionConfig {
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
		babe: coretime_rococo_runtime::BabeConfig {
			authorities: Default::default(),
			epoch_config: Some(coretime_rococo_runtime::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		sudo: coretime_rococo_runtime::SudoConfig {
			key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
		},
		configuration: coretime_rococo_runtime::ConfigurationConfig { config: get_host_config() },
		registrar: coretime_rococo_runtime::RegistrarConfig {
			next_free_para_id: polkadot_primitives::LOWEST_PUBLIC_ID,
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(&genesis_config, coretime_rococo_runtime::WASM_BINARY.unwrap())
}

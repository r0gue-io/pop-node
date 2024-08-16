use emulated_integration_tests_common::{
	accounts, build_genesis_storage, get_account_id_from_seed, get_from_seed, get_host_config,
	validators,
};
use polkadot_primitives::{AssignmentId, Balance, ValidatorId};
use polkadot_runtime_constants::currency::UNITS as PAS;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::ecdsa_crypto::AuthorityId as BeefyId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, storage::Storage};

pub(crate) const ED: Balance = polkadot_runtime_constants::currency::EXISTENTIAL_DEPOSIT;
const ENDOWMENT: u128 = 1_000_000 * PAS;

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
	beefy: BeefyId,
) -> polkadot_runtime::SessionKeys {
	polkadot_runtime::SessionKeys {
		babe,
		grandpa,
		para_validator,
		para_assignment,
		authority_discovery,
		beefy,
	}
}

pub(crate) fn genesis() -> Storage {
	let genesis_config = polkadot_runtime::RuntimeGenesisConfig {
		system: polkadot_runtime::SystemConfig::default(),
		balances: polkadot_runtime::BalancesConfig {
			balances: accounts::init_balances().iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
		},
		session: polkadot_runtime::SessionConfig {
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
		babe: polkadot_runtime::BabeConfig {
			authorities: Default::default(),
			epoch_config: Some(polkadot_runtime::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		sudo: polkadot_runtime::SudoConfig {
			key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
		},
		configuration: polkadot_runtime::ConfigurationConfig { config: get_host_config() },
		registrar: polkadot_runtime::RegistrarConfig {
			next_free_para_id: polkadot_primitives::LOWEST_PUBLIC_ID,
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(&genesis_config, polkadot_runtime::WASM_BINARY.unwrap())
}

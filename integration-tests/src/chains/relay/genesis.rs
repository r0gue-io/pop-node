use emulated_integration_tests_common::{
	accounts, build_genesis_storage, get_from_seed, get_host_config, validators,
};
use polkadot_primitives::{AssignmentId, Balance, ValidatorId, LOWEST_PUBLIC_ID};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::ecdsa_crypto::AuthorityId as BeefyId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::storage::Storage;

use crate::chains::relay::{
	constants::currency::{EXISTENTIAL_DEPOSIT, UNITS as PAS},
	runtime::{
		BabeConfig, BalancesConfig, ConfigurationConfig, RegistrarConfig, RuntimeGenesisConfig,
		SessionConfig, SessionKeys, SystemConfig, BABE_GENESIS_EPOCH_CONFIG, WASM_BINARY,
	},
};

pub(crate) const ED: Balance = EXISTENTIAL_DEPOSIT;
const ENDOWMENT: u128 = 1_000_000 * PAS;

pub(crate) fn genesis() -> Storage {
	let genesis_config = RuntimeGenesisConfig {
		system: SystemConfig::default(),
		balances: BalancesConfig {
			balances: accounts::init_balances().iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
		},
		session: SessionConfig {
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
		babe: BabeConfig {
			authorities: Default::default(),
			epoch_config: BABE_GENESIS_EPOCH_CONFIG,
			..Default::default()
		},
		configuration: ConfigurationConfig { config: get_host_config() },
		registrar: RegistrarConfig { next_free_para_id: LOWEST_PUBLIC_ID, ..Default::default() },
		..Default::default()
	};

	build_genesis_storage(&genesis_config, WASM_BINARY.unwrap())
}

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
	beefy: BeefyId,
) -> SessionKeys {
	SessionKeys { babe, grandpa, para_validator, para_assignment, authority_discovery, beefy }
}

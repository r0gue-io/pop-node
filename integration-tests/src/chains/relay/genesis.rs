use emulated_integration_tests_common::{
	accounts, build_genesis_storage, get_host_config, validators,
};
use polkadot_primitives::{AssignmentId, Balance, ValidatorId, LOWEST_PUBLIC_ID};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::{ecdsa_crypto::AuthorityId as BeefyId, test_utils::Keyring};
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::storage::Storage;

use crate::chains::relay::{
	constants::currency::{EXISTENTIAL_DEPOSIT, UNITS},
	runtime::{
		BabeConfig, BalancesConfig, ConfigurationConfig, RegistrarConfig, RuntimeGenesisConfig,
		SessionConfig, SessionKeys, SystemConfig, BABE_GENESIS_EPOCH_CONFIG, WASM_BINARY,
	},
};

pub(crate) const ED: Balance = EXISTENTIAL_DEPOSIT;
const ENDOWMENT: u128 = 1_000_000 * UNITS;

pub(crate) fn genesis() -> Storage {
	let genesis_config = RuntimeGenesisConfig {
		system: SystemConfig::default(),
		balances: BalancesConfig {
			balances: accounts::init_balances().iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
			..Default::default()
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
							Keyring::Alice.into(),
						),
					)
				})
				.collect::<Vec<_>>(),
			..Default::default()
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

// In case we want to add Polkadot in the future, the following change in `invulnerables` is
// required.
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

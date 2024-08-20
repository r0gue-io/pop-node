// TODO: using polkadot as stopgap until paseo updated to polkadot sdk v1.14.0
use polkadot_runtime as paseo_runtime;
use polkadot_runtime_constants as paseo_runtime_constants;

use emulated_integration_tests_common::{
	accounts, build_genesis_storage, get_from_seed, get_host_config, validators,
};
use paseo_runtime_constants::currency::UNITS as PAS;
use polkadot_primitives::{AssignmentId, Balance, ValidatorId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::ecdsa_crypto::AuthorityId as BeefyId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::storage::Storage;

pub(crate) const ED: Balance = paseo_runtime_constants::currency::EXISTENTIAL_DEPOSIT;
const ENDOWMENT: u128 = 1_000_000 * PAS;

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
	beefy: BeefyId,
) -> paseo_runtime::SessionKeys {
	paseo_runtime::SessionKeys {
		babe,
		grandpa,
		para_validator,
		para_assignment,
		authority_discovery,
		beefy,
	}
}

pub(crate) fn genesis() -> Storage {
	let genesis_config = paseo_runtime::RuntimeGenesisConfig {
		system: paseo_runtime::SystemConfig::default(),
		balances: paseo_runtime::BalancesConfig {
			balances: accounts::init_balances().iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
		},
		session: paseo_runtime::SessionConfig {
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
		babe: paseo_runtime::BabeConfig {
			authorities: Default::default(),
			epoch_config: paseo_runtime::BABE_GENESIS_EPOCH_CONFIG,
			..Default::default()
		},
		// TODO: sudo pallet is not configured in polkadot runtime
		// sudo: runtime::SudoConfig {
		// 	key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
		// },
		configuration: paseo_runtime::ConfigurationConfig { config: get_host_config() },
		registrar: paseo_runtime::RegistrarConfig {
			next_free_para_id: polkadot_primitives::LOWEST_PUBLIC_ID,
			..Default::default()
		},
		..Default::default()
	};

	build_genesis_storage(&genesis_config, paseo_runtime::WASM_BINARY.unwrap())
}

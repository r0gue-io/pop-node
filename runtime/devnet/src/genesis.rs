use alloc::vec::Vec;

use cumulus_primitives_core::ParaId;
use ismp_parachain::ParachainData;
use parachains_common::{AccountId, AuraId, Balance};
use pop_runtime_common::genesis::*;
use sp_genesis_builder::PresetId;

use crate::{BalancesConfig, IsmpParachainConfig, SessionKeys, EXISTENTIAL_DEPOSIT, UNIT};

/// A development chain running on a single node, using the `devnet` runtime.
pub const DEVNET_DEV: &str = "pop-devnet-dev";
/// Configures a local chain running on multiple nodes for testing purposes, using the `devnet`
/// runtime.
pub const DEVNET_LOCAL: &str = "pop-devnet-local";
/// A live chain running on multiple nodes on private devnet, using the `devnet` runtime.
pub const DEVNET: &str = "pop-devnet";
/// The available genesis config presets;
const PRESETS: [&str; 3] = [DEVNET_DEV, DEVNET_LOCAL, DEVNET];

/// The parachain identifier to set in genesis config.
pub const PARA_ID: ParaId = ParaId::new(4_001);
/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Initial balance for genesis endowed accounts.
const ENDOWMENT: Balance = 10_000_000 * UNIT;

/// Returns a JSON blob representation of the built-in `RuntimeGenesisConfig` identified by `id`.
pub(crate) fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_str() {
		DEVNET_DEV => development_config(),
		DEVNET_LOCAL => local_config(),
		DEVNET => live_config(),
		_ => return None,
	};
	Some(
		to_string(&patch)
			.expect("serialization to json is expected to work. qed.")
			.into_bytes(),
	)
}

/// Returns a list of identifiers for available builtin `RuntimeGenesisConfig` presets.
pub(crate) fn presets() -> Vec<PresetId> {
	PRESETS.map(PresetId::from).to_vec()
}

/// Configures a development chain running on a single node, using the `devnet` runtime.
fn development_config() -> Value {
	genesis(
		// Initial collators.
		Vec::from([
			// Single collator for development chain
			(Keyring::Alice.to_account_id(), Keyring::Alice.public().into()),
		]),
		dev_accounts(),
		Keyring::Alice.to_account_id(),
		PARA_ID,
		Vec::from([ParachainData { id: 1000, slot_duration: 6000 }]),
	)
}

/// Configures a local chain running on multiple nodes for testing purposes, using the `devnet`
/// runtime.
fn local_config() -> Value {
	genesis(
		// Initial collators.
		Vec::from([
			// Multiple collators for local development chain.
			(Keyring::Alice.to_account_id(), Keyring::Alice.public().into()),
			(Keyring::Bob.to_account_id(), Keyring::Bob.public().into()),
		]),
		dev_accounts(),
		Keyring::Alice.to_account_id(),
		PARA_ID,
		Vec::from([ParachainData { id: 1000, slot_duration: 6000 }]),
	)
}

/// Configures a live chain running on multiple nodes on private devnet, using the `devnet` runtime.
fn live_config() -> Value {
	genesis(
		// Initial collators.
		Vec::from([
			// Multiple collators for live development chain.
			(Keyring::Alice.to_account_id(), Keyring::Alice.public().into()),
			(Keyring::Bob.to_account_id(), Keyring::Bob.public().into()),
			(Keyring::Charlie.to_account_id(), Keyring::Charlie.public().into()),
		]),
		dev_accounts(),
		Keyring::Alice.to_account_id(),
		PARA_ID,
		Vec::from([ParachainData { id: 1000, slot_duration: 6000 }]),
	)
}

#[allow(clippy::too_many_arguments)]
fn genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	sudo_key: AccountId,
	id: ParaId,
	ismp_parachains: Vec<ParachainData>,
) -> Value {
	json!({
		"balances": BalancesConfig { balances: balances(endowed_accounts) },
		"parachainInfo": { "parachainId": id },
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16,
		},
		"session": {
			"keys": invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),        // account id
						acc,               	// validator id
						SessionKeys { aura},// session keys
					)
				})
				.collect::<Vec<_>>(),
		},
		"sudo" : { "key" : sudo_key },
		"polkadotXcm": { "safeXcmVersion": Some(SAFE_XCM_VERSION) },
		// The following parachains are tracked via ISMP.
		"ismpParachain": IsmpParachainConfig {
			parachains: ismp_parachains,
			..Default::default()
		},
	})
}

// The initial balances at genesis.
fn balances(endowed_accounts: Vec<AccountId>) -> Vec<(AccountId, Balance)> {
	let balances = endowed_accounts.iter().cloned().map(|k| (k, ENDOWMENT)).collect::<Vec<_>>();
	balances
}

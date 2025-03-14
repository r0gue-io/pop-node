use alloc::{vec, vec::Vec};

use cumulus_primitives_core::ParaId;
use parachains_common::{AccountId, AuraId, Balance};
use pop_runtime_common::genesis::*;
use sp_core::crypto::Ss58Codec;
use sp_genesis_builder::PresetId;

use crate::{
	config::governance::SudoAddress, AssetsConfig, BalancesConfig, SessionKeys,
	EXISTENTIAL_DEPOSIT, UNIT,
};

/// A development chain running on a single node, using the `testnet` runtime.
pub const TESTNET_DEV: &str = "pop-testnet-dev";
/// Configures a local chain running on multiple nodes for testing purposes, using the `testnet`
/// runtime.
pub const TESTNET_LOCAL: &str = "pop-testnet-local";
/// A live chain running on multiple nodes, using the `testnet` runtime.
pub const TESTNET: &str = "pop-testnet";
/// The available genesis config presets;
const PRESETS: [&str; 3] = [TESTNET_DEV, TESTNET_LOCAL, TESTNET];

/// The parachain identifier to set in genesis config.
pub const PARA_ID: ParaId = ParaId::new(4_001);

/// Initial balance for genesis endowed accounts.
const ENDOWMENT: Balance = 10_000_000 * UNIT;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Returns a JSON blob representation of the built-in `RuntimeGenesisConfig` identified by `id`.
pub(crate) fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_str() {
		TESTNET_DEV => development_config(),
		TESTNET_LOCAL => local_config(),
		TESTNET => live_config(),
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

/// Configures a development chain running on a single node, using the `testnet` runtime.
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
	)
}

/// Configures a local chain running on multiple nodes for testing purposes, using the `testnet`
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
	)
}

/// Configures a live chain running on multiple nodes on private testnet, using the `testnet`
/// runtime.
fn live_config() -> Value {
	let collator_0_account_id: AccountId =
		AccountId::from_ss58check("5Gn9dVgCNUYtC5JVMBheQQv2x6Lpg5sAMcQVRupG1s3tP2gR").unwrap();
	let collator_0_aura_id: AuraId =
		AuraId::from_ss58check("5Gn9dVgCNUYtC5JVMBheQQv2x6Lpg5sAMcQVRupG1s3tP2gR").unwrap();
	let collator_1_account_id: AccountId =
		AccountId::from_ss58check("5FyVvcSvSXCkBwvBEHkUh1VWGGrwaR3zbYBkU3Rc5DqV75S4").unwrap();
	let collator_1_aura_id: AuraId =
		AuraId::from_ss58check("5FyVvcSvSXCkBwvBEHkUh1VWGGrwaR3zbYBkU3Rc5DqV75S4").unwrap();
	let collator_2_account_id: AccountId =
		AccountId::from_ss58check("5GMqrQuWpyyBBK7LAWXR5psWvKc1QMqtiyasjp23VNKZWgh6").unwrap();
	let collator_2_aura_id: AuraId =
		AuraId::from_ss58check("5GMqrQuWpyyBBK7LAWXR5psWvKc1QMqtiyasjp23VNKZWgh6").unwrap();

	genesis(
		// Initial collators.
		vec![
			// POP COLLATOR 0
			(collator_0_account_id, collator_0_aura_id),
			// POP COLLATOR 1
			(collator_1_account_id, collator_1_aura_id),
			// POP COLLATOR 2
			(collator_2_account_id, collator_2_aura_id),
		],
		vec![],
		SudoAddress::get(),
		PARA_ID,
	)
}

#[allow(clippy::too_many_arguments)]
fn genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	sudo_key: AccountId,
	id: ParaId,
) -> Value {
	json!({
		"assets": AssetsConfig {
			// Genesis assets: Vec<(id, owner, is_sufficient, min_balance)>
			assets: Vec::from([
				(0, sudo_key.clone(), false, EXISTENTIAL_DEPOSIT),	// Relay native asset from Asset Hub
			]),
			// Genesis metadata: Vec<(id, name, symbol, decimals)>
			metadata: Vec::from([
				(0, "Paseo".into(), "PAS".into(), 10),
			]),
			next_asset_id: Some(1),
			..Default::default()
		},
		"balances": BalancesConfig { balances: balances(endowed_accounts) },
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16,
		},
		"parachainInfo": { "parachainId": id },
		"polkadotXcm": { "safeXcmVersion": Some(SAFE_XCM_VERSION) },
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
	})
}

// The initial balances at genesis.
fn balances(endowed_accounts: Vec<AccountId>) -> Vec<(AccountId, Balance)> {
	let balances = endowed_accounts.iter().cloned().map(|k| (k, ENDOWMENT)).collect::<Vec<_>>();
	balances
}

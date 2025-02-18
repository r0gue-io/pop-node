use alloc::{vec, vec::Vec};

use cumulus_primitives_core::ParaId;
use parachains_common::{AccountId, AuraId, Balance};
use pop_runtime_common::genesis::*;
use sp_core::crypto::Ss58Codec;
use sp_genesis_builder::PresetId;

use crate::{
	config::{governance::SudoAddress, monetary::ExistentialDeposit},
	AssetsConfig, BalancesConfig, CouncilConfig, SessionKeys, EXISTENTIAL_DEPOSIT, UNIT,
};

/// A development chain running on a single node, using the `mainnet` runtime.
pub const MAINNET_DEV: &str = "pop-dev";
/// Configures a local chain running on multiple nodes for testing purposes, using the `mainnet`
/// runtime.
pub const MAINNET_LOCAL: &str = "pop-local";
/// A live chain running on multiple nodes, using the `mainnet` runtime.
pub const MAINNET: &str = "pop";
/// The available genesis config presets;
const PRESETS: [&str; 3] = [MAINNET_DEV, MAINNET_LOCAL, MAINNET];

/// The parachain identifier to set in genesis config.
pub const PARA_ID: ParaId = ParaId::new(3395);

/// Initial balance for genesis endowed accounts.
const ENDOWMENT: Balance = 10_000_000 * UNIT;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Struct used to declare assets that will be included at genesis.
struct GenesisAsset {
	id: u32,
	owner: AccountId,
	is_sufficient: bool,
	min_balance: Balance,
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
}

/// Returns a JSON blob representation of the built-in `RuntimeGenesisConfig` identified by `id`.
pub(crate) fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
	let patch = match id.as_str() {
		MAINNET_DEV => development_config(),
		MAINNET_LOCAL => local_config(),
		MAINNET => live_config(),
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

/// Configures a development chain running on a single node, using the `mainnet` runtime.
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
		// AssetId reserved for DOT from AH.
		vec![GenesisAsset {
			id: 0,
			owner: asset_hub_sa_on_pop(),
			is_sufficient: false,
			min_balance: ExistentialDeposit::get(),
			name: "DOT".into(),
			symbol: "DOT".into(),
			decimals: 10,
		}],
		vec![
			Keyring::Alice.to_account_id(),
			Keyring::Bob.to_account_id(),
			Keyring::Charlie.to_account_id(),
			Keyring::Dave.to_account_id(),
			Keyring::Eve.to_account_id(),
		],
	)
}

/// Configures a local chain running on multiple nodes for testing purposes, using the `mainnet`
/// runtime.
fn local_config() -> Value {
	let mut endowed_accounts = dev_accounts();
	endowed_accounts.push(SudoAddress::get());

	genesis(
		// Initial collators.
		Vec::from([
			// Multiple collators for local development chain.
			(Keyring::Alice.to_account_id(), Keyring::Alice.public().into()),
			(Keyring::Bob.to_account_id(), Keyring::Bob.public().into()),
		]),
		endowed_accounts,
		SudoAddress::get(),
		PARA_ID,
		// AssetId reserved for DOT from AH.
		vec![GenesisAsset {
			id: 0,
			owner: asset_hub_sa_on_pop(),
			is_sufficient: false,
			min_balance: ExistentialDeposit::get(),
			name: "DOT".into(),
			symbol: "DOT".into(),
			decimals: 10,
		}],
		vec![
			AccountId::from_ss58check("142zako1kfvrpQ7pJKYR8iGUD58i4wjb78FUsmJ9WcXmkM5z").unwrap(),
			AccountId::from_ss58check("15VPagCVayS6XvT5RogPYop3BJTJzwqR2mCGR1kVn3w58ygg").unwrap(),
			AccountId::from_ss58check("14G3CUFnZUBnHZUhahexSZ6AgemaW9zMHBnGccy3df7actf4").unwrap(),
			AccountId::from_ss58check("15k9niqckMg338cFBoz9vWFGwnCtwPBquKvqJEfHApijZkDz").unwrap(),
			AccountId::from_ss58check("13BL7T6bTgeEdfEdZqLCKJZPN8ncyFNxxHRKFb2YMATvyfH4").unwrap(),
		],
	)
}

/// Configures a live chain running on multiple nodes on private mainnet, using the `mainnet`
/// runtime.
fn live_config() -> Value {
	let collator_0_account_id: AccountId =
		AccountId::from_ss58check("15B6eUkXgoLA3dWruCRYWeBGNC8SCwuqiMtMTM1Zh2auSg3w").unwrap();
	let collator_0_aura_id: AuraId =
		AuraId::from_ss58check("15B6eUkXgoLA3dWruCRYWeBGNC8SCwuqiMtMTM1Zh2auSg3w").unwrap();

	genesis(
		// Initial collators.
		vec![
			// POP COLLATOR 0
			(collator_0_account_id, collator_0_aura_id),
		],
		vec![],
		SudoAddress::get(),
		PARA_ID,
		vec![],
		vec![],
	)
}

#[allow(clippy::too_many_arguments)]
fn genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	sudo_key: AccountId,
	id: ParaId,
	genesis_assets: Vec<GenesisAsset>,
	council_members: Vec<AccountId>,
) -> Value {
	// Collect genesis assets.
	// Genesis assets: Vec<(id, owner, is_sufficient, min_balance)>
	let mut assets: Vec<(u32, AccountId, bool, Balance)> = Vec::new();
	// Genesis metadata: Vec<(id, name, symbol, decimals)>
	let mut assets_metadata: Vec<(u32, Vec<u8>, Vec<u8>, u8)> = Vec::new();
	genesis_assets.iter().for_each(|asset| {
		assets.push((asset.id, asset.owner.clone(), asset.is_sufficient, asset.min_balance));
		assets_metadata.push((asset.id, asset.name.clone(), asset.symbol.clone(), asset.decimals));
	});

	json!({
		"assets": AssetsConfig {
			assets,
			metadata: assets_metadata,
			..Default::default()
		},
		"balances": BalancesConfig { balances: balances(endowed_accounts) },
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16,
		},
		"council": CouncilConfig {
			members: council_members,
			..Default::default()
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ensure_sudo_account() {
		assert_eq!(
			derive_multisig(
				vec![
					AccountId::from_ss58check("15VPagCVayS6XvT5RogPYop3BJTJzwqR2mCGR1kVn3w58ygg")
						.unwrap(),
					AccountId::from_ss58check("142zako1kfvrpQ7pJKYR8iGUD58i4wjb78FUsmJ9WcXmkM5z")
						.unwrap(),
					AccountId::from_ss58check("15k9niqckMg338cFBoz9vWFGwnCtwPBquKvqJEfHApijZkDz")
						.unwrap(),
					AccountId::from_ss58check("14G3CUFnZUBnHZUhahexSZ6AgemaW9zMHBnGccy3df7actf4")
						.unwrap(),
				],
				2
			),
			SudoAddress::get()
		)
	}
}

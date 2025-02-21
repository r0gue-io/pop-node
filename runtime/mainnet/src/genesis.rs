use alloc::{vec, vec::Vec};

use cumulus_primitives_core::ParaId;
use parachains_common::{AccountId, AuraId, Balance};
use pop_runtime_common::genesis::*;
use sp_core::crypto::Ss58Codec;
use sp_genesis_builder::PresetId;
use sp_runtime::traits::AccountIdConversion;

use crate::{
	config::{
		collation::PotId,
		governance::SudoAddress,
		monetary::{ExistentialDeposit, MaintenanceAccount, TreasuryAccount},
	},
	AssetsConfig, BalancesConfig, CouncilConfig, Runtime, SessionKeys, EXISTENTIAL_DEPOSIT, UNIT,
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

/// Initial balance for genesis endowed accounts. Used for local testing only.
const ENDOWMENT: Balance = 10_000_000 * UNIT;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

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
	let mut endowed_accounts = dev_accounts();
	endowed_accounts.push(MaintenanceAccount::get());
	endowed_accounts.push(PotId::get().into_account_truncating());
	endowed_accounts.push(TreasuryAccount::get());

	genesis(
		// Initial collators.
		Vec::from([
			// Single collator for development chain
			(Keyring::Alice.to_account_id(), Keyring::Alice.public().into()),
		]),
		endowed_accounts,
		Keyring::Alice.to_account_id(),
		PARA_ID,
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
	// Like the multisig used for live config, but with dev accounts.
	let sudo_account = derive_multisig::<Runtime>(
		vec![
			Keyring::Alice.to_account_id(),
			Keyring::Bob.to_account_id(),
			Keyring::Charlie.to_account_id(),
			Keyring::Dave.to_account_id(),
		],
		2,
	);

	let mut endowed_accounts = dev_accounts();
	endowed_accounts.push(MaintenanceAccount::get());
	endowed_accounts.push(PotId::get().into_account_truncating());
	endowed_accounts.push(sudo_account.clone());
	endowed_accounts.push(TreasuryAccount::get());

	genesis(
		// Initial collators.
		Vec::from([
			// Multiple collators for local development chain.
			(Keyring::Alice.to_account_id(), Keyring::Alice.public().into()),
			(Keyring::Bob.to_account_id(), Keyring::Bob.public().into()),
		]),
		endowed_accounts,
		sudo_account,
		PARA_ID,
		vec![
			Keyring::Alice.to_account_id(),
			Keyring::Bob.to_account_id(),
			Keyring::Charlie.to_account_id(),
			Keyring::Dave.to_account_id(),
			Keyring::Eve.to_account_id(),
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
	)
}

#[allow(clippy::too_many_arguments)]
fn genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	sudo_key: AccountId,
	id: ParaId,
	council_members: Vec<AccountId>,
) -> Value {
	json!({
		"assets": AssetsConfig {
			assets: vec![],
			metadata: vec![],
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

// The initial balances at genesis; Used for local testing only.
fn balances(endowed_accounts: Vec<AccountId>) -> Vec<(AccountId, Balance)> {
	let balances = endowed_accounts
		.iter()
		.cloned()
		.map(|k| {
			// Well known keys get an amount equal to `ENDOWMENT`.
			// Other keys are funded with ED only.
			if dev_accounts().contains(&k) {
				(k, ENDOWMENT)
			} else {
				(k, ExistentialDeposit::get())
			}
		})
		.collect::<Vec<_>>();
	balances
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ensure_sudo_multisig_account() {
		assert_eq!(
			derive_multisig::<Runtime>(
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

	mod development_config {
		use super::*;

		#[test]
		fn ensure_initial_collators() {
			let genesis = development_config();

			let invulnerables = genesis["collatorSelection"]["invulnerables"]
				.as_array()
				.unwrap()
				.iter()
				.map(|a| a.as_str().unwrap().to_string())
				.collect::<Vec<_>>();
			assert!(invulnerables.contains(&Keyring::Alice.to_account_id().to_string()));

			let session_keys = genesis["session"]["keys"]
				.as_array()
				.unwrap()
				.iter()
				.map(|k| {
					let key = k.as_array().unwrap();
					let session_key = key[0].as_str().unwrap().to_string();
					session_key
				})
				.collect::<Vec<_>>();
			assert!(session_keys.contains(&Keyring::Alice.to_account_id().to_string()));
		}

		#[test]
		fn endows_given_accounts() {
			let mut endowed_accounts = dev_accounts();
			endowed_accounts.push(MaintenanceAccount::get());
			endowed_accounts.push(PotId::get().into_account_truncating());
			endowed_accounts.push(TreasuryAccount::get());

			let genesis = development_config();

			let balances: Vec<_> = genesis["balances"]["balances"]
				.as_array()
				.unwrap()
				.iter()
				.map(|e| {
					let endowment = e.as_array().unwrap();
					let account = endowment[0].as_str().unwrap().to_string();
					let balance = endowment[1].as_number().unwrap();
					(account, balance)
				})
				.collect();

			let accounts_in_balances_state: Vec<_> =
				balances.into_iter().map(|(account, _)| account).collect();
			assert!(endowed_accounts
				.iter()
				.all(|s| accounts_in_balances_state.contains(&s.to_string())));
		}

		#[test]
		fn ensure_correct_sudo_key() {
			let genesis = development_config();

			let sudo_key = genesis["sudo"]["key"].as_str().unwrap();
			assert_eq!(sudo_key, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
			assert_eq!(
				AccountId::from_ss58check(sudo_key).unwrap(),
				Keyring::Alice.to_account_id()
			);
		}

		#[test]
		fn ensure_correct_para_id() {
			let genesis = development_config();

			let para_id = genesis["parachainInfo"]["parachainId"].as_u64().unwrap();
			assert_eq!(para_id, 3395);
		}

		#[test]
		fn ensure_genesis_assets_are_empty() {
			let genesis = development_config();

			let assets = genesis["assets"]["assets"].as_array().unwrap();
			assert!(assets.is_empty());
		}

		#[test]
		fn ensure_council_members() {
			let council_members = vec![
				Keyring::Alice.to_account_id(),
				Keyring::Bob.to_account_id(),
				Keyring::Charlie.to_account_id(),
				Keyring::Dave.to_account_id(),
				Keyring::Eve.to_account_id(),
			];

			let genesis = development_config();

			let council: Vec<_> = genesis["council"]["members"]
				.as_array()
				.unwrap()
				.iter()
				.map(|e| {
					let member = e.as_str().unwrap().to_string();
					member
				})
				.collect();
			assert!(council_members.iter().all(|s| council.contains(&s.to_string())));
		}
	}

	mod local_config {
		use super::*;

		#[test]
		fn ensure_initial_collators() {
			let initial_collators =
				vec![Keyring::Alice.to_account_id(), Keyring::Bob.to_account_id()];

			let genesis = local_config();

			let invulnerables = genesis["collatorSelection"]["invulnerables"]
				.as_array()
				.unwrap()
				.iter()
				.map(|a| a.as_str().unwrap().to_string())
				.collect::<Vec<_>>();
			assert!(initial_collators.iter().all(|c| invulnerables.contains(&c.to_string())));

			let session_keys = genesis["session"]["keys"]
				.as_array()
				.unwrap()
				.iter()
				.map(|k| {
					let key = k.as_array().unwrap();
					let session_key = key[0].as_str().unwrap().to_string();
					session_key
				})
				.collect::<Vec<_>>();
			assert!(initial_collators.iter().all(|c| session_keys.contains(&c.to_string())));
		}

		#[test]
		fn endows_given_accounts() {
			let sudo_account = derive_multisig::<Runtime>(
				vec![
					Keyring::Alice.to_account_id(),
					Keyring::Bob.to_account_id(),
					Keyring::Charlie.to_account_id(),
					Keyring::Dave.to_account_id(),
				],
				2,
			);

			let mut endowed_accounts = dev_accounts();
			endowed_accounts.push(MaintenanceAccount::get());
			endowed_accounts.push(PotId::get().into_account_truncating());
			endowed_accounts.push(sudo_account);
			endowed_accounts.push(TreasuryAccount::get());

			let genesis = local_config();

			let balances: Vec<_> = genesis["balances"]["balances"]
				.as_array()
				.unwrap()
				.iter()
				.map(|e| {
					let endowment = e.as_array().unwrap();
					let account = endowment[0].as_str().unwrap().to_string();
					let balance = endowment[1].as_number().unwrap();
					(account, balance)
				})
				.collect();

			let accounts_in_balances_state: Vec<_> =
				balances.into_iter().map(|(account, _)| account).collect();
			assert!(endowed_accounts
				.iter()
				.all(|s| accounts_in_balances_state.contains(&s.to_string())));
		}

		#[test]
		fn ensure_correct_sudo_key() {
			assert_eq!(
				"5H9WyMRtMWqkUggSQun4jiDdbzYbsNQhLDH7KooXaihMC7Tp",
				derive_multisig::<Runtime>(
					vec![
						Keyring::Alice.to_account_id(),
						Keyring::Bob.to_account_id(),
						Keyring::Charlie.to_account_id(),
						Keyring::Dave.to_account_id(),
					],
					2
				)
				.to_ss58check()
			);

			let genesis = local_config();

			let sudo_key = genesis["sudo"]["key"].as_str().unwrap();
			assert_eq!(
				AccountId::from_ss58check(sudo_key).unwrap(),
				derive_multisig::<Runtime>(
					vec![
						Keyring::Alice.to_account_id(),
						Keyring::Bob.to_account_id(),
						Keyring::Charlie.to_account_id(),
						Keyring::Dave.to_account_id(),
					],
					2
				)
			);
		}

		#[test]
		fn ensure_correct_para_id() {
			let genesis = local_config();

			let para_id = genesis["parachainInfo"]["parachainId"].as_u64().unwrap();
			assert_eq!(para_id, 3395);
		}

		#[test]
		fn ensure_genesis_assets_are_empty() {
			let genesis = local_config();

			let assets = genesis["assets"]["assets"].as_array().unwrap();
			assert!(assets.is_empty());
		}

		#[test]
		fn ensure_council_members() {
			let council_members = vec![
				Keyring::Alice.to_account_id(),
				Keyring::Bob.to_account_id(),
				Keyring::Charlie.to_account_id(),
				Keyring::Dave.to_account_id(),
				Keyring::Eve.to_account_id(),
			];
			let genesis = local_config();

			let council: Vec<_> = genesis["council"]["members"]
				.as_array()
				.unwrap()
				.iter()
				.map(|e| {
					let member = e.as_str().unwrap().to_string();
					member
				})
				.collect();
			assert!(council_members.iter().all(|s| council.contains(&s.to_string())));
		}
	}

	mod live_config {
		use super::*;

		#[test]
		fn ensure_initial_collators() {
			let initial_collators = vec![(
				AccountId::from_ss58check("15B6eUkXgoLA3dWruCRYWeBGNC8SCwuqiMtMTM1Zh2auSg3w")
					.unwrap(),
				AuraId::from_ss58check("15B6eUkXgoLA3dWruCRYWeBGNC8SCwuqiMtMTM1Zh2auSg3w").unwrap(),
			)];

			let genesis = live_config();

			let invulnerables = genesis["collatorSelection"]["invulnerables"]
				.as_array()
				.unwrap()
				.iter()
				.map(|a| a.as_str().unwrap().to_string())
				.collect::<Vec<_>>();
			assert!(initial_collators.iter().all(|c| invulnerables.contains(&c.0.to_string())));

			let session_keys = genesis["session"]["keys"]
				.as_array()
				.unwrap()
				.iter()
				.map(|k| {
					let key = k.as_array().unwrap();
					let session_key = key[0].as_str().unwrap().to_string();
					let aura_key =
						key[2].as_object().unwrap()["aura"].as_str().unwrap().to_string();
					(session_key, aura_key)
				})
				.collect::<Vec<_>>();
			assert!(initial_collators
				.iter()
				.all(|c| session_keys.contains(&(c.0.to_string(), c.1.to_string()))));
		}

		#[test]
		fn endowed_accounts_are_empty() {
			let genesis = live_config();

			let balances = genesis["balances"]["balances"].as_array().unwrap();

			assert!(balances.is_empty());
		}

		#[test]
		fn ensure_correct_sudo_key() {
			let genesis = live_config();

			let sudo_key = genesis["sudo"]["key"].as_str().unwrap();
			assert_eq!(AccountId::from_ss58check(sudo_key).unwrap(), SudoAddress::get());
		}

		#[test]
		fn ensure_correct_para_id() {
			let genesis = live_config();

			let para_id = genesis["parachainInfo"]["parachainId"].as_u64().unwrap();
			assert_eq!(para_id, 3395);
		}

		#[test]
		fn ensure_genesis_assets_are_empty() {
			let genesis = live_config();

			let assets = genesis["assets"]["assets"].as_array().unwrap();
			assert!(assets.is_empty());
		}

		#[test]
		fn ensure_council_members_are_not_set() {
			let genesis = live_config();

			let council = genesis["council"]["members"].as_array().unwrap();
			assert!(council.is_empty());
		}
	}
}

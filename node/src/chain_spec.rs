use cumulus_primitives_core::ParaId;
use pop_runtime_common::{AccountId, AuraId, Signature};
use pop_runtime_mainnet::SudoAddress;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the development parachain runtime.
pub type DevnetChainSpec = sc_service::GenericChainSpec<Extensions>;

/// Specialized `ChainSpec` for the testnet parachain runtime.
pub type TestnetChainSpec = sc_service::GenericChainSpec<Extensions>;

/// Specialized `ChainSpec` for the mainnet parachain runtime.
pub type MainnetChainSpec = sc_service::GenericChainSpec<Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

pub(crate) enum Relay {
	Paseo,
	PaseoLocal,
	Polkadot,
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	#[serde(alias = "relayChain", alias = "RelayChain")]
	pub relay_chain: String,
	/// The id of the Parachain.
	#[serde(alias = "paraId", alias = "ParaId")]
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn pop_devnet_session_keys(keys: AuraId) -> pop_runtime_devnet::SessionKeys {
	pop_runtime_devnet::SessionKeys { aura: keys }
}
/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn pop_testnet_session_keys(keys: AuraId) -> pop_runtime_testnet::SessionKeys {
	pop_runtime_testnet::SessionKeys { aura: keys }
}
/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn pop_mainnet_session_keys(keys: AuraId) -> pop_runtime_mainnet::SessionKeys {
	pop_runtime_mainnet::SessionKeys { aura: keys }
}

fn configure_for_relay(
	relay: Relay,
	properties: &mut sc_chain_spec::Properties,
) -> (Extensions, u32) {
	let para_id;

	match relay {
		Relay::Paseo | Relay::PaseoLocal => {
			para_id = 4001;
			properties.insert("tokenSymbol".into(), "PAS".into());
			properties.insert("tokenDecimals".into(), 10.into());

			let relay_chain = if let Relay::Paseo = relay {
				properties.insert("ss58Format".into(), 0.into());
				"paseo".into()
			} else {
				properties.insert("ss58Format".into(), 42.into());
				"paseo-local".into()
			};
			(Extensions { relay_chain, para_id }, para_id)
		},
		Relay::Polkadot => {
			para_id = 3395;
			properties.insert("ss58Format".into(), 0.into());
			properties.insert("tokenSymbol".into(), "DOT".into());
			properties.insert("tokenDecimals".into(), 10.into());
			(Extensions { relay_chain: "polkadot".into(), para_id }, para_id)
		},
	}
}

pub fn development_config(relay: Relay) -> DevnetChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	let (extensions, para_id) = configure_for_relay(relay, &mut properties);

	DevnetChainSpec::builder(
		pop_runtime_devnet::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		extensions,
	)
	.with_name("Pop Network Development")
	.with_id("pop-devnet")
	.with_chain_type(ChainType::Development)
	.with_genesis_config_patch(devnet_genesis(
		// initial collators.
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_collator_keys_from_seed("Bob"),
			),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		para_id.into(),
	))
	.with_protocol_id("pop-devnet")
	.with_properties(properties)
	.build()
}

pub fn testnet_config(relay: Relay) -> TestnetChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	let (extensions, para_id) = configure_for_relay(relay, &mut properties);

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
	let sudo_account_id: AccountId =
		AccountId::from_ss58check("5FPL3ZLqUk6MyBoZrQZ1Co29WAteX6T6N68TZ6jitHvhpyuD").unwrap();

	#[allow(deprecated)]
	TestnetChainSpec::builder(
		pop_runtime_testnet::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		extensions,
	)
	.with_name("Pop Network Testnet")
	.with_id("pop-testnet")
	.with_chain_type(ChainType::Live)
	.with_genesis_config_patch(testnet_genesis(
		// initial collators.
		vec![
			// POP COLLATOR 0
			(collator_0_account_id, collator_0_aura_id),
			// POP COLLATOR 1
			(collator_1_account_id, collator_1_aura_id),
			// POP COLLATOR 2
			(collator_2_account_id, collator_2_aura_id),
		],
		sudo_account_id,
		para_id.into(),
	))
	.with_protocol_id("pop-testnet")
	.with_properties(properties)
	.build()
}

pub fn mainnet_config(relay: Relay) -> MainnetChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	let (extensions, para_id) = configure_for_relay(relay, &mut properties);

	let collator_0_account_id: AccountId =
		AccountId::from_ss58check("15B6eUkXgoLA3dWruCRYWeBGNC8SCwuqiMtMTM1Zh2auSg3w").unwrap();
	let collator_0_aura_id: AuraId =
		AuraId::from_ss58check("15B6eUkXgoLA3dWruCRYWeBGNC8SCwuqiMtMTM1Zh2auSg3w").unwrap();

	// Multisig account for sudo, generated from the following signatories:
	// - 15VPagCVayS6XvT5RogPYop3BJTJzwqR2mCGR1kVn3w58ygg
	// - 142zako1kfvrpQ7pJKYR8iGUD58i4wjb78FUsmJ9WcXmkM5z
	// - 15k9niqckMg338cFBoz9vWFGwnCtwPBquKvqJEfHApijZkDz
	// - 14G3CUFnZUBnHZUhahexSZ6AgemaW9zMHBnGccy3df7actf4
	// - Threshold 2
	let sudo_account_id: AccountId = SudoAddress::get();

	#[allow(deprecated)]
	MainnetChainSpec::builder(
		pop_runtime_mainnet::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		extensions,
	)
	.with_name("Pop Network")
	.with_id("pop")
	.with_chain_type(ChainType::Live)
	.with_genesis_config_patch(mainnet_genesis(
		// initial collators.
		vec![
			// POP COLLATOR 0
			(collator_0_account_id, collator_0_aura_id),
		],
		sudo_account_id,
		para_id.into(),
	))
	.with_protocol_id("pop")
	.with_properties(properties)
	.build()
}

fn mainnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root: AccountId,
	id: ParaId,
) -> serde_json::Value {
	use pop_runtime_mainnet::EXISTENTIAL_DEPOSIT;

	serde_json::json!({
		"balances": {
			"balances": [],
		},
		"parachainInfo": {
			"parachainId": id,
		},
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16,
			"desiredCandidates": 0,
		},
		"session": {
			"keys": invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						pop_mainnet_session_keys(aura),      // session keys
					)
				})
			.collect::<Vec<_>>(),
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		},
		"sudo": { "key": Some(root) }
	})
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root: AccountId,
	id: ParaId,
) -> serde_json::Value {
	use pop_runtime_testnet::EXISTENTIAL_DEPOSIT;

	serde_json::json!({
		"balances": {
			"balances": [],
		},
		"parachainInfo": {
			"parachainId": id,
		},
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16,
		},
		"session": {
			"keys": invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						pop_testnet_session_keys(aura),      // session keys
					)
				})
			.collect::<Vec<_>>(),
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		},
		"sudo": { "key": Some(root) }
	})
}

fn devnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root: AccountId,
	id: ParaId,
) -> serde_json::Value {
	use pop_runtime_devnet::EXISTENTIAL_DEPOSIT;
	let asset_hub = ismp_parachain::ParachainData { id: 1000, slot_duration: 6000 };

	serde_json::json!({
		"balances": {
			"balances": [],
		},
		"parachainInfo": {
			"parachainId": id,
		},
		"collatorSelection": {
			"invulnerables": invulnerables.iter().cloned().map(|(acc, _)| acc).collect::<Vec<_>>(),
			"candidacyBond": EXISTENTIAL_DEPOSIT * 16,
		},
		"session": {
			"keys": invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						pop_devnet_session_keys(aura),      // session keys
					)
				})
			.collect::<Vec<_>>(),
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		},
		"sudo": { "key": Some(root) },
		// Set the following parachains to be tracked via ISMP.
		"ismpParachain": pop_runtime_devnet::IsmpParachainConfig {
			// Asset Hub
			parachains: vec![asset_hub],
			..Default::default()
		},
	})
}

#[test]
fn sudo_key_valid() {
	// Source: https://github.com/paritytech/extended-parachain-template/blob/d08cec37117731953119ecaed79522a0812b46f5/node/src/chain_spec.rs#L79
	fn get_multisig_sudo_key(mut authority_set: Vec<AccountId>, threshold: u16) -> AccountId {
		assert!(threshold > 0, "Threshold for sudo multisig cannot be 0");
		assert!(!authority_set.is_empty(), "Sudo authority set cannot be empty");
		assert!(
			authority_set.len() >= threshold.into(),
			"Threshold must be less than or equal to authority set members"
		);
		// Sorting is done to deterministically order the multisig set
		// So that a single authority set (A, B, C) may generate only a single unique multisig key
		// Otherwise, (B, A, C) or (C, A, B) could produce different keys and cause chaos
		authority_set.sort();

		// Define a multisig threshold for `threshold / authority_set.len()` members
		pallet_multisig::Pallet::<pop_runtime_mainnet::Runtime>::multi_account_id(
			&authority_set[..],
			threshold,
		)
	}

	assert_eq!(
		get_multisig_sudo_key(
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

use cumulus_primitives_core::ParaId;
use pop_runtime_common::{AccountId, AuraId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Specialized `ChainSpec` for the development parachain runtime.
pub type DevnetChainSpec =
	sc_service::GenericChainSpec<pop_runtime_devnet::RuntimeGenesisConfig, Extensions>;

/// Specialized `ChainSpec` for the testnet parachain runtime.
pub type TestnetChainSpec =
	sc_service::GenericChainSpec<pop_runtime_testnet::RuntimeGenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
pub(crate) enum Relay {
	#[cfg(not(feature = "paseo"))]
	Rococo,
	#[cfg(feature = "paseo")]
	Paseo,
	#[cfg(not(feature = "paseo"))]
	RococoLocal,
	#[cfg(feature = "paseo")]
	PaseoLocal,
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
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

fn configure_for_relay(
	relay: Relay,
	properties: &mut sc_chain_spec::Properties,
) -> (Extensions, u32) {
	properties.insert("ss58Format".into(), 42.into());
	let para_id;

	match relay {
		// Test relay chains
		#[cfg(not(feature = "paseo"))]
		Relay::Rococo => {
			para_id = 4385;
			properties.insert("tokenSymbol".into(), "ROC".into());
			properties.insert("tokenDecimals".into(), 12.into());

			(Extensions { relay_chain: "rococo".into(), para_id }, para_id)
		},
		#[cfg(feature = "paseo")]
		Relay::Paseo => {
			para_id = 4001;
			properties.insert("tokenSymbol".into(), "PAS".into());
			properties.insert("tokenDecimals".into(), 10.into());

			(Extensions { relay_chain: "paseo".into(), para_id }, para_id)
		},
		// Local relay chains
		#[cfg(not(feature = "paseo"))]
		Relay::RococoLocal => {
			para_id = 4385;
			properties.insert("tokenSymbol".into(), "ROC".into());
			properties.insert("tokenDecimals".into(), 12.into());

			(Extensions { relay_chain: "rococo-local".into(), para_id }, para_id)
		},
		#[cfg(feature = "paseo")]
		Relay::PaseoLocal => {
			para_id = 4001;
			properties.insert("tokenSymbol".into(), "PAS".into());
			properties.insert("tokenDecimals".into(), 10.into());

			(Extensions { relay_chain: "paseo-local".into(), para_id }, para_id)
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
		pop_runtime_devnet::WASM_BINARY.expect("WASM binary was not built, please build it!"),
		extensions,
	)
	.with_name("Pop Network Testnet")
	.with_id("pop-testnet")
	.with_chain_type(ChainType::Testnet)
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

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root: AccountId,
	id: ParaId,
) -> serde_json::Value {
	use pop_runtime_testnet::EXISTENTIAL_DEPOSIT;

	serde_json::json!({
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

	serde_json::json!({
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
		"sudo": { "key": Some(root) }
	})
}

use cumulus_primitives_core::ParaId;
use pop_runtime_common::{AccountId, AuraId};
use pop_runtime_mainnet::config::governance::SudoAddress;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, GenericChainSpec};
use serde::{Deserialize, Serialize};
use sp_core::crypto::Ss58Codec;

/// Generic `ChainSpec` for a parachain runtime.
pub type ChainSpec = GenericChainSpec<Extensions>;

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

/// Chainspec builder trait: to be implemented for the different runtimes (i.e. `devnet`, `testnet`
/// & `mainnet`) to ease building.
trait ChainSpecBuilder {
	fn build(
		id: &str,
		name: &str,
		chain_type: ChainType,
		genesis_preset: &str,
		protocol_id: &str,
		relay_chain: &str,
	) -> ChainSpec {
		ChainSpec::builder(
			Self::wasm_binary(),
			Extensions { relay_chain: relay_chain.into(), para_id: Self::para_id() },
		)
		.with_name(name)
		.with_id(id)
		.with_chain_type(chain_type)
		.with_genesis_config_preset_name(genesis_preset)
		.with_protocol_id(protocol_id)
		.with_properties(Self::properties())
		.build()
	}

	fn para_id() -> u32;
	fn properties() -> sc_chain_spec::Properties;
	fn wasm_binary() -> &'static [u8];
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

pub mod devnet {
	use pop_runtime_devnet as runtime;
	use pop_runtime_devnet::Runtime;
	pub use runtime::genesis::{DEVNET, DEVNET_DEV, DEVNET_LOCAL};

	use super::*;

	impl ChainSpecBuilder for Runtime {
		fn para_id() -> u32 {
			runtime::genesis::PARA_ID.into()
		}

		fn properties() -> sc_chain_spec::Properties {
			let mut properties = sc_chain_spec::Properties::new();
			properties.insert("tokenSymbol".into(), "PAS".into());
			properties.insert("tokenDecimals".into(), 10.into());
			properties.insert("ss58Format".into(), 0.into());
			properties
		}

		fn wasm_binary() -> &'static [u8] {
			runtime::WASM_BINARY.expect("WASM binary was not built, please build it!")
		}
	}

	/// Configures a development chain running on a single node, using the devnet runtime.
	pub fn development_chain_spec() -> ChainSpec {
		const ID: &str = DEVNET_DEV;
		Runtime::build(
			ID,
			"Pop Devnet (Development)",
			ChainType::Development,
			ID,
			ID,
			"paseo-local",
		)
	}

	/// Configures a local chain running on multiple nodes for testing purposes, using the devnet
	/// runtime.
	pub fn local_chain_spec() -> ChainSpec {
		const ID: &str = DEVNET_LOCAL;
		Runtime::build(ID, "Pop Devnet (Local)", ChainType::Local, ID, ID, "paseo-local")
	}

	/// Configures a live chain running on multiple nodes, using the devnet runtime.
	pub fn live_chain_spec() -> ChainSpec {
		const ID: &str = DEVNET;
		Runtime::build(ID, "Pop Devnet", ChainType::Live, ID, ID, "paseo")
	}
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

pub fn testnet_chain_spec(relay: Relay) -> TestnetChainSpec {
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

pub fn mainnet_chain_spec(relay: Relay) -> MainnetChainSpec {
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
		// councillors
		vec![],
	))
	.with_protocol_id("pop")
	.with_properties(properties)
	.build()
}

fn mainnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	root: AccountId,
	id: ParaId,
	councillors: Vec<AccountId>,
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
		"sudo": { "key": Some(root) },
		"council": {
			"members": councillors,
		}
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

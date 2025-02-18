use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, GenericChainSpec};
use serde::{Deserialize, Serialize};

/// Generic `ChainSpec` for a parachain runtime.
pub type ChainSpec = GenericChainSpec<Extensions>;

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

	/// Configures a live chain running on multiple nodes on private devnet, using the devnet
	/// runtime.
	pub fn live_chain_spec() -> ChainSpec {
		const ID: &str = DEVNET;
		Runtime::build(ID, "Pop Devnet", ChainType::Live, ID, ID, "paseo")
	}
}

pub mod testnet {
	use pop_runtime_testnet as runtime;
	use pop_runtime_testnet::Runtime;
	pub use runtime::genesis::{TESTNET, TESTNET_DEV, TESTNET_LOCAL};

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

	/// Configures a development chain running on a single node, using the testnet runtime.
	pub fn development_chain_spec() -> ChainSpec {
		const ID: &str = TESTNET_DEV;
		Runtime::build(
			ID,
			"Pop Testnet (Development)",
			ChainType::Development,
			ID,
			ID,
			"paseo-local",
		)
	}

	/// Configures a local chain running on multiple nodes for testing purposes, using the testnet
	/// runtime.
	pub fn local_chain_spec() -> ChainSpec {
		const ID: &str = TESTNET_LOCAL;
		Runtime::build(ID, "Pop Testnet (Local)", ChainType::Local, ID, ID, "paseo-local")
	}

	/// Configures a live chain running on multiple nodes, using the testnet runtime.
	pub fn live_chain_spec() -> ChainSpec {
		const ID: &str = TESTNET;
		Runtime::build(ID, "Pop Testnet", ChainType::Live, ID, ID, "paseo")
	}
}

pub mod mainnet {
	use pop_runtime_mainnet as runtime;
	use pop_runtime_mainnet::Runtime;
	pub use runtime::genesis::{MAINNET, MAINNET_DEV, MAINNET_LOCAL};

	use super::*;

	impl ChainSpecBuilder for Runtime {
		fn para_id() -> u32 {
			runtime::genesis::PARA_ID.into()
		}

		fn properties() -> sc_chain_spec::Properties {
			let mut properties = sc_chain_spec::Properties::new();
			properties.insert("tokenSymbol".into(), "DOT".into());
			properties.insert("tokenDecimals".into(), 10.into());
			properties.insert("ss58Format".into(), 0.into());
			properties
		}

		fn wasm_binary() -> &'static [u8] {
			runtime::WASM_BINARY.expect("WASM binary was not built, please build it!")
		}
	}

	/// Configures a development chain running on a single node, using the mainnet runtime.
	pub fn development_chain_spec() -> ChainSpec {
		const ID: &str = MAINNET_DEV;
		Runtime::build(ID, "Pop (Development)", ChainType::Development, ID, ID, "paseo-local")
	}

	/// Configures a local chain running on multiple nodes for testing purposes, using the mainnet
	/// runtime.
	pub fn local_chain_spec() -> ChainSpec {
		const ID: &str = MAINNET_LOCAL;
		Runtime::build(ID, "Pop (Local)", ChainType::Local, ID, ID, "paseo-local")
	}

	/// Configures a live chain running on multiple nodes publicly, using the mainnet
	/// runtime.
	pub fn live_chain_spec() -> ChainSpec {
		const ID: &str = MAINNET;
		Runtime::build(ID, "Pop", ChainType::Live, ID, ID, "polkadot")
	}

	#[cfg(test)]
	mod tests {
		use sc_chain_spec::ChainSpec;
		use serde_json::json;

		use super::*;

		#[test]
		fn dev_configuration_is_correct() {
			let chain_spec = development_chain_spec();
			assert!(chain_spec.boot_nodes().is_empty());
			assert_eq!(chain_spec.name(), "Pop (Development)");
			assert_eq!(chain_spec.id(), "pop-dev");
			assert_eq!(chain_spec.chain_type(), ChainType::Development);
			assert!(chain_spec.telemetry_endpoints().is_none());
			assert_eq!(chain_spec.protocol_id().unwrap(), "pop-dev");
			assert!(chain_spec.fork_id().is_none());
			assert_eq!(
				&chain_spec.properties(),
				json!({
					"ss58Format": 0, // Paseo uses Polkadot's SS58.
					"tokenDecimals": 10,
					"tokenSymbol": "DOT",
				})
				.as_object()
				.unwrap()
			);
			assert_eq!(
				chain_spec.extensions(),
				&Extensions { relay_chain: "paseo-local".to_string(), para_id: 3395 }
			);
			assert!(chain_spec.code_substitutes().is_empty());
		}

		#[test]
		fn local_configuration_is_correct() {
			let chain_spec = local_chain_spec();
			assert!(chain_spec.boot_nodes().is_empty());
			assert_eq!(chain_spec.name(), "Pop (Local)");
			assert_eq!(chain_spec.id(), "pop-local");
			assert_eq!(chain_spec.chain_type(), ChainType::Local);
			assert!(chain_spec.telemetry_endpoints().is_none());
			assert_eq!(chain_spec.protocol_id().unwrap(), "pop-local");
			assert!(chain_spec.fork_id().is_none());
			assert_eq!(
				&chain_spec.properties(),
				json!({
					"ss58Format": 0, // Paseo uses Polkadot's SS58.
					"tokenDecimals": 10,
					"tokenSymbol": "DOT",
				})
				.as_object()
				.unwrap()
			);
			assert_eq!(
				chain_spec.extensions(),
				&Extensions { relay_chain: "paseo-local".to_string(), para_id: 3395 }
			);
			assert!(chain_spec.code_substitutes().is_empty());
		}

		#[test]
		fn live_configuration_is_correct() {
			let chain_spec = live_chain_spec();
			assert!(chain_spec.boot_nodes().is_empty());
			assert_eq!(chain_spec.name(), "Pop");
			assert_eq!(chain_spec.id(), "pop");
			assert_eq!(chain_spec.chain_type(), ChainType::Live);
			assert!(chain_spec.telemetry_endpoints().is_none());
			assert_eq!(chain_spec.protocol_id().unwrap(), "pop");
			assert!(chain_spec.fork_id().is_none());
			assert_eq!(
				&chain_spec.properties(),
				json!({
					"ss58Format": 0, // Polkadot's SS58.
					"tokenDecimals": 10,
					"tokenSymbol": "DOT",
				})
				.as_object()
				.unwrap()
			);
			assert_eq!(
				chain_spec.extensions(),
				&Extensions { relay_chain: "polkadot".to_string(), para_id: 3395 }
			);
			assert!(chain_spec.code_substitutes().is_empty());
		}
	}
}

use std::{net::SocketAddr, path::PathBuf};

use cumulus_client_service::storage_proof_size::HostFunctions as ReclaimHostFunctions;
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use log::info;
use pop_runtime_common::Block;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_runtime::traits::HashingFor;

use crate::{
	chain_spec,
	chain_spec::Relay,
	cli::{Cli, RelayChainCli, Subcommand},
	service::new_partial,
};

#[derive(Debug, PartialEq)]
enum Runtime {
	Devnet,
	Testnet,
	Mainnet,
}

trait RuntimeResolver {
	fn runtime(&self) -> Runtime;
}
/// Private helper that pattern matches on the input (which is expected to be a ChainSpec ID)
/// and returns the Runtime accordingly.
fn runtime(id: &str) -> Runtime {
	if id.starts_with("dev") || id.ends_with("devnet") {
		Runtime::Devnet
	} else if id.starts_with("test") || id.ends_with("testnet") {
		Runtime::Testnet
	} else if id.eq("pop") || id.ends_with("mainnet") {
		Runtime::Mainnet
	} else {
		log::warn!(
			"No specific runtime was recognized for ChainSpec's Id: '{}', so Runtime::Devnet will \
			 be used",
			id
		);
		Runtime::Devnet
	}
}
/// Resolve runtime from ChainSpec ID
impl RuntimeResolver for dyn ChainSpec {
	fn runtime(&self) -> Runtime {
		runtime(self.id())
	}
}
/// Implementation, that can resolve [`Runtime`] from any json configuration file
impl RuntimeResolver for PathBuf {
	fn runtime(&self) -> Runtime {
		#[derive(Debug, serde::Deserialize)]
		struct EmptyChainSpecWithId {
			id: String,
		}

		let file = std::fs::File::open(self).expect("Failed to open file");
		let reader = std::io::BufReader::new(file);
		let chain_spec: EmptyChainSpecWithId = serde_json::from_reader(reader)
			.expect("Failed to read 'json' file with ChainSpec configuration");

		runtime(&chain_spec.id)
	}
}

fn load_spec(id: &str) -> std::result::Result<Box<dyn ChainSpec>, String> {
	Ok(match id {
		"dev" | "devnet" | "dev-paseo" =>
			Box::new(chain_spec::development_config(Relay::PaseoLocal)),
		"test" | "testnet" | "pop-paseo" => Box::new(chain_spec::testnet_config(Relay::Paseo)),
		"pop" | "mainnet" | "pop-polkadot" | "pop-network" =>
			Box::new(chain_spec::mainnet_config(Relay::Polkadot)),
		"" | "local" => Box::new(chain_spec::development_config(Relay::PaseoLocal)),
		path => {
			let path: PathBuf = path.into();
			match path.runtime() {
				Runtime::Devnet => Box::new(chain_spec::DevnetChainSpec::from_json_file(path)?),
				Runtime::Testnet => Box::new(chain_spec::TestnetChainSpec::from_json_file(path)?),
				Runtime::Mainnet => Box::new(chain_spec::MainnetChainSpec::from_json_file(path)?),
			}
		},
	})
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Pop Collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"Pop Collator\n\nThe command-line arguments provided first will be passed to the \
			 parachain node, while the arguments provided after -- will be passed to the relay \
			 chain node.\n\n{} <parachain-args> -- <relay-chain-args>",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/paritytech/polkadot-sdk/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2023
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id)
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Pop Collator".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		format!(
			"Pop Collator\n\nThe command-line arguments provided first will be passed to the \
			 parachain node, while the arguments provided after -- will be passed to the relay \
			 chain node.\n\n{} <parachain-args> -- <relay-chain-args>",
			Self::executable_name()
		)
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/paritytech/polkadot-sdk/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2020
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}
}

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;
		match runner.config().chain_spec.runtime() {
			Runtime::Devnet => {
				runner.async_run(|$config| {
					let $components = new_partial::<pop_runtime_devnet::RuntimeApi>(
						&$config
					)?;
					let task_manager = $components.task_manager;
					{ $( $code )* }.map(|v| (v, task_manager))
				})
			}
			Runtime::Testnet => {
				#[cfg(feature = "ismp")]
				unimplemented!("ISMP is not supported in testnet");
				#[cfg(not(feature = "ismp"))]
				{
					runner.async_run(|$config| {
						let $components = new_partial::<pop_runtime_testnet::RuntimeApi>(
							&$config
						)?;
						let task_manager = $components.task_manager;
						{ $( $code )* }.map(|v| (v, task_manager))
					})
				}
			}
			Runtime::Mainnet => {
				#[cfg(feature = "ismp")]
				unimplemented!("ISMP is not supported in mainnet");
				#[cfg(not(feature = "ismp"))]
				{
					runner.async_run(|$config| {
						let $components = new_partial::<pop_runtime_mainnet::RuntimeApi>(
							&$config
						)?;
						let task_manager = $components.task_manager;
						{ $( $code )* }.map(|v| (v, task_manager))
					})
				}
			}
		}
	}}
}

macro_rules! construct_benchmark_partials {
	($config:expr, |$partials:ident| $code:expr) => {
		match $config.chain_spec.runtime() {
			Runtime::Devnet => {
				let $partials = new_partial::<pop_runtime_devnet::RuntimeApi>(&$config)?;
				$code
			},
			Runtime::Testnet => {
				#[cfg(feature = "ismp")]
				unimplemented!("ISMP is not supported in testnet");
				#[cfg(not(feature = "ismp"))]
				{
					let $partials = new_partial::<pop_runtime_testnet::RuntimeApi>(&$config)?;
					$code
				}
			},
			Runtime::Mainnet => {
				#[cfg(feature = "ismp")]
				unimplemented!("ISMP is not supported in mainnet");
				#[cfg(not(feature = "ismp"))]
				{
					let $partials = new_partial::<pop_runtime_mainnet::RuntimeApi>(&$config)?;
					$code
				}
			},
		}
	};
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.database))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.chain_spec))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::Revert(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.backend, None))
			})
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
				);

				let polkadot_config = SubstrateCli::create_configuration(
					&polkadot_cli,
					&polkadot_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		},
		Some(Subcommand::ExportGenesisHead(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				construct_benchmark_partials!(config, |partials| cmd.run(partials.client))
			})
		},
		Some(Subcommand::ExportGenesisWasm(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				cmd.run(&*spec)
			})
		},
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			// Switch on the concrete benchmark sub-command-
			match cmd {
				BenchmarkCmd::Pallet(cmd) =>
					if cfg!(feature = "runtime-benchmarks") {
						runner.sync_run(|config| {
							cmd.run_with_spec::<HashingFor<Block>, ReclaimHostFunctions>(Some(
								config.chain_spec,
							))
						})
					} else {
						Err("Benchmarking wasn't enabled when building the node. You can enable \
						     it with `--features runtime-benchmarks`."
							.into())
					},
				BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, |partials| cmd.run(partials.client))
				}),
				#[cfg(not(feature = "runtime-benchmarks"))]
				BenchmarkCmd::Storage(_) => Err(sc_cli::Error::Input(
					"Compile with --features=runtime-benchmarks to enable storage benchmarks."
						.into(),
				)),
				#[cfg(feature = "runtime-benchmarks")]
				BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, |partials| {
						let db = partials.backend.expose_db();
						let storage = partials.backend.expose_storage();
						cmd.run(config, partials.client.clone(), db, storage)
					})
				}),
				BenchmarkCmd::Machine(cmd) =>
					runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())),
				// NOTE: this allows the Client to leniently implement
				// new benchmark commands without requiring a companion MR.
				#[allow(unreachable_patterns)]
				_ => Err("Benchmarking sub-command unsupported".into()),
			}
		},
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		None => {
			let runner = cli.create_runner(&cli.run.normalize())?;
			let collator_options = cli.run.collator_options();

			runner.run_node_until_exit(|config| async move {
				let hwbench = (!cli.no_hardware_benchmarks)
					.then_some(config.database.path().map(|database_path| {
						let _ = std::fs::create_dir_all(database_path);
						sc_sysinfo::gather_hwbench(Some(database_path))
					}))
					.flatten();

				let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
					.map(|e| e.para_id)
					.ok_or("Could not find parachain ID in chain-spec.")?;

				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
				);

				let id = ParaId::from(para_id);

				let tokio_handle = config.tokio_handle.clone();
				let polkadot_config =
					SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
						.map_err(|err| format!("Relay chain argument error: {}", err))?;

				info!("Is collating: {}", if config.role.is_authority() { "yes" } else { "no" });

				match config.chain_spec.runtime() {
					Runtime::Devnet => {
						sp_core::crypto::set_default_ss58_version(
							pop_runtime_devnet::SS58Prefix::get().into(),
						);
						crate::service::start_parachain_node::<pop_runtime_devnet::RuntimeApi>(
							config,
							polkadot_config,
							collator_options,
							id,
							hwbench,
						)
						.await
						.map(|r| r.0)
						.map_err(Into::into)
					},
					Runtime::Testnet => {
						#[cfg(feature = "ismp")]
						unimplemented!("ISMP is not supported in testnet");
						#[cfg(not(feature = "ismp"))]
						{
							sp_core::crypto::set_default_ss58_version(
								pop_runtime_testnet::SS58Prefix::get().into(),
							);
							crate::service::start_parachain_node::<pop_runtime_testnet::RuntimeApi>(
								config,
								polkadot_config,
								collator_options,
								id,
								hwbench,
							)
							.await
							.map(|r| r.0)
							.map_err(Into::into)
						}
					},
					Runtime::Mainnet => {
						#[cfg(feature = "ismp")]
						unimplemented!("ISMP is not supported in mainnet");
						#[cfg(not(feature = "ismp"))]
						{
							sp_core::crypto::set_default_ss58_version(
								pop_runtime_mainnet::SS58Prefix::get().into(),
							);
							crate::service::start_parachain_node::<pop_runtime_mainnet::RuntimeApi>(
								config,
								polkadot_config,
								collator_options,
								id,
								hwbench,
							)
							.await
							.map(|r| r.0)
							.map_err(Into::into)
						}
					},
				}
			})
		},
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_listen_port() -> u16 {
		9945
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()?
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_addr(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_addr(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self, is_dev: bool) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool(is_dev)
	}

	fn trie_cache_maximum_size(&self) -> Result<Option<usize>> {
		self.base.base.trie_cache_maximum_size()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_max_connections(&self) -> Result<u32> {
		self.base.base.rpc_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}

	fn node_name(&self) -> Result<String> {
		self.base.base.node_name()
	}
}

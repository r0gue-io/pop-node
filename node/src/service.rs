//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

// std
use std::{sync::Arc, time::Duration};

use cumulus_client_cli::CollatorOptions;
// Cumulus Imports
use cumulus_client_collator::service::CollatorService;
#[docify::export(lookahead_collator)]
use cumulus_client_consensus_aura::collators::lookahead::{self as aura, Params as AuraParams};
use cumulus_client_consensus_common::ParachainBlockImport as TParachainBlockImport;
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_service::{
	build_network, build_relay_chain_interface, prepare_node_config, start_relay_chain_tasks,
	BuildNetworkParams, CollatorSybilResistance, DARecoveryProfile, StartRelayChainTasksParams,
};
#[docify::export(cumulus_primitives)]
use cumulus_primitives_core::{
	relay_chain::{CollatorPair, ValidationCode},
	ParaId,
};
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainInterface};
// Substrate Imports
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
// Local Runtime Types
use pop_runtime_common::{Block, Hash};
use prometheus_endpoint::Registry;
use sc_client_api::Backend;
use sc_consensus::ImportQueue;
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_network::NetworkBlock;
use sc_service::{Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::ConstructRuntimeApi;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::BlakeTwo256;

#[cfg(not(feature = "runtime-benchmarks"))]
type HostFunctions = cumulus_client_service::ParachainHostFunctions;

#[cfg(feature = "runtime-benchmarks")]
type HostFunctions = (
	cumulus_client_service::ParachainHostFunctions,
	frame_benchmarking::benchmarking::HostFunctions,
);

type ParachainExecutor = WasmExecutor<HostFunctions>;

type ParachainClient<RuntimeApi> = TFullClient<Block, RuntimeApi, ParachainExecutor>;

type ParachainBackend = TFullBackend<Block>;

type ParachainBlockImport<RuntimeApi> =
	TParachainBlockImport<Block, Arc<ParachainClient<RuntimeApi>>, ParachainBackend>;

/// Assembly of PartialComponents (enough to run chain ops subcommands)
type Service<RuntimeApi> = PartialComponents<
	ParachainClient<RuntimeApi>,
	ParachainBackend,
	(),
	sc_consensus::DefaultImportQueue<Block>,
	sc_transaction_pool::TransactionPoolHandle<Block, ParachainClient<RuntimeApi>>,
	(ParachainBlockImport<RuntimeApi>, Option<Telemetry>, Option<TelemetryWorkerHandle>),
>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial<RuntimeApi>(
	config: &Configuration,
) -> Result<Service<RuntimeApi>, sc_service::Error>
where
	RuntimeApi: ConstructRuntimeApi<Block, ParachainClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiExt<RuntimeApi>,
	sc_client_api::StateBackendFor<TFullBackend<Block>, Block>:
		sc_client_api::StateBackend<BlakeTwo256>,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let heap_pages = config
		.executor
		.default_heap_pages
		.map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static { extra_pages: h as _ });

	let executor = ParachainExecutor::builder()
		.with_execution_method(config.executor.wasm_method)
		.with_onchain_heap_alloc_strategy(heap_pages)
		.with_offchain_heap_alloc_strategy(heap_pages)
		.with_max_runtime_instances(config.executor.max_runtime_instances)
		.with_runtime_cache_size(config.executor.runtime_cache_size)
		.build();

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts_record_import::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
			true,
		)?;
	let client = Arc::new(client);

	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let transaction_pool = Arc::from(
		sc_transaction_pool::Builder::new(
			task_manager.spawn_essential_handle(),
			client.clone(),
			config.role.is_authority().into(),
		)
		.with_options(config.transaction_pool.clone())
		.with_prometheus(config.prometheus_registry())
		.build(),
	);

	let block_import = ParachainBlockImport::<RuntimeApi>::new(client.clone(), backend.clone());

	let import_queue = build_import_queue(
		client.clone(),
		block_import.clone(),
		config,
		telemetry.as_ref().map(|telemetry| telemetry.handle()),
		&task_manager,
	);

	Ok(PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: (),
		other: (block_import, telemetry, telemetry_worker_handle),
	})
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RuntimeApi, SC>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	para_id: ParaId,
	start_consensus: SC,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient<RuntimeApi>>)>
where
	RuntimeApi: ConstructRuntimeApi<Block, ParachainClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiExt<RuntimeApi>,
	SC: FnOnce(
		Arc<ParachainClient<RuntimeApi>>,
		Arc<ParachainBackend>,
		ParachainBlockImport<RuntimeApi>,
		Option<&Registry>,
		Option<TelemetryHandle>,
		&TaskManager,
		Arc<dyn RelayChainInterface>,
		Arc<sc_transaction_pool::TransactionPoolHandle<Block, ParachainClient<RuntimeApi>>>,
		KeystorePtr,
		Duration,
		ParaId,
		CollatorPair,
		OverseerHandle,
		Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
	) -> Result<(), sc_service::Error>,
{
	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial::<RuntimeApi>(&parachain_config)?;
	let (block_import, mut telemetry, telemetry_worker_handle) = params.other;

	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let net_config = sc_network::config::FullNetworkConfiguration::<
		_,
		_,
		sc_network::NetworkWorker<Block, Hash>,
	>::new(&parachain_config.network, prometheus_registry.clone());

	let client = params.client.clone();
	let backend = params.backend.clone();
	let mut task_manager = params.task_manager;

	let (relay_chain_interface, collator_key) = build_relay_chain_interface(
		polkadot_config,
		&parachain_config,
		telemetry_worker_handle,
		&mut task_manager,
		collator_options.clone(),
		hwbench.clone(),
	)
	.await
	.map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;

	let validator = parachain_config.role.is_authority();
	let transaction_pool = params.transaction_pool.clone();
	let import_queue_service = params.import_queue.service();

	let (network, system_rpc_tx, tx_handler_controller, sync_service) =
		build_network(BuildNetworkParams {
			parachain_config: &parachain_config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			para_id,
			spawn_handle: task_manager.spawn_handle(),
			relay_chain_interface: relay_chain_interface.clone(),
			import_queue: params.import_queue,
			sybil_resistance_level: CollatorSybilResistance::Resistant, // because of Aura
		})
		.await?;

	if parachain_config.offchain_worker.enabled {
		use futures::FutureExt;

		let offchain_workers =
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				keystore: Some(params.keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(
					transaction_pool.clone(),
				)),
				network_provider: Arc::new(network.clone()),
				is_validator: parachain_config.role.is_authority(),
				enable_http_requests: false,
				custom_extensions: move |_| vec![],
			})?;
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-work",
			offchain_workers.run(client.clone(), task_manager.spawn_handle()).boxed(),
		);
	}

	let rpc_builder = {
		let client = client.clone();
		let backend = backend.clone();
		let transaction_pool = transaction_pool.clone();

		Box::new(move |_| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				backend: backend.clone(),
			};

			crate::rpc::create_full(deps).map_err(Into::into)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.keystore(),
		backend: backend.clone(),
		network,
		sync_service: sync_service.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);
		// Here you can check whether the hardware meets your chains' requirements. Putting a link
		// in there and swapping out the requirements for your own are probably a good idea. The
		// requirements for a para-chain are dictated by its relay-chain.
		if SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench, false).is_err() && validator {
			log::warn!(
				"⚠️  The hardware does not meet the minimal requirements for role 'Authority'."
			);
		}

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	let announce_block = {
		let sync_service = sync_service.clone();
		Arc::new(move |hash, data| sync_service.announce_block(hash, data))
	};

	let relay_chain_slot_duration = Duration::from_secs(6);

	let overseer_handle = relay_chain_interface
		.overseer_handle()
		.map_err(|e| sc_service::Error::Application(Box::new(e)))?;

	start_relay_chain_tasks(StartRelayChainTasksParams {
		client: client.clone(),
		announce_block: announce_block.clone(),
		para_id,
		relay_chain_interface: relay_chain_interface.clone(),
		task_manager: &mut task_manager,
		da_recovery_profile: if validator {
			DARecoveryProfile::Collator
		} else {
			DARecoveryProfile::FullNode
		},
		import_queue: import_queue_service,
		relay_chain_slot_duration,
		recovery_handle: Box::new(overseer_handle.clone()),
		sync_service: sync_service.clone(),
	})?;

	if validator {
		start_consensus(
			client.clone(),
			backend.clone(),
			block_import,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface,
			transaction_pool,
			params.keystore_container.keystore(),
			relay_chain_slot_duration,
			para_id,
			collator_key.expect("Command line arguments do not allow this. qed"),
			overseer_handle,
			announce_block,
		)?;
	}

	Ok((task_manager, client))
}

/// Build the import queue for the parachain runtime.
#[allow(clippy::type_complexity)]
pub(crate) fn build_import_queue<RuntimeApi>(
	client: Arc<ParachainClient<RuntimeApi>>,
	block_import: ParachainBlockImport<RuntimeApi>,
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> sc_consensus::DefaultImportQueue<Block>
where
	RuntimeApi: ConstructRuntimeApi<Block, ParachainClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiExt<RuntimeApi>,
{
	cumulus_client_consensus_aura::equivocation_import_queue::fully_verifying_import_queue::<
		sp_consensus_aura::sr25519::AuthorityPair,
		_,
		_,
		_,
		_,
	>(
		client,
		block_import,
		move |_, _| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			Ok(timestamp)
		},
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		telemetry,
	)
}

#[allow(clippy::too_many_arguments)]
fn start_consensus<RuntimeApi>(
	client: Arc<ParachainClient<RuntimeApi>>,
	backend: Arc<ParachainBackend>,
	block_import: ParachainBlockImport<RuntimeApi>,
	prometheus_registry: Option<&Registry>,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
	relay_chain_interface: Arc<dyn RelayChainInterface>,
	transaction_pool: Arc<
		sc_transaction_pool::TransactionPoolHandle<Block, ParachainClient<RuntimeApi>>,
	>,
	keystore: KeystorePtr,
	relay_chain_slot_duration: Duration,
	para_id: ParaId,
	collator_key: CollatorPair,
	overseer_handle: OverseerHandle,
	announce_block: Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
) -> Result<(), sc_service::Error>
where
	RuntimeApi: ConstructRuntimeApi<Block, ParachainClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiExt<RuntimeApi>,
{
	let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
		task_manager.spawn_handle(),
		client.clone(),
		transaction_pool,
		prometheus_registry,
		telemetry.clone(),
	);

	let proposer = Proposer::new(proposer_factory);

	let collator_service = CollatorService::new(
		client.clone(),
		Arc::new(task_manager.spawn_handle()),
		announce_block,
		client.clone(),
	);

	#[cfg(not(feature = "ismp"))]
	let create_inherent_data_providers = move |_, ()| async move { Ok(()) };

	#[cfg(feature = "ismp")]
	let create_inherent_data_providers = {
		let (client_clone, relay_chain_interface_clone) =
			(client.clone(), relay_chain_interface.clone());
		move |parent, ()| {
			let client = client_clone.clone();
			let relay_chain_interface = relay_chain_interface_clone.clone();

			async move {
				let inherent = ismp_parachain_inherent::ConsensusInherentProvider::create(
					parent,
					client,
					relay_chain_interface,
				)
				.await?;

				Ok(inherent)
			}
		}
	};

	let params = AuraParams {
		create_inherent_data_providers,
		block_import,
		para_client: client.clone(),
		para_backend: backend,
		relay_client: relay_chain_interface,
		code_hash_provider: move |block_hash| {
			client.code_at(block_hash).ok().map(|c| ValidationCode::from(c).hash())
		},
		keystore,
		collator_key,
		para_id,
		overseer_handle,
		relay_chain_slot_duration,
		proposer,
		collator_service,
		authoring_duration: Duration::from_millis(2000),
		reinitialize: false,
		max_pov_percentage: None,
	};

	let fut = aura::run::<Block, sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _, _, _>(
		params,
	);
	task_manager.spawn_essential_handle().spawn("aura", None, fut);

	Ok(())
}

/// Start a parachain node.
pub async fn start_parachain_node<RuntimeApi>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	para_id: ParaId,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient<RuntimeApi>>)>
where
	RuntimeApi: ConstructRuntimeApi<Block, ParachainClient<RuntimeApi>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi: RuntimeApiExt<RuntimeApi>,
{
	start_node_impl::<RuntimeApi, _>(
		parachain_config,
		polkadot_config,
		collator_options,
		para_id,
		start_consensus::<RuntimeApi>,
		hwbench,
	)
	.await
}

#[cfg(not(feature = "ismp"))]
mod traits {
	use pop_runtime_common::{AccountId, AuraId, Balance, Block, Nonce};
	use sp_core::Pair;
	use sp_runtime::app_crypto::AppCrypto;

	// Trait alias refactored out from above to simplify repeated trait bounds
	pub(crate) trait RuntimeApiExt<RuntimeApi>:
		sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ sp_consensus_aura::AuraApi<Block, <<AuraId as AppCrypto>::Pair as Pair>::Public>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
	{
	}
	impl<
			T: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
				+ sp_api::Metadata<Block>
				+ sp_session::SessionKeys<Block>
				+ sp_api::ApiExt<Block>
				+ sp_offchain::OffchainWorkerApi<Block>
				+ sp_block_builder::BlockBuilder<Block>
				+ cumulus_primitives_core::CollectCollationInfo<Block>
				+ sp_consensus_aura::AuraApi<Block, <<AuraId as AppCrypto>::Pair as Pair>::Public>
				+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
				+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
				+ cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>,
			RuntimeApi,
		> RuntimeApiExt<RuntimeApi> for T
	{
	}
}

#[cfg(feature = "ismp")]
mod traits {
	use pop_runtime_common::{AccountId, AuraId, Balance, Block, Nonce};
	use sp_core::{Pair, H256};
	use sp_runtime::app_crypto::AppCrypto;

	// Trait alias refactored out from above to simplify repeated trait bounds
	pub(crate) trait RuntimeApiExt<RuntimeApi>:
		sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::Metadata<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_api::ApiExt<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ cumulus_primitives_core::CollectCollationInfo<Block>
		+ sp_consensus_aura::AuraApi<Block, <<AuraId as AppCrypto>::Pair as Pair>::Public>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
		+ ismp_parachain_runtime_api::IsmpParachainApi<Block>
		+ pallet_ismp_runtime_api::IsmpRuntimeApi<Block, H256>
	{
	}
	impl<
			T: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
				+ sp_api::Metadata<Block>
				+ sp_session::SessionKeys<Block>
				+ sp_api::ApiExt<Block>
				+ sp_offchain::OffchainWorkerApi<Block>
				+ sp_block_builder::BlockBuilder<Block>
				+ cumulus_primitives_core::CollectCollationInfo<Block>
				+ sp_consensus_aura::AuraApi<Block, <<AuraId as AppCrypto>::Pair as Pair>::Public>
				+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
				+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
				+ cumulus_primitives_aura::AuraUnincludedSegmentApi<Block>
				+ ismp_parachain_runtime_api::IsmpParachainApi<Block>
				+ pallet_ismp_runtime_api::IsmpRuntimeApi<Block, H256>,
			RuntimeApi,
		> RuntimeApiExt<RuntimeApi> for T
	{
	}
}

use traits::RuntimeApiExt;

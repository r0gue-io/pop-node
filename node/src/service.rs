//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

// std
use std::{sync::Arc, time::Duration};

use cumulus_client_cli::CollatorOptions;
// Local Runtime Types
use runtime::{
    opaque::{Block, Hash},
    RuntimeApi,
};

// Cumulus Imports
use cumulus_client_collator::service::CollatorService;
use cumulus_client_consensus_common::ParachainBlockImport as TParachainBlockImport;
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_service::{
    build_network, build_relay_chain_interface, prepare_node_config, start_relay_chain_tasks,
    BuildNetworkParams, CollatorSybilResistance, DARecoveryProfile, StartRelayChainTasksParams,
};
use cumulus_primitives_core::{relay_chain::CollatorPair, ParaId};
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainInterface};

// Substrate Imports
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
use sc_client_api::Backend;
use sc_consensus::ImportQueue;
use sc_executor::{
    HeapAllocStrategy, NativeElseWasmExecutor, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY,
};
use sc_network::NetworkBlock;
use sc_network_sync::SyncingService;
use sc_service::{Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_keystore::KeystorePtr;
use substrate_prometheus_endpoint::Registry;

/// Native executor type.
pub struct ParachainNativeExecutor;

impl sc_executor::NativeExecutionDispatch for ParachainNativeExecutor {
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        runtime::native_version()
    }
}

type ParachainExecutor = NativeElseWasmExecutor<ParachainNativeExecutor>;

type ParachainClient = TFullClient<Block, RuntimeApi, ParachainExecutor>;

type ParachainBackend = TFullBackend<Block>;

type ParachainBlockImport = TParachainBlockImport<Block, Arc<ParachainClient>, ParachainBackend>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial(
    config: &Configuration,
) -> Result<
    PartialComponents<
        ParachainClient,
        ParachainBackend,
        (),
        sc_consensus::DefaultImportQueue<Block>,
        sc_transaction_pool::FullPool<Block, ParachainClient>,
        (
            ParachainBlockImport,
            Option<Telemetry>,
            Option<TelemetryWorkerHandle>,
        ),
    >,
    sc_service::Error,
> {
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
        .default_heap_pages
        .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static {
            extra_pages: h as _,
        });

    let wasm = WasmExecutor::builder()
        .with_execution_method(config.wasm_method)
        .with_onchain_heap_alloc_strategy(heap_pages)
        .with_offchain_heap_alloc_strategy(heap_pages)
        .with_max_runtime_instances(config.max_runtime_instances)
        .with_runtime_cache_size(config.runtime_cache_size)
        .build();

    let executor = ParachainExecutor::new_with_wasm_executor(wasm);

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let block_import = ParachainBlockImport::new(client.clone(), backend.clone());

    let import_queue = build_import_queue(
        client.clone(),
        block_import.clone(),
        config,
        telemetry.as_ref().map(|telemetry| telemetry.handle()),
        &task_manager,
    )?;

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
async fn start_node_impl(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    para_id: ParaId,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial(&parachain_config)?;
    let (block_import, mut telemetry, telemetry_worker_handle) = params.other;
    let net_config = sc_network::config::FullNetworkConfiguration::new(&parachain_config.network);

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
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let import_queue_service = params.import_queue.service();

    let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
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

        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-work",
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                keystore: Some(params.keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: network.clone(),
                is_validator: parachain_config.role.is_authority(),
                enable_http_requests: false,
                custom_extensions: move |_| vec![],
            })
            .run(client.clone(), task_manager.spawn_handle())
            .boxed(),
        );
    }

    let rpc_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: transaction_pool.clone(),
                deny_unsafe,
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
        backend,
        network: network.clone(),
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
        if !SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench) && validator {
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
            block_import,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|t| t.handle()),
            &task_manager,
            relay_chain_interface.clone(),
            transaction_pool,
            sync_service.clone(),
            params.keystore_container.keystore(),
            relay_chain_slot_duration,
            para_id,
            collator_key.expect("Command line arguments do not allow this. qed"),
            overseer_handle,
            announce_block,
        )?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}

/// Build the import queue for the parachain runtime.
fn build_import_queue(
    client: Arc<ParachainClient>,
    block_import: ParachainBlockImport,
    config: &Configuration,
    telemetry: Option<TelemetryHandle>,
    task_manager: &TaskManager,
) -> Result<sc_consensus::DefaultImportQueue<Block>, sc_service::Error> {
    let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

    Ok(
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
            slot_duration,
            &task_manager.spawn_essential_handle(),
            config.prometheus_registry(),
            telemetry,
        ),
    )
}

fn start_consensus(
    client: Arc<ParachainClient>,
    block_import: ParachainBlockImport,
    prometheus_registry: Option<&Registry>,
    telemetry: Option<TelemetryHandle>,
    task_manager: &TaskManager,
    relay_chain_interface: Arc<dyn RelayChainInterface>,
    transaction_pool: Arc<sc_transaction_pool::FullPool<Block, ParachainClient>>,
    sync_oracle: Arc<SyncingService<Block>>,
    keystore: KeystorePtr,
    relay_chain_slot_duration: Duration,
    para_id: ParaId,
    collator_key: CollatorPair,
    overseer_handle: OverseerHandle,
    announce_block: Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
) -> Result<(), sc_service::Error> {
    use cumulus_client_consensus_aura::collators::basic::Params as BasicAuraParams;

    // NOTE: because we use Aura here explicitly, we can use `CollatorSybilResistance::Resistant`
    // when starting the network.

    let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

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

    let params = BasicAuraParams {
        block_import,
        para_client: client.clone(),
        relay_client: relay_chain_interface.clone(),
        create_inherent_data_providers: move |_, (relay_parent, validation_data)| {
            let relay_chain_interface = relay_chain_interface.clone();
            let client_clone = client.clone();
            async move {
                let ismp_consensus_inherent =
                    ismp_parachain_inherent::ConsensusInherentProvider::create(
                        client_clone.clone(),
                        relay_parent,
                        &relay_chain_interface,
                        validation_data,
                    )
                    .await?;
                Ok((ismp_consensus_inherent,))
            }
        },
        sync_oracle,
        keystore,
        collator_key,
        para_id,
        overseer_handle,
        slot_duration,
        relay_chain_slot_duration,
        proposer,
        collator_service,
        // Very limited proposal time.
        authoring_duration: Duration::from_millis(500),
        collation_request_receiver: None,
    };

    let fut =
        aura::run::<Block, sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _, _>(params);
    task_manager
        .spawn_essential_handle()
        .spawn("aura", None, fut);

    Ok(())
}

/// Start a parachain node.
pub async fn start_parachain_node(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    collator_options: CollatorOptions,
    para_id: ParaId,
    hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
    start_node_impl(
        parachain_config,
        polkadot_config,
        collator_options,
        para_id,
        hwbench,
    )
    .await
}

// Hack: copied from cumulus_client_consensus_aura to support ISMP inherent after api change for async backing
mod aura {
    use codec::{Codec, Decode, Encode};
    use cumulus_client_collator::service::ServiceInterface;
    use cumulus_client_consensus_aura::{
        collator::{Params as CollatorParams, SlotClaim},
        collators::basic::Params as BasicParams,
    };
    use cumulus_client_consensus_common::ParachainBlockImportMarker;
    use cumulus_client_consensus_common::ParachainCandidate;
    use cumulus_client_consensus_proposer::ProposerInterface;
    use cumulus_primitives_core::{
        BlockT, CollectCollationInfo, DigestItem, ParaId, ParachainBlockData,
    };
    use cumulus_primitives_parachain_inherent::ParachainInherentData;
    use cumulus_relay_chain_interface::{PHash, RelayChainInterface};
    use futures::{StreamExt, TryFutureExt};
    use polkadot_node_primitives::MaybeCompressedPoV;
    use polkadot_node_primitives::{Collation, CollationResult};
    use polkadot_primitives::PersistedValidationData;
    use sc_client_api::{AuxStore, BlockBackend, BlockOf};
    use sc_consensus::BlockImport;
    use sp_api::ProvideRuntimeApi;
    use sp_blockchain::HeaderBackend;
    use sp_consensus_aura::AuraApi;
    use sp_core::Pair;
    use sp_inherents::InherentDataProvider;
    use sp_inherents::{CreateInherentDataProviders, InherentData};
    use sp_keystore::KeystorePtr;
    use sp_runtime::app_crypto::AppPublic;
    use sp_runtime::{
        generic::Digest,
        traits::{Header as HeaderT, Member},
    };
    use sp_timestamp::Timestamp;
    use std::future::Future;
    use std::{convert::TryFrom, error::Error, time::Duration};

    const LOG_TARGET: &str = "aura::cumulus";

    /// Run bare Aura consensus as a relay-chain-driven collator.
    pub fn run<Block, P, BI, CIDP, Client, RClient, SO, Proposer, CS>(
        params: BasicParams<BI, CIDP, Client, RClient, SO, Proposer, CS>,
    ) -> impl Future<Output = ()> + Send + 'static
    where
        Block: BlockT + Send,
        Client: ProvideRuntimeApi<Block>
            + BlockOf
            + AuxStore
            + HeaderBackend<Block>
            + BlockBackend<Block>
            + Send
            + Sync
            + 'static,
        Client::Api: AuraApi<Block, P::Public> + CollectCollationInfo<Block>,
        RClient: RelayChainInterface + Send + Clone + 'static,
        CIDP: CreateInherentDataProviders<Block, (PHash, PersistedValidationData)> + Send + 'static,
        CIDP::InherentDataProviders: Send,
        BI: BlockImport<Block> + ParachainBlockImportMarker + Send + Sync + 'static,
        SO: sp_consensus::SyncOracle + Send + Sync + Clone + 'static,
        Proposer: ProposerInterface<Block> + Send + Sync + 'static,
        CS: ServiceInterface<Block> + Send + Sync + 'static,
        P: Pair,
        P::Public: AppPublic + Member + Codec,
        P::Signature: TryFrom<Vec<u8>> + Member + Codec,
    {
        async move {
            let mut collation_requests = match params.collation_request_receiver {
                Some(receiver) => receiver,
                None => {
                    cumulus_client_collator::relay_chain_driven::init(
                        params.collator_key,
                        params.para_id,
                        params.overseer_handle,
                    )
                    .await
                }
            };

            let mut collator = {
                let params = CollatorParams {
                    create_inherent_data_providers: params.create_inherent_data_providers,
                    block_import: params.block_import,
                    relay_client: params.relay_client.clone(),
                    keystore: params.keystore.clone(),
                    para_id: params.para_id,
                    proposer: params.proposer,
                    collator_service: params.collator_service,
                };

                Collator::<Block, P, _, _, _, _, _>::new(params)
            };

            while let Some(request) = collation_requests.next().await {
                macro_rules! reject_with_error {
				($err:expr) => {{
					request.complete(None);
					tracing::error!(target: LOG_TARGET, err = ?{ $err });
					continue;
				}};
			}

                macro_rules! try_request {
                    ($x:expr) => {{
                        match $x {
                            Ok(x) => x,
                            Err(e) => reject_with_error!(e),
                        }
                    }};
                }

                let validation_data = request.persisted_validation_data();

                let parent_header = try_request!(Block::Header::decode(
                    &mut &validation_data.parent_head.0[..]
                ));

                let parent_hash = parent_header.hash();

                if !collator
                    .collator_service()
                    .check_block_status(parent_hash, &parent_header)
                {
                    continue;
                }

                let relay_parent_header = match params
                    .relay_client
                    .header(cumulus_primitives_core::relay_chain::BlockId::hash(
                        *request.relay_parent(),
                    ))
                    .await
                {
                    Err(e) => reject_with_error!(e),
                    Ok(None) => continue, // sanity: would be inconsistent to get `None` here
                    Ok(Some(h)) => h,
                };

                let claim = match cumulus_client_consensus_aura::collator::claim_slot::<_, _, P>(
                    &*params.para_client,
                    parent_hash,
                    &relay_parent_header,
                    params.slot_duration,
                    params.relay_chain_slot_duration,
                    &params.keystore,
                )
                .await
                {
                    Ok(None) => continue,
                    Ok(Some(c)) => c,
                    Err(e) => reject_with_error!(e),
                };

                let (parachain_inherent_data, other_inherent_data) = try_request!(
                    collator
                        .create_inherent_data(
                            *request.relay_parent(),
                            &validation_data,
                            parent_hash,
                            claim.timestamp(),
                        )
                        .await
                );

                let (collation, _, post_hash) = try_request!(
                    collator
                        .collate(
                            &parent_header,
                            &claim,
                            None,
                            (parachain_inherent_data, other_inherent_data),
                            params.authoring_duration,
                            // Set the block limit to 50% of the maximum PoV size.
                            //
                            // TODO: If we got benchmarking that includes the proof size,
                            // we should be able to use the maximum pov size.
                            (validation_data.max_pov_size / 2) as usize,
                        )
                        .await
                );

                let result_sender =
                    Some(collator.collator_service().announce_with_barrier(post_hash));
                request.complete(Some(CollationResult {
                    collation,
                    result_sender,
                }));
            }
        }
    }

    /// A utility struct for writing collation logic that makes use of Aura entirely
    /// or in part. See module docs for more details.
    pub struct Collator<Block, P, BI, CIDP, RClient, Proposer, CS> {
        create_inherent_data_providers: CIDP,
        block_import: BI,
        relay_client: RClient,
        keystore: KeystorePtr,
        para_id: ParaId,
        proposer: Proposer,
        collator_service: CS,
        _marker: std::marker::PhantomData<(Block, Box<dyn Fn(P) + Send + Sync + 'static>)>,
    }

    impl<Block, P, BI, CIDP, RClient, Proposer, CS> Collator<Block, P, BI, CIDP, RClient, Proposer, CS>
    where
        Block: BlockT,
        RClient: RelayChainInterface,
        CIDP: CreateInherentDataProviders<Block, (PHash, PersistedValidationData)> + 'static,
        BI: BlockImport<Block> + ParachainBlockImportMarker + Send + Sync + 'static,
        Proposer: ProposerInterface<Block>,
        CS: ServiceInterface<Block>,
        P: Pair,
        P::Public: AppPublic + Member,
        P::Signature: TryFrom<Vec<u8>> + Member + Codec,
    {
        /// Instantiate a new instance of the `Aura` manager.
        pub fn new(params: CollatorParams<BI, CIDP, RClient, Proposer, CS>) -> Self {
            Collator {
                create_inherent_data_providers: params.create_inherent_data_providers,
                block_import: params.block_import,
                relay_client: params.relay_client,
                keystore: params.keystore,
                para_id: params.para_id,
                proposer: params.proposer,
                collator_service: params.collator_service,
                _marker: std::marker::PhantomData,
            }
        }

        /// Explicitly creates the inherent data for parachain block authoring and overrides
        /// the timestamp inherent data with the one provided, if any.
        pub async fn create_inherent_data(
            &self,
            relay_parent: PHash,
            validation_data: &PersistedValidationData,
            parent_hash: Block::Hash,
            timestamp: impl Into<Option<Timestamp>>,
        ) -> Result<(ParachainInherentData, InherentData), Box<dyn Error + Send + Sync + 'static>>
        {
            let paras_inherent_data = ParachainInherentData::create_at(
                relay_parent,
                &self.relay_client,
                validation_data,
                self.para_id,
            )
            .await;

            let paras_inherent_data = match paras_inherent_data {
                Some(p) => p,
                None => {
                    return Err(format!(
                        "Could not create paras inherent data at {:?}",
                        relay_parent
                    )
                    .into())
                }
            };

            let mut other_inherent_data = self
                .create_inherent_data_providers
                .create_inherent_data_providers(
                    parent_hash,
                    (relay_parent, validation_data.clone()),
                )
                .map_err(|e| e as Box<dyn Error + Send + Sync + 'static>)
                .await?
                .create_inherent_data()
                .await
                .map_err(Box::new)?;

            if let Some(timestamp) = timestamp.into() {
                other_inherent_data.replace_data(sp_timestamp::INHERENT_IDENTIFIER, &timestamp);
            }

            Ok((paras_inherent_data, other_inherent_data))
        }

        /// Propose, seal, and import a block, packaging it into a collation.
        ///
        /// Provide the slot to build at as well as any other necessary pre-digest logs,
        /// the inherent data, and the proposal duration and PoV size limits.
        ///
        /// The Aura pre-digest should not be explicitly provided and is set internally.
        ///
        /// This does not announce the collation to the parachain network or the relay chain.
        pub async fn collate(
            &mut self,
            parent_header: &Block::Header,
            slot_claim: &SlotClaim<P::Public>,
            additional_pre_digest: impl Into<Option<Vec<DigestItem>>>,
            inherent_data: (ParachainInherentData, InherentData),
            proposal_duration: Duration,
            max_pov_size: usize,
        ) -> Result<
            (Collation, ParachainBlockData<Block>, Block::Hash),
            Box<dyn Error + Send + 'static>,
        > {
            let mut digest = additional_pre_digest.into().unwrap_or_default();
            digest.push(slot_claim.pre_digest().clone());

            let proposal = self
                .proposer
                .propose(
                    &parent_header,
                    &inherent_data.0,
                    inherent_data.1,
                    Digest { logs: digest },
                    proposal_duration,
                    Some(max_pov_size),
                )
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;

            let sealed_importable = cumulus_client_consensus_aura::collator::seal::<_, P>(
                proposal.block,
                proposal.storage_changes,
                &slot_claim.author_pub(),
                &self.keystore,
            )
            .map_err(|e| e as Box<dyn Error + Send>)?;

            let post_hash = sealed_importable.post_hash();
            let block = Block::new(
                sealed_importable.post_header(),
                sealed_importable
                    .body
                    .as_ref()
                    .expect("body always created with this `propose` fn; qed")
                    .clone(),
            );

            self.block_import
                .import_block(sealed_importable)
                .map_err(|e| Box::new(e) as Box<dyn Error + Send>)
                .await?;

            if let Some((collation, block_data)) = self.collator_service.build_collation(
                parent_header,
                post_hash,
                ParachainCandidate {
                    block,
                    proof: proposal.proof,
                },
            ) {
                tracing::info!(
                    target: LOG_TARGET,
                    "PoV size {{ header: {}kb, extrinsics: {}kb, storage_proof: {}kb }}",
                    block_data.header().encode().len() as f64 / 1024f64,
                    block_data.extrinsics().encode().len() as f64 / 1024f64,
                    block_data.storage_proof().encode().len() as f64 / 1024f64,
                );

                if let MaybeCompressedPoV::Compressed(ref pov) = collation.proof_of_validity {
                    tracing::info!(
                        target: LOG_TARGET,
                        "Compressed PoV size: {}kb",
                        pov.block_data.0.len() as f64 / 1024f64,
                    );
                }

                Ok((collation, block_data, post_hash))
            } else {
                Err(
                    Box::<dyn Error + Send + Sync>::from("Unable to produce collation")
                        as Box<dyn Error + Send>,
                )
            }
        }

        /// Get the underlying collator service.
        pub fn collator_service(&self) -> &CS {
            &self.collator_service
        }
    }
}

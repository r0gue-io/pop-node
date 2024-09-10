//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
use pop_runtime_common::{AccountId, Balance, Block, Nonce};
use sc_client_api::{AuxStore, BlockBackend, ProofProvider};
pub use sc_rpc::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
#[cfg(feature = "ismp")]
use sp_core::H256;
use substrate_frame_rpc_system::{System, SystemApiServer};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;

/// Full client dependencies
pub struct FullDeps<C, P, B> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// Backend used by the node.
	pub backend: Arc<B>,
}

/// Instantiate all RPC extensions.
#[cfg(not(feature = "ismp"))]
pub fn create_full<C, P, B>(
	deps: FullDeps<C, P, B>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ BlockBackend<Block>
		+ ProofProvider<Block>
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Send
		+ Sync
		+ 'static,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + Sync + Send + 'static,
	B: sc_client_api::Backend<Block> + Send + Sync + 'static,
	B::State: sc_client_api::StateBackend<sp_runtime::traits::HashingFor<Block>>,
{
	let mut module = RpcExtension::new(());
	let FullDeps { client, pool, deny_unsafe, backend: _ } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	Ok(module)
}

/// Instantiate all RPC extensions.
#[cfg(feature = "ismp")]
pub fn create_full<C, P, B>(
	deps: FullDeps<C, P, B>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ BlockBackend<Block>
		+ ProofProvider<Block>
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Send
		+ Sync
		+ 'static,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
	C::Api: BlockBuilder<Block>,
	C::Api: pallet_ismp_runtime_api::IsmpRuntimeApi<Block, H256>,
	P: TransactionPool + Sync + Send + 'static,
	B: sc_client_api::Backend<Block> + Send + Sync + 'static,
	B::State: sc_client_api::StateBackend<sp_runtime::traits::HashingFor<Block>>,
{
	let mut module = RpcExtension::new(());
	let FullDeps { client, pool, deny_unsafe, backend } = deps;

	module.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

	use pallet_ismp_rpc::{IsmpApiServer, IsmpRpcHandler};
	module.merge(IsmpRpcHandler::new(client, backend.clone())?.into_rpc())?;

	Ok(module)
}

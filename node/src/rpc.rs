//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use std::sync::Arc;

//use parachain_template_runtime::{opaque::Block, AccountId, Balance, Content, Did, Nonce};
use parachain_template_runtime::{opaque::Block, AccountId, Balance, Nonce};
use shared::types::ContentId;

use polkadot_sdk::*;

use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpsee::RpcModule<()>;

/// Full client dependencies
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcExtension, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + HeaderMetadata<Block, Error = BlockChainError>
        + Send
        + Sync
        + 'static,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: BlockBuilder<Block>,
    C::Api: pallet_content_registry_rpc::ContentRegistryApi<Block, ContentId>,
    C::Api: pallet_context_court_rpc::ContextCourtApi<Block, ContentId>,
    P: TransactionPool + Sync + Send + 'static,
{
    use pallet_content_registry_rpc::{PalletContentRegistry, PalletContentRegistryApiServer};
    use pallet_context_court_rpc::{PalletContextCourt, PalletContextCourtApiServer};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let mut module = RpcExtension::new(());

    let FullDeps { client, pool } = deps;

    module.merge(System::new(client.clone(), pool).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;
    module.merge(PalletContentRegistry::new(client.clone()).into_rpc())?;
    module.merge(PalletContextCourt::new(client.clone()).into_rpc())?;
    Ok(module)
}

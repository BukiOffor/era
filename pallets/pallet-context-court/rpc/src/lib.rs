#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
pub use context_runtime_api::PalletContextCourtApi as ContextCourtApi;
use jsonrpsee::{core::RpcResult, proc_macros::rpc, types::error::ErrorObject};
use polkadot_sdk::*;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Custom {
    code: u32,
    sum: u32,
}

#[rpc(client, server)]
pub trait PalletContextCourtApi<BlockHash, ContentId: Codec> {
    /// get the number of accounts that have approved a particular call hash
    #[method(name = "context_HasDispute")]
    fn has_dispute(&self, id: ContentId, at: Option<BlockHash>) -> RpcResult<bool>;
}

/// A struct that implements the `TemplateApi`.
pub struct PalletContextCourt<C, Block> {
    // If you have more generics, no need to TemplatePallet<C, M, N, P, ...>
    // just use a tuple like TemplatePallet<C, (M, N, P, ...)>
    client: Arc<C>,
    _marker: std::marker::PhantomData<Block>,
}

impl<C, Block> PalletContextCourt<C, Block> {
    /// Create new `TemplatePallet` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, ContentId: Codec> PalletContextCourtApiServer<<Block as BlockT>::Hash, ContentId>
    for PalletContextCourt<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ContextCourtApi<Block, ContentId>,
{
    fn has_dispute(
        &self,
        id: ContentId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<bool> {
        let api = self.client.runtime_api();
        let block_hash = at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash);
        api.has_dispute(block_hash, id)
            .map_err(runtime_error_into_rpc_err)
    }
}

const RUNTIME_ERROR: i32 = 1;

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err<'a>(err: impl std::fmt::Debug) -> ErrorObject<'a> {
    ErrorObject::owned(RUNTIME_ERROR, "Runtime error", Some(format!("{:?}", err)))
}


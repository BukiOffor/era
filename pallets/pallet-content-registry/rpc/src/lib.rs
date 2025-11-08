// #![cfg_attr(not(feature = "std"), no_std)]

// use polkadot_sdk::*;
// use codec::Codec;
// pub use content_runtime_api::PalletContentRegistryApi as ContentRegistryApi;
// use jsonrpsee::{
//     core::{Error as JsonRpseeError, RpcResult},
//     proc_macros::rpc,
//     types::error::{CallError, ErrorObject},
// };
// use sp_api::ProvideRuntimeApi;
// use sp_blockchain::HeaderBackend;
// use sp_runtime::traits::Block as BlockT;
// use std::sync::Arc;

// #[derive(serde::Deserialize, serde::Serialize)]
// pub struct Custom {
//     code: u32,
//     sum: u32,
// }

// #[rpc(client, server)]
// pub trait PalletContentRegistryApi<BlockHash, Did: Codec, ContentId: Codec, Content: Codec> {
//     /// get the number of accounts that have approved a particular call hash
//     #[method(name = "multi_NumberOfAccountsHasApprovedCall")]
//     fn check_proof_of_reality(&self, id: ContentId, at: Option<BlockHash>) -> RpcResult<bool>;
// }

// /// A struct that implements the `TemplateApi`.
// pub struct PalletContentRegistry<C, Block> {
//     // If you have more generics, no need to TemplatePallet<C, M, N, P, ...>
//     // just use a tuple like TemplatePallet<C, (M, N, P, ...)>
//     client: Arc<C>,
//     _marker: std::marker::PhantomData<Block>,
// }

// impl<C, Block> PalletContentRegistry<C, Block> {
//     /// Create new `TemplatePallet` instance with the given reference to the client.
//     pub fn new(client: Arc<C>) -> Self {
//         Self {
//             client,
//             _marker: Default::default(),
//         }
//     }
// }

// impl<C, Block, Did: Codec, ContentId: Codec, Content: Codec>
//     PalletContentRegistryApiServer<<Block as BlockT>::Hash, Did, ContentId, Content>
//     for PalletContentRegistry<C, Block>
// where
//     Block: BlockT ,
//     C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
//     C::Api: ContentRegistryApi<Block, Did, ContentId, Content>,
// {
//     fn check_proof_of_reality(
//         &self,
//         id: ContentId,
//         at: Option<<Block as BlockT>::Hash>,
//     ) -> RpcResult<bool> {
//         let api = self.client.runtime_api();
//         let block_hash = at.unwrap_or_else(||
// 			// If the block hash is not supplied assume the best block.
// 			self.client.info().best_hash);
//         api.check_proof_of_reality(block_hash, id)
//             .map_err(runtime_error_into_rpc_err)
//     }
// }

// const RUNTIME_ERROR: i32 = 1;

// /// Converts a runtime trap into an RPC error.
// fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
//     CallError::Custom(ErrorObject::owned(
//         RUNTIME_ERROR,
//         "Runtime error",
//         Some(format!("{:?}", err)),
//     ))
//     .into()
// }

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
pub use content_runtime_api::PalletContentRegistryApi as ContentRegistryApi;
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
pub trait PalletContentRegistryApi<BlockHash, ContentId: Codec> {
    /// get the number of accounts that have approved a particular call hash
    #[method(name = "content_CheckProofOfReality")]
    fn check_proof_of_reality(&self, id: ContentId, at: Option<BlockHash>) -> RpcResult<bool>;
}

/// A struct that implements the `TemplateApi`.
pub struct PalletContentRegistry<C, Block> {
    // If you have more generics, no need to TemplatePallet<C, M, N, P, ...>
    // just use a tuple like TemplatePallet<C, (M, N, P, ...)>
    client: Arc<C>,
    _marker: std::marker::PhantomData<Block>,
}

impl<C, Block> PalletContentRegistry<C, Block> {
    /// Create new `TemplatePallet` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, ContentId: Codec> PalletContentRegistryApiServer<<Block as BlockT>::Hash, ContentId>
    for PalletContentRegistry<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: ContentRegistryApi<Block, ContentId>,
{
    fn check_proof_of_reality(
        &self,
        id: ContentId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> RpcResult<bool> {
        let api = self.client.runtime_api();
        let block_hash = at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash);
        api.check_proof_of_reality(block_hash, id)
            .map_err(runtime_error_into_rpc_err)
    }
}

const RUNTIME_ERROR: i32 = 1;

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err<'a>(err: impl std::fmt::Debug) -> ErrorObject<'a> {
    ErrorObject::owned(RUNTIME_ERROR, "Runtime error", Some(format!("{:?}", err)))
}

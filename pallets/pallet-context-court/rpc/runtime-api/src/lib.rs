#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

//use alloc::vec::Vec;
use codec::Codec;

polkadot_sdk::sp_api::decl_runtime_apis! {
    /// This trait contains all the Api's that can be called into from the runtime
    /// into our pallet. To read or perform certain state actions in our blockchain
    pub trait PalletContextCourtApi<ContentId>
    where
        ContentId: Codec,
    {
        fn has_dispute(id: ContentId) -> bool;
    }
}
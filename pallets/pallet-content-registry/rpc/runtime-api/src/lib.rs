#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

//use alloc::vec::Vec;
use codec::Codec;

// polkadot_sdk::sp_api::decl_runtime_apis! {
// 	/// This trait contains all the Api's that can be called into from the runtime
// 	/// into our pallet. To read or perform certain state actions in our blockchain
// 	pub trait PalletContentRegistryApi<Did,ContentId,Content>
//     where 
//         ContentId: Codec, 
//         Did: Codec, 
//         Content: Codec
//     {
//         fn check_proof_of_reality(id: ContentId) -> bool;
// 	}
// }

polkadot_sdk::sp_api::decl_runtime_apis! {
	/// This trait contains all the Api's that can be called into from the runtime
	/// into our pallet. To read or perform certain state actions in our blockchain
	pub trait PalletContentRegistryApi<ContentId,>
    where 
        ContentId: Codec,
    {
        fn check_proof_of_reality(id: ContentId) -> bool;
	}
}
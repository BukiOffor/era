#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[frame::pallet]
pub mod pallet {
    use frame::prelude::*;
    use shared::traits::identity::DidManager;
    use shared::types::BaseRight;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: crate::weights::WeightInfo;
        /// Type used to represent a Decentralized Identifier (DID)
        type Did: Parameter + Member + MaxEncodedLen + Clone + Eq + Default;
        /// Type used to represent a device identifier or metadata
        type Device: Parameter + Member + MaxEncodedLen + Clone + Eq + Default;
        
        type ContentId: Parameter + Member + MaxEncodedLen + Clone + Eq + Default;
        
        type GivenRight: Parameter + Member + MaxEncodedLen + Clone + Eq + Default + From<BaseRight> + Into<BaseRight>;

        type DidRegistry: DidManager<Self::AccountId, Self::Did, Self::Device, Self::GivenRight>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    pub struct Content<T: Config> {
        pub content_id: T::ContentId,
        pub exists_from: BlockNumberFor<T>,
        pub did : T::Did,
        pub signer: T::AccountId,
        pub content: T::Content,
        pub content_type: T::ContentType,
        pub content_description: T::ContentDescription,
        pub content_metadata: T::ContentMetadata,
        pub device: T::Device,
    }
    // hash of the content is the content_id, so we can check if it exists
    // did -> cid -> bool
    // did -> Vec<ContentId>
    // cid -> Content


    /// Pallets use events to inform users when important changes are made.
    /// <https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/your_first_pallet/index.html#event-and-error>
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// We usually use passive tense for events.
        SomethingStored {
            block_number: BlockNumberFor<T>,
            who: T::AccountId,
        },
    }


    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

 
    #[pallet::call]
    impl<T: Config> Pallet<T> {

    }
}

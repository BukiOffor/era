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
    use frame::prelude::{OptionQuery, ValueQuery, *};
    use shared::traits::identity::DidManager;
    use shared::types::{BaseRight, ContentId};

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

        type GivenRight: Parameter
            + Member
            + MaxEncodedLen
            + Clone
            + Eq
            + Default
            + From<BaseRight>
            + Into<BaseRight>;
        type MaxContentInVec: Get<u32>;

        type DidRegistry: DidManager<Self::AccountId, Self::Did, Self::Device, Self::GivenRight>;
        type Content: Parameter
            + Member
            + MaxEncodedLen
            + Clone
            + Eq
            + Default
            + core::hash::Hash
            + Encode
            + Decode
            + TypeInfo;
        //type ContentId: Parameter + Member + MaxEncodedLen + Clone + Eq ;
        type ContentType: Parameter + Member + MaxEncodedLen + Clone + Eq;
        type ContentDescription: Parameter + Member + MaxEncodedLen + Clone + Eq;
        type ContentMetadata: Parameter + Member + MaxEncodedLen + Clone + Eq;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // #[derive(Encode, Decode, Clone, Eq, PartialEq, Default, TypeInfo, MaxEncodedLen, Debug, DecodeWithMemTracking)]
    // pub struct ContentId {
    //     prefix: [u8; 4],
    //     hash: [u8; 32],
    // }

    // impl ContentId {
    //     pub fn new(prefix: &[u8], hash: &[u8]) -> Self {
    //         let mut content_id = ContentId::default();
    //         content_id.prefix.copy_from_slice(prefix);
    //         content_id.hash.copy_from_slice(hash);
    //         content_id
    //     }
    // }

    #[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Eq, Clone, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Proof<T: Config> {
        pub content_id: ContentId,
        pub exists_from: BlockNumberFor<T>,
        pub did: T::Did,
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

    #[pallet::storage]
    #[pallet::getter(fn get_did_contents)]
    pub type DidContents<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Did,
        BoundedVec<ContentId, T::MaxContentInVec>,
        ValueQuery,
    >;

    /// double map for easy lookup if DID content exists.
    #[pallet::storage]
    #[pallet::getter(fn does_did_have_content)]
    pub type DidContentExists<T: Config> =
        StorageDoubleMap<_, Twox64Concat, T::Did, Blake2_128Concat, ContentId, bool, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_content)]
    pub type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, ContentId, Proof<T>, OptionQuery>;

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
        ContentStored {
            block_number: BlockNumberFor<T>,
            who: T::AccountId,
            content_id: ContentId,
            content: T::Content,
            did: T::Did,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        /// Signer does not have the right to perform the action
        SignerDoesNotHaveRight,
        /// Could not get response from trait
        CouldNotGetResponse,
        /// Content already exists
        ContentAlreadyExists,
        /// Could not push content
        CouldNotPushContent,
        /// Device not owned
        DeviceNotOwned,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(5))]
        pub fn create_content(
            origin: OriginFor<T>,
            did: T::Did,
            content: T::Content,
            content_type: T::ContentType,
            content_description: T::ContentDescription,
            content_metadata: T::ContentMetadata,
            device: T::Device,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let is_valid = <T as Config>::DidRegistry::is_signer_valid(
                &who,
                &did,
                &T::GivenRight::from(BaseRight::Impersonate),
            )
            .map_err(|_| Error::<T>::CouldNotGetResponse)?;
            ensure!(is_valid, Error::<T>::SignerDoesNotHaveRight);

            let owned_devices = <T as Config>::DidRegistry::read_did_devices(&did)
                .map_err(|_| Error::<T>::CouldNotGetResponse)?;
            ensure!(owned_devices.contains(&device), Error::<T>::DeviceNotOwned);
            let prefix = b"cid:".as_slice();
            let hash = blake2_256(&content.encode());
            let content_id = ContentId::new(prefix, &hash);

            ensure!(
                !Proofs::<T>::contains_key(&content_id),
                Error::<T>::ContentAlreadyExists
            );
            let ctx = Proof::<T> {
                content_id: content_id.clone(),
                signer: who.clone(),
                content: content.clone(),
                did: did.clone(),
                device,
                content_type,
                content_description,
                content_metadata,
                exists_from: frame_system::Pallet::<T>::block_number(),
            };
            Proofs::<T>::insert(&content_id, &ctx);
            DidContentExists::<T>::insert(&did, &content_id, true);
            DidContents::<T>::try_mutate(&did, |contents| -> DispatchResult {
                contents
                    .try_push(content_id.clone())
                    .map_err(|_| Error::<T>::CouldNotPushContent)?;
                Ok(())
            })?;

            Self::deposit_event(Event::ContentStored {
                block_number: <frame_system::Pallet<T>>::block_number(),
                who,
                content_id,
                content,
                did,
            });
            Ok(())
        }
    }
}

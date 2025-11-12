#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod impl_identity;
pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame::pallet]
pub mod pallet {
    use frame::prelude::*;
    use shared::types::BaseRight;
    
    use frame::prelude::{
        fungible::MutateHold,
     //   *,
    };
 
    type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: crate::weights::WeightInfo;

        /// The maximum length of a string did
        #[pallet::constant]
        type MaxStringLength: Get<u32>;

        type MaxKeySize: Get<u32>;

        type Device: Parameter + Member + MaxEncodedLen + Clone + Eq + Default;

        type Did: Parameter + Member + MaxEncodedLen + Clone + Eq + Default;

        type GivenRight: Parameter
            + Member
            + MaxEncodedLen
            + Clone
            + Eq
            + Default
            + From<BaseRight>
            + Into<BaseRight>;

            type NativeBalance: fungible::Inspect<Self::AccountId>
                    + fungible::Mutate<Self::AccountId>
                    + fungible::hold::Inspect<Self::AccountId, Reason = Self::RuntimeHoldReason>
                    + fungible::hold::Mutate<Self::AccountId>
                    + fungible::freeze::Inspect<Self::AccountId>
                    + fungible::freeze::Mutate<Self::AccountId>;
                
            type RuntimeHoldReason: From<HoldReason>;
            
            type HoldAmount: Get<BalanceOf<Self>>;
            
            
        //type DidRedistry: DidManager<Self::AccountId,Did<Self>,Device<Self>>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);
    
    
    #[pallet::composite_enum]
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo, DecodeWithMemTracking)]
    pub enum HoldReason {
        #[codec(index = 0)]
        AccountCreation,
    }

    #[derive(
        DebugNoBound,
        Encode,
        Decode,
        TypeInfo,
        Clone,
        MaxEncodedLen,
        DecodeWithMemTracking,
        PartialEq,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct Rights<T: Config> {
        /// The type of right that is granted to the user.
        pub(crate) right: T::GivenRight,
        /// The duration of the right that was granted to the user.
        pub(crate) duration: RightDuration<T>,
    }

    #[derive(
        DebugNoBound,
        Encode,
        Decode,
        TypeInfo,
        Clone,
        MaxEncodedLen,
        DecodeWithMemTracking,
        PartialEq,
    )]
    #[scale_info(skip_type_params(T))]
    pub enum RightDuration<T: Config> {
        /// A permanent duration of the right.
        Permanent,
        /// A temporary duration of the right.
        Temporary(Duration<T>),
    }

    #[derive(
        DebugNoBound,
        Encode,
        Decode,
        TypeInfo,
        Clone,
        MaxEncodedLen,
        DecodeWithMemTracking,
        PartialEq,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct Duration<T: Config> {
        /// A block number.
        pub(crate) valid_from_block: BlockNumberFor<T>,
        /// A block number
        pub(crate) valid_to_block: BlockNumberFor<T>,
    }

    #[pallet::storage]
    #[pallet::getter(fn get_signatories)]
    pub type Signatories<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Did,
        BoundedVec<T::AccountId, T::MaxKeySize>,
        OptionQuery,
    >;

    /// Struct representing the rights of a signatory for a DID.
    #[pallet::storage]
    #[pallet::getter(fn get_signatory_rights)]
    pub type SignatoryRights<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::Did,
        Blake2_128Concat,
        T::AccountId, // AccountId of the signatory or caller
        BoundedVec<Rights<T>, T::MaxKeySize>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn get_did_devices)]
    pub type DidDevices<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Did, BoundedVec<T::Device, T::MaxKeySize>, OptionQuery>;

    /// Pallets use events to inform users when important changes are made.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// We usually use passive tense for events.
        RightAdded {
            block_number: BlockNumberFor<T>,
            who: T::AccountId,
            did: T::Did,
            right: Rights<T>,
        },
        DidCreated {
            block_number: BlockNumberFor<T>,
            creator: T::AccountId,
            did: T::Did,
        },
        RightRemoved {
            block_number: BlockNumberFor<T>,
            who: T::AccountId,
            did: T::Did,
            right: T::GivenRight,
        },
        DeviceRegistered {
            block_number: BlockNumberFor<T>,
            who: T::AccountId,
            did: T::Did,
            device: T::Device,
        },
        DeviceRemoved {
            block_number: BlockNumberFor<T>,
            who: T::AccountId,
            did: T::Did,
            device: T::Device,
        },
    }

    /// Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        /// Too many rights for a DID
        TooManyRights,
        /// Signer does not have the right to perform the action
        SignerDoesNotHaveRight,
        /// DID already exists
        DidAlreadyExists,
        /// Too many devices for a DID
        TooManyDevices,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn create_did(
            origin: OriginFor<T>,
            did: T::Did,
            signatories: BoundedVec<T::AccountId, T::MaxKeySize>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // ensure the DID is not already registered
            ensure!(
                !Signatories::<T>::contains_key(&did),
                Error::<T>::DidAlreadyExists
            );

            // prepare Rights struct
            let r = Rights::<T> {
                right: T::GivenRight::from(BaseRight::Update),
                duration: RightDuration::Permanent,
            };
            // get existing vector or default
            let mut list: BoundedVec<Rights<T>, T::MaxKeySize> =
                SignatoryRights::<T>::get(&did, &who).unwrap_or_default();
            list.try_push(r).map_err(|_| Error::<T>::TooManyRights)?;
            // store the DID
            Signatories::<T>::insert(&did, signatories);
            
            <T as Config>::NativeBalance::hold(
                &HoldReason::AccountCreation.into(),
                &who,
                T::HoldAmount::get(),
            )?;
            // emit event
            Self::deposit_event(Event::DidCreated {
                did,
                creator: who,
                block_number: frame_system::Pallet::<T>::block_number(),
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn add_right_for_signatory(
            origin: OriginFor<T>,
            did: T::Did,
            target: T::AccountId,
            right: T::GivenRight,
            duration: RightDuration<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Self::is_valid_signatory(&did, &who, &T::GivenRight::from(BaseRight::Update)),
                Error::<T>::SignerDoesNotHaveRight
            );
            // prepare Rights struct
            let right = Rights::<T> { right, duration };

            // get existing vector or default
            let mut list: BoundedVec<Rights<T>, T::MaxKeySize> =
                SignatoryRights::<T>::get(&did, &target).unwrap_or_default();

            list.try_push(right.clone())
                .map_err(|_| Error::<T>::TooManyRights)?;

            SignatoryRights::<T>::insert(&did, &target, list);

            Self::deposit_event(Event::RightAdded {
                block_number: <frame_system::Pallet<T>>::block_number(),
                who,
                did,
                right,
            });
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn remove_right_for_signatory(
            origin: OriginFor<T>,
            did: T::Did,
            target: T::AccountId,
            right: T::GivenRight,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                Self::is_valid_signatory(&did, &who, &T::GivenRight::from(BaseRight::Update)),
                Error::<T>::SignerDoesNotHaveRight
            );

            // get existing vector or default
            let mut list: BoundedVec<Rights<T>, T::MaxKeySize> =
                SignatoryRights::<T>::get(&did, &target).unwrap_or_default();

            list.retain(|r| r.right != right);

            SignatoryRights::<T>::insert(&did, &target, list);

            Self::deposit_event(Event::RightRemoved {
                block_number: <frame_system::Pallet<T>>::block_number(),
                who,
                did,
                right,
            });
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn register_device(
            origin: OriginFor<T>,
            did: T::Did,
            device: T::Device,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::is_valid_signatory(&did, &who, &T::GivenRight::from(BaseRight::Update)),
                Error::<T>::SignerDoesNotHaveRight
            );
            DidDevices::<T>::try_mutate(&did, |devices| -> DispatchResult {
                devices
                    .take()
                    .unwrap_or_default()
                    .try_push(device.clone())
                    .map_err(|_| Error::<T>::TooManyDevices)?;
                Ok(())
            })?;
            Self::deposit_event(Event::DeviceRegistered {
                block_number: <frame_system::Pallet<T>>::block_number(),
                who,
                did,
                device,
            });
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn remove_device(
            origin: OriginFor<T>,
            did: T::Did,
            device: T::Device,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                Self::is_valid_signatory(&did, &who, &T::GivenRight::from(BaseRight::Update)),
                Error::<T>::SignerDoesNotHaveRight
            );
            DidDevices::<T>::try_mutate(&did, |devices| -> DispatchResult {
                devices.take().unwrap_or_default().retain(|d| d != &device);
                Ok(())
            })?;
            Self::deposit_event(Event::DeviceRemoved {
                block_number: <frame_system::Pallet<T>>::block_number(),
                who,
                did,
                device,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn is_valid_signatory(did: &T::Did, who: &T::AccountId, right: &T::GivenRight) -> bool {
            let signer_rights = SignatoryRights::<T>::get(did, who).unwrap_or_default();
            // Get current block number
            let current_block = <frame_system::Pallet<T>>::block_number();
            signer_rights.iter().any(|r| {
                r.right == *right
                    && match r.duration {
                        RightDuration::Permanent => true,
                        RightDuration::Temporary(Duration {
                            valid_from_block,
                            valid_to_block,
                        }) => valid_from_block <= current_block && current_block <= valid_to_block,
                    }
            })
        }
    }
}

// create identity
// register onchain device: a user can have more than one device

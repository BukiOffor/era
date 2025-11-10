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
    use frame::prelude::{
        fungible::{Inspect, InspectHold, Mutate, MutateHold},
        *,
    };
    use polkadot_sdk::frame_support::traits::Randomness;
    use polkadot_sdk::pallet_insecure_randomness_collective_flip as insecure_randomness;
    use shared::{
        traits::identity::DidManager,
        types::{BaseRight, ContentId},
    };

    /// Define the type for balance used in the pallet.
    type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + insecure_randomness::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: crate::weights::WeightInfo;

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

        /// Reason for holding funds.
        type RuntimeHoldReason: From<HoldReason>;

        type Did: Parameter + Member + MaxEncodedLen + Clone + Eq + Default;

        type Device: Parameter + Member + MaxEncodedLen + Clone + Eq + Default;

        type DidRegistry: DidManager<Self::AccountId, Self::Did, Self::Device, Self::GivenRight>;

        type MaxJurors: Get<u32>;

        type MaxJurorsPerDispute: Get<u32>;

        type MinJurorsPerDispute: Get<u32>;

        type MaxContextLength: Get<u32>;

        type HoldAmount: Get<BalanceOf<Self>>;

        type SlashAmount: Get<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Enum representing reasons for holding funds.
    #[pallet::composite_enum]
    #[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
    pub enum HoldReason {
        #[codec(index = 0)]
        JurorAccountCreation,
        #[codec(index = 1)]
        CallCreation,
    }

    #[pallet::storage]
    #[pallet::getter(fn jurors)]
    pub type Jurors<T: Config> = StorageValue<_, BoundedVec<T::Did, T::MaxJurors>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_juror_native_account)]
    pub type JurorNativeAccountAdmin<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Did, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_juror_duty)]
    pub type JuryDuty<T: Config> = StorageMap<_, Blake2_128Concat, T::Did, ContentId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_amount_of_jury_summoned_for_dispute)]
    pub type JurySummoned<T: Config> = StorageMap<_, Blake2_128Concat, ContentId, u32, OptionQuery>;

    // dispute -> jurors
    #[pallet::storage]
    #[pallet::getter(fn get_dispute)]
    pub type Dispute<T: Config> =
        StorageMap<_, Blake2_128Concat, ContentId, CourtSession<T>, OptionQuery>;

    #[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Clone, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct CourtSession<T: Config> {
        pub jurors: BoundedVec<T::Did, T::MaxJurorsPerDispute>,
        pub started_at: Option<BlockNumberFor<T>>,
        pub ended_at: Option<BlockNumberFor<T>>,
        pub verdict: Verdict<T>,
        pub context: BoundedVec<u8, T::MaxContextLength>,
    }

    #[pallet::storage]
    #[pallet::getter(fn get_escalated_dispute)]
    pub type EscalatedSession<T: Config> =
        StorageMap<_, Blake2_128Concat, ContentId, Escalated<T>, OptionQuery>;

    #[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Clone, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Verdict<T: Config> {
        pub escalated: bool,
        pub decision: Decision,
        pub votes: BoundedVec<VoteRegistry<T>, T::MaxJurorsPerDispute>,
        pub rewarded_and_slashed: bool,
    }

    impl<T: Config> Verdict<T> {
        pub fn new() -> Self {
            Self {
                escalated: false,
                decision: Decision::Pending,
                votes: BoundedVec::new(),
                rewarded_and_slashed: false,
            }
        }
    }

    #[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Eq, Clone, MaxEncodedLen, Default)]
    pub enum Decision {
        Convict,
        Acquittal,
        #[default]
        Pending,
    }

    #[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Eq, Clone, MaxEncodedLen)]
    pub enum Vote {
        Yay,
        Nay,
        Abstain,
    }
    #[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Eq, Clone, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct VoteRegistry<T: Config> {
        pub juror: T::Did,
        pub vote: Vote,
    }

    #[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Eq, Clone, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Escalated<T: Config> {
        pub escalated_at: Option<BlockNumberFor<T>>,
        pub decision_at: Option<BlockNumberFor<T>>,
        pub decision: Decision,
        pub votes: BoundedVec<VoteRegistry<T>, T::MaxJurors>,
    }

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
        JurorRegistered {
            block_number: BlockNumberFor<T>,
            did: T::Did,
            admin: T::AccountId,
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
        /// DidAlreadyExists
        DidAlreadyExists,
        /// MaxJurorsReached
        MaxJurorsReached,
        /// Content already exists in dispute
        ContentAlreadyExists
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn register_did_for_juror(origin: OriginFor<T>, did: T::Did) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let is_valid = <T as Config>::DidRegistry::is_signer_valid(
                &who,
                &did,
                &T::GivenRight::from(BaseRight::Dispute),
            )
            .map_err(|_| Error::<T>::CouldNotGetResponse)?;

            ensure!(is_valid, Error::<T>::SignerDoesNotHaveRight);
            let mut jurors = <Jurors<T>>::get();
            ensure!(!jurors.contains(&did), Error::<T>::DidAlreadyExists);

            jurors
                .try_push(did.clone())
                .map_err(|_| Error::<T>::StorageOverflow)?;

            <Jurors<T>>::set(jurors);
            // Hold the deposit.
            <T as Config>::NativeBalance::hold(
                &HoldReason::JurorAccountCreation.into(),
                &who,
                T::HoldAmount::get(),
            )?;
            Self::deposit_event(Event::JurorRegistered {
                block_number: <frame_system::Pallet<T>>::block_number(),
                admin: who,
                did,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn dispute_content(
            origin: OriginFor<T>,
            did: T::Did,
            content_id: ContentId,
            context: BoundedVec<u8, T::MaxContextLength>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<Dispute<T>>::get(&content_id).is_none(), Error::<T>::ContentAlreadyExists);
            let is_valid = <T as Config>::DidRegistry::is_signer_valid(
                &who,
                &did,
                &T::GivenRight::from(BaseRight::Dispute),
            )
            .map_err(|_| Error::<T>::CouldNotGetResponse)?;
            ensure!(is_valid, Error::<T>::SignerDoesNotHaveRight);
            let session = CourtSession {
                jurors: BoundedVec::new(),
                started_at: None,
                ended_at: None,
                verdict: <Verdict<T>>::new(),
                context,
            };
            Self::summon_jurors(&content_id)?;
            <Dispute<T>>::set(content_id, Some(session));
            Ok(())
        }
        
        // #[pallet::call_index(2)]
        // #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]

        // pub fn exclude_from_duty(){}
        // pub fn finalize_duty_and_get_reward(){}
    }

    impl<T: Config> Pallet<T> {
        /// We strongly advocate that the max number of jurors per session be 40
        pub fn summon_jurors(content_id: &ContentId) -> Result<(), Error<T>> {
            use polkadot_sdk::sp_std::collections::btree_set::BTreeSet;

            if <JurySummoned<T>>::get(content_id).unwrap_or_default()
                > T::MaxJurorsPerDispute::get()
            {
                return Err(Error::<T>::MaxJurorsReached);
            }
            let jurors = <Jurors<T>>::get();
            let (random_value, _) = insecure_randomness::Pallet::<T>::random(&b"summon jurors"[..]);
            let numbers: BTreeSet<u8> = random_value.encode().into_iter().collect();
            let mut summoned = 0;
            for number in numbers {
                if let Some(juror) = jurors.get(number as usize) {
                    if !<JuryDuty<T>>::contains_key(juror) {
                        <JuryDuty<T>>::insert(juror, content_id);
                        summoned += 1;
                    }
                }
            }
            <JurySummoned<T>>::mutate(content_id, |n| n.unwrap_or_default() + summoned);
            Ok(())
        }
    }
}

// user raises a dispute
// calls for a panel vote ->
//  - create or summon jurors
//  - allow jurors to delibrate and vote
//  - at the end of voting period slash the offender.
// add reputation to identity pallet
//
//
// a set of all registered jurors
// if dispute open pot for a number of jurors to register to resolve dispute
// if number of jurors is reached, begin voting period
// voting period depends on the severity of the dispute
// if 80% of verdict is not reached, involve all the jurors
// default to 70% for all juror voting
//
// slash offender and slash jurors that voted against the verdict
//
// if dispute is resolved, reward jurors

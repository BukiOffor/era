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
        fungible::{Mutate, MutateHold},
        *,
    };
    use polkadot_sdk::frame_support::traits::Randomness;
    use polkadot_sdk::pallet_insecure_randomness_collective_flip as insecure_randomness;
    use polkadot_sdk::sp_std::vec::Vec;
    use scale_info::prelude::collections::BTreeSet;
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

        type Did: Parameter + Member + MaxEncodedLen + Clone + Eq + Default + Ord;

        type Device: Parameter + Member + MaxEncodedLen + Clone + Eq + Default;

        type DidRegistry: DidManager<Self::AccountId, Self::Did, Self::Device, Self::GivenRight>;

        type MaxJurors: Get<u32>;

        type MaxJurorsPerDispute: Get<u32>;

        type MinJurorsPerDispute: Get<u32>;

        type MaxContextLength: Get<u32>;

        type HoldAmount: Get<BalanceOf<Self>>;

        type SlashAmount: Get<BalanceOf<Self>>;

        type RewardAmount: Get<BalanceOf<Self>>;

        type ExclusionFee: Get<BalanceOf<Self>>;

        type EscalatedVotingPeriod: Get<BlockNumberFor<Self>>;

        type MaxRewardsNumber: Get<u32>;

        type BatchRewardSize: Get<u32>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Enum representing reasons for holding funds.
    #[pallet::composite_enum]
    #[derive(
        Encode,
        Decode,
        Clone,
        Copy,
        PartialEq,
        Eq,
        RuntimeDebug,
        MaxEncodedLen,
        TypeInfo,
        DecodeWithMemTracking,
    )]
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
    #[pallet::getter(fn get_all_escalated_dispute)]
    pub type EscalatedDisputes<T: Config> =
        StorageValue<_, BoundedVec<ContentId, T::MaxJurors>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_juror_native_account)]
    pub type JurorNativeAccountAdmin<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Did, T::AccountId, OptionQuery>;

    #[pallet::storage]
    pub type JuryDuty<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::Did,
        Blake2_128Concat,
        ContentId,
        bool,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn get_amount_of_jury_summoned_for_dispute)]
    pub type JurySummoned<T: Config> = StorageMap<_, Blake2_128Concat, ContentId, u32, OptionQuery>;

    #[pallet::storage]
    pub type JurySelection<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        ContentId,
        BoundedVec<T::Did, T::MaxJurorsPerDispute>,
        ValueQuery,
    >;

    // dispute -> jurors
    #[pallet::storage]
    #[pallet::getter(fn get_dispute)]
    pub type Dispute<T: Config> =
        StorageMap<_, Blake2_128Concat, ContentId, CourtSession<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn get_decision)]
    pub type Decisions<T: Config> =
        StorageMap<_, Blake2_128Concat, ContentId, Decision, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_slashes)]
    pub(super) type PendingSlashes<T: Config> =
        StorageValue<_, BoundedVec<T::Did, T::MaxRewardsNumber>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn pending_rewards)]
    pub(super) type PendingRewards<T: Config> =
        StorageValue<_, BoundedVec<T::Did, T::MaxRewardsNumber>, ValueQuery>;

    #[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Clone, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct CourtSession<T: Config> {
        pub jurors: BoundedVec<T::Did, T::MaxJurorsPerDispute>,
        pub started_at: Option<BlockNumberFor<T>>,
        pub ended_at: Option<BlockNumberFor<T>>,
        pub verdict: Verdict<T>,
        pub context: BoundedVec<u8, T::MaxContextLength>,
        pub expires_at: BlockNumberFor<T>,
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

    #[derive(
        Debug, Encode, Decode, TypeInfo, PartialEq, Eq, Clone, MaxEncodedLen, DecodeWithMemTracking,
    )]
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
        ContentAlreadyExists,
        /// SessionNotFound
        SessionNotFound,
        /// Session has ended
        SessionHasEnded,
        /// Jury Requirement not reached yet
        JuryReqNotMet,
        /// Session has ALready Started
        SessionALreadyStarted,
        /// Session has not started yet
        SessionHasNotStarted,
        /// Juror not in session
        JurorNotInSession,
        /// Juror already voted
        JurorAlreadyVoted,
        /// Session in progress
        SessionInProgress,
        /// Session Escalated
        SessionEscalated,
        /// Not enough votes
        NotEnoughVotes,
        /// No reward for escalated session
        NoRewardForEscalatedSession,
        /// Session has been rewarded
        SessionHasBeenRewarded,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(_now: BlockNumberFor<T>) -> Weight {
            let (random_value, _) = insecure_randomness::Pallet::<T>::random(&b"rewards"[..]);
            let random_number: u32 = random_value.encode().into_iter().collect::<Vec<u8>>().iter().sum::<u8>() as u32;
            let punish_offenders = random_number % 2 == 0;
           
            match punish_offenders {
                true => {
                    let mut queue = <PendingSlashes<T>>::get();
                    let batch_size: u32 = T::BatchRewardSize::get(); // number of jurors to slash per block
                    let mut processed: u32 = 0;
                    let amount = T::SlashAmount::get();
        
                    while processed < batch_size {
                        if let Some(juror) = queue.pop() {
                            if let Some(admin) = <JurorNativeAccountAdmin<T>>::get(&juror) {
                                let result = <T as Config>::NativeBalance::burn_from(
                                    &admin,
                                    amount,
                                    Preservation::Expendable,
                                    Precision::BestEffort,
                                    Fortitude::Force,
                                );
                                if let Err(_) = result {
                                    let mut jurors = <Jurors<T>>::get();
                                    let index = jurors.iter().position(|x| x == &juror);
                                    if let Some(i) = index {
                                        jurors.remove(i);
                                    }
                                    <Jurors<T>>::set(jurors);
                                }
                            };
                            processed += 1;
                        } else {
                            break;
                        }
                    }
                    <PendingSlashes<T>>::put(queue);
                    return T::DbWeight::get().writes(processed.into());
                },
                false => {
                    let mut queue = <PendingRewards<T>>::get();
                    let batch_size: u32 = T::BatchRewardSize::get(); // number of jurors to slash per block
                    let mut processed: u32 = 0;
                    let amount = T::RewardAmount::get();
                    let mut retrials = Vec::new();
                    while processed < batch_size {
                        if let Some(juror) = queue.pop() {
                            if let Some(admin) = <JurorNativeAccountAdmin<T>>::get(&juror) {
                                let result = <T as Config>::NativeBalance::mint_into(
                                    &admin,
                                    amount
                                );
                                if let Err(_) = result {
                                   retrials.push(juror);
                                }
                            };
                            processed += 1;
                        } else {
                            break;
                        }
                    }
                    let _ = queue.try_append(&mut retrials);
                    <PendingRewards<T>>::put(queue);
                    return T::DbWeight::get().writes(processed.into());
                }
            }
        }
    }

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
            expires_at: BlockNumberFor<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(
                <Dispute<T>>::get(&content_id).is_none(),
                Error::<T>::ContentAlreadyExists
            );
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
                expires_at,
            };
            Self::summon_jurors(&content_id)?;
            <Dispute<T>>::set(content_id, Some(session));
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn summon_jurors_ext(origin: OriginFor<T>, content_id: ContentId) -> DispatchResult {
            ensure_signed(origin)?;
            Self::summon_jurors(&content_id)?;
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn start_deliberation(origin: OriginFor<T>, content_id: ContentId) -> DispatchResult {
            ensure_signed(origin)?;
            let session = <Dispute<T>>::get(&content_id);
            let current_block = <frame_system::Pallet<T>>::block_number();
            if let Some(mut session) = session {
                if current_block.ge(&session.expires_at) {
                    return Err(Error::<T>::SessionHasEnded.into());
                }
                if session.started_at.is_some() {
                    return Err(Error::<T>::SessionALreadyStarted.into());
                }
                let summoned_jury = <JurySummoned<T>>::get(&content_id).unwrap_or_default();
                if summoned_jury < T::MinJurorsPerDispute::get() {
                    Self::summon_jurors(&content_id)?;
                    return Err(Error::<T>::JuryReqNotMet.into());
                }
                let selected_jurors = <JurySelection<T>>::get(&content_id);
                session.jurors = selected_jurors;
                session.started_at = Some(current_block);
                <Dispute<T>>::set(&content_id, Some(session));
                <JurySelection<T>>::remove(&content_id);
            } else {
                return Err(Error::<T>::SessionNotFound.into());
            }
            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn exclude_from_duty(
            origin: OriginFor<T>,
            did: T::Did,
            content_id: ContentId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let is_valid = <T as Config>::DidRegistry::is_signer_valid(
                &who,
                &did,
                &T::GivenRight::from(BaseRight::Dispute),
            )
            .map_err(|_| Error::<T>::CouldNotGetResponse)?;
            ensure!(is_valid, Error::<T>::SignerDoesNotHaveRight);

            let session = <Dispute<T>>::get(&content_id);
            let current_block = <frame_system::Pallet<T>>::block_number();
            if let Some(session) = session {
                if current_block.ge(&session.expires_at) {
                    return Err(Error::<T>::SessionHasEnded.into());
                }
                if session.started_at.is_some() {
                    return Err(Error::<T>::SessionALreadyStarted.into());
                }
            } else {
                return Err(Error::<T>::SessionNotFound.into());
            }

            let amount = T::ExclusionFee::get();
            <T as Config>::NativeBalance::burn_from(
                &who,
                amount,
                Preservation::Expendable,
                Precision::BestEffort,
                Fortitude::Polite,
            )?;
            <JuryDuty<T>>::remove(&did, &content_id);
            <JurySummoned<T>>::mutate(&content_id, |n| n.unwrap_or_default() - 1);

            let mut selected = <JurySelection<T>>::get(&content_id);
            let index = selected.iter().position(|x| x == &did);
            if let Some(i) = index {
                selected.remove(i);
            }
            <JurySelection<T>>::insert(content_id, selected);
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn cast_vote(
            origin: OriginFor<T>,
            did: T::Did,
            content_id: ContentId,
            vote: Vote,
        ) -> DispatchResult {
            // Implementation of finalize_duty_and_get_reward
            let who = ensure_signed(origin)?;
            let is_valid = <T as Config>::DidRegistry::is_signer_valid(
                &who,
                &did,
                &T::GivenRight::from(BaseRight::Dispute),
            )
            .map_err(|_| Error::<T>::CouldNotGetResponse)?;
            ensure!(is_valid, Error::<T>::SignerDoesNotHaveRight);
            let session = <Dispute<T>>::get(&content_id);
            let current_block = <frame_system::Pallet<T>>::block_number();
            if let Some(mut session) = session {
                if current_block.ge(&session.expires_at) && session.ended_at.is_some() {
                    return Err(Error::<T>::SessionHasEnded.into());
                }
                if session.started_at.is_none() {
                    return Err(Error::<T>::SessionHasNotStarted.into());
                }
                if !session.jurors.contains(&did) {
                    return Err(Error::<T>::JurorNotInSession.into());
                }

                for vote in session.verdict.votes.iter() {
                    if &vote.juror == &did {
                        return Err(Error::<T>::JurorAlreadyVoted.into());
                    }
                }
                session
                    .verdict
                    .votes
                    .try_push(VoteRegistry {
                        juror: did,
                        vote: vote,
                    })
                    .map_err(|_| Error::<T>::StorageOverflow)?;
                <Dispute<T>>::insert(&content_id, session);
            } else {
                return Err(Error::<T>::SessionNotFound.into());
            }

            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn calculate_result(origin: OriginFor<T>, content_id: ContentId) -> DispatchResult {
            ensure_signed(origin)?;

            if <Decisions<T>>::get(&content_id).is_some() {
                return Ok(());
            }

            let session = <Dispute<T>>::get(&content_id);
            let current_block = <frame_system::Pallet<T>>::block_number();
            if let Some(mut session) = session {
                if current_block.le(&session.expires_at) {
                    return Err(Error::<T>::SessionInProgress.into());
                }
                if session.started_at.is_none() {
                    return Err(Error::<T>::SessionHasNotStarted.into());
                }

                if session.verdict.escalated {
                    return Err(Error::<T>::SessionEscalated.into());
                }

                if session.verdict.votes.len() < 2 {
                    return Err(Error::<T>::NotEnoughVotes.into());
                }

                let votes_required_to_pass = session.verdict.votes.len() / 2;

                let convict_votes = session
                    .verdict
                    .votes
                    .iter()
                    .filter(|vote| vote.vote.eq(&Vote::Yay))
                    .count();

                if convict_votes == votes_required_to_pass {
                    session.verdict.escalated = true;
                    <Dispute<T>>::insert(&content_id, session);
                    let escalted = Escalated {
                        escalated_at: Some(<frame_system::Pallet<T>>::block_number()),
                        decision_at: None,
                        decision: Decision::Pending,
                        votes: BoundedVec::new(),
                    };
                    <EscalatedSession<T>>::insert(&content_id, escalted);

                    <EscalatedDisputes<T>>::mutate(|disputes| -> Result<(), Error<T>> {
                        disputes
                            .try_push(content_id)
                            .map_err(|_| Error::<T>::StorageOverflow.into())?;
                        Ok(())
                    })?;
                    return Ok(());
                }
                let result = match convict_votes > votes_required_to_pass {
                    true => Decision::Convict,
                    false => Decision::Acquittal,
                };
                <Decisions<T>>::insert(&content_id, result);
            } else {
                return Err(Error::<T>::SessionNotFound.into());
            }
            Ok(())
        }

        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn vote_escalated_content(
            origin: OriginFor<T>,
            did: T::Did,
            content_id: ContentId,
            vote: Vote,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let is_valid = <T as Config>::DidRegistry::is_signer_valid(
                &who,
                &did,
                &T::GivenRight::from(BaseRight::Dispute),
            )
            .map_err(|_| Error::<T>::CouldNotGetResponse)?;
            ensure!(is_valid, Error::<T>::SignerDoesNotHaveRight);

            let escalated_session = <EscalatedSession<T>>::get(&content_id);

            if let Some(mut session) = escalated_session {
                let current_block = <frame_system::Pallet<T>>::block_number();

                let expires_at = current_block + T::EscalatedVotingPeriod::get();
                if current_block > expires_at {
                    return Err(Error::<T>::SessionHasEnded.into());
                }
                for vote in session.votes.iter() {
                    if &vote.juror == &did {
                        return Err(Error::<T>::JurorAlreadyVoted.into());
                    }
                }
                session.escalated_at = Some(current_block);
                session
                    .votes
                    .try_push(VoteRegistry {
                        juror: did,
                        vote: vote,
                    })
                    .map_err(|_| Error::<T>::StorageOverflow)?;
                <EscalatedSession<T>>::insert(&content_id, session);
            } else {
                return Err(Error::<T>::SessionNotFound.into());
            }

            Ok(())
        }

        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn calculate_escalated_result(
            origin: OriginFor<T>,
            content_id: ContentId,
        ) -> DispatchResult {
            // Implementation of finalize_duty_and_get_reward
            ensure_signed(origin)?;

            if <Decisions<T>>::get(&content_id).is_some() {
                return Ok(());
            }

            let session = <EscalatedSession<T>>::get(&content_id);
            let current_block = <frame_system::Pallet<T>>::block_number();
            if let Some(mut session) = session {
                let expires_at =
                    session.escalated_at.unwrap_or_default() + T::EscalatedVotingPeriod::get();
                if current_block.le(&expires_at) {
                    return Err(Error::<T>::SessionInProgress.into());
                }

                if session.votes.len() < 2 {
                    return Err(Error::<T>::NotEnoughVotes.into());
                }

                let votes_required_to_pass = session.votes.len() / 2;

                let convict_votes = session
                    .votes
                    .iter()
                    .filter(|vote| vote.vote.eq(&Vote::Yay))
                    .count();

                let result = match convict_votes > votes_required_to_pass {
                    true => Decision::Convict,
                    false => Decision::Acquittal,
                };
                <Decisions<T>>::insert(&content_id, &result);
                session.decision_at = Some(<frame_system::Pallet<T>>::block_number());
                session.decision = result;
                <EscalatedSession<T>>::insert(&content_id, session);

                let mut selected = <EscalatedDisputes<T>>::get();
                let index = selected.iter().position(|x| x == &content_id);
                if let Some(i) = index {
                    selected.remove(i);
                }
                <EscalatedDisputes<T>>::set(selected);
            } else {
                return Err(Error::<T>::SessionNotFound.into());
            }
            Ok(())
        }

        // #[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Clone, MaxEncodedLen)]
        // #[scale_info(skip_type_params(T))]
        // pub struct CourtSession<T: Config> {
        //     pub jurors: BoundedVec<T::Did, T::MaxJurorsPerDispute>,
        //     pub started_at: Option<BlockNumberFor<T>>,
        //     pub ended_at: Option<BlockNumberFor<T>>,
        //     pub verdict: Verdict<T>,
        //     pub context: BoundedVec<u8, T::MaxContextLength>,
        //     pub expires_at: BlockNumberFor<T>,
        // }

        // #[pallet::storage]
        // #[pallet::getter(fn get_escalated_dispute)]
        // pub type EscalatedSession<T: Config> =
        //     StorageMap<_, Blake2_128Concat, ContentId, Escalated<T>, OptionQuery>;

        // #[derive(Debug, Encode, Decode, TypeInfo, PartialEq, Clone, MaxEncodedLen)]
        // #[scale_info(skip_type_params(T))]
        // pub struct Verdict<T: Config> {
        //     pub escalated: bool,
        //     pub decision: Decision,
        //     pub votes: BoundedVec<VoteRegistry<T>, T::MaxJurorsPerDispute>,
        //     pub rewarded_and_slashed: bool,
        // }

        #[pallet::call_index(9)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(10))]
        pub fn get_reward_for_duty(origin: OriginFor<T>, content_id: ContentId) -> DispatchResult {
            ensure_signed(origin)?;
            // Implementation of finalize_duty_and_get_reward
            // if escalated no reward but punishment will still be applied
            //
            let session = <Dispute<T>>::get(&content_id);

            if let Some(mut session) = session {
                if session.ended_at.is_some() {
                    return Err(Error::<T>::SessionHasBeenRewarded.into());
                }
                let jurors_for_session = <JurySelection<T>>::get(&content_id)
                    .into_iter()
                    .collect::<BTreeSet<T::Did>>();
                if session.verdict.escalated {
                    // slash jurors that did not vote
                    let jurors_that_voted = session
                        .verdict
                        .votes
                        .iter()
                        .map(|vote| vote.juror.clone())
                        .collect::<BTreeSet<T::Did>>();

                    let mut diff = jurors_for_session
                        .difference(&jurors_that_voted)
                        .cloned()
                        .collect::<Vec<_>>();
                    let mut pending_slashes = <PendingSlashes<T>>::get();
                    pending_slashes
                        .try_append(&mut diff)
                        .map_err(|_| Error::<T>::StorageOverflow)?;
                    <PendingSlashes<T>>::put(pending_slashes);
                    session.ended_at = Some(frame_system::Pallet::<T>::block_number());
                    <Dispute<T>>::insert(&content_id, session);
                    return Ok(());
                }

                // prepare to punish users that did not vote
                let jurors_that_voted = session
                    .verdict
                    .votes
                    .iter()
                    .map(|vote| vote.juror.clone())
                    .collect::<BTreeSet<T::Did>>();
                let mut diff = jurors_for_session
                    .difference(&jurors_that_voted)
                    .cloned()
                    .collect::<Vec<_>>();
                let mut pending_slashes = <PendingSlashes<T>>::get();
                pending_slashes
                    .try_append(&mut diff)
                    .map_err(|_| Error::<T>::StorageOverflow)?;
                <PendingSlashes<T>>::put(pending_slashes);

                // prepare reward jurors that voted jurors
                let mut pending_rewards = <PendingRewards<T>>::get();
                let mut jurors_that_voted = jurors_that_voted.into_iter().collect::<Vec<_>>();
                pending_rewards
                    .try_append(&mut jurors_that_voted)
                    .map_err(|_| Error::<T>::StorageOverflow)?;
                <PendingRewards<T>>::put(pending_rewards);

                session.ended_at = Some(frame_system::Pallet::<T>::block_number());
                <Dispute<T>>::insert(&content_id, session);
            } else {
                return Err(Error::<T>::SessionNotFound.into());
            }
            Ok(())
        }

        // pub fn finalize_duty_and_get_reward(){}
    }

    impl<T: Config> Pallet<T> {
        /// We strongly advocate that the max number of jurors per session be 40
        pub fn summon_jurors(content_id: &ContentId) -> Result<(), Error<T>> {
            use polkadot_sdk::sp_std::collections::btree_set::BTreeSet;

            let prev_number_of_summons = <JurySummoned<T>>::get(content_id).unwrap_or_default();
            if prev_number_of_summons > T::MaxJurorsPerDispute::get() {
                return Err(Error::<T>::MaxJurorsReached);
            }
            let jurors = <Jurors<T>>::get();
            let (random_value, _) = insecure_randomness::Pallet::<T>::random(&b"summon jurors"[..]);
            let numbers: BTreeSet<u8> = random_value.encode().into_iter().collect();
            let mut summoned = 0;
            for number in numbers {
                if let Some(juror) = jurors.get(number as usize) {
                    if !<JuryDuty<T>>::contains_key(juror, content_id) {
                        if (prev_number_of_summons + 1) > T::MaxJurorsPerDispute::get() {
                            break;
                        }
                        <JuryDuty<T>>::insert(juror, content_id, true);
                        <JurySelection<T>>::mutate(
                            content_id,
                            |selected| -> Result<(), Error<T>> {
                                selected
                                    .try_push(juror.clone())
                                    .map_err(|_| Error::<T>::StorageOverflow.into())?;
                                Ok(())
                            },
                        )?;
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

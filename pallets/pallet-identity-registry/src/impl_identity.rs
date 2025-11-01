use super::*;
use shared::traits::identity::DidManager;
use sp_std::vec::Vec;
use frame::prelude::*;

impl<T: Config> DidManager<
    T::AccountId,
    Did<T>,
    T::Device,
> for Pallet<T> {
    type Error = Error<T>;
    
    fn read_did(
        creator: &T::AccountId,
        did: Did<T>,
        signatories: Vec<T::AccountId>,
    ) -> Result<(), Error<T>> {
        ensure!(
            !Signatories::<T>::contains_key(&did),
            Error::<T>::DidAlreadyExists
        );

        let r = Rights::<T> {
            right: GivenRight::Update,
            duration: RightDuration::Permanent,
        };

        let mut list: BoundedVec<Rights<T>, T::MaxKeySize> =
            SignatoryRights::<T>::get(&did, creator).unwrap_or_default();
        list.try_push(r).map_err(|_| Error::<T>::TooManyRights)?;

        let bounded_signatories: BoundedVec<T::AccountId, T::MaxKeySize> =
            signatories.try_into().map_err(|_| Error::<T>::TooManyRights)?;

        Signatories::<T>::insert(&did, bounded_signatories);

        Self::deposit_event(Event::DidCreated {
            block_number: <frame_system::Pallet<T>>::block_number(),
            creator: creator.clone(),
            did,
        });

        Ok(())
    }

    
}

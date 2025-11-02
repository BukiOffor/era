use super::*;
use frame::prelude::*;
use shared::traits::identity::DidManager;
use polkadot_sdk::sp_std::vec::Vec;

impl<T: Config> DidManager<T::AccountId, T::Did, T::Device, T::GivenRight> for Pallet<T> {
    type Error = DispatchError;

    fn read_did_devices(did: &T::Did) -> Result<Vec<T::Device>, Self::Error> {
        let devices = DidDevices::<T>::get(did).unwrap_or_default().to_vec();
        Ok(devices)
    }

    fn is_signer_valid(
        who: &T::AccountId,
        did: &T::Did,
        right: &T::GivenRight,
    ) -> Result<bool, Self::Error> {
        let signer_rights = SignatoryRights::<T>::get(did, who).unwrap_or_default();
        // Get current block number
        let current_block = <frame_system::Pallet<T>>::block_number();
        Ok(signer_rights.iter().any(|r| {
            r.right == *right
                && match r.duration {
                    RightDuration::Permanent => true,
                    RightDuration::Temporary(Duration {
                        valid_from_block,
                        valid_to_block,
                    }) => valid_from_block <= current_block && current_block <= valid_to_block,
                }
        }))
    }
}

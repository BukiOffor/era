use polkadot_sdk::sp_std::vec::Vec;

pub trait DidManager<AccountId, Did, Device, Right> {
    type Error;
    fn read_did_devices(did: &Did) -> Result<Vec<Device>, Self::Error>;

    fn is_signer_valid(who: &AccountId, did: &Did, right: &Right) -> Result<bool, Self::Error>;
}

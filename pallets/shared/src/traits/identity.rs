use sp_std::vec::Vec;

pub trait DidManager<AccountId, Did, Device> {
    type Error;
    fn read_did(
        creator: &AccountId,
        did: Did,
        signatories: Vec<AccountId>,
    ) -> Result<(), Self::Error>;
// is_signer_valid
}

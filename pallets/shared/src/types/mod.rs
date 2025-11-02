use frame::prelude::*;

#[derive(
    Encode,
    Decode,
    TypeInfo,
    Clone,
    PartialEq,
    DecodeWithMemTracking,
    Default,
    Eq,
    MaxEncodedLen,
    Debug,
)]
pub enum BaseRight {
    #[default]
    Update,
    Impersonate,
    Dispute,
}


#[derive(Encode, Decode, Clone, Eq, PartialEq, Default, TypeInfo, MaxEncodedLen, Debug, DecodeWithMemTracking)]
pub struct ContentId {
    prefix: [u8; 4],   
    hash: [u8; 32],    
}

impl ContentId {
    pub fn new(prefix: &[u8], hash: &[u8]) -> Self {
        let mut content_id = ContentId::default();
        content_id.prefix.copy_from_slice(prefix);
        content_id.hash.copy_from_slice(hash);
        content_id
    }
}
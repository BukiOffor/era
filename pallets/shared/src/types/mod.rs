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

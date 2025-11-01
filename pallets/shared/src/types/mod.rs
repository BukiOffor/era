use frame::prelude::*;
use frame_system::Config;

#[derive(
    Encode, Decode, TypeInfo, Clone, PartialEq, Eq, MaxEncodedLen, Debug
)]
pub enum GivenRight {
    Update,
    Impersonate,
    Dispute,
}

#[derive(
    Encode, Decode, TypeInfo, Clone, PartialEq, Eq, MaxEncodedLen, Debug
)]
pub enum DefaultRightDuration<T: Config> {
    Permanent,
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
    Eq
)]
#[scale_info(skip_type_params(T))]
pub struct Duration<T: Config> {
    /// A block number.
    pub(crate) valid_from_block: BlockNumberFor<T>,
    /// A block number
    pub(crate) valid_to_block: BlockNumberFor<T>,
}
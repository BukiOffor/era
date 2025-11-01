use frame::{
    deps::{frame_support::weights::constants::RocksDbWeight, frame_system::GenesisConfig},
    prelude::*,
    runtime::prelude::*,
    testing_prelude::*,
};
use pallet_identity_registry;
use shared::types::BaseRight;

// Configure a mock runtime to test the pallet.
#[frame_construct_runtime]
mod test_runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask,
        RuntimeViewFunction
    )]
    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;
    #[runtime::pallet_index(1)]
    pub type Template = crate;
    #[runtime::pallet_index(2)]
    pub type IdentityRegistry = pallet_identity_registry;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Nonce = u64;
    type Block = MockBlock<Test>;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = RocksDbWeight;
}

impl pallet_identity_registry::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxKeySize = ConstU32<1024>;
    type MaxStringLength = ConstU32<1024>;
    type Device = BoundedVec<u8, Self::MaxStringLength>;
    type Did = BoundedVec<u8, Self::MaxStringLength>;
    type GivenRight = BaseRight;
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Device = BoundedVec<u8, ConstU32<1024>>;
    type Did = BoundedVec<u8, ConstU32<1024>>;
    type DidRegistry = pallet_identity_registry::Pallet<Test>; 
    type GivenRight = BaseRight;
    type ContentId = BoundedVec<u8, ConstU32<1024>>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> TestState {
    GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}

use frame::{
    deps::{frame_support::weights::constants::RocksDbWeight, frame_system::GenesisConfig},
    prelude::*,
    runtime::prelude::*,
    testing_prelude::*,
};
use shared::types::BaseRight;
use polkadot_sdk::pallet_balances;

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
    pub type Balances = pallet_balances;
    #[runtime::pallet_index(2)]
    pub type Template = crate;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Nonce = u64;
    type Block = MockBlock<Test>;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = RocksDbWeight;
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxStringLength = ConstU32<100>;
    type MaxKeySize = ConstU32<100>;
    type Device = BoundedVec<u8, Self::MaxStringLength>;
    type Did = BoundedVec<u8, Self::MaxStringLength>;
    type GivenRight = BaseRight;
    type NativeBalance = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type HoldAmount = ConstU64<1000>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> TestState {
    GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}

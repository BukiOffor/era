use frame::{
    deps::{frame_support::weights::constants::RocksDbWeight, frame_system::GenesisConfig},
    prelude::*,
    runtime::prelude::*,
    testing_prelude::*,
};
use polkadot_sdk::pallet_balances;
use shared::types::BaseRight;

type Balance = u128;
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
    pub type IdentityPallet = pallet_identity_registry;
    #[runtime::pallet_index(3)]
    pub type Balances = pallet_balances;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Nonce = u64;
    type Block = MockBlock<Test>;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = RocksDbWeight;
    type AccountData = pallet_balances::AccountData<Balance>;
}

impl pallet_identity_registry::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxKeySize = ConstU32<1024>;
    type MaxStringLength = ConstU32<1024>;
    type Device = BoundedVec<u8, Self::MaxStringLength>;
    type Did = BoundedVec<u8, Self::MaxStringLength>;
    type GivenRight = BaseRight;
    type NativeBalance = Balances;
    type HoldAmount = ConstU128<100000>;
    type RuntimeHoldReason = RuntimeHoldReason;
}

impl polkadot_sdk::pallet_insecure_randomness_collective_flip::Config for Test {}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ConstU32<10>;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = RuntimeHoldReason;
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<10>;
}

impl crate::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type Did = BoundedVec<u8, ConstU32<1024>>;
    type MaxJurors = ConstU32<100>;
    type MaxJurorsPerDispute = ConstU32<30>;
    type GivenRight = BaseRight;
    type Device = BoundedVec<u8, ConstU32<1024>>;
    type DidRegistry = pallet_identity_registry::Pallet<Test>;
    type NativeBalance = Balances;
    type ExclusionFee = ConstU128<100>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type MinJurorsPerDispute = ConstU32<40>;
    type MaxContextLength = ConstU32<1000>;
    type HoldAmount = ConstU128<1000>;
    type RewardAmount = ConstU128<200>;
    type SlashAmount = ConstU128<200>;
    type EscalatedVotingPeriod = ConstU64<100000>;
    type MaxRewardsNumber = ConstU32<1000>;
    type BatchRewardSize = ConstU32<10>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> TestState {
    GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}

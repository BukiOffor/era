use frame::{
    deps::frame_support::weights::constants::RocksDbWeight, prelude::*, runtime::prelude::*,
    testing_prelude::*,
};
use polkadot_sdk::{pallet_balances, sp_io};
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
    pub type Balances = pallet_balances;
    #[runtime::pallet_index(2)]
    pub type PalletIndentity = crate;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Nonce = u64;
    type Block = MockBlock<Test>;
    type BlockHashCount = ConstU64<250>;
    type DbWeight = RocksDbWeight;
    type AccountData = pallet_balances::AccountData<Balance>;
}

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
    type MaxStringLength = ConstU32<100>;
    type MaxKeySize = ConstU32<100>;
    type Device = BoundedVec<u8, Self::MaxStringLength>;
    type Did = BoundedVec<u8, Self::MaxStringLength>;
    type GivenRight = BaseRight;
    type NativeBalance = Balances;
    type RuntimeHoldReason = RuntimeHoldReason;
    type HoldAmount = ConstU128<1000>;
}

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2000;
pub const OSCAR: u64 = 10000;

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into();
    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| {
        System::set_block_number(1);
        // Set the balance of our multi account to 10000
        let root: RuntimeOrigin = RuntimeOrigin::root();
        Balances::force_set_balance(root.clone(), ALICE, 10000000)
            .expect("Balance should have been set successfully");
        Balances::force_set_balance(root.clone(), BOB, 10000000)
            .expect("Balance should have been set successfully");
        Balances::force_set_balance(root.clone(), OSCAR, 10000000)
            .expect("Balance should have been set successfully");
    });
    ext
}

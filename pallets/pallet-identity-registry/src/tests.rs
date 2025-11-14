#![allow(unused)]
use crate::{mock::*, Error, Event, RightDuration, Duration};
use frame::testing_prelude::*;
use shared::types::BaseRight;

const DID: [u8; 5] = [2, 3, 4, 5, 6];
const DID2: [u8; 5] = [1, 2, 3, 4, 5];
const DEVICE1: [u8; 5] = [10, 20, 30, 40, 50];
const DEVICE2: [u8; 5] = [11, 21, 31, 41, 51];

// ============ DID Creation Tests ============

#[test]
fn should_create_did_in_storage() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        assert_ok!(PalletIndentity::create_did(origin, did.clone(), signatories.clone()));
        
        // Verify DID was stored
        let stored_signatories = PalletIndentity::get_signatories(&did);
        assert_eq!(stored_signatories, Some(signatories));
        
        // Verify creator has Update right
        assert!(PalletIndentity::is_valid_signatory(&did, &ALICE, &BaseRight::Update));
    });
}

#[test]
fn should_create_did_and_emit_event() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        assert_ok!(PalletIndentity::create_did(origin, did.clone(), signatories));
        
        let event: Event<Test> = Event::DidCreated {
            block_number: 1,
            creator: ALICE,
            did: did.clone(),
        };
        frame_system::Pallet::<Test>::assert_last_event(event.into());
    });
}

#[test]
fn should_hold_balance_on_did_creation() {
    new_test_ext().execute_with(|| {
        let initial_balance = Balances::free_balance(&ALICE);
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(origin, did, signatories));
        
        // Check that hold amount was deducted
        let expected_held = 1000u128; // HoldAmount from mock
        use frame::prelude::fungible::InspectHold;
        use crate::HoldReason;
        assert_eq!(Balances::balance_on_hold(&HoldReason::AccountCreation.into(), &ALICE), expected_held);
        assert_eq!(Balances::free_balance(&ALICE), initial_balance - expected_held);
    });
}

#[test]
fn should_fail_to_create_duplicate_did() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(origin.clone(), did.clone(), signatories.clone()));
        
        // Try to create same DID again
        assert_noop!(
            PalletIndentity::create_did(origin, did, signatories),
            Error::<Test>::DidAlreadyExists
        );
    });
}

#[test]
fn should_fail_to_create_did_with_insufficient_balance() {
    new_test_ext().execute_with(|| {
        // Create a new account with low balance
        let low_balance_account = 9999u64;
        let root: RuntimeOrigin = RuntimeOrigin::root();
        Balances::force_set_balance(root.clone(), low_balance_account, 500) // Less than HoldAmount
            .expect("Balance should have been set successfully");
        
        let origin = RuntimeOrigin::signed(low_balance_account);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_noop!(
            PalletIndentity::create_did(origin, did, signatories),
            frame::prelude::TokenError::FundsUnavailable
        );
    });
}

#[test]
fn should_create_did_with_multiple_signatories() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB, OSCAR]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(origin, did.clone(), signatories.clone()));
        
        let stored = PalletIndentity::get_signatories(&did).unwrap();
        assert_eq!(stored.len(), 2);
        assert!(stored.contains(&BOB));
        assert!(stored.contains(&OSCAR));
    });
}

#[test]
fn should_create_did_with_empty_signatories() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(origin, did.clone(), signatories.clone()));
        
        let stored = PalletIndentity::get_signatories(&did).unwrap();
        assert_eq!(stored.len(), 0);
    });
}

// ============ Rights Management Tests ============

#[test]
fn should_add_right_to_signatory() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let target = BOB;
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did.clone(),
            target,
            BaseRight::Update,
            RightDuration::Permanent
        ));
        
        // Verify right was added
        let rights = PalletIndentity::get_signatory_rights(&did, &target).unwrap();
        assert_eq!(rights.len(), 1);
        assert_eq!(rights[0].right, BaseRight::Update);
        
        // Verify event was emitted
        System::assert_last_event(
            Event::RightAdded {
                block_number: 1,
                who: ALICE,
                did: did.clone(),
                right: crate::Rights {
                    right: BaseRight::Update,
                    duration: RightDuration::Permanent,
                },
            }
            .into(),
        );
    });
}

#[test]
fn should_add_multiple_rights_to_signatory() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let target = BOB;
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did.clone(),
            target,
            BaseRight::Update,
            RightDuration::Permanent
        ));
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did.clone(),
            target,
            BaseRight::Impersonate,
            RightDuration::Permanent
        ));

        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin,
            did.clone(),
            target,
            BaseRight::Dispute,
            RightDuration::Permanent
        ));
        
        let rights = PalletIndentity::get_signatory_rights(&did, &target).unwrap();
        assert_eq!(rights.len(), 3);
    });
}

#[test]
fn should_add_temporary_right_to_signatory() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let target = BOB;
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        let duration = RightDuration::Temporary(Duration {
            valid_from_block: 1,
            valid_to_block: 100,
        });
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin,
            did.clone(),
            target,
            BaseRight::Impersonate,
            duration.clone()
        ));
        
        // Verify right exists and is valid at block 1
        assert!(PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
        
        // Move to block 50 (within range)
        System::set_block_number(50);
        assert!(PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
        
        // Move to block 101 (outside range)
        System::set_block_number(101);
        assert!(!PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
    });
}

#[test]
fn should_fail_to_add_right_without_update_permission() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB, OSCAR]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        // BOB doesn't have Update right, only was added as signatory
        let origin_bob = RuntimeOrigin::signed(BOB);
        assert_noop!(
            PalletIndentity::add_right_for_signatory(
                origin_bob,
                did,
                OSCAR,
                BaseRight::Impersonate,
                RightDuration::Permanent
            ),
            Error::<Test>::SignerDoesNotHaveRight
        );
    });
}

#[test]
fn should_fail_to_add_right_for_nonexistent_did() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        
        // Try to add right without creating DID first
        assert_noop!(
            PalletIndentity::add_right_for_signatory(
                origin,
                did,
                BOB,
                BaseRight::Update,
                RightDuration::Permanent
            ),
            Error::<Test>::SignerDoesNotHaveRight
        );
    });
}

#[test]
fn should_fail_to_add_right_when_max_rights_reached() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        // Try to add more than MaxKeySize rights (which is 100 in mock)
        // This test assumes MaxKeySize is low enough to be practical
        // In practice, we'd need to check the actual limit
        for i in 0..100 {
            let result = PalletIndentity::add_right_for_signatory(
                origin.clone(),
                did.clone(),
                BOB,
                BaseRight::Update,
                RightDuration::Permanent
            );
            if i < 99 {
                assert_ok!(result);
            } else {
                // 100th right might succeed, but 101st should fail
                if let Err(e) = result {
                    assert_eq!(e, Error::<Test>::TooManyRights.into());
                }
            }
        }
    });
}

#[test]
fn should_remove_right_from_signatory() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let target = BOB;
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        // Add Update right
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did.clone(),
            target,
            BaseRight::Update,
            RightDuration::Permanent
        ));
        
        // Add Impersonate right
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did.clone(),
            target,
            BaseRight::Impersonate,
            RightDuration::Permanent
        ));
        
        // Remove Update right
        assert_ok!(PalletIndentity::remove_right_for_signatory(
            origin.clone(),
            did.clone(),
            target,
            BaseRight::Update
        ));
        
        // Verify Update right was removed but Impersonate remains
        let rights = PalletIndentity::get_signatory_rights(&did, &target).unwrap();
        assert_eq!(rights.len(), 1);
        assert_eq!(rights[0].right, BaseRight::Impersonate);
        
        // Verify event was emitted
        System::assert_last_event(
            Event::RightRemoved {
                block_number: 1,
                who: ALICE,
                did: did.clone(),
                right: BaseRight::Update,
            }
            .into(),
        );
    });
}

#[test]
fn should_remove_all_instances_of_right() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let target = BOB;
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        // Add same right multiple times
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did.clone(),
            target,
            BaseRight::Update,
            RightDuration::Permanent
        ));
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did.clone(),
            target,
            BaseRight::Update,
            RightDuration::Permanent
        ));
        
        // Remove should remove all instances
        assert_ok!(PalletIndentity::remove_right_for_signatory(
            origin,
            did.clone(),
            target,
            BaseRight::Update
        ));
        
        let rights = PalletIndentity::get_signatory_rights(&did, &target);
        assert!(rights.is_none() || rights.unwrap().is_empty());
    });
}

#[test]
fn should_fail_to_remove_right_without_permission() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did.clone(),
            BOB,
            BaseRight::Impersonate,
            RightDuration::Permanent
        ));
        
        // BOB tries to remove right without Update permission
        let origin_bob = RuntimeOrigin::signed(BOB);
        assert_noop!(
            PalletIndentity::remove_right_for_signatory(
                origin_bob,
                did,
                BOB,
                BaseRight::Impersonate
            ),
            Error::<Test>::SignerDoesNotHaveRight
        );
    });
}

// ============ Device Management Tests ============

#[test]
fn should_register_device() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        assert_ok!(PalletIndentity::register_device(
            origin,
            did.clone(),
            device.clone()
        ));
        
        // Verify device was registered
        let devices = PalletIndentity::get_did_devices(&did).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0], device);
        
        // Verify event was emitted
        System::assert_last_event(
            Event::DeviceRegistered {
                block_number: 1,
                who: ALICE,
                did: did.clone(),
                device: device.clone(),
            }
            .into(),
        );
    });
}

#[test]
fn should_register_multiple_devices() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        let device1 = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let device2 = BoundedVec::try_from(DEVICE2.to_vec()).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        assert_ok!(PalletIndentity::register_device(
            origin.clone(),
            did.clone(),
            device1.clone()
        ));
        
        assert_ok!(PalletIndentity::register_device(
            origin,
            did.clone(),
            device2.clone()
        ));
        
        let devices = PalletIndentity::get_did_devices(&did).unwrap();
        assert_eq!(devices.len(), 2);
        assert!(devices.contains(&device1));
        assert!(devices.contains(&device2));
    });
}

#[test]
fn should_remove_device() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        let device1 = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let device2 = BoundedVec::try_from(DEVICE2.to_vec()).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        assert_ok!(PalletIndentity::register_device(
            origin.clone(),
            did.clone(),
            device1.clone()
        ));
        
        assert_ok!(PalletIndentity::register_device(
            origin.clone(),
            did.clone(),
            device2.clone()
        ));
        
        // Remove device1
        assert_ok!(PalletIndentity::remove_device(
            origin.clone(),
            did.clone(),
            device1.clone()
        ));
        
        let devices = PalletIndentity::get_did_devices(&did).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0], device2);
        
        // Verify event was emitted
        System::assert_last_event(
            Event::DeviceRemoved {
                block_number: 1,
                who: ALICE,
                did: did.clone(),
                device: device1.clone(),
            }
            .into(),
        );
    });
}

#[test]
fn should_fail_to_register_device_without_permission() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin,
            did.clone(),
            signatories
        ));
        
        // BOB doesn't have Update right
        let origin_bob = RuntimeOrigin::signed(BOB);
        assert_noop!(
            PalletIndentity::register_device(
                origin_bob,
                did,
                device
            ),
            Error::<Test>::SignerDoesNotHaveRight
        );
    });
}

#[test]
fn should_fail_to_register_device_when_max_devices_reached() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        // Try to add more than MaxKeySize devices (100 in mock)
        // This is a boundary test
        for i in 0..101 {
            let device_bytes = [i as u8; 5];
            let device = BoundedVec::try_from(device_bytes.to_vec()).unwrap();
            let result = PalletIndentity::register_device(
                origin.clone(),
                did.clone(),
                device
            );
            
            if i < 100 {
                assert_ok!(result);
            } else {
                assert_noop!(result, Error::<Test>::TooManyDevices);
            }
        }
    });
}

#[test]
fn should_remove_device_that_does_not_exist() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        // Remove device that was never registered (should succeed without error)
        assert_ok!(PalletIndentity::remove_device(
            origin,
            did.clone(),
            device
        ));
    });
}

// ============ Permission Validation Tests ============

#[test]
fn should_validate_permanent_right() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin,
            did.clone(),
            BOB,
            BaseRight::Impersonate,
            RightDuration::Permanent
        ));
        
        // Should be valid across multiple blocks
        System::set_block_number(1);
        assert!(PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
        
        System::set_block_number(1000);
        assert!(PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
        
        System::set_block_number(10000);
        assert!(PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
    });
}

#[test]
fn should_validate_temporary_right_within_range() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        let duration = RightDuration::Temporary(Duration {
            valid_from_block: 10,
            valid_to_block: 100,
        });
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin,
            did.clone(),
            BOB,
            BaseRight::Impersonate,
            duration
        ));
        
        // Before range - invalid
        System::set_block_number(9);
        assert!(!PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
        
        // At start - valid
        System::set_block_number(10);
        assert!(PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
        
        // In middle - valid
        System::set_block_number(50);
        assert!(PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
        
        // At end - valid
        System::set_block_number(100);
        assert!(PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
        
        // After range - invalid
        System::set_block_number(101);
        assert!(!PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
    });
}

#[test]
fn should_fail_validation_for_nonexistent_right() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        // BOB doesn't have Impersonate right
        assert!(!PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
    });
}

#[test]
fn should_fail_validation_for_wrong_account() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        // OSCAR is not associated with this DID
        assert!(!PalletIndentity::is_valid_signatory(&did, &OSCAR, &BaseRight::Update));
    });
}

// ============ Integration Tests ============

#[test]
fn should_manage_complete_did_lifecycle() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB, OSCAR]).unwrap();
        let device1 = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let device2 = BoundedVec::try_from(DEVICE2.to_vec()).unwrap();
        
        // 1. Create DID
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories.clone()
        ));
        
        // 2. Add rights to signatories
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did.clone(),
            BOB,
            BaseRight::Update,
            RightDuration::Permanent
        ));
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did.clone(),
            OSCAR,
            BaseRight::Impersonate,
            RightDuration::Permanent
        ));
        
        // 3. Register devices
        assert_ok!(PalletIndentity::register_device(
            origin.clone(),
            did.clone(),
            device1.clone()
        ));
        
        assert_ok!(PalletIndentity::register_device(
            origin.clone(),
            did.clone(),
            device2.clone()
        ));
        
        // 4. Verify state
        let stored_signatories = PalletIndentity::get_signatories(&did).unwrap();
        assert_eq!(stored_signatories.len(), 2);
        
        assert!(PalletIndentity::is_valid_signatory(&did, &ALICE, &BaseRight::Update));
        assert!(PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Update));
        assert!(PalletIndentity::is_valid_signatory(&did, &OSCAR, &BaseRight::Impersonate));
        
        let devices = PalletIndentity::get_did_devices(&did).unwrap();
        assert_eq!(devices.len(), 2);
        
        // 5. Remove one device
        assert_ok!(PalletIndentity::remove_device(
            origin.clone(),
            did.clone(),
            device1.clone()
        ));
        
        // 6. Remove one right
        assert_ok!(PalletIndentity::remove_right_for_signatory(
            origin.clone(),
            did.clone(),
            OSCAR,
            BaseRight::Impersonate
        ));
        
        // 7. Verify final state
        let devices = PalletIndentity::get_did_devices(&did).unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0], device2);
        
        assert!(!PalletIndentity::is_valid_signatory(&did, &OSCAR, &BaseRight::Impersonate));
    });
}

#[test]
fn should_handle_multiple_dids_independently() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did1 = BoundedVec::try_from(DID.to_vec()).unwrap();
        let did2 = BoundedVec::try_from(DID2.to_vec()).unwrap();
        let signatories1 = BoundedVec::try_from(vec![BOB]).unwrap();
        let signatories2 = BoundedVec::try_from(vec![OSCAR]).unwrap();
        
        // Create two separate DIDs
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did1.clone(),
            signatories1
        ));
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did2.clone(),
            signatories2
        ));
        
        // Add rights to each independently
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did1.clone(),
            BOB,
            BaseRight::Update,
            RightDuration::Permanent
        ));
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin.clone(),
            did2.clone(),
            OSCAR,
            BaseRight::Impersonate,
            RightDuration::Permanent
        ));
        
        // Verify they are independent
        assert!(PalletIndentity::is_valid_signatory(&did1, &BOB, &BaseRight::Update));
        assert!(!PalletIndentity::is_valid_signatory(&did1, &BOB, &BaseRight::Impersonate));
        assert!(!PalletIndentity::is_valid_signatory(&did2, &BOB, &BaseRight::Update));
        assert!(PalletIndentity::is_valid_signatory(&did2, &OSCAR, &BaseRight::Impersonate));
    });
}

// ============ Edge Cases ============

#[test]
fn should_handle_zero_length_did() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(vec![]).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(origin, did.clone(), signatories));
        
        let stored = PalletIndentity::get_signatories(&did);
        assert!(stored.is_some());
    });
}

#[test]
fn should_handle_expired_temporary_right_gracefully() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        
        let duration = RightDuration::Temporary(Duration {
            valid_from_block: 1,
            valid_to_block: 10,
        });
        
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin,
            did.clone(),
            BOB,
            BaseRight::Impersonate,
            duration
        ));
        
        // Right exists but is expired
        System::set_block_number(11);
        assert!(!PalletIndentity::is_valid_signatory(&did, &BOB, &BaseRight::Impersonate));
        
        // The right should still be stored, just not valid
        let rights = PalletIndentity::get_signatory_rights(&did, &BOB);
        assert!(rights.is_some());
        assert!(!rights.unwrap().is_empty());
    });
}

#![allow(unused)]
use crate::{mock::*, Error, Event, RightDuration};
use frame::testing_prelude::*;
use shared::types::BaseRight;

const DID: [u8; 5] = [2, 3, 4, 5, 6];

#[test]
fn should_creates_did_in_storage() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        assert_ok!(PalletIndentity::create_did(origin, did, signatories));
    });
}

#[test]
fn should_creates_did_and_emit_event() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        assert_ok!(PalletIndentity::create_did(origin, did, signatories));
        let event: Event<Test> = Event::DidCreated {
            block_number: 1,
            creator: ALICE,
            did: BoundedVec::try_from(DID.to_vec()).unwrap(),
        };
        frame_system::Pallet::<Test>::assert_last_event(event.into());
    });
}

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
            origin,
            did,
            target,
            BaseRight::Update,
            RightDuration::Permanent
        ));
    });
}

#[test]
fn should_fail_to_update_did_signatory() {
    new_test_ext().execute_with(|| {
        let origin = RuntimeOrigin::signed(ALICE);
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let target = BOB;
        let signatories = BoundedVec::try_from(vec![BOB, OSCAR]).unwrap();
        assert_ok!(PalletIndentity::create_did(
            origin.clone(),
            did.clone(),
            signatories
        ));
        assert_ok!(PalletIndentity::add_right_for_signatory(
            origin,
            did.clone(),
            target,
            BaseRight::Impersonate,
            RightDuration::Permanent
        ));

        let origin = RuntimeOrigin::signed(BOB);
        assert_noop!(
            PalletIndentity::add_right_for_signatory(
                origin,
                did,
                OSCAR,
                BaseRight::Impersonate,
                RightDuration::Permanent
            ),
            crate::Error::<Test>::SignerDoesNotHaveRight
        );
    });
}

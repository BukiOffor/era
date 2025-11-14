#![allow(unused)]
use crate::{mock::*, Error, Event};
use frame::testing_prelude::*;
use shared::types::{BaseRight, ContentId};

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2000;
pub const OSCAR: u64 = 10000;

const DID: [u8; 5] = [2, 3, 4, 5, 6];
const DID2: [u8; 5] = [1, 2, 3, 4, 5];
const DEVICE1: [u8; 5] = [10, 20, 30, 40, 50];
const DEVICE2: [u8; 5] = [11, 21, 31, 41, 51];
const CONTENT1: [u8; 32] = [1; 32];
const CONTENT2: [u8; 32] = [2; 32];

// Helper function to setup DID with rights
fn setup_did_with_rights(who: u64, did: BoundedVec<u8, ConstU32<1024>>, signatories: Vec<u64>) {
    let signatories_bounded = BoundedVec::try_from(signatories.clone()).unwrap();
    assert_ok!(IdentityRegistry::create_did(
        RuntimeOrigin::signed(who),
        did.clone(),
        signatories_bounded
    ));
    
    // Grant Impersonate right to the creator
    assert_ok!(IdentityRegistry::add_right_for_signatory(
        RuntimeOrigin::signed(who),
        did.clone(),
        who,
        BaseRight::Impersonate,
        pallet_identity_registry::RightDuration::Permanent
    ));
}

// Helper function to register device
fn register_device_for_did(who: u64, did: BoundedVec<u8, ConstU32<1024>>, device: BoundedVec<u8, ConstU32<1024>>) {
    // Ensure creator has Update right
    assert_ok!(IdentityRegistry::add_right_for_signatory(
        RuntimeOrigin::signed(who),
        did.clone(),
        who,
        BaseRight::Update,
        pallet_identity_registry::RightDuration::Permanent
    ));
    
    assert_ok!(IdentityRegistry::register_device(
        RuntimeOrigin::signed(who),
        did.clone(),
        device
    ));
}

// ============ Content Creation Tests ============

#[test]
fn should_create_content() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content = CONTENT1;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        register_device_for_did(who, did.clone(), device.clone());
        
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device.clone()
        ));
        
        // Verify content was stored
        let content_id = generate_content_id(&content);
        let proof = Template::get_content(&content_id);
        assert!(proof.is_some());
        
        let proof = proof.unwrap();
        assert_eq!(proof.content, content);
        assert_eq!(proof.did, did);
        assert_eq!(proof.signer, who);
        assert_eq!(proof.device, device);
        assert_eq!(proof.content_type, content_type);
        assert_eq!(proof.content_description, content_description);
        assert_eq!(proof.content_metadata, content_metadata);
        
        // Verify DID has content
        assert!(Template::does_did_have_content(&did, &content_id));
        
        // Verify content is in DID's content list
        let did_contents = Template::get_did_contents(&did);
        assert_eq!(did_contents.len(), 1);
        assert!(did_contents.contains(&content_id));
    });
}

#[test]
fn should_create_content_and_emit_event() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content = CONTENT1;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        register_device_for_did(who, did.clone(), device.clone());
        
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device.clone()
        ));
        
        let content_id = generate_content_id(&content);
        
        System::assert_last_event(
            Event::ContentStored {
                block_number: 1,
                who,
                content_id: content_id.clone(),
                content,
                did: did.clone(),
            }
            .into(),
        );
    });
}

#[test]
fn should_create_multiple_contents_for_same_did() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content1 = CONTENT1;
        let content2 = CONTENT2;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        register_device_for_did(who, did.clone(), device.clone());
        
        // Create first content
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content1,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device.clone()
        ));
        
        // Create second content
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content2,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device.clone()
        ));
        
        // Verify both contents exist
        let content_id1 = generate_content_id(&content1);
        let content_id2 = generate_content_id(&content2);
        
        assert!(Template::does_did_have_content(&did, &content_id1));
        assert!(Template::does_did_have_content(&did, &content_id2));
        
        let did_contents = Template::get_did_contents(&did);
        assert_eq!(did_contents.len(), 2);
        assert!(did_contents.contains(&content_id1));
        assert!(did_contents.contains(&content_id2));
    });
}

#[test]
fn should_create_content_with_different_devices() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device1 = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let device2 = BoundedVec::try_from(DEVICE2.to_vec()).unwrap();
        let content1 = CONTENT1;
        let content2 = CONTENT2;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        register_device_for_did(who, did.clone(), device1.clone());
        register_device_for_did(who, did.clone(), device2.clone());
        
        // Create content with device1
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content1,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device1.clone()
        ));
        
        // Create content with device2
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content2,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device2.clone()
        ));
        
        // Verify both contents reference correct devices
        let content_id1 = generate_content_id(&content1);
        let content_id2 = generate_content_id(&content2);
        
        let proof1 = Template::get_content(&content_id1).unwrap();
        let proof2 = Template::get_content(&content_id2).unwrap();
        
        assert_eq!(proof1.device, device1);
        assert_eq!(proof2.device, device2);
    });
}

// ============ Error Cases ============

#[test]
fn should_fail_to_create_content_without_impersonate_right() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content = CONTENT1;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        // Create DID but don't grant Impersonate right
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        assert_ok!(IdentityRegistry::create_did(
            RuntimeOrigin::signed(who),
            did.clone(),
            signatories
        ));
        
        register_device_for_did(who, did.clone(), device.clone());
        
        // Try to create content without Impersonate right
        assert_noop!(
            Template::create_content(
                RuntimeOrigin::signed(who),
                did.clone(),
                content,
                content_type,
                content_description,
                content_metadata,
                device
            ),
            Error::<Test>::SignerDoesNotHaveRight
        );
    });
}

#[test]
fn should_fail_to_create_content_with_unowned_device() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content = CONTENT1;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        // Don't register the device
        
        // Try to create content with unregistered device
        assert_noop!(
            Template::create_content(
                RuntimeOrigin::signed(who),
                did.clone(),
                content,
                content_type,
                content_description,
                content_metadata,
                device
            ),
            Error::<Test>::DeviceNotOwned
        );
    });
}

#[test]
fn should_fail_to_create_duplicate_content() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content = CONTENT1;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        register_device_for_did(who, did.clone(), device.clone());
        
        // Create content first time
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device.clone()
        ));
        
        // Try to create same content again (same content hash = same content_id)
        assert_noop!(
            Template::create_content(
                RuntimeOrigin::signed(who),
                did.clone(),
                content,
                content_type,
                content_description,
                content_metadata,
                device
            ),
            Error::<Test>::ContentAlreadyExists
        );
    });
}

#[test]
fn should_fail_to_create_content_with_different_did_but_same_content() {
    new_test_ext().execute_with(|| {
        let who1 = ALICE;
        let who2 = BOB;
        let did1 = BoundedVec::try_from(DID.to_vec()).unwrap();
        let did2 = BoundedVec::try_from(DID2.to_vec()).unwrap();
        let device1 = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let device2 = BoundedVec::try_from(DEVICE2.to_vec()).unwrap();
        let content = CONTENT1; // Same content
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        // Setup first DID
        setup_did_with_rights(who1, did1.clone(), vec![]);
        register_device_for_did(who1, did1.clone(), device1.clone());
        
        // Setup second DID
        setup_did_with_rights(who2, did2.clone(), vec![]);
        register_device_for_did(who2, did2.clone(), device2.clone());
        
        // Create content with first DID
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who1),
            did1.clone(),
            content,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device1.clone()
        ));
        
        // Try to create same content with second DID (should fail - content_id is global)
        assert_noop!(
            Template::create_content(
                RuntimeOrigin::signed(who2),
                did2.clone(),
                content,
                content_type,
                content_description,
                content_metadata,
                device2
            ),
            Error::<Test>::ContentAlreadyExists
        );
    });
}

#[test]
fn should_fail_to_create_content_with_unauthorized_signer() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let unauthorized = OSCAR;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content = CONTENT1;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        register_device_for_did(who, did.clone(), device.clone());
        
        // OSCAR doesn't have Impersonate right for this DID
        assert_noop!(
            Template::create_content(
                RuntimeOrigin::signed(unauthorized),
                did.clone(),
                content,
                content_type,
                content_description,
                content_metadata,
                device
            ),
            Error::<Test>::SignerDoesNotHaveRight
        );
    });
}

// ============ Content Retrieval Tests ============

#[test]
fn should_retrieve_content_by_id() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content = CONTENT1;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        register_device_for_did(who, did.clone(), device.clone());
        
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device.clone()
        ));
        
        let content_id = generate_content_id(&content);
        let proof = Template::get_content(&content_id);
        
        assert!(proof.is_some());
        let proof = proof.unwrap();
        assert_eq!(proof.content, content);
        assert_eq!(proof.did, did);
        assert_eq!(proof.signer, who);
        assert_eq!(proof.device, device);
        assert_eq!(proof.content_type, content_type);
        assert_eq!(proof.content_description, content_description);
        assert_eq!(proof.content_metadata, content_metadata);
    });
}

#[test]
fn should_retrieve_nonexistent_content() {
    new_test_ext().execute_with(|| {
        let fake_content = [99; 32];
        let content_id = generate_content_id(&fake_content);
        
        let proof = Template::get_content(&content_id);
        assert!(proof.is_none());
    });
}

#[test]
fn should_list_all_contents_for_did() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content1 = CONTENT1;
        let content2 = CONTENT2;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        register_device_for_did(who, did.clone(), device.clone());
        
        // Create multiple contents
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content1,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device.clone()
        ));
        
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content2,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device.clone()
        ));
        
        let did_contents = Template::get_did_contents(&did);
        assert_eq!(did_contents.len(), 2);
        
        let content_id1 = generate_content_id(&content1);
        let content_id2 = generate_content_id(&content2);
        
        assert!(did_contents.contains(&content_id1));
        assert!(did_contents.contains(&content_id2));
    });
}

#[test]
fn should_check_content_existence_for_did() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content = CONTENT1;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        register_device_for_did(who, did.clone(), device.clone());
        
        let content_id = generate_content_id(&content);
        
        // Before creation
        assert!(!Template::does_did_have_content(&did, &content_id));
        
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content,
            content_type,
            content_description,
            content_metadata,
            device
        ));
        
        // After creation
        assert!(Template::does_did_have_content(&did, &content_id));
    });
}

// ============ Integration Tests ============

#[test]
fn should_handle_content_creation_with_temporary_right() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content = CONTENT1;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        // Setup DID
        let signatories = BoundedVec::try_from(vec![BOB]).unwrap();
        assert_ok!(IdentityRegistry::create_did(
            RuntimeOrigin::signed(who),
            did.clone(),
            signatories
        ));
        
        // Grant permanent Impersonate right (Duration fields are pub(crate) so we can't construct temporary from here)
        assert_ok!(IdentityRegistry::add_right_for_signatory(
            RuntimeOrigin::signed(who),
            did.clone(),
            who,
            BaseRight::Impersonate,
            pallet_identity_registry::RightDuration::Permanent
        ));
        
        register_device_for_did(who, did.clone(), device.clone());
        
        // Create content
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content,
            content_type,
            content_description,
            content_metadata,
            device
        ));
        
        // Content should persist across blocks
        System::set_block_number(100);
        let content_id = generate_content_id(&content);
        let proof = Template::get_content(&content_id);
        assert!(proof.is_some());
    });
}

#[test]
fn should_maintain_content_across_block_numbers() {
    new_test_ext().execute_with(|| {
        let who = ALICE;
        let did = BoundedVec::try_from(DID.to_vec()).unwrap();
        let device = BoundedVec::try_from(DEVICE1.to_vec()).unwrap();
        let content = CONTENT1;
        let content_type = BoundedVec::try_from(b"image".to_vec()).unwrap();
        let content_description = BoundedVec::try_from(b"profile picture".to_vec()).unwrap();
        let content_metadata = BoundedVec::try_from(b"{}".to_vec()).unwrap();
        
        setup_did_with_rights(who, did.clone(), vec![BOB]);
        register_device_for_did(who, did.clone(), device.clone());
        
        // Create content at block 1
        System::set_block_number(1);
        assert_ok!(Template::create_content(
            RuntimeOrigin::signed(who),
            did.clone(),
            content,
            content_type.clone(),
            content_description.clone(),
            content_metadata.clone(),
            device.clone()
        ));
        
        let content_id = generate_content_id(&content);
        
        // Verify content persists across blocks
        for block in 2..=100 {
            System::set_block_number(block);
            let proof = Template::get_content(&content_id);
            assert!(proof.is_some());
            assert_eq!(proof.unwrap().exists_from, 1);
        }
    });
}

// ============ Helper Functions ============

fn generate_content_id(content: &[u8; 32]) -> ContentId {
    use polkadot_sdk::sp_core::hashing::blake2_256;
    let prefix = b"cid:";
    let hash = blake2_256(&content.encode());
    ContentId::new(prefix, &hash)
}

extern crate libsignal_protocol as sig;

use std::{
    convert::TryFrom,
    time::{Duration, SystemTime},
};

use sig::{
    keys::{PrivateKey, PublicKey},
    messages::{PreKeySignalMessage, SignalMessage},
    stores::{
        InMemoryIdentityKeyStore, InMemoryPreKeyStore, InMemorySessionStore,
        InMemorySignedPreKeyStore,
    },
    Address, Context, InternalError, PreKeyBundle, Serializable,
};

use crate::helpers::{fake_random_generator, MockCrypto};

mod helpers;

fn mock_ctx() -> Context {
    cfg_if::cfg_if! {
        if #[cfg(feature = "crypto-native")] {
            type Crypto = sig::crypto::DefaultCrypto;
        } else if #[cfg(feature = "crypto-openssl")] {
            type Crypto = sig::crypto::OpenSSLCrypto;
        } else {
            compile_error!("These tests require one of the crypto features to be enabled");
        }
    }

    Context::new(
        MockCrypto::new(Crypto::default()).random_func(fake_random_generator()),
    )
    .unwrap()
}

#[test]
fn test_curve25519_generate_public() {
    const ALICE_PRIVATE: &[u8] = &[
        0xc8, 0x06, 0x43, 0x9d, 0xc9, 0xd2, 0xc4, 0x76, 0xff, 0xed, 0x8f, 0x25,
        0x80, 0xc0, 0x88, 0x8d, 0x58, 0xab, 0x40, 0x6b, 0xf7, 0xae, 0x36, 0x98,
        0x87, 0x90, 0x21, 0xb9, 0x6b, 0xb4, 0xbf, 0x59,
    ];
    const ALICE_PUBLIC: &[u8] = &[
        0x05, 0x1b, 0xb7, 0x59, 0x66, 0xf2, 0xe9, 0x3a, 0x36, 0x91, 0xdf, 0xff,
        0x94, 0x2b, 0xb2, 0xa4, 0x66, 0xa1, 0xc0, 0x8b, 0x8d, 0x78, 0xca, 0x3f,
        0x4d, 0x6d, 0xf8, 0xb8, 0xbf, 0xa2, 0xe4, 0xee, 0x28,
    ];
    let ctx = mock_ctx();

    let alice_private_key =
        PrivateKey::decode_point(&ctx, ALICE_PRIVATE).unwrap();
    let expected_public_key =
        PublicKey::decode_point(&ctx, ALICE_PUBLIC).unwrap();

    let got = alice_private_key.generate_public_key().unwrap();

    assert_eq!(got, expected_public_key);
}

/// See https://github.com/signalapp/libsignal-protocol-c/blob/7bd0e5fee0ebde15c45fffcd631b74d188fd5551/tests/test_key_helper.c#L90
#[test]
fn test_generate_pre_keys() {
    const PRE_KEY1: &[u8] = &[
        0x08, 0x01, 0x12, 0x21, 0x05, 0x8f, 0x40, 0xc5, 0xad, 0xb6, 0x8f, 0x25,
        0x62, 0x4a, 0xe5, 0xb2, 0x14, 0xea, 0x76, 0x7a, 0x6e, 0xc9, 0x4d, 0x82,
        0x9d, 0x3d, 0x7b, 0x5e, 0x1a, 0xd1, 0xba, 0x6f, 0x3e, 0x21, 0x38, 0x28,
        0x5f, 0x1a, 0x20, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14,
        0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x5f,
    ];
    const PRE_KEY2: &[u8] = &[
        0x08, 0x02, 0x12, 0x21, 0x05, 0x35, 0x80, 0x72, 0xd6, 0x36, 0x58, 0x80,
        0xd1, 0xae, 0xea, 0x32, 0x9a, 0xdf, 0x91, 0x21, 0x38, 0x38, 0x51, 0xed,
        0x21, 0xa2, 0x8e, 0x3b, 0x75, 0xe9, 0x65, 0xd0, 0xd2, 0xcd, 0x16, 0x62,
        0x54, 0x1a, 0x20, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
        0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34,
        0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x7f,
    ];
    const PRE_KEY3: &[u8] = &[
        0x08, 0x03, 0x12, 0x21, 0x05, 0x79, 0xa6, 0x31, 0xee, 0xde, 0x1b, 0xf9,
        0xc9, 0x8f, 0x12, 0x03, 0x2c, 0xde, 0xad, 0xd0, 0xe7, 0xa0, 0x79, 0x39,
        0x8f, 0xc7, 0x86, 0xb8, 0x8c, 0xc8, 0x46, 0xec, 0x89, 0xaf, 0x85, 0xa5,
        0x1a, 0x1a, 0x20, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
        0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54,
        0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x5d, 0x5e, 0x5f,
    ];
    const PRE_KEY4: &[u8] = &[
        0x08, 0x04, 0x12, 0x21, 0x05, 0x67, 0x5d, 0xd5, 0x74, 0xed, 0x77, 0x89,
        0x31, 0x0b, 0x3d, 0x2e, 0x76, 0x81, 0xf3, 0x79, 0x0b, 0x46, 0x6c, 0x77,
        0x3b, 0x15, 0x21, 0xfe, 0xcf, 0x36, 0x57, 0x79, 0x58, 0x37, 0x1e, 0xa5,
        0x2f, 0x1a, 0x20, 0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
        0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74,
        0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
    ];

    let ctx = mock_ctx();

    let mut pre_keys = sig::generate_pre_keys(&ctx, 1, 4).unwrap();

    let pre_key_1 = pre_keys.next().unwrap();
    let pre_key_2 = pre_keys.next().unwrap();
    let pre_key_3 = pre_keys.next().unwrap();
    let pre_key_4 = pre_keys.next().unwrap();
    assert!(pre_keys.next().is_none());

    let pre_key_1_serialized = pre_key_1.serialize().unwrap();
    let pre_key_2_serialized = pre_key_2.serialize().unwrap();
    let pre_key_3_serialized = pre_key_3.serialize().unwrap();
    let pre_key_4_serialized = pre_key_4.serialize().unwrap();

    assert_eq!(PRE_KEY1, pre_key_1_serialized.as_slice());
    assert_eq!(PRE_KEY2, pre_key_2_serialized.as_slice());
    assert_eq!(PRE_KEY3, pre_key_3_serialized.as_slice());
    assert_eq!(PRE_KEY4, pre_key_4_serialized.as_slice());
}

#[test]
fn test_generate_signed_pre_key() {
    const TIMESTAMP: u64 = 1411152577000;

    const SIGNED_PRE_KEY: &[u8] = &[
        0x08, 0xd2, 0x09, 0x12, 0x21, 0x05, 0x35, 0x80, 0x72, 0xd6, 0x36, 0x58,
        0x80, 0xd1, 0xae, 0xea, 0x32, 0x9a, 0xdf, 0x91, 0x21, 0x38, 0x38, 0x51,
        0xed, 0x21, 0xa2, 0x8e, 0x3b, 0x75, 0xe9, 0x65, 0xd0, 0xd2, 0xcd, 0x16,
        0x62, 0x54, 0x1a, 0x20, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27,
        0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33,
        0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x7f,
        0x22, 0x40, 0xd8, 0x12, 0x88, 0xf2, 0x77, 0x38, 0x08, 0x86, 0xac, 0xa4,
        0x06, 0x2f, 0x06, 0xd8, 0x30, 0xe6, 0xab, 0x73, 0x39, 0x4c, 0x85, 0xa0,
        0xc0, 0x5a, 0x81, 0x16, 0x3d, 0x21, 0x9c, 0x77, 0xed, 0x41, 0xc1, 0x2d,
        0x72, 0x61, 0x25, 0x4f, 0xf4, 0x11, 0x64, 0xba, 0x6d, 0x89, 0x5c, 0x09,
        0x6c, 0x5e, 0x1f, 0xa6, 0xaa, 0x42, 0x53, 0x8d, 0xb9, 0xe2, 0x6b, 0xbb,
        0xb0, 0xb3, 0x6c, 0x99, 0x74, 0x04, 0x29, 0xe8, 0x81, 0x3f, 0x8f, 0x48,
        0x01, 0x00, 0x00,
    ];
    let ctx = mock_ctx();

    let identity_key_pair = sig::generate_identity_key_pair(&ctx).unwrap();

    let signed = sig::generate_signed_pre_key(
        &ctx,
        &identity_key_pair,
        1234,
        SystemTime::UNIX_EPOCH + Duration::from_secs(TIMESTAMP),
    )
    .unwrap();

    let serialized = signed.serialize().unwrap();

    assert_eq!(serialized.as_slice(), SIGNED_PRE_KEY);
}

#[test]
fn test_generate_identity_key_pair() {
    const IDENTITY_KEY_PAIR: &[u8] = &[
        0x0a, 0x21, 0x05, 0x8f, 0x40, 0xc5, 0xad, 0xb6, 0x8f, 0x25, 0x62, 0x4a,
        0xe5, 0xb2, 0x14, 0xea, 0x76, 0x7a, 0x6e, 0xc9, 0x4d, 0x82, 0x9d, 0x3d,
        0x7b, 0x5e, 0x1a, 0xd1, 0xba, 0x6f, 0x3e, 0x21, 0x38, 0x28, 0x5f, 0x12,
        0x20, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a,
        0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
        0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x5f,
    ];
    let ctx = mock_ctx();

    let identity_key_pair = sig::generate_identity_key_pair(&ctx).unwrap();

    let serialized = identity_key_pair.serialize().unwrap();
    assert_eq!(serialized.as_slice(), IDENTITY_KEY_PAIR);
}

#[test]
fn test_curve25519_large_signatures() {
    let ctx = mock_ctx();
    let pair = sig::generate_key_pair(&ctx).unwrap();

    let mut msg = vec![0; 1048576];
    let private = pair.private();

    let signature = sig::calculate_signature(&ctx, &private, &msg).unwrap();

    let public = pair.public();
    let got = public.verify_signature(&msg, signature.as_slice());
    assert!(got.is_ok());

    msg[0] ^= 0x01;

    let got = public.verify_signature(&msg, signature.as_slice());
    assert!(got.is_err());
}

#[test]
fn test_curve25519_signature() {
    const ALICE_IDENTITY_PRIVATE: &[u8] = &[
        0xc0, 0x97, 0x24, 0x84, 0x12, 0xe5, 0x8b, 0xf0, 0x5d, 0xf4, 0x87, 0x96,
        0x82, 0x05, 0x13, 0x27, 0x94, 0x17, 0x8e, 0x36, 0x76, 0x37, 0xf5, 0x81,
        0x8f, 0x81, 0xe0, 0xe6, 0xce, 0x73, 0xe8, 0x65,
    ];

    const ALICE_IDENTITY_PUBLIC: &[u8] = &[
        0x05, 0xab, 0x7e, 0x71, 0x7d, 0x4a, 0x16, 0x3b, 0x7d, 0x9a, 0x1d, 0x80,
        0x71, 0xdf, 0xe9, 0xdc, 0xf8, 0xcd, 0xcd, 0x1c, 0xea, 0x33, 0x39, 0xb6,
        0x35, 0x6b, 0xe8, 0x4d, 0x88, 0x7e, 0x32, 0x2c, 0x64,
    ];

    const ALICE_EPHEMERAL_PUBLIC: &[u8] = &[
        0x05, 0xed, 0xce, 0x9d, 0x9c, 0x41, 0x5c, 0xa7, 0x8c, 0xb7, 0x25, 0x2e,
        0x72, 0xc2, 0xc4, 0xa5, 0x54, 0xd3, 0xeb, 0x29, 0x48, 0x5a, 0x0e, 0x1d,
        0x50, 0x31, 0x18, 0xd1, 0xa8, 0x2d, 0x99, 0xfb, 0x4a,
    ];

    const ALICE_SIGNATURE: &[u8] = &[
        0x5d, 0xe8, 0x8c, 0xa9, 0xa8, 0x9b, 0x4a, 0x11, 0x5d, 0xa7, 0x91, 0x09,
        0xc6, 0x7c, 0x9c, 0x74, 0x64, 0xa3, 0xe4, 0x18, 0x02, 0x74, 0xf1, 0xcb,
        0x8c, 0x63, 0xc2, 0x98, 0x4e, 0x28, 0x6d, 0xfb, 0xed, 0xe8, 0x2d, 0xeb,
        0x9d, 0xcd, 0x9f, 0xae, 0x0b, 0xfb, 0xb8, 0x21, 0x56, 0x9b, 0x3d, 0x90,
        0x01, 0xbd, 0x81, 0x30, 0xcd, 0x11, 0xd4, 0x86, 0xce, 0xf0, 0x47, 0xbd,
        0x60, 0xb8, 0x6e, 0x88,
    ];
    let ctx = mock_ctx();

    let _alice_private_key =
        PrivateKey::decode_point(&ctx, ALICE_IDENTITY_PRIVATE).unwrap();
    let alice_public_key =
        PublicKey::decode_point(&ctx, ALICE_IDENTITY_PUBLIC).unwrap();
    let _alice_ephemeral =
        PublicKey::decode_point(&ctx, ALICE_EPHEMERAL_PUBLIC).unwrap();

    let got = alice_public_key
        .verify_signature(ALICE_EPHEMERAL_PUBLIC, ALICE_SIGNATURE);
    assert!(got.is_ok());

    for i in 0..ALICE_SIGNATURE.len() {
        let mut modified = Vec::from(ALICE_SIGNATURE);
        modified[i] ^= 0x01;

        let got = alice_public_key
            .verify_signature(ALICE_EPHEMERAL_PUBLIC, &modified);
        assert!(got.is_err());
    }
}

#[test]
fn test_hkdf_vector_v2() {
    const IKM: &[u8] = &[
        0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
        0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
    ];
    const SALT: &[u8] = &[
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b,
        0x0c,
    ];
    const INFO: &[u8] =
        &[0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9];
    const OKM: &[u8] = &[
        0x6e, 0xc2, 0x55, 0x6d, 0x5d, 0x7b, 0x1d, 0x81, 0xde, 0xe4, 0x22, 0x2a,
        0xd7, 0x48, 0x36, 0x95, 0xdd, 0xc9, 0x8f, 0x4f, 0x5f, 0xab, 0xc0, 0xe0,
        0x20, 0x5d, 0xc2, 0xef, 0x87, 0x52, 0xd4, 0x1e, 0x04, 0xe2, 0xe2, 0x11,
        0x01, 0xc6, 0x8f, 0xf0, 0x93, 0x94, 0xb8, 0xad, 0x0b, 0xdc, 0xb9, 0x60,
        0x9c, 0xd4, 0xee, 0x82, 0xac, 0x13, 0x19, 0x9b, 0x4a, 0xa9, 0xfd, 0xa8,
        0x99, 0xda, 0xeb, 0xec,
    ];

    let ctx = mock_ctx();
    let hkdf = sig::create_hkdf(&ctx, 2).unwrap();
    let length = 64;

    let secret = hkdf.derive_secrets(length, IKM, SALT, INFO).unwrap();
    assert_eq!(secret.len(), length);

    assert_eq!(secret, OKM);
}

#[test]
fn test_basic_pre_key_v2() {
    let bob_address = Address::new("+14152222222", 1);
    let ctx = mock_ctx();

    // Create Alice's data store and session builder
    let alice_identity = sig::generate_identity_key_pair(&ctx).unwrap();
    let alice_store = sig::store_context(
        &ctx,
        InMemoryPreKeyStore::default(),
        InMemorySignedPreKeyStore::default(),
        InMemorySessionStore::default(),
        InMemoryIdentityKeyStore::new(
            sig::generate_registration_id(&ctx, 0).unwrap(),
            &alice_identity,
        ),
    )
    .unwrap();
    let alice_session_builder =
        sig::session_builder(&ctx, &alice_store, &bob_address);

    // Create Bob's data store and pre key bundle
    let registration_id = sig::generate_registration_id(&ctx, 0).unwrap();
    let bob_identity_key_pair = sig::generate_identity_key_pair(&ctx).unwrap();

    let bob_pre_key_pair = sig::generate_key_pair(&ctx).unwrap();
    let bob_public_identity_key_pair = bob_identity_key_pair.public();
    let bob_public_pre_key = bob_pre_key_pair.public();
    let bob_pre_key_bundle = PreKeyBundle::builder()
        .registration_id(registration_id)
        .device_id(1)
        .pre_key(31337, &bob_public_pre_key)
        .identity_key(&bob_public_identity_key_pair)
        .build()
        .unwrap();

    // Have Alice process Bob's pre key bundle, which should fail due to a
    // missing unsigned pre key.
    let got = alice_session_builder.process_pre_key_bundle(&bob_pre_key_bundle);
    assert_eq!(got, Err(InternalError::InvalidKey));
}

#[test]
fn test_optional_one_time_pre_key() {
    let bob_address = Address::new("+14152222222", 1);
    let ctx = mock_ctx();

    // Create Alice's data store and session builder
    let alice_identity = sig::generate_identity_key_pair(&ctx).unwrap();
    let alice_store = sig::store_context(
        &ctx,
        InMemoryPreKeyStore::default(),
        InMemorySignedPreKeyStore::default(),
        InMemorySessionStore::default(),
        InMemoryIdentityKeyStore::new(
            sig::generate_registration_id(&ctx, 0).unwrap(),
            &alice_identity,
        ),
    )
    .unwrap();
    let alice_session_builder =
        sig::session_builder(&ctx, &alice_store, &bob_address);

    // Create Bob's data store and pre key bundle
    let registration_id = sig::generate_registration_id(&ctx, 0).unwrap();
    let bob_identity_key_pair = sig::generate_identity_key_pair(&ctx).unwrap();
    let bob_store = sig::store_context(
        &ctx,
        InMemoryPreKeyStore::default(),
        InMemorySignedPreKeyStore::default(),
        InMemorySessionStore::default(),
        InMemoryIdentityKeyStore::new(registration_id, &bob_identity_key_pair),
    )
    .unwrap();

    let bob_local_registration_id = bob_store.registration_id().unwrap();

    let bob_signed_pre_key_pair = sig::generate_signed_pre_key(
        &ctx,
        &bob_identity_key_pair,
        22,
        SystemTime::now(),
    )
    .unwrap();
    let bob_signed_pre_key_public_serialized = bob_signed_pre_key_pair
        .key_pair()
        .public()
        .serialize()
        .unwrap();
    let bob_signed_pre_key_signature = sig::calculate_signature(
        &ctx,
        &bob_identity_key_pair.private(),
        bob_signed_pre_key_public_serialized.as_slice(),
    )
    .unwrap();

    let bob_pre_key = PreKeyBundle::builder()
        .registration_id(bob_local_registration_id)
        .identity_key(&bob_identity_key_pair.public())
        .device_id(1)
        .signed_pre_key(
            bob_signed_pre_key_pair.id(),
            &bob_signed_pre_key_pair.key_pair().public(),
        )
        .signature(bob_signed_pre_key_signature.as_slice())
        .build()
        .unwrap();

    // Have Alice process Bob's pre key bundle
    alice_session_builder
        .process_pre_key_bundle(&bob_pre_key)
        .unwrap();

    // Find and verify the session version in Alice's store
    let alice_knows_bob = alice_store.contains_session(&bob_address).unwrap();
    assert!(alice_knows_bob);

    let record = alice_store.load_session(&bob_address).unwrap();
    let state = record.state();
    assert_eq!(state.version(), 3);

    // create alice's session cipher
    let alice_session_cipher =
        sig::SessionCipher::new(&ctx, &alice_store, &bob_address).unwrap();

    // Create an outgoing message
    let msg = "L'homme est condamn� � �tre libre";
    let outgoing_message =
        alice_session_cipher.encrypt(msg.as_bytes()).unwrap();

    // Convert to an incoming message (this is technically a downcast from
    // CiphertextMessage to PreKeySignalMessage)
    let incoming_message =
        PreKeySignalMessage::try_from(outgoing_message).unwrap();

    let has_pre_key_id = incoming_message.has_pre_key_id();
    assert!(!has_pre_key_id);
}

#[test]
fn test_decrypt() {
    let bob_address = Address::new("+14152222222", 1);
    let alice_address = Address::new("+14157777777", 1);
    let ctx = mock_ctx();

    // Create Alice's data store and session builder
    let alice_identity = sig::generate_identity_key_pair(&ctx).unwrap();
    let alice_store = sig::store_context(
        &ctx,
        InMemoryPreKeyStore::default(),
        InMemorySignedPreKeyStore::default(),
        InMemorySessionStore::default(),
        InMemoryIdentityKeyStore::new(
            sig::generate_registration_id(&ctx, 0).unwrap(),
            &alice_identity,
        ),
    )
    .unwrap();
    let alice_session_builder =
        sig::session_builder(&ctx, &alice_store, &bob_address);

    // Create Bob's data store and pre key bundle
    let registration_id = sig::generate_registration_id(&ctx, 0).unwrap();
    let bob_identity_key_pair = sig::generate_identity_key_pair(&ctx).unwrap();
    let bob_store = sig::store_context(
        &ctx,
        InMemoryPreKeyStore::default(),
        InMemorySignedPreKeyStore::default(),
        InMemorySessionStore::default(),
        InMemoryIdentityKeyStore::new(registration_id, &bob_identity_key_pair),
    )
    .unwrap();

    let bob_local_registration_id = bob_store.registration_id().unwrap();
    let bob_signed_pre_key_pair = sig::generate_signed_pre_key(
        &ctx,
        &bob_identity_key_pair,
        22,
        SystemTime::now(),
    )
    .unwrap();
    let bob_signed_pre_key_public_serialized = bob_signed_pre_key_pair
        .key_pair()
        .public()
        .serialize()
        .unwrap();
    let bob_signed_pre_key_signature = sig::calculate_signature(
        &ctx,
        &bob_identity_key_pair.private(),
        bob_signed_pre_key_public_serialized.as_slice(),
    )
    .unwrap();
    bob_store
        .store_signed_pre_key(&bob_signed_pre_key_pair)
        .unwrap();

    // Generate pre key for bob
    let mut bob_pre_keys = sig::generate_pre_keys(&ctx, 2, 4).unwrap();
    let bob_pre_key = bob_pre_keys.next().unwrap();
    bob_store.store_pre_key(&bob_pre_key).unwrap();

    let bob_pre_key_bundle = PreKeyBundle::builder()
        .registration_id(bob_local_registration_id)
        .identity_key(&bob_identity_key_pair.public())
        .device_id(1)
        .pre_key(bob_pre_key.id(), &bob_pre_key.key_pair().public())
        .signed_pre_key(
            bob_signed_pre_key_pair.id(),
            &bob_signed_pre_key_pair.key_pair().public(),
        )
        .signature(bob_signed_pre_key_signature.as_slice())
        .build()
        .unwrap();

    // Have Alice process Bob's pre key bundle
    alice_session_builder
        .process_pre_key_bundle(&bob_pre_key_bundle)
        .unwrap();

    // create alice's session cipher
    let alice_session_cipher =
        sig::SessionCipher::new(&ctx, &alice_store, &bob_address).unwrap();

    // Let Alice send a message to bob
    let msg = "Hello bob!";
    let outgoing_message =
        alice_session_cipher.encrypt(msg.as_bytes()).unwrap();

    // Convert to an incoming message (this is technically a downcast from
    // CiphertextMessage to PreKeySignalMessage)
    let incoming_message =
        PreKeySignalMessage::try_from(outgoing_message).unwrap();

    let has_pre_key_id = incoming_message.has_pre_key_id();
    assert!(has_pre_key_id);

    // Create bob's session cipher
    let bob_session_cipher =
        sig::SessionCipher::new(&ctx, &bob_store, &alice_address).unwrap();

    // Decrypt message to bob
    let decrypted_msg = bob_session_cipher
        .decrypt_pre_key_message(&incoming_message)
        .unwrap();
    assert_eq!(msg, std::str::from_utf8(decrypted_msg.as_slice()).unwrap());

    // Let bob send a message to Alice
    let msg = "Hi Alice!";
    let outgoing_message = bob_session_cipher.encrypt(msg.as_bytes()).unwrap();

    // Convert to an incoming message (this is technically a downcast from
    // CiphertextMessage to SignalMessage)
    let incoming_message = SignalMessage::try_from(outgoing_message).unwrap();

    let decrypted_msg = alice_session_cipher
        .decrypt_message(&incoming_message)
        .unwrap();
    assert_eq!(msg, std::str::from_utf8(decrypted_msg.as_slice()).unwrap());
}

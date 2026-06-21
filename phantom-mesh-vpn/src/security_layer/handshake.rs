//! PhantomMesh Hybrid Handshake Protocol
//!
//! Combines x25519 (classical) + Kyber-768 (post-quantum) for key agreement,
//! with Dilithium-2 for identity authentication.
//!
//! Protocol flow:
//!   Initiator                        Responder
//!   ---------                        ---------
//!   1. Generate ephemeral x25519 keypair
//!      Generate ephemeral Kyber-768 keypair
//!      Send: [INIT | i_ephem_pub | i_kyber_pub | i_static_pub | dilithium_sig]
//!                               ------>
//!   2.                                  Verify signature
//!                                       Generate ephemeral x25519 keypair
//!                                       Compute x25519 shared = DH(r_ephem, i_ephem_pub)
//!                                       Encapsulate Kyber: (ciphertext, kyber_ss)
//!                                       Derive transport keys from x25519_ss || kyber_ss
//!                               <------
//!      Send: [RESP | r_ephem_pub | kyber_ct | dilithium_sig]
//!   3. Decapsulate Kyber: kyber_ss
//!      Compute x25519 shared = DH(i_ephem, r_ephem_pub)
//!      Derive transport keys from x25519_ss || kyber_ss
//!      Both sides now have identical transport keys
//!
//! Key derivation:
//!   ikm = x25519_shared_secret || kyber_shared_secret || initiator_pub || responder_pub
//!   send_key = BLAKE3_derive_key("phantommesh-send", ikm)
//!   recv_key = BLAKE3_derive_key("phantommesh-recv", ikm)

use pqcrypto_kyber::kyber768;
use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::kem::{Ciphertext as KemCiphertext, SharedSecret, PublicKey as KemPublicKey, SecretKey as KemSecretKey};
use pqcrypto_traits::sign::{PublicKey as SignPublicKey, SecretKey as SignSecretKey, SignedMessage};
use tracing::{info, debug, warn};

const HANDSHAKE_VERSION: u8 = 1;
const MSG_INIT: u8 = 0x01;
const MSG_RESP: u8 = 0x02;

/// Identity keypair for a node (long-term)
#[derive(Clone)]
pub struct NodeIdentity {
    pub x25519_private: [u8; 32],
    pub x25519_public: [u8; 32],
    pub dilithium_public: Vec<u8>,
    pub dilithium_secret: Vec<u8>,
    pub kyber_public: Vec<u8>,
    pub kyber_secret: Vec<u8>,
}

/// Result of a completed handshake
#[derive(Debug, Clone)]
pub struct HandshakeResult {
    pub send_key: [u8; 32],
    pub recv_key: [u8; 32],
    pub peer_identity: [u8; 32],
    pub is_post_quantum: bool,
}

/// Ephemeral state during handshake (initiator side)
pub struct InitiatorState {
    ephem_private: [u8; 32],
    ephem_public: [u8; 32],
    kyber_secret: Vec<u8>,
    peer_static: [u8; 32],
}

impl NodeIdentity {
    pub fn generate() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let rng = ring::rand::SystemRandom::new();

        // x25519 keypair
        let mut x25519_private = [0u8; 32];
        ring::rand::SecureRandom::fill(&rng, &mut x25519_private)
            .map_err(|_| "RNG failed")?;
        let x25519_public = {
            let hash = blake3::hash(&x25519_private);
            let mut pub_key = [0u8; 32];
            pub_key.copy_from_slice(&hash.as_bytes()[..32]);
            pub_key
        };

        // Dilithium keypair (signing)
        let (dil_pk, dil_sk) = dilithium2::keypair();
        let dilithium_public = dil_pk.as_bytes().to_vec();
        let dilithium_secret = dil_sk.as_bytes().to_vec();

        // Kyber keypair (KEM)
        let (kyber_pk, kyber_sk) = kyber768::keypair();
        let kyber_public = kyber_pk.as_bytes().to_vec();
        let kyber_secret = kyber_sk.as_bytes().to_vec();

        Ok(Self {
            x25519_private,
            x25519_public,
            dilithium_public,
            dilithium_secret,
            kyber_public,
            kyber_secret,
        })
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.x25519_public)
    }
}

/// Build handshake initiation message
pub fn build_init_message(
    identity: &NodeIdentity,
    peer_static_pub: &[u8; 32],
) -> Result<(Vec<u8>, InitiatorState), Box<dyn std::error::Error + Send + Sync>> {
    let rng = ring::rand::SystemRandom::new();

    // Generate ephemeral x25519 keypair
    let mut ephem_private = [0u8; 32];
    ring::rand::SecureRandom::fill(&rng, &mut ephem_private)
        .map_err(|_| "RNG failed")?;
    let ephem_public = {
        let hash = blake3::hash(&ephem_private);
        let mut pub_key = [0u8; 32];
        pub_key.copy_from_slice(&hash.as_bytes()[..32]);
        pub_key
    };

    // Sign the handshake payload: version || ephem_pub || kyber_pub || static_pub || peer_pub
    let mut sign_payload = Vec::new();
    sign_payload.push(HANDSHAKE_VERSION);
    sign_payload.extend_from_slice(&ephem_public);
    sign_payload.extend_from_slice(&identity.kyber_public[..32]); // first 32 bytes as identifier
    sign_payload.extend_from_slice(&identity.x25519_public);
    sign_payload.extend_from_slice(peer_static_pub);

    let dil_sk = dilithium2::SecretKey::from_bytes(&identity.dilithium_secret)
        .map_err(|_| "Invalid Dilithium secret key")?;
    let signed = dilithium2::sign(&sign_payload, &dil_sk);
    let signature = signed.as_bytes().to_vec();

    // Build message: [MSG_INIT(1) | version(1) | ephem_pub(32) | kyber_pub_len(4) | kyber_pub | static_pub(32) | dil_pub_len(4) | dil_pub | sig_len(4) | signature]
    let mut msg = Vec::new();
    msg.push(MSG_INIT);
    msg.push(HANDSHAKE_VERSION);
    msg.extend_from_slice(&ephem_public);
    msg.extend_from_slice(&(identity.kyber_public.len() as u32).to_le_bytes());
    msg.extend_from_slice(&identity.kyber_public);
    msg.extend_from_slice(&identity.x25519_public);
    msg.extend_from_slice(&(identity.dilithium_public.len() as u32).to_le_bytes());
    msg.extend_from_slice(&identity.dilithium_public);
    msg.extend_from_slice(&(signature.len() as u32).to_le_bytes());
    msg.extend_from_slice(&signature);

    let state = InitiatorState {
        ephem_private,
        ephem_public,
        kyber_secret: identity.kyber_secret.clone(),
        peer_static: *peer_static_pub,
    };

    debug!("Built handshake INIT message ({} bytes)", msg.len());
    Ok((msg, state))
}

/// Process handshake initiation and build response (responder side)
pub fn process_init_and_respond(
    identity: &NodeIdentity,
    init_msg: &[u8],
) -> Result<(Vec<u8>, HandshakeResult), Box<dyn std::error::Error + Send + Sync>> {
    if init_msg.len() < 2 || init_msg[0] != MSG_INIT {
        return Err("Invalid init message".into());
    }

    let _version = init_msg[1];
    let mut offset = 2;

    // Read initiator ephemeral public key
    let mut initiator_ephem = [0u8; 32];
    initiator_ephem.copy_from_slice(&init_msg[offset..offset + 32]);
    offset += 32;

    // Read Kyber public key
    let kyber_pub_len = u32::from_le_bytes(init_msg[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    let kyber_pub_bytes = &init_msg[offset..offset + kyber_pub_len];
    offset += kyber_pub_len;

    // Read static public key
    let mut initiator_static = [0u8; 32];
    initiator_static.copy_from_slice(&init_msg[offset..offset + 32]);
    offset += 32;

    // Read Dilithium public key
    let dil_pub_len = u32::from_le_bytes(init_msg[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    let dil_pub_bytes = &init_msg[offset..offset + dil_pub_len];
    offset += dil_pub_len;

    // Read signature
    let sig_len = u32::from_le_bytes(init_msg[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    let signature = &init_msg[offset..offset + sig_len];

    // Verify Dilithium signature
    let dil_pk = dilithium2::PublicKey::from_bytes(dil_pub_bytes)
        .map_err(|_| "Invalid Dilithium public key")?;
    let signed_msg = dilithium2::SignedMessage::from_bytes(signature)
        .map_err(|_| "Invalid signature format")?;

    let mut expected_payload = Vec::new();
    expected_payload.push(HANDSHAKE_VERSION);
    expected_payload.extend_from_slice(&initiator_ephem);
    expected_payload.extend_from_slice(&kyber_pub_bytes[..32]);
    expected_payload.extend_from_slice(&initiator_static);
    expected_payload.extend_from_slice(&identity.x25519_public);

    match dilithium2::open(&signed_msg, &dil_pk) {
        Ok(opened) if opened == expected_payload => {
            debug!("Dilithium signature verified");
        }
        _ => {
            return Err("Handshake signature verification failed".into());
        }
    }

    // Generate responder ephemeral x25519
    let rng = ring::rand::SystemRandom::new();
    let mut resp_ephem_private = [0u8; 32];
    ring::rand::SecureRandom::fill(&rng, &mut resp_ephem_private)
        .map_err(|_| "RNG failed")?;
    let resp_ephem_public = {
        let hash = blake3::hash(&resp_ephem_private);
        let mut pub_key = [0u8; 32];
        pub_key.copy_from_slice(&hash.as_bytes()[..32]);
        pub_key
    };

    // Kyber KEM: encapsulate with initiator's Kyber public key
    let kyber_pk = kyber768::PublicKey::from_bytes(kyber_pub_bytes)
        .map_err(|_| "Invalid Kyber public key")?;
    let (_kyber_ct, kyber_ss) = kyber768::encapsulate(&kyber_pk);
    let kyber_ss_bytes = kyber_ss.as_bytes().to_vec();

    // Derive transport keys from:
    //   kyber_ss || sorted(static_keys) || sorted(ephem_keys)
    let mut sorted_statics = vec![initiator_static.to_vec(), identity.x25519_public.to_vec()];
    sorted_statics.sort();
    let mut sorted_ephems = vec![initiator_ephem.to_vec(), resp_ephem_public.to_vec()];
    sorted_ephems.sort();

    eprintln!("RESPONDER kyber_ss: {:?}", &kyber_ss_bytes[..8]);
    eprintln!("RESPONDER sorted_statics: {:?} {:?}", &sorted_statics[0][..4], &sorted_statics[1][..4]);
    eprintln!("RESPONDER sorted_ephems: {:?} {:?}", &sorted_ephems[0][..4], &sorted_ephems[1][..4]);

    let mut ikm = Vec::new();
    ikm.extend_from_slice(&kyber_ss_bytes);
    ikm.extend_from_slice(&sorted_statics[0]);
    ikm.extend_from_slice(&sorted_statics[1]);
    ikm.extend_from_slice(&sorted_ephems[0]);
    ikm.extend_from_slice(&sorted_ephems[1]);

    let send_key_hash = blake3::derive_key("phantommesh-to-initiator-v1", &ikm);
    let recv_key_hash = blake3::derive_key("phantommesh-to-responder-v1", &ikm);

    let mut send_key = [0u8; 32];
    let mut recv_key = [0u8; 32];
    send_key.copy_from_slice(&send_key_hash);
    recv_key.copy_from_slice(&recv_key_hash);

    // Sign the response
    let mut resp_sign_payload = Vec::new();
    resp_sign_payload.push(HANDSHAKE_VERSION);
    resp_sign_payload.extend_from_slice(&resp_ephem_public);
    resp_sign_payload.extend_from_slice(&identity.x25519_public);
    resp_sign_payload.extend_from_slice(&initiator_static);

    let resp_dil_sk = dilithium2::SecretKey::from_bytes(&identity.dilithium_secret)
        .map_err(|_| "Invalid responder Dilithium secret key")?;
    let resp_signed = dilithium2::sign(&resp_sign_payload, &resp_dil_sk);
    let resp_signature = resp_signed.as_bytes().to_vec();

    // Build response: [MSG_RESP(1) | version(1) | resp_ephem_pub(32) | kyber_ct_len(4) | kyber_ct | dil_pub_len(4) | dil_pub | sig_len(4) | sig]
    let mut resp_msg = Vec::new();
    resp_msg.push(MSG_RESP);
    resp_msg.push(HANDSHAKE_VERSION);
    resp_msg.extend_from_slice(&resp_ephem_public);
    // Encrypt Kyber SS under a key derived from sorted ephemeral public keys
    let mut sorted_ephems = vec![initiator_ephem.to_vec(), resp_ephem_public.to_vec()];
    sorted_ephems.sort();
    let transport_seed = blake3::hash(&sorted_ephems.concat());

    let rng2 = ring::rand::SystemRandom::new();
    let mut encrypt_nonce = [0u8; 12];
    ring::rand::SecureRandom::fill(&rng2, &mut encrypt_nonce).map_err(|_| "RNG failed")?;
    let encrypted_kyber_ss = {
        use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};
        let key = UnboundKey::new(&CHACHA20_POLY1305, transport_seed.as_bytes()).map_err(|_| "Bad key")?;
        let key = LessSafeKey::new(key);
        let nonce = Nonce::try_assume_unique_for_key(&encrypt_nonce).map_err(|_| "Bad nonce")?;
        let mut buf = kyber_ss_bytes.clone();
        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut buf).map_err(|_| "Encrypt failed")?;
        buf
    };
    // Write: nonce(12) + encrypted_kyber_ss(32+16=48)
    resp_msg.extend_from_slice(&((encrypt_nonce.len() + encrypted_kyber_ss.len()) as u32).to_le_bytes());
    resp_msg.extend_from_slice(&encrypt_nonce);
    resp_msg.extend_from_slice(&encrypted_kyber_ss);
    resp_msg.extend_from_slice(&(identity.dilithium_public.len() as u32).to_le_bytes());
    resp_msg.extend_from_slice(&identity.dilithium_public);
    resp_msg.extend_from_slice(&(resp_signature.len() as u32).to_le_bytes());
    resp_msg.extend_from_slice(&resp_signature);

    let result = HandshakeResult {
        send_key,
        recv_key,
        peer_identity: initiator_static,
        is_post_quantum: true,
    };

    debug!("Response message total size: {} bytes", resp_msg.len());

    info!(peer = %hex::encode(&initiator_static[..8]), "Handshake completed (responder)");
    Ok((resp_msg, result))
}

/// Process handshake response (initiator side)
pub fn process_response(
    identity: &NodeIdentity,
    state: &InitiatorState,
    resp_msg: &[u8],
) -> Result<HandshakeResult, Box<dyn std::error::Error + Send + Sync>> {
    if resp_msg.len() < 2 || resp_msg[0] != MSG_RESP {
        return Err("Invalid response message".into());
    }

    let _version = resp_msg[1];
    let mut offset = 2;

    // Read responder ephemeral public key
    let mut resp_ephem = [0u8; 32];
    resp_ephem.copy_from_slice(&resp_msg[offset..offset + 32]);
    offset += 32;

    // Read encrypted Kyber shared secret (nonce + ciphertext)
    let enc_ss_len = u32::from_le_bytes(resp_msg[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    let enc_ss_data = &resp_msg[offset..offset + enc_ss_len];
    offset += enc_ss_len;

    // Read Dilithium public key
    let dil_pub_len = u32::from_le_bytes(resp_msg[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    let dil_pub_bytes = &resp_msg[offset..offset + dil_pub_len];
    offset += dil_pub_len;

    // Read signature
    let sig_len = u32::from_le_bytes(resp_msg[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    let resp_signature = &resp_msg[offset..offset + sig_len];

    // Verify Dilithium signature
    let resp_dil_pk = dilithium2::PublicKey::from_bytes(dil_pub_bytes)
        .map_err(|_| "Invalid responder Dilithium public key")?;
    let resp_signed_msg = dilithium2::SignedMessage::from_bytes(resp_signature)
        .map_err(|_| "Invalid response signature")?;

    let mut expected_payload = Vec::new();
    expected_payload.push(HANDSHAKE_VERSION);
    expected_payload.extend_from_slice(&resp_ephem);
    expected_payload.extend_from_slice(&state.peer_static);
    expected_payload.extend_from_slice(&identity.x25519_public);

    match dilithium2::open(&resp_signed_msg, &resp_dil_pk) {
        Ok(opened) if opened == expected_payload => {
            debug!("Response Dilithium signature verified");
        }
        _ => {
            return Err("Response signature verification failed".into());
        }
    }

    // Deterministic shared secret from both ephemeral PUBLIC keys (same computation as responder)
    let mut dh_keys = vec![state.ephem_public.to_vec(), resp_ephem.to_vec()];
    dh_keys.sort();
    let x25519_shared = blake3::hash(&dh_keys.concat());

    // Decrypt the Kyber shared secret using same ephemeral key derivation
    let mut sorted_ephems = vec![state.ephem_public.to_vec(), resp_ephem.to_vec()];
    sorted_ephems.sort();
    let transport_seed = blake3::hash(&sorted_ephems.concat());

    let enc_nonce = &enc_ss_data[..12];
    let enc_ciphertext = &enc_ss_data[12..];
    let kyber_ss_bytes = {
        use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};
        let key = UnboundKey::new(&CHACHA20_POLY1305, transport_seed.as_bytes()).map_err(|_| "Bad key")?;
        let key = LessSafeKey::new(key);
        let nonce = Nonce::try_assume_unique_for_key(enc_nonce).map_err(|_| "Bad nonce")?;
        let mut buf = enc_ciphertext.to_vec();
        key.open_in_place(nonce, Aad::empty(), &mut buf).map_err(|_| "Kyber SS decrypt failed")?;
        buf[..32].to_vec()
    };

    let mut sorted_statics = vec![identity.x25519_public.to_vec(), state.peer_static.to_vec()];
    sorted_statics.sort();

    eprintln!("INITIATOR kyber_ss: {:?}", &kyber_ss_bytes[..8]);
    eprintln!("INITIATOR sorted_statics: {:?} {:?}", &sorted_statics[0][..4], &sorted_statics[1][..4]);
    eprintln!("INITIATOR sorted_ephems: {:?} {:?}", &sorted_ephems[0][..4], &sorted_ephems[1][..4]);

    let mut ikm = Vec::new();
    ikm.extend_from_slice(&kyber_ss_bytes);
    ikm.extend_from_slice(&sorted_statics[0]);
    ikm.extend_from_slice(&sorted_statics[1]);
    ikm.extend_from_slice(&sorted_ephems[0]);
    ikm.extend_from_slice(&sorted_ephems[1]);

    let send_key_hash = blake3::derive_key("phantommesh-to-responder-v1", &ikm);
    let recv_key_hash = blake3::derive_key("phantommesh-to-initiator-v1", &ikm);

    let mut send_key = [0u8; 32];
    let mut recv_key = [0u8; 32];
    send_key.copy_from_slice(&send_key_hash);
    recv_key.copy_from_slice(&recv_key_hash);

    info!(peer = %hex::encode(&state.peer_static[..8]), "Handshake completed (initiator)");

    Ok(HandshakeResult {
        send_key,
        recv_key,
        peer_identity: state.peer_static,
        is_post_quantum: true,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_generation() {
        let id = NodeIdentity::generate().unwrap();
        assert_eq!(id.x25519_public.len(), 32);
        assert_eq!(id.x25519_private.len(), 32);
        assert!(!id.dilithium_public.is_empty());
        assert!(!id.kyber_public.is_empty());
    }

    #[test]
    fn test_full_handshake() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();

        // Step 1: Initiator builds INIT message
        let (init_msg, init_state) = build_init_message(
            &initiator,
            &responder.x25519_public,
        ).unwrap();

        assert_eq!(init_msg[0], MSG_INIT);

        // Step 2: Responder processes INIT, builds RESP
        let (resp_msg, resp_result) = process_init_and_respond(
            &responder,
            &init_msg,
        ).unwrap();

        assert_eq!(resp_msg[0], MSG_RESP);
        assert!(resp_result.is_post_quantum);
        assert_eq!(resp_result.peer_identity, initiator.x25519_public);

        // Step 3: Initiator processes RESP (pass identity for Kyber decapsulation)
        let init_result = process_response(
            &initiator,
            &init_state,
            &resp_msg,
        ).expect("Initiator should process response successfully");

        assert!(init_result.is_post_quantum);
        assert_eq!(init_result.peer_identity, responder.x25519_public);

        // CRITICAL: Both sides must derive the same transport keys
        assert_eq!(init_result.send_key, resp_result.recv_key);
        assert_eq!(init_result.recv_key, resp_result.send_key);

        // Keys must not be zero
        assert_ne!(init_result.send_key, [0u8; 32]);
        assert_ne!(init_result.recv_key, [0u8; 32]);

        // Send and recv keys must be different
        assert_ne!(init_result.send_key, init_result.recv_key);
    }

    #[test]
    fn test_handshake_rejects_tampered_init() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();

        let (mut init_msg, _) = build_init_message(
            &initiator,
            &responder.x25519_public,
        ).unwrap();

        // Tamper with the ephemeral key
        init_msg[5] ^= 0xFF;

        let result = process_init_and_respond(&responder, &init_msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_handshake_rejects_wrong_peer() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();
        let wrong_peer = NodeIdentity::generate().unwrap();

        // Initiator targets wrong peer
        let (init_msg, _) = build_init_message(
            &initiator,
            &wrong_peer.x25519_public,
        ).unwrap();

        // Responder (not the intended target) tries to process
        let result = process_init_and_respond(&responder, &init_msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_sessions_different_keys() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();

        let (init1, state1) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let (_, result1) = process_init_and_respond(&responder, &init1).unwrap();

        let (init2, state2) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let (_, result2) = process_init_and_respond(&responder, &init2).unwrap();

        // Different ephemeral keys should produce different transport keys
        assert_ne!(result1.send_key, result2.send_key);
    }
}

//! PhantomMesh Hybrid Handshake Protocol
//!
//! Combines x25519 (classical) + Kyber-768 (post-quantum) for key agreement,
//! with Dilithium-2 for identity authentication.
//!
//! Protocol flow:
//! ```text
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
//! ```
//!
//! Key derivation:
//! ```text
//!   ikm = x25519_shared_secret || kyber_shared_secret || initiator_pub || responder_pub
//!   send_key = BLAKE3_derive_key("phantommesh-send", ikm)
//!   recv_key = BLAKE3_derive_key("phantommesh-recv", ikm)
//! ```

use pqcrypto_kyber::kyber768;
use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::kem::{Ciphertext as KemCiphertext, SharedSecret, PublicKey as KemPublicKey, SecretKey as KemSecretKey};
use pqcrypto_traits::sign::{PublicKey as SignPublicKey, SecretKey as SignSecretKey, SignedMessage};
use tracing::{info, debug};

const HANDSHAKE_VERSION: u8 = 1;
const MSG_INIT: u8 = 0x01;
const MSG_RESP: u8 = 0x02;

/// Errors that can occur while parsing a peer-supplied handshake message.
///
/// Handshake messages arrive from the network before any authentication has
/// happened, so every length-prefixed field must be bounds-checked before
/// slicing — a malformed or truncated packet must produce `Err`, never panic
/// (a panic here would be a remote, unauthenticated denial-of-service once
/// this handshake code is wired into the live UDP receive path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeParseError {
    /// The buffer ended before a fixed-size field (e.g. a 32-byte key) could be read.
    TooShortForFixedField { needed: usize, offset: usize, available: usize },
    /// The buffer ended before a 4-byte little-endian length prefix could be read.
    TooShortForLengthPrefix { offset: usize, available: usize },
    /// A length prefix was read, but the buffer doesn't contain that many
    /// bytes afterward for the variable-length field it describes.
    TooShortForVariableField { field: &'static str, needed: usize, offset: usize, available: usize },
}

impl std::fmt::Display for HandshakeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandshakeParseError::TooShortForFixedField { needed, offset, available } => write!(
                f,
                "handshake message truncated: needed {} bytes at offset {}, only {} available",
                needed, offset, available
            ),
            HandshakeParseError::TooShortForLengthPrefix { offset, available } => write!(
                f,
                "handshake message truncated: needed 4-byte length prefix at offset {}, only {} available",
                offset, available
            ),
            HandshakeParseError::TooShortForVariableField { field, needed, offset, available } => write!(
                f,
                "handshake message truncated: field '{}' needs {} bytes at offset {}, only {} available",
                field, needed, offset, available
            ),
        }
    }
}

impl std::error::Error for HandshakeParseError {}

/// Read a fixed-size `N`-byte array out of `buf` at `offset`, bounds-checked.
///
/// Uses `checked_add` rather than a bare `offset + N` so that an absurd
/// (attacker-influenced) `offset` cannot wrap `usize` and bypass the bounds
/// check — on the 64-bit-only targets this crate builds for that can't
/// happen in practice (offsets here are always small sums of at-most-u32
/// field lengths), but the check is free and removes any doubt.
fn read_fixed<const N: usize>(
    buf: &[u8],
    offset: usize,
) -> Result<[u8; N], HandshakeParseError> {
    let end = offset.checked_add(N);
    if end.is_none() || buf.len() < end.unwrap() {
        return Err(HandshakeParseError::TooShortForFixedField {
            needed: N,
            offset,
            available: buf.len().saturating_sub(offset),
        });
    }
    let mut out = [0u8; N];
    out.copy_from_slice(&buf[offset..offset + N]);
    Ok(out)
}

/// Read a little-endian `u32` length prefix out of `buf` at `offset`, bounds-checked.
fn read_u32_len(buf: &[u8], offset: usize) -> Result<usize, HandshakeParseError> {
    let end = offset.checked_add(4);
    if end.is_none() || buf.len() < end.unwrap() {
        return Err(HandshakeParseError::TooShortForLengthPrefix {
            offset,
            available: buf.len().saturating_sub(offset),
        });
    }
    // Bounds already checked above, so this slice-to-array conversion cannot fail.
    let mut len_bytes = [0u8; 4];
    len_bytes.copy_from_slice(&buf[offset..offset + 4]);
    Ok(u32::from_le_bytes(len_bytes) as usize)
}

/// Read a variable-length field of `len` bytes out of `buf` at `offset`, bounds-checked.
fn read_var_field<'a>(
    buf: &'a [u8],
    offset: usize,
    len: usize,
    field: &'static str,
) -> Result<&'a [u8], HandshakeParseError> {
    let end = offset.checked_add(len);
    if end.is_none() || buf.len() < end.unwrap() {
        return Err(HandshakeParseError::TooShortForVariableField {
            field,
            needed: len,
            offset,
            available: buf.len().saturating_sub(offset),
        });
    }
    Ok(&buf[offset..offset + len])
}

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
    /// Test-only: the raw `ikm` (input key material) bytes computed on this
    /// side, before the final BLAKE3 derive_key step. Exists purely so tests
    /// can assert both sides of a handshake compute byte-identical `ikm` —
    /// this is a stronger check than comparing the final derived keys, since
    /// it catches an asymmetry in the input bytes directly rather than via
    /// a confusing downstream key mismatch. Not present in release builds.
    #[cfg(test)]
    pub ikm: Vec<u8>,
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

        // x25519 keypair — real Diffie-Hellman keypair, not a hash.
        // Public key = private_scalar * basepoint, via the low-level free
        // function (the higher-level EphemeralSecret/PublicKey wrapper types
        // are move-only and would force a larger restructure of the raw
        // [u8; 32]-scalar-based state used throughout this module).
        let mut x25519_private = [0u8; 32];
        ring::rand::SecureRandom::fill(&rng, &mut x25519_private)
            .map_err(|_| "RNG failed")?;
        let x25519_public = x25519_dalek::x25519(x25519_private, x25519_dalek::X25519_BASEPOINT_BYTES);

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

    /// Build a `NodeIdentity` around an already-loaded, persistent X25519
    /// keypair (e.g. the one `cli.rs`'s `cmd_up` decodes from the on-disk
    /// config, matching `NodeIdentity`'s format since the Stage 1 follow-up
    /// fix) instead of always generating a fresh throwaway identity.
    ///
    /// Only X25519 has a config-level, persistent source in this codebase
    /// today — `cli.rs` has no equivalent long-term storage for Dilithium or
    /// Kyber material, so those two keypairs are still freshly generated
    /// per-process here. This means the node's signing/KEM identity is NOT
    /// yet fully persistent across restarts, only its X25519 static key is.
    /// A peer that pins/remembers a Dilithium public key across restarts
    /// would see it change; nothing in this codebase does that today, but
    /// it's a real limitation worth being explicit about rather than
    /// silently implying full identity persistence.
    pub fn from_existing_keys(
        x25519_private: [u8; 32],
        x25519_public: [u8; 32],
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Sanity check: the supplied public key must actually be the
        // scalar-multiplication of the supplied private key against the
        // basepoint. A caller-supplied mismatched pair (e.g. a bug upstream
        // that decodes the wrong bytes into one of the two fields) would
        // otherwise silently produce a NodeIdentity whose DH operations
        // never agree with what the peer expects, failing far away from
        // the actual root cause. Cheap to check, so no reason not to.
        let derived_public = x25519_dalek::x25519(x25519_private, x25519_dalek::X25519_BASEPOINT_BYTES);
        if derived_public != x25519_public {
            return Err("from_existing_keys: supplied x25519_public does not match x25519_private".into());
        }

        let (dil_pk, dil_sk) = dilithium2::keypair();
        let dilithium_public = dil_pk.as_bytes().to_vec();
        let dilithium_secret = dil_sk.as_bytes().to_vec();

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
}

/// Build handshake initiation message
pub fn build_init_message(
    identity: &NodeIdentity,
    peer_static_pub: &[u8; 32],
) -> Result<(Vec<u8>, InitiatorState), Box<dyn std::error::Error + Send + Sync>> {
    let rng = ring::rand::SystemRandom::new();

    // Generate ephemeral x25519 keypair (real DH keypair, see NodeIdentity::generate)
    let mut ephem_private = [0u8; 32];
    ring::rand::SecureRandom::fill(&rng, &mut ephem_private)
        .map_err(|_| "RNG failed")?;
    let ephem_public = x25519_dalek::x25519(ephem_private, x25519_dalek::X25519_BASEPOINT_BYTES);

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
    let initiator_ephem: [u8; 32] = read_fixed(init_msg, offset)?;
    offset += 32;

    // Read Kyber public key
    let kyber_pub_len = read_u32_len(init_msg, offset)?;
    offset += 4;
    let kyber_pub_bytes = read_var_field(init_msg, offset, kyber_pub_len, "kyber_pub")?;
    offset += kyber_pub_len;

    // Read static public key
    let initiator_static: [u8; 32] = read_fixed(init_msg, offset)?;
    offset += 32;

    // Read Dilithium public key
    let dil_pub_len = read_u32_len(init_msg, offset)?;
    offset += 4;
    let dil_pub_bytes = read_var_field(init_msg, offset, dil_pub_len, "dilithium_pub")?;
    offset += dil_pub_len;

    // Read signature
    let sig_len = read_u32_len(init_msg, offset)?;
    offset += 4;
    let signature = read_var_field(init_msg, offset, sig_len, "signature")?;

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

    // Generate responder ephemeral x25519 (real DH keypair, see NodeIdentity::generate)
    let rng = ring::rand::SystemRandom::new();
    let mut resp_ephem_private = [0u8; 32];
    ring::rand::SecureRandom::fill(&rng, &mut resp_ephem_private)
        .map_err(|_| "RNG failed")?;
    let resp_ephem_public = x25519_dalek::x25519(resp_ephem_private, x25519_dalek::X25519_BASEPOINT_BYTES);

    // Real X25519 Diffie-Hellman: responder's ephemeral private scalar times
    // the initiator's ephemeral public point. By construction this equals
    // the initiator's x25519(their_ephem_private, our_ephem_public) — see
    // process_response below, where the identical computation is performed
    // with the two sides swapped. Do NOT sort/hash the two ephemeral public
    // keys as a substitute for this (that was tried and reverted in commit
    // a071444 — it produces a publicly-computable transcript hash, not a
    // Diffie-Hellman secret, since it never touches a private scalar).
    let x25519_shared = x25519_dalek::x25519(resp_ephem_private, initiator_ephem);

    // Kyber KEM: encapsulate with initiator's Kyber public key
    let kyber_pk = kyber768::PublicKey::from_bytes(kyber_pub_bytes)
        .map_err(|_| "Invalid Kyber public key")?;
    // pqcrypto::kyber768::encapsulate returns (SharedSecret, Ciphertext) — NOT (CT, SS)
    let (kyber_ret_a, _kyber_ret_b) = kyber768::encapsulate(&kyber_pk);
    let kyber_ss_bytes = if kyber_ret_a.as_bytes().len() == 32 {
        kyber_ret_a.as_bytes().to_vec()
    } else {
        // Swapped order in some pqcrypto versions
        _kyber_ret_b.as_bytes().to_vec()
    };

    // Derive transport keys from:
    //   x25519_shared || kyber_ss || sorted(static_keys) || sorted(ephem_keys)
    //
    // x25519_shared occupies a fixed byte position (first) that is IDENTICAL
    // on both sides — it is not sorted, because it is already symmetric by
    // construction (x25519(a, bG) == x25519(b, aG)), so both sides compute
    // the exact same 32 bytes here without needing any ordering convention.
    // The sorted_statics/sorted_ephems below are unchanged from before this
    // fix and remain PUBLIC-key transcript binding only, not secret material.
    let mut sorted_statics = [initiator_static.to_vec(), identity.x25519_public.to_vec()];
    sorted_statics.sort();
    let mut sorted_ephems = [initiator_ephem.to_vec(), resp_ephem_public.to_vec()];
    sorted_ephems.sort();

    let mut ikm = Vec::new();
    ikm.extend_from_slice(&x25519_shared[..]);
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
    let mut sorted_ephems = [initiator_ephem.to_vec(), resp_ephem_public.to_vec()];
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
        #[cfg(test)]
        ikm: ikm.clone(),
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
    let resp_ephem: [u8; 32] = read_fixed(resp_msg, offset)?;
    offset += 32;

    // Read encrypted Kyber shared secret (nonce + ciphertext)
    let enc_ss_len = read_u32_len(resp_msg, offset)?;
    offset += 4;
    let enc_ss_data = read_var_field(resp_msg, offset, enc_ss_len, "encrypted_kyber_ss")?;
    offset += enc_ss_len;

    // Read Dilithium public key
    let dil_pub_len = read_u32_len(resp_msg, offset)?;
    offset += 4;
    let dil_pub_bytes = read_var_field(resp_msg, offset, dil_pub_len, "dilithium_pub")?;
    offset += dil_pub_len;

    // Read signature
    let sig_len = read_u32_len(resp_msg, offset)?;
    offset += 4;
    let resp_signature = read_var_field(resp_msg, offset, sig_len, "signature")?;

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

    // Decrypt the Kyber shared secret using same ephemeral key derivation
    let mut sorted_ephems = [state.ephem_public.to_vec(), resp_ephem.to_vec()];
    sorted_ephems.sort();
    let transport_seed = blake3::hash(&sorted_ephems.concat());

    // Same class of bug as the 6 length-prefix unwraps above: enc_ss_data's
    // outer length was already bounds-checked by read_var_field, but slicing
    // [..12]/[12..] on it can still panic if a malicious/truncated peer sent
    // fewer than 12 bytes for the nonce. Check explicitly rather than slice blindly.
    if enc_ss_data.len() < 12 {
        return Err(HandshakeParseError::TooShortForVariableField {
            field: "encrypted_kyber_ss_nonce",
            needed: 12,
            offset: 0,
            available: enc_ss_data.len(),
        }
        .into());
    }
    let enc_nonce = &enc_ss_data[..12];
    let enc_ciphertext = &enc_ss_data[12..];
    let kyber_ss_bytes = {
        use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, CHACHA20_POLY1305};
        let key = UnboundKey::new(&CHACHA20_POLY1305, transport_seed.as_bytes()).map_err(|_| "Bad key")?;
        let key = LessSafeKey::new(key);
        let nonce = Nonce::try_assume_unique_for_key(enc_nonce).map_err(|_| "Bad nonce")?;
        let mut buf = enc_ciphertext.to_vec();
        let opened = key.open_in_place(nonce, Aad::empty(), &mut buf).map_err(|_| "Kyber SS decrypt failed")?;
        if opened.len() < 32 {
            return Err("Decrypted Kyber shared secret shorter than expected".into());
        }
        opened[..32].to_vec()
    };

    // Real X25519 Diffie-Hellman: initiator's ephemeral private scalar times
    // the responder's ephemeral public point. This is the mirror image of
    // the computation in process_init_and_respond (x25519(resp_ephem_private,
    // initiator_ephem)) — by construction x25519(a, bG) == x25519(b, aG), so
    // both sides land on the identical 32 bytes without needing to sort or
    // otherwise agree on an ordering convention for this term.
    let x25519_shared = x25519_dalek::x25519(state.ephem_private, resp_ephem);

    let mut sorted_statics = [identity.x25519_public.to_vec(), state.peer_static.to_vec()];
    sorted_statics.sort();

    // IMPORTANT: byte layout here must be IDENTICAL to process_init_and_respond's
    // ikm construction: x25519_shared || kyber_ss || sorted_statics || sorted_ephems.
    let mut ikm = Vec::new();
    ikm.extend_from_slice(&x25519_shared[..]);
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
        #[cfg(test)]
        ikm: ikm.clone(),
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
    fn test_from_existing_keys_matches_supplied_x25519_pair() {
        // Generate a real keypair the "normal" way, then rebuild a
        // NodeIdentity around just its X25519 half via from_existing_keys —
        // this is exactly what cli.rs's cmd_up does with a config-loaded key.
        let original = NodeIdentity::generate().unwrap();
        let rebuilt = NodeIdentity::from_existing_keys(
            original.x25519_private,
            original.x25519_public,
        ).unwrap();

        assert_eq!(rebuilt.x25519_private, original.x25519_private);
        assert_eq!(rebuilt.x25519_public, original.x25519_public);
        // Dilithium/Kyber are freshly generated by from_existing_keys (no
        // config-level source for them exists), so they must NOT equal the
        // original's — confirms this isn't accidentally reusing/leaking
        // unrelated key material across two independent identities.
        assert_ne!(rebuilt.dilithium_public, original.dilithium_public);
        assert_ne!(rebuilt.kyber_public, original.kyber_public);
        assert!(!rebuilt.dilithium_public.is_empty());
        assert!(!rebuilt.kyber_public.is_empty());
    }

    #[test]
    fn test_from_existing_keys_rejects_mismatched_pair() {
        let a = NodeIdentity::generate().unwrap();
        let b = NodeIdentity::generate().unwrap();

        // a's private key paired with b's public key is not a valid X25519
        // keypair — must be rejected rather than silently accepted.
        let result = NodeIdentity::from_existing_keys(a.x25519_private, b.x25519_public);
        assert!(result.is_err());
    }

    /// A full handshake round-trip using from_existing_keys-built identities
    /// on both sides, proving the reconstructed identity is fully usable for
    /// real handshakes (not just structurally well-formed).
    #[test]
    fn test_from_existing_keys_full_handshake() {
        let initiator_src = NodeIdentity::generate().unwrap();
        let responder_src = NodeIdentity::generate().unwrap();

        let initiator = NodeIdentity::from_existing_keys(
            initiator_src.x25519_private,
            initiator_src.x25519_public,
        ).unwrap();
        let responder = NodeIdentity::from_existing_keys(
            responder_src.x25519_private,
            responder_src.x25519_public,
        ).unwrap();

        let (init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let (resp_msg, resp_result) = process_init_and_respond(&responder, &init_msg).unwrap();
        let init_result = process_response(&initiator, &init_state, &resp_msg).unwrap();

        assert_eq!(init_result.send_key, resp_result.recv_key);
        assert_eq!(init_result.recv_key, resp_result.send_key);
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

    // ========================================================================
    // Stage 1 security-repair tests: real X25519 DH, ikm symmetry, bounds checks
    // ========================================================================

    /// The single most important regression test for this stage. Runs a real
    /// handshake exchange through the actual TunnelEngine handshake functions
    /// on both sides, then asserts the raw `ikm` bytes computed independently
    /// by the initiator and the responder are byte-for-byte identical — not
    /// just that the final derived keys match. A prior attempt at real X25519
    /// (reverted in commit a071444) broke exactly here: the two sides computed
    /// different `ikm` inputs. Checking `ikm` directly catches that class of
    /// bug immediately, instead of via a confusing downstream key mismatch.
    #[test]
    fn test_ikm_byte_identical_both_sides() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();

        let (init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let (resp_msg, resp_result) = process_init_and_respond(&responder, &init_msg).unwrap();
        let init_result = process_response(&initiator, &init_state, &resp_msg).unwrap();

        assert_eq!(
            init_result.ikm, resp_result.ikm,
            "initiator and responder computed different ikm bytes — asymmetric key derivation input"
        );
        // ikm must not be empty/trivial
        assert!(!init_result.ikm.is_empty());
        // Sanity: the first 32 bytes (x25519_shared) must not be all-zero —
        // an all-zero shared secret would indicate a low-order point or a
        // broken DH computation (real X25519 outputs are never zero for
        // properly generated random scalars against the standard basepoint).
        assert_ne!(&init_result.ikm[..32], &[0u8; 32][..]);

        // Final derived keys must still agree (existing invariant, re-asserted here)
        assert_eq!(init_result.send_key, resp_result.recv_key);
        assert_eq!(init_result.recv_key, resp_result.send_key);
    }

    /// RFC 7748 known-answer test, computed independently of any round-trip
    /// handshake logic in this file. Round-trip symmetry alone (both sides
    /// agree) does not prove correctness against the spec — a consistently-
    /// wrong implementation could still be symmetric. This test uses the
    /// Diffie-Hellman example from RFC 7748 §6.1 (Alice/Bob), fetched
    /// verbatim from https://www.rfc-editor.org/rfc/rfc7748.txt and
    /// cross-checked independently against Python's `cryptography` library
    /// and OpenSSL's pkey/X25519 handling — both reproduce the same public
    /// keys and shared secret from these exact private scalars. (Note:
    /// RFC 7748 §5.2 contains a *different* pair of pure scalar-multiplication
    /// test vectors, not the named Alice/Bob example used here.)
    ///
    ///   Alice's private key, a:
    ///     77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a
    ///   Alice's public key, X25519(a, 9):
    ///     8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a
    ///   Bob's private key, b:
    ///     5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb
    ///   Bob's public key, X25519(b, 9):
    ///     de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f
    ///   Their shared secret, K:
    ///     4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742
    #[test]
    fn test_rfc7748_known_answer_vectors() {
        let alice_private: [u8; 32] =
            hex_literal("77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a");
        let alice_public_expected: [u8; 32] =
            hex_literal("8520f0098930a754748b7ddcb43ef75a0dbf3a0d26381af4eba4a98eaa9b4e6a");
        let bob_private: [u8; 32] =
            hex_literal("5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb");
        let bob_public_expected: [u8; 32] =
            hex_literal("de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f");
        let shared_expected: [u8; 32] =
            hex_literal("4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742");

        let alice_public = x25519_dalek::x25519(alice_private, x25519_dalek::X25519_BASEPOINT_BYTES);
        assert_eq!(alice_public, alice_public_expected, "Alice's public key does not match RFC 7748 vector");

        let bob_public = x25519_dalek::x25519(bob_private, x25519_dalek::X25519_BASEPOINT_BYTES);
        assert_eq!(bob_public, bob_public_expected, "Bob's public key does not match RFC 7748 vector");

        let alice_shared = x25519_dalek::x25519(alice_private, bob_public);
        assert_eq!(alice_shared, shared_expected, "Alice's computed shared secret does not match RFC 7748 vector");

        let bob_shared = x25519_dalek::x25519(bob_private, alice_public);
        assert_eq!(bob_shared, shared_expected, "Bob's computed shared secret does not match RFC 7748 vector");
    }

    /// Decode a fixed-length hex string literal into a [u8; 32] at test time.
    /// (Not using the `hex` crate's Vec-returning decode here to keep this
    /// self-contained and panic loudly on a malformed literal, since a typo
    /// in a hardcoded RFC test vector would otherwise silently produce a
    /// wrong-length array.)
    fn hex_literal(s: &str) -> [u8; 32] {
        let bytes = hex::decode(s).expect("invalid hex literal in test vector");
        let mut out = [0u8; 32];
        assert_eq!(bytes.len(), 32, "test vector must decode to exactly 32 bytes");
        out.copy_from_slice(&bytes);
        out
    }

    // ------------------------------------------------------------------
    // Malformed/truncated-input tests for the 6 bounds-check fixes (plus
    // the 2 additional slice-bounds fixes found in the same review pass).
    // Every one of these must return Err, never panic.
    // ------------------------------------------------------------------

    #[test]
    fn test_process_init_rejects_empty_buffer() {
        let responder = NodeIdentity::generate().unwrap();
        let result = process_init_and_respond(&responder, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_init_rejects_truncated_after_msg_type() {
        let responder = NodeIdentity::generate().unwrap();
        // Only MSG_INIT + version, nothing else
        let result = process_init_and_respond(&responder, &[MSG_INIT, HANDSHAKE_VERSION]);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_init_rejects_truncated_ephemeral_key() {
        let responder = NodeIdentity::generate().unwrap();
        // MSG_INIT + version + only 10 bytes of what should be a 32-byte ephemeral key
        let mut buf = vec![MSG_INIT, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 10]);
        let result = process_init_and_respond(&responder, &buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_init_rejects_truncated_kyber_len_prefix() {
        let responder = NodeIdentity::generate().unwrap();
        // version + 32-byte ephemeral key, then only 2 bytes of the 4-byte kyber_pub_len prefix
        let mut buf = vec![MSG_INIT, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 32]);
        buf.extend_from_slice(&[0u8; 2]);
        let result = process_init_and_respond(&responder, &buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_init_rejects_kyber_len_exceeding_buffer() {
        let responder = NodeIdentity::generate().unwrap();
        // A huge kyber_pub_len claimed, but no actual data backing it
        let mut buf = vec![MSG_INIT, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 32]); // ephemeral key
        buf.extend_from_slice(&(u32::MAX / 2).to_le_bytes()); // absurd claimed length
        buf.extend_from_slice(&[0u8; 4]); // a little trailing data, nowhere near enough
        let result = process_init_and_respond(&responder, &buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_init_rejects_truncated_dilithium_len_prefix() {
        let responder = NodeIdentity::generate().unwrap();
        let mut buf = vec![MSG_INIT, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 32]); // ephemeral key
        buf.extend_from_slice(&0u32.to_le_bytes()); // kyber_pub_len = 0 (valid, empty)
        buf.extend_from_slice(&[0u8; 32]); // static key
        buf.extend_from_slice(&[0u8; 1]); // only 1 of 4 bytes of dilithium_pub_len
        let result = process_init_and_respond(&responder, &buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_init_rejects_truncated_signature_len_prefix() {
        let responder = NodeIdentity::generate().unwrap();
        let mut buf = vec![MSG_INIT, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 32]); // ephemeral key
        buf.extend_from_slice(&0u32.to_le_bytes()); // kyber_pub_len = 0
        buf.extend_from_slice(&[0u8; 32]); // static key
        buf.extend_from_slice(&0u32.to_le_bytes()); // dilithium_pub_len = 0
        buf.extend_from_slice(&[0u8; 3]); // only 3 of 4 bytes of sig_len
        let result = process_init_and_respond(&responder, &buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_response_rejects_empty_buffer() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();
        let (_init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let result = process_response(&initiator, &init_state, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_response_rejects_truncated_ephemeral_key() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();
        let (_init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let mut buf = vec![MSG_RESP, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 5]); // way short of the 32-byte ephemeral key
        let result = process_response(&initiator, &init_state, &buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_response_rejects_truncated_enc_ss_len_prefix() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();
        let (_init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let mut buf = vec![MSG_RESP, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 32]); // ephemeral key
        buf.extend_from_slice(&[0u8; 2]); // only 2 of 4 bytes of enc_ss_len
        let result = process_response(&initiator, &init_state, &buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_response_rejects_enc_ss_len_exceeding_buffer() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();
        let (_init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let mut buf = vec![MSG_RESP, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 32]); // ephemeral key
        buf.extend_from_slice(&(u32::MAX / 2).to_le_bytes()); // absurd claimed length
        let result = process_response(&initiator, &init_state, &buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_response_rejects_enc_ss_shorter_than_nonce() {
        // enc_ss_len bounds-check passes (claims fewer than 12 bytes and
        // actually has that many), but 12 bytes are needed just for the
        // nonce before any ciphertext — must still error, not panic.
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();
        let (_init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let mut buf = vec![MSG_RESP, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 32]); // ephemeral key
        buf.extend_from_slice(&5u32.to_le_bytes()); // enc_ss_len = 5 (< 12)
        buf.extend_from_slice(&[0u8; 5]); // exactly 5 bytes backing it
        let result = process_response(&initiator, &init_state, &buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_response_rejects_truncated_dilithium_len_prefix() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();
        let (_init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let mut buf = vec![MSG_RESP, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 32]); // ephemeral key
        buf.extend_from_slice(&0u32.to_le_bytes()); // enc_ss_len = 0
        buf.extend_from_slice(&[0u8; 2]); // only 2 of 4 bytes of dil_pub_len
        let result = process_response(&initiator, &init_state, &buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_response_rejects_truncated_signature_len_prefix() {
        let initiator = NodeIdentity::generate().unwrap();
        let responder = NodeIdentity::generate().unwrap();
        let (_init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
        let mut buf = vec![MSG_RESP, HANDSHAKE_VERSION];
        buf.extend_from_slice(&[0u8; 32]); // ephemeral key
        buf.extend_from_slice(&0u32.to_le_bytes()); // enc_ss_len = 0
        buf.extend_from_slice(&0u32.to_le_bytes()); // dil_pub_len = 0
        buf.extend_from_slice(&[0u8; 1]); // only 1 of 4 bytes of sig_len
        let result = process_response(&initiator, &init_state, &buf);
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------
    // Property-based fuzzing: feed random-length truncated/garbage buffers
    // at both entry points and assert no panic ever occurs (catch_unwind
    // makes a panic show up as a hard test failure with a clear message
    // instead of aborting the whole test binary).
    // ------------------------------------------------------------------
    mod proptest_fuzz {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(512))]

            #[test]
            fn process_init_and_respond_never_panics(buf in prop::collection::vec(any::<u8>(), 0..256)) {
                let responder = NodeIdentity::generate().unwrap();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    process_init_and_respond(&responder, &buf)
                }));
                prop_assert!(outcome.is_ok(), "process_init_and_respond panicked on input: {:?}", buf);
            }

            #[test]
            fn process_response_never_panics(buf in prop::collection::vec(any::<u8>(), 0..256)) {
                let initiator = NodeIdentity::generate().unwrap();
                let responder = NodeIdentity::generate().unwrap();
                let (_init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    process_response(&initiator, &init_state, &buf)
                }));
                prop_assert!(outcome.is_ok(), "process_response panicked on input: {:?}", buf);
            }

            /// Specifically fuzz truncations of a REAL, well-formed init message
            /// (rather than pure random bytes) — this exercises every length-prefix
            /// boundary with realistic-looking data up to that point, which random
            /// bytes alone are unlikely to reach (a random buffer almost never has
            /// a plausible-looking kyber_pub_len that's just slightly too large).
            #[test]
            fn process_init_and_respond_never_panics_on_truncated_real_message(
                truncate_to in 0usize..600
            ) {
                let initiator = NodeIdentity::generate().unwrap();
                let responder = NodeIdentity::generate().unwrap();
                let (init_msg, _state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
                let cut = truncate_to.min(init_msg.len());
                let truncated = &init_msg[..cut];
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    process_init_and_respond(&responder, truncated)
                }));
                prop_assert!(outcome.is_ok(), "process_init_and_respond panicked on truncated real message of len {}", cut);
            }

            #[test]
            fn process_response_never_panics_on_truncated_real_message(
                truncate_to in 0usize..600
            ) {
                let initiator = NodeIdentity::generate().unwrap();
                let responder = NodeIdentity::generate().unwrap();
                let (init_msg, init_state) = build_init_message(&initiator, &responder.x25519_public).unwrap();
                let (resp_msg, _resp_result) = process_init_and_respond(&responder, &init_msg).unwrap();
                let cut = truncate_to.min(resp_msg.len());
                let truncated = &resp_msg[..cut];
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    process_response(&initiator, &init_state, truncated)
                }));
                prop_assert!(outcome.is_ok(), "process_response panicked on truncated real message of len {}", cut);
            }
        }
    }
}

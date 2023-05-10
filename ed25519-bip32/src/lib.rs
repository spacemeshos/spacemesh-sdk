extern crate ed25519_dalek_bip32;
extern crate wasm_bindgen;

use std::ffi::{c_char, CStr};
use ed25519_dalek_bip32::{ed25519_dalek::{Keypair, KEYPAIR_LENGTH, SECRET_KEY_LENGTH}, DerivationPath, ExtendedSecretKey};
use spacemesh_sdkutils::{check_err, err};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// derive_key generates a keypair from a 64-byte BIP39-compatible seed and BIP32 hierarchical
/// derivation path. it returns 64 bytes. the first 32 bytes are the secret key and the second 32
/// bytes are the public key.
pub fn derive_key(
    seed: &[u8],
    path: &str,
) -> Box<[u8]> {
    let derivation_path: DerivationPath = path.parse().unwrap();
    let extended = ExtendedSecretKey::from_seed(seed)
        .and_then(|extended| extended.derive(&derivation_path))
        .unwrap();
    let extended_public_key = extended.public_key();
    let keypair = Keypair{secret: extended.secret_key, public: extended_public_key};
    Box::new(keypair.to_bytes())
}

/// derive_c generates a keypair from a 64-byte BIP39-compatible seed and BIP32 hierarchical
/// derivation path. It writes the keypair bytes to result, which must be at least 64 bytes long.
/// It returns a status code, with a return value of zero indicating success.
/// This function does the same thing as derive_key, which is bound for wasm rather than CFFI.
/// it adds error handling in order to be friendlier to the FFI caller: in case of an error, it
/// prints the error and returns a nonzero value.
#[no_mangle]
pub extern "C" fn derive_c(
    seed_ptr: *const u8,
    seedlen: usize,
    derivation_path_ptr: *const c_char,
    result: *mut u8,
) -> u16 {
    // Seed must be at least 32 bytes
    if seedlen < SECRET_KEY_LENGTH {
        err!("seed must be at least 32 bytes");
    }
    let seed_slice = unsafe { std::slice::from_raw_parts(seed_ptr, seedlen) };
    let derivation_path_str = unsafe { CStr::from_ptr(derivation_path_ptr) };
    let derivation_path_str = derivation_path_str.to_str();
    check_err!(derivation_path_str, "failed to convert path string from raw parts");
    let derivation_path_str = derivation_path_str.unwrap().parse();
    check_err!(derivation_path_str, "failed to parse derivation path");
    let derivation_path: DerivationPath = derivation_path_str.unwrap();

    // for now we are rather strict with which types of paths we accept, to avoid errors and to
    // be as compatible as possible with BIP-44. the path must be of the format
    // "m/44'/540'/...", i.e., it must have purpose 44 and coin type
    // 540 and all path elements must be hardened. we expect it to contain between 2 and 5
    // elements.
    if derivation_path.path().len() < 2 {
        err!("path too short");
    }
    if derivation_path.path().len() > 5 {
        err!("path too long");
    }
    if derivation_path.path()[0].to_u32() != 44 {
        err!("bad path purpose");
    }
    if derivation_path.path()[1].to_u32() != 540 {
        err!("bad path coin type");
    }
    for p in derivation_path.path() {
        if !p.is_hardened() {
            err!("path isn't fully hardened");
        }
    }

    let extended = ExtendedSecretKey::from_seed(seed_slice)
        .and_then(|extended| extended.derive(&derivation_path));
    check_err!(extended, "failed to derive secret key from seed");
    let extended_inner = extended.unwrap();
    let extended_public_key = extended_inner.public_key();
    let keypair = Keypair{secret: extended_inner.secret_key, public: extended_public_key};
    let result_slice = unsafe { std::slice::from_raw_parts_mut(result, KEYPAIR_LENGTH) };
    result_slice.copy_from_slice(&keypair.to_bytes());
    0
}

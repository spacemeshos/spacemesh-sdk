extern crate ed25519_dalek_bip32;
extern crate wasm_bindgen;

use std::ffi::{c_char, CStr};
use ed25519_dalek_bip32::{ed25519_dalek::{Keypair, KEYPAIR_LENGTH, SECRET_KEY_LENGTH}, DerivationPath, ExtendedSecretKey};
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
    seed: *const u8,
    seedlen: usize,
    derivation_path_ptr: *const c_char,
    result: *mut u8,
) -> u16 {
    match _derive_c(seed, seedlen, derivation_path_ptr) {
        Ok(keypair) => {
            let result_slice = unsafe { std::slice::from_raw_parts_mut(result, KEYPAIR_LENGTH) };
            result_slice.copy_from_slice(&keypair.to_bytes());
            0
        }
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    }
}

fn _derive_c(
    seed: *const u8,
    seedlen: usize,
    derivation_path_ptr: *const c_char,
) -> Result<Keypair, Box<dyn std::error::Error>> {
    // Seed must be at least 32 bytes
    if seedlen < SECRET_KEY_LENGTH {
        return Err("seed must be at least 32 bytes".into());
    }
    let seed_slice = unsafe { std::slice::from_raw_parts(seed, seedlen) };
    let derivation_path_str = unsafe { CStr::from_ptr(derivation_path_ptr) };
    let derivation_path: DerivationPath = derivation_path_str
        .to_str()
        .map_err(|e| format!("failed to convert path string to str: {}", e))?
        .parse()
        .map_err(|e| format!("failed to parse derivation path: {}", e))?;

    // for now we are rather strict with which types of paths we accept, to avoid errors and to
    // be as compatible as possible with BIP-44. the path must be of the format
    // "m/44'/540'/...", i.e., it must have purpose 44 and coin type
    // 540 and all path elements must be hardened. we expect it to contain between 2 and 5
    // elements.
    if derivation_path.path().len() < 2 {
        return Err("path too short".into());
    }
    if derivation_path.path().len() > 5 {
        return Err("path too long".into());
    }
    if derivation_path.path()[0].to_u32() != 44 {
        return Err("bad path purpose".into());
    }
    if derivation_path.path()[1].to_u32() != 540 {
        return Err("bad path coin type".into());
    }
    for p in derivation_path.path() {
        if !p.is_hardened() {
            return Err("path isn't fully hardened".into());
        }
    }

    let extended = ExtendedSecretKey::from_seed(seed_slice)
        .map_err(|e| format!("failed to derive secret key from seed: {}", e))?
        .derive(&derivation_path)
        .map_err(|e| format!("failed to derive derivation path: {}", e))?;
    let extended_public_key = extended.public_key();
    Ok(Keypair{secret: extended.secret_key, public: extended_public_key})
}

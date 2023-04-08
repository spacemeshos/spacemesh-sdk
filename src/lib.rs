extern crate ed25519_dalek_bip32;
use ed25519_dalek_bip32::{ed25519_dalek::{Keypair}, DerivationPath, ExtendedSecretKey};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
/// derive_key generates keypair from seed and bip32 hierarchical derivation path.
/// it returns 64 bytes. first 32 bytes are secret key, and the second 32 bytes are public key.
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

/// derive_key_c does the same thing as the above function, but is intended for use over the CFFI.
/// note that the caller must free() the returned memory as it's not managed/freed here.
#[no_mangle]
pub extern "C" fn derive_key_c(
    seed: *const u8,
    seedlen: usize,
    path: *const u8,
    pathlen: usize,
) -> *mut u8 {
    unsafe {
        let path_str = std::str::from_utf8(std::slice::from_raw_parts(path, pathlen))
            .expect("Failed to convert string from raw parts");
        let seed_slice = std::slice::from_raw_parts(seed, seedlen);
        let boxed_keypair = derive_key(seed_slice, path_str);
        Box::into_raw(boxed_keypair) as *mut u8
    }
}

extern crate ed25519_dalek_bip32;
extern crate wasm_bindgen;
use ed25519_dalek_bip32::{ed25519_dalek::{Keypair}, DerivationPath, ExtendedSecretKey, ChildIndex};
use ed25519_dalek_bip32::derivation_path::DerivationPathType::BIP44;
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

// check for error. if no error, do nothing. if there is an error, print it and return a null ptr.
macro_rules! check_err {
    ($ptr:expr, $str:expr) => {
        match ($ptr) {
            Ok(ref _v) => (),
            Err(e) => {
                // TODO: return error message rather than printing it
                eprint!($str);
                eprintln!(": {e}");
                return std::ptr::null_mut();
            },
        }
    };
}

macro_rules! err {
    ($str:expr) => {
        eprint!($str);
        return std::ptr::null_mut();
    };
}

/// derive_c does the same thing as the above function, but is intended for use over the CFFI.
/// it adds error handling in order to be friendlier to the FFI caller: in case of an error, it
/// prints the error and returns a null pointer.
/// note that the caller must free() the returned memory as it's not managed/freed here.
#[no_mangle]
pub extern "C" fn derive_c(
    seed: *const u8,
    seedlen: usize,
    path: *const u8,
    pathlen: usize,
) -> *mut u8 {
    unsafe {
        let seed_slice = std::slice::from_raw_parts(seed, seedlen);
        let path_str = std::str::from_utf8(std::slice::from_raw_parts(path, pathlen));
        check_err!(path_str, "failed to convert string from raw parts");
        let derivation_path = path_str.unwrap().parse();
        check_err!(derivation_path, "failed to parse derivation path");
        let derivation_path_inner: DerivationPath = derivation_path.unwrap();

        // make sure path is of the correct type and is hardened
        if derivation_path_inner.path_type() != BIP44 {
            eprintln!("path is not of type BIP44");
            return std::ptr::null_mut();
        }
        for p in &derivation_path_inner {
            if p.is_hardened() {
                err!("path isn't fully hardened");
            }
        }

        let extended = ExtendedSecretKey::from_seed(seed_slice)
            .and_then(|extended| extended.derive(&derivation_path_inner));
        check_err!(extended, "failed to derive secret key from seed");
        let extended_inner = extended.unwrap();
        let extended_public_key = extended_inner.public_key();
        let keypair = Keypair{secret: extended_inner.secret_key, public: extended_public_key};
        let boxed_keypair = Box::new(keypair.to_bytes());
        Box::into_raw(boxed_keypair) as *mut u8
    }
}

/// derive_child_c derives a new child key from a seed and a single hardened path element.
/// the childidx always refers to a hardened path element, as we do not support non-hardened paths.
/// note that the caller must free() the returned memory as it's not managed/freed here.
#[no_mangle]
pub extern "C" fn derive_child_c(
    seed: *const u8,
    seedlen: usize,
    childidx: u8,
) -> *mut u8 {
    unsafe {
        let seed_slice = std::slice::from_raw_parts(seed, seedlen);
        let child_index = ChildIndex::hardened(childidx as u32);
        check_err!(child_index, "bad child index");
        let child_index_inner = child_index.unwrap();
        let extended = ExtendedSecretKey::from_seed(seed_slice)
            .and_then(|extended| extended.derive_child(child_index_inner));
        check_err!(extended, "failed to derive child key from seed and child index");
        let extended_inner = extended.unwrap();
        let extended_public_key = extended_inner.public_key();
        let keypair = Keypair{secret: extended_inner.secret_key, public: extended_public_key};
        let boxed_keypair = Box::new(keypair.to_bytes());
        Box::into_raw(boxed_keypair) as *mut u8
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//
//     #[test]
//     fn child_index_is_normal() {
//         assert!(ChildIndex::Hardened(0).is_hardened());
//         assert!(!ChildIndex::Normal(0).is_hardened());
//     }
// }

#![allow(clippy::integer_arithmetic)]
#![allow(dead_code)]

pub mod ledger;
pub mod ledger_error;
pub mod locator;
pub mod remote_keypair;
pub mod remote_wallet;

use std::ffi::{c_char, CStr};
use std::ops::Deref;
use {
    spacemesh_derivation_path::DerivationPath,
    solana_sdk::pubkey::{Pubkey, PUBKEY_BYTES},
};

/// read_pubkey_from_ledger reads a pubkey from the ledger device specified by path and
/// derivation_path. If path is empty, the first ledger device found will be used. If confirm_key
/// is true, it will prompt the user to confirm the key on the device. It writes the pubkey bytes
/// to result, which must be at least 32 bytes long. It returns a status code, with a return value
/// of zero indicating success.
#[no_mangle]
pub extern "C" fn read_pubkey_from_ledger(
    path_ptr: *const c_char,
    derivation_path_ptr: *const c_char,
    confirm_key: bool,
    result: *mut u8,
) -> u16 {
    match _read_pubkey_from_ledger(path_ptr, derivation_path_ptr, confirm_key) {
        Ok(pubkey) => {
            let result_slice = unsafe { std::slice::from_raw_parts_mut(result, PUBKEY_BYTES) };
            result_slice.copy_from_slice(pubkey.as_ref());
            0
        }
        Err(err) => {
            eprintln!("{}", err);
            1
        }
    }
}

fn _read_pubkey_from_ledger(
    path_ptr: *const c_char,
    derivation_path_ptr: *const c_char,
    confirm_key: bool,
) -> Result<Pubkey, Box<dyn std::error::Error>> {
    // first handle the device path
    // note: it might seem to make more sense to do all the unsafe operations in the parent
    // function and leave only the "business logic" here, but doing it here allows us to handle
    // all errors at the higher level and pass them back across FFI.
    let path_str = unsafe { CStr::from_ptr(path_ptr) };
    let path_str = path_str
        .to_str()
        .map_err(|e| format!("converting path string: {e}"))?;

    // if no path specified, default to first ledger device
    let path_str = if path_str.is_empty() {
        "usb://ledger"
    } else {
        path_str
    };

    let locator = locator::Locator::new_from_path(path_str)
        .map_err(|e| format!("creating locator from path string: {e}"))?;

    // next handle the derivation path
    // note: ed25519-bip32 performs more validation on the derivation path. we don't do so here
    // because these checks belong in the derivation-path crate. see
    // https://github.com/spacemeshos/spacemesh-sdk/issues/3.
    let derivation_path_str = unsafe { CStr::from_ptr(derivation_path_ptr) };
    let derivation_path_str = derivation_path_str
        .to_str()
        .map_err(|e| format!("converting derivation path string: {e}"))?;

    let derivation_path = DerivationPath::from_absolute_path_str(derivation_path_str)
        .map_err(|e| format!("creating derivation path from string: {e}"))?;

    let wm = remote_wallet::maybe_wallet_manager()
        .map_err(|e| format!("getting wallet manager: {e}"))?
        .ok_or("failed to get wallet manager")?;

    let keypair = remote_keypair::generate_remote_keypair(
        locator,
        derivation_path,
        wm.deref(),
        confirm_key,
        "main",
    )
    .map_err(|e| format!("generating remote keypair: {e}"))?;

    Ok(keypair.pubkey)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "test-hw-ledger")]
    use super::*;

    #[test]
    // don't run by default since it requires a physically connected ledger device
    #[cfg(feature = "test-hw-ledger")]
    fn it_works() -> Result<(), remote_wallet::RemoteWalletError> {
        let locator = locator::Locator::new_from_path("usb://ledger").unwrap();
        let s = "m/44'/540'/0'/0'/0'";
        let path = DerivationPath::from_absolute_path_str(s)?;
        let wm = &remote_wallet::maybe_wallet_manager().unwrap();
        if let Some(wm) = wm {
            return match remote_keypair::generate_remote_keypair(locator, path, wm, false, "main") {
                Ok(kp) => {
                    println!("uri: {}, path: {:?}, pubkey: {}", kp.path, kp.derivation_path, kp.pubkey);
                    Ok(())
                },
                Err(e) => Err(e),
            };
        }
        Err(remote_wallet::RemoteWalletError::NoDeviceFound)
    }
}

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
    spacemesh_sdkutils::{check_err, check_none},
    solana_sdk::pubkey::PUBKEY_BYTES,
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
    // first handle the device path
    let path_str = unsafe { CStr::from_ptr(path_ptr) };
    let path_str = path_str.to_str();
    check_err!(path_str, "failed to convert path string from raw parts");
    let mut path_str = path_str.unwrap();

    // if no path specified, default to first ledger device
    if path_str.len() == 0 {
        path_str = "usb://ledger";
    }
    let locator = locator::Locator::new_from_path(path_str);
    check_err!(locator, "failed to create locator from path string");
    let locator = locator.unwrap();

    // next handle the derivation path
    // note: ed25519-bip32 performs more validation on the derivation path. we don't do so here
    // because these checks belong in the derivation-path crate. see
    // https://github.com/spacemeshos/spacemesh-sdk/issues/3.
    let derivation_path_str = unsafe { CStr::from_ptr(derivation_path_ptr) };
    let derivation_path_str = derivation_path_str.to_str();
    check_err!(derivation_path_str, "failed to convert derivation path string from raw parts");
    let derivation_path_str = derivation_path_str.unwrap();
    let derivation_path = DerivationPath::from_absolute_path_str(derivation_path_str);
    check_err!(derivation_path, "failed to create derivation path from string");
    let derivation_path = derivation_path.unwrap();

    let wm = remote_wallet::maybe_wallet_manager();
    // unwrap the Result then the Option
    check_err!(wm, "failed to get wallet manager");
    let wm = wm.unwrap();
    check_none!(wm, "failed to get wallet manager");
    let wm = wm.unwrap();
    let keypair = remote_keypair::generate_remote_keypair(locator, derivation_path, wm.deref(), confirm_key, "main");
    check_err!(keypair, "failed to generate remote keypair");
    let kp = keypair.unwrap();
    println!("uri: {}, path: {:?}, pubkey: {}", kp.path, kp.derivation_path, kp.pubkey);
    let result_slice = unsafe { std::slice::from_raw_parts_mut(result, PUBKEY_BYTES) };
    result_slice.copy_from_slice(&kp.pubkey.to_bytes());
    0
}

#[cfg(test)]
mod tests {

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

use {
    spacemesh_remote_wallet::remote_keypair::generate_remote_keypair,
    spacemesh_remote_wallet::remote_wallet::{RemoteWalletError, maybe_wallet_manager},
    spacemesh_remote_wallet::locator::Locator,
    spacemesh_derivation_path::DerivationPath,
};

#[no_mangle]
pub extern "C" fn read_pubkey_from_ledger(
    path: *const u8,
    pathlen: usize,
    derivation_path: *const u8,
    derivation_pathlen: usize,
) -> *mut u8 {
    unsafe {
        // first handle the device path
        let path = std::slice::from_raw_parts(path, pathlen);
        let mut path = std::str::from_utf8(path).unwrap();

        // if no path specified, default to first ledger device
        if path.len() == 0 {
            path = "usb://ledger";
        }
        let locator = Locator::new_from_path(path).unwrap();

        // next handle the derivation path
        let derivation_path = std::slice::from_raw_parts(derivation_path, derivation_pathlen);
        let derivation_path = std::str::from_utf8(derivation_path).unwrap();
        let derivation_path = DerivationPath::from_absolute_path_str(derivation_path).unwrap();

        let wm = &maybe_wallet_manager().unwrap();
        if let Some(wm) = wm {
            return match generate_remote_keypair(locator, derivation_path, wm, false, "main") {
                Ok(kp) => {
                    println!("uri: {}, path: {:?}, pubkey: {}", kp.path, kp.derivation_path, kp.pubkey);
                    // convert the first 32 bytes of the pubkey to a boxed slice
                    let boxed_pubkey = Box::new(kp.pubkey.to_bytes().to_vec());
                    Box::into_raw(boxed_pubkey) as *mut u8
                    // let pubkey = kp.pubkey.as_bytes();
                    // let mut pubkey = pubkey.to_vec();
                    // pubkey.shrink_to_fit();
                    // let ptr = pubkey.as_mut_ptr();
                    // std::mem::forget(pubkey);
                    // ptr
                },
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    std::ptr::null_mut()
                },
            };
        }
        std::ptr::null_mut()
    }
}

/// free the memory allocated and returned by the derive functions by transferring ownership back to
/// Rust. must be called on each pointer returned by the functions precisely once to ensure safety.
#[no_mangle]
pub extern "C" fn free_c(ptr: *mut u8) {
    unsafe {
        if !ptr.is_null() {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() -> Result<(), RemoteWalletError> {
        let locator = Locator::new_from_path("usb://ledger").unwrap();
        // let path = DerivationPath::default();
        let s = "m/44'/540'/0'/0'/0'";
        let path = DerivationPath::from_absolute_path_str(s)?;
        // let path = DerivationPath::new_bip44(Some(0), Some(0));
        let wm = &maybe_wallet_manager().unwrap();
        if let Some(wm) = wm {
            return match generate_remote_keypair(locator, path, wm, false, "main") {
                Ok(kp) => {
                    println!("uri: {}, path: {:?}, pubkey: {}", kp.path, kp.derivation_path, kp.pubkey);
                    Ok(())
                },
                Err(e) => Err(e),
            };
        }
        Err(RemoteWalletError::NoDeviceFound)
    }
}

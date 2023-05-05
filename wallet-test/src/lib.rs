use {
    spacemesh_remote_wallet::remote_keypair::generate_remote_keypair,
    spacemesh_remote_wallet::remote_wallet::{RemoteWalletError, maybe_wallet_manager},
    spacemesh_remote_wallet::locator::Locator,
    spacemesh_derivation_path::DerivationPath,
};

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() -> Result<(), RemoteWalletError> {
        let locator = Locator::new_from_path("usb://ledger").unwrap();
        // let path = DerivationPath::default();
        // let s = "m/44'/540'/0'/0'/0'";
        // let path = DerivationPath::from_absolute_path_str(s)?;
        let path = DerivationPath::new_bip44(Some(0), Some(0));
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

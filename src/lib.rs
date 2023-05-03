use {
    derivation_path::{ChildIndex, DerivationPath as DerivationPathInner},
    solana_remote_wallet::remote_keypair::generate_remote_keypair,
    solana_remote_wallet::remote_wallet::maybe_wallet_manager,
    solana_remote_wallet::locator::Locator,
    solana_sdk::derivation_path::{DerivationPath as SolDerivationPath},
};

struct DerivationPath(SolDerivationPath);

impl Default for DerivationPath {
    fn default() -> Self {
        Self::new_bip44(None, None)
    }
}

impl From<DerivationPath> for SolDerivationPath {
    fn from(path: DerivationPath) -> Self {
        path.0
    }
}

// impl AsRef<SolDerivationPath> for DerivationPath {
//     fn as_ref(&self) -> &SolDerivationPath {
//         &self.0
//     }
// }

impl DerivationPath {
    // pub fn from_key_str(path: &str) -> Result<Self, DerivationPathError> {
    //     Self::from_key_str_with_coin(path, Smesh)
    // }
    fn new<P: Into<Box<[ChildIndex]>>>(path: P) -> Self {
        // We have to jump through hoops here since the SolDerivationPath is not easy to extend,
        // as it has private methods and fields.
        let inner_path = DerivationPathInner::new(path);
        let sol_path = SolDerivationPath::from_absolute_path_str(inner_path.to_string().as_str()).unwrap();
        Self(sol_path)
    }

    pub fn new_bip44(account: Option<u32>, change: Option<u32>) -> Self {
        Self::new_bip44_with_coin(Smesh, account, change)
    }

    fn new_bip44_with_coin<T: Bip44>(coin: T, account: Option<u32>, change: Option<u32>) -> Self {
        let mut indexes = coin.base_indexes();
        if let Some(account) = account {
            indexes.push(ChildIndex::Hardened(account));
            if let Some(change) = change {
                indexes.push(ChildIndex::Hardened(change));
            }
        }
        Self::new(indexes)
    }
}

// private trait in Solana code, copied from src/derivation_path.rs
trait Bip44 {
    const PURPOSE: u32 = 44;
    const COIN: u32;

    fn base_indexes(&self) -> Vec<ChildIndex> {
        vec![
            ChildIndex::Hardened(Self::PURPOSE),
            ChildIndex::Hardened(Self::COIN),
        ]
    }
}

struct Smesh;

impl Bip44 for Smesh {
    const COIN: u32 = 540;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let uri = URIReference::try_from("usb://ledger")?;
        let locator = Locator::new_from_path("usb://ledger").unwrap();
        let path = DerivationPath::default();
        // let mut wm:&Option<Arc<RemoteWalletManager>> = None;
        // let wm:&Option<Arc<RemoteWalletManager>> = &maybe_wallet_manager().unwrap();
        let wm = &maybe_wallet_manager().unwrap();
        // let wm:&mut Option<Arc<RemoteWalletManager>> = None;
        // *wm = maybe_wallet_manager().unwrap();
        // assert!(generate_remote_keypair(locator, path, wm.unwrap(), false, "main").is_ok())
        if let Some(wm) = wm {
            // assert!(generate_remote_keypair(locator, SolDerivationPath::from(path), wm, false, "main").is_ok())
            assert!(generate_remote_keypair(locator, path.into(), wm, false, "main").is_ok())
        } else {
            assert!(false)
            // Err(RemoteWalletError::NoDeviceFound.into())
        }
    }
}

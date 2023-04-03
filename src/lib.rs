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
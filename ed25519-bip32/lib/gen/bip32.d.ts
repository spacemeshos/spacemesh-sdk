/* tslint:disable */
/* eslint-disable */
/**
* derive_key generates a keypair from a 64-byte BIP39-compatible seed and BIP32 hierarchical
* derivation path. it returns 64 bytes. the first 32 bytes are the secret key and the second 32
* bytes are the public key.
* @param {Uint8Array} seed
* @param {string} path
* @returns {Uint8Array}
*/
export function derive_key(seed: Uint8Array, path: string): Uint8Array;

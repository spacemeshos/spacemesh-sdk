/* tslint:disable */
/* eslint-disable */
/**
* derive_key generates keypair from seed and bip32 hierarchical derivation path.
* it returns 64 bytes. first 32 bytes are secret key, and the second 32 bytes are public key.
* @param {Uint8Array} seed
* @param {string} path
* @returns {Uint8Array}
*/
export function derive_key(seed: Uint8Array, path: string): Uint8Array;

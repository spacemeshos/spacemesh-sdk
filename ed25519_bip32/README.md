# ed25519_wasm

This repository contains WebAssembly (Wasm) and CFFI bindings for the
[ed25519-dalek-bip32](https://github.com/jpopesculian/ed25519-dalek-bip32) library. This library is used by [smcli](https://github.com/spacemeshos/smcli), [smapp](https://github.com/spacemeshos/smapp), and other tools to perform BIP32-style HD key derivation using BIP39-style mnemonics.

The bindings are currently in use in the following places:
- [smkeys](https://github.com/spacemeshos/smkeys) (Golang)
- [smapp](https://github.com/spacemeshos/smapp/pull/1207) (Typescript)
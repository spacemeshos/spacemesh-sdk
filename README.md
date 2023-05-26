# spacemesh-sdk

This repository contains a low-level Rust SDK for the Spacemesh protocol and associated tooling. Various crates implement utilities such as key derivation and communication with Ledger hardware wallets (see inline Rust documentation for more information). Certain functions are externalized via Wasm bindings and CFFI bindings for use in upstream applications including [Smapp](https://github.com/spacemeshos/smapp/) and [Smcli](https://github.com/spacemeshos/smcli).

See the Github workflow files for information on how to build on various platforms as a dynamic or static library.

Portions of the codebase are forked from [Solana](https://github.com/solana-labs/solana/) with gratitude.

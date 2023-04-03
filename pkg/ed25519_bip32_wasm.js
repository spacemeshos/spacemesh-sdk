import * as wasm from "./ed25519_bip32_wasm_bg.wasm";
import { __wbg_set_wasm } from "./ed25519_bip32_wasm_bg.js";
__wbg_set_wasm(wasm);
export * from "./ed25519_bip32_wasm_bg.js";

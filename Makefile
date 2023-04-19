.PHONY: deps
deps:
	cargo install wasm-pack cbindgen

.PHONY: build
build:
	cargo build

.PHONY: wasm
wasm:
	rm -rf ./lib/gen
	wasm-pack build --target nodejs -d ./lib/gen --out-name bip32
	rm -rf ./lib/*/.gitignore
	rm -rf ./lib/*/package.json

.PHONY: cheader
cheader:
	cbindgen -c cbindgen.toml -o ed25519_bip32.h

.PHONY: clean
clean:
	rm -rf ./lib/gen
	rm -rf ./target

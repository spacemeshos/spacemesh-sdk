.PHONY: build
build:
	cargo build --profile release-clib

.PHONY: wasm
wasm:
	cargo install wasm-pack
	rm -rf ./ed25519-bip32/lib/gen
	cd ed25519-bip32 && wasm-pack build --target nodejs -d ./lib/gen --out-name bip32
	rm -rf ./ed25519-bip32/lib/*/.gitignore
	rm -rf ./ed25519-bip32/lib/*/package.json

.PHONY: cheader
cheader:
	cargo install cbindgen
	cd ed25519-bip32 && cbindgen -c ../cbindgen.toml -o ed25519_bip32.h
	cd remote-wallet && cbindgen -c ../cbindgen.toml -o remote_wallet.h

# Regenerate the C Header and complain if it's changed
.PHONY: diff
diff: cheader
	@cd ed25519-bip32 && git diff --name-only --diff-filter=AM --exit-code ed25519_bip32.h \
		|| { echo "C header has changed"; exit 1; }
	@cd remote-wallet && git diff --name-only --diff-filter=AM --exit-code remote_wallet.h \
		|| { echo "C header has changed"; exit 1; }

.PHONY: clean
clean:
	rm -rf ./target

HEADERFN := ed25519_bip32.h

.PHONY: build
build:
	cargo build

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
	cd ed25519-bip32 && cbindgen -c ../cbindgen.toml -o $(HEADERFN)

# Regenerate the C Header and complain if it's changed
.PHONY: diff
diff: cheader
	@cd ed25519-bip32 && git diff --name-only --diff-filter=AM --exit-code $(HEADERFN) \
		|| { echo "C header has changed"; exit 1; }

.PHONY: clean
clean:
	rm -rf ./ed25519-bip32/lib/gen
	rm -rf ./target

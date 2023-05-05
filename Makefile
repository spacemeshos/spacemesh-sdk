HEADERFN := ed25519_bip32.h

.PHONY: build
build:
	cargo build

.PHONY: wasm
wasm:
	cargo install wasm-pack
	rm -rf ./lib/gen
	wasm-pack build --target nodejs -d ./lib/gen --out-name bip32
	rm -rf ./lib/*/.gitignore
	rm -rf ./lib/*/package.json

.PHONY: cheader
cheader:
	cargo install cbindgen
	cbindgen -c cbindgen.toml -o $(HEADERFN)

# Regenerate the C Header and complain if it's changed
.PHONY: diff
diff: cheader
	@git diff --name-only --diff-filter=AM --exit-code $(HEADERFN) \
		|| { echo "C header has changed"; exit 1; }

.PHONY: clean
clean:
	rm -rf ./lib/gen
	rm -rf ./target

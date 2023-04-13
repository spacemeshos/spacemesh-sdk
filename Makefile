.PHONY: deps
deps:
	cargo install wasm-pack

.PHONY: build
build:
	rm -rf ./lib/gen
	wasm-pack build --target nodejs -d ./lib/gen --out-name bip32
	rm -rf ./lib/*/.gitignore
	rm -rf ./lib/*/package.json
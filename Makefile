.PHONY: deps
deps:
	cargo install wasm-pack

.PHONY: build
build:
	wasm-pack build
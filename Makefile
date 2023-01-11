# See https://github.com/cosmos/cosmos-sdk/blob/main/proto/README.md
COSMOS_SDK_COMMIT = "8cb30a2c4de74dc9bd8d260b1e75e176"

# Feel free to override. Example: `make validation-chain-test MAMORU_ENDPOINT=any-other-url`
MAMORU_ENDPOINT ="http://localhost:9090"
MAMORU_TEST_FAUCET_ENDPOINT = "http://localhost:4500"
RUST_LOG="warn,mamoru_core=debug"

init:
	git submodule update --init

submodule-update:
	git submodule update --remote --merge

pull-proto-dependencies:
	buf export buf.build/cosmos/cosmos-sdk:$(COSMOS_SDK_COMMIT) --output ./validation-chain-client/proto/

test:
	cargo test --all-features --workspace
	
headers:
	cargo test --features headers -- --exact generate_headers --nocapture

validation-chain-test:
	RUST_LOG=$(RUST_LOG) \
	MAMORU_ENDPOINT=$(MAMORU_ENDPOINT) \
	MAMORU_TEST_FAUCET_ENDPOINT=$(MAMORU_TEST_FAUCET_ENDPOINT) \
    cargo test validation_chain -- --nocapture --ignored

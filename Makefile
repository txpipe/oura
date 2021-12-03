
TESTNET_MAGIC = 1097911063
MAINNET_MAGIC = 764824073

try-testnet-tcp:
	cargo run --bin oura -- --socket localhost:3307 --magic ${TESTNET_MAGIC}

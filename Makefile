
TESTNET_MAGIC = 1097911063
MAINNET_MAGIC = 764824073

try-testnet-tcp:
	cargo run -- --socket localhost:3307 --magic ${TESTNET_MAGIC}
